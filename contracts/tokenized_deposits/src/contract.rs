use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{
    Address, Env, IntoVal, MuxedAddress, String, Symbol,
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contract, contractimpl, token, vec as svec,
};

use crate::{
    constants::*,
    error::TDError,
    events,
    market::{self, MarketClient},
    storage::{self, AllowanceValue, ShareTokenMetadata},
};

// A tokenized deposit for the lending market: a share token that grows in price and can be
// borrowed against.
//
// # Per-participant obligations
//
// The vault does not pool liquidity into one position. Each participant gets their own market
// obligation, owned by the vault but namespaced by a seed derived from their address (see
// `market::participant_key`). This isolates liquidation risk -- if one participant over-borrows and
// is liquidated, only their own collateral is seized and other holders are untouched.
//
// # No local balance ledger, and no `total_supply`
//
// `balance()` reads live from the market: it *is* the participant's `jToken` count, not a mirrored
// copy. This is deliberate. A liquidator seizing `jTokens` mutates market state directly with no
// callback to this contract, so any cached balance would silently drift and over-report. Reading
// through makes liquidations self-reporting, and the ledger cannot lie.
//
// The same reasoning rules out a stored `total_supply`: an honest value would be the sum over every
// participant's live position, and the market exposes no cheap aggregate. It is therefore not
// implemented. SEP-41 does not require it.
//
// # Deviation: transfers are health-gated, so shares are not freely fungible
//
// A transfer moves value between two market obligations and the market enforces the sender's health
// on the way out. A participant who has borrowed against their shares cannot move the portion
// backing that debt -- the call reverts.
//
// This is intended: debt must never silently follow shares to an unwitting recipient. But it means
// integrators that assume `transfer` always succeeds given sufficient balance (most AMMs) may
// break. This contract deliberately makes no SEP-56 conformance claim.
//
// # MVP limitation: transfers route through withdraw/deposit
//
// The market has no primitive for moving `jTokens` between obligations, so a transfer is a
// withdrawal from the sender followed by a deposit to the recipient. That inherits three real
// costs: the pool's withdrawal fee is charged on every transfer, transfers can fail when pool
// liquidity is low regardless of the sender's health, and rounding applies twice. A market-level
// `transfer_j_tokens` would remove all three; until then transfers are an occasional operation,
// not a hot path
#[contract]
pub struct TokenizedDepositsContract;

#[contractimpl]
impl TokenizedDepositsContract {
    pub fn __constructor(
        e: Env,
        admin: Address,
        asset: Address,
        market: Address,
        pool: Address,
        name: String,
        symbol: String,
    ) -> Result<(), TDError> {
        if name.is_empty() || name.len() > MAX_NAME_LENGTH {
            return Err(TDError::InvalidInitialization);
        }
        if symbol.is_empty() || symbol.len() > MAX_SYMBOL_LENGTH {
            return Err(TDError::InvalidInitialization);
        }

        // Shares are `jTokens`, which the market denominates in the underlying's own decimals
        let decimals = token::TokenClient::new(&e, &asset).decimals();
        if decimals > MAX_DECIMALS {
            return Err(TDError::InvalidInitialization);
        }

        storage::set_admin(&e, &admin);
        storage::set_asset(&e, &asset);
        storage::set_market(&e, &market);
        storage::set_pool(&e, &pool);
        storage::set_metadata(&e, &ShareTokenMetadata { name, symbol, decimals });
        storage::bump_instance(&e);

        Ok(())
    }

    // -- Deposit / redeem --

    // Deposits `assets` of the underlying and credits shares to `receiver`.
    //
    // The assets land in `receiver`'s own sub-obligation, so their position is liquidation-isolated
    // from every other holder from this point on
    pub fn deposit(e: Env, assets: i128, receiver: Address, from: Address) -> Result<i128, TDError> {
        from.require_auth();
        Self::require_positive(assets)?;

        if storage::get_deposits_paused(&e) {
            return Err(TDError::DepositsPaused);
        }
        storage::bump_instance(&e);

        let (asset, market, pool) =
            (storage::get_asset(&e), storage::get_market(&e), storage::get_pool(&e));
        let vault = e.current_contract_address();

        let before = market::participant_j_tokens(&e, &market, &pool, &receiver);

        token::TokenClient::new(&e, &asset).transfer(&from, MuxedAddress::from(&vault), &assets);
        Self::authorize_market_pull(&e, &asset, &market, assets);
        MarketClient::new(&e, &market).deposit(
            &market::participant_key(&e, &receiver),
            &pool,
            &assets,
            &None,
        );

        // Minted shares are the observed delta rather than a locally computed figure, so the
        // market's own rounding stays the single source of truth
        let minted = market::participant_j_tokens(&e, &market, &pool, &receiver)
            .checked_sub(before)
            .ok_or(TDError::OverOrUnderflow)?;
        if minted <= 0 {
            return Err(TDError::ZeroShares);
        }

        events::deposit(&e, from.clone(), from, receiver, assets, minted);

        Ok(minted)
    }

    // Burns shares from `owner` and pays the underlying to `receiver`.
    //
    // Returns the amount actually received, which is net of the pool's withdrawal fee and capped by
    // the market to whatever keeps `owner`'s obligation healthy
    pub fn redeem(e: Env, shares: i128, receiver: Address, owner: Address) -> Result<i128, TDError> {
        owner.require_auth();
        Self::require_positive(shares)?;
        storage::bump_instance(&e);

        let (asset, market, pool) =
            (storage::get_asset(&e), storage::get_market(&e), storage::get_pool(&e));
        let vault = e.current_contract_address();

        let held = market::participant_j_tokens(&e, &market, &pool, &owner);
        if shares > held {
            return Err(TDError::InsufficientBalance);
        }

        let assets = Self::assets_for_shares(&e, &market, &pool, &owner, shares)?;
        if assets <= 0 {
            return Err(TDError::ZeroAssets);
        }

        // The market caps oversized withdrawals silently rather than reverting, so a redemption
        // that health or liquidity cannot honour in full is rejected outright. Otherwise the caller
        // would be told the redemption succeeded while receiving less than their shares are worth
        if market::max_withdrawable(&e, &market, &pool, &owner) < assets {
            return Err(TDError::ExceedsMaxWithdraw);
        }

        let asset_client = token::TokenClient::new(&e, &asset);
        let balance_before = asset_client.balance(&vault);

        MarketClient::new(&e, &market).withdraw(
            &market::participant_key(&e, &owner),
            &pool,
            &assets,
            &None,
        );

        // The market caps withdrawals by health and available liquidity, so it may pay less than
        // asked. Forwarding the observed delta -- never the requested figure -- stops the vault from
        // paying one holder out of another's liquidity
        let received = asset_client
            .balance(&vault)
            .checked_sub(balance_before)
            .ok_or(TDError::OverOrUnderflow)?;
        if received <= 0 {
            return Err(TDError::ZeroAssets);
        }

        asset_client.transfer(&vault, MuxedAddress::from(&receiver), &received);

        let burned = held
            .checked_sub(market::participant_j_tokens(&e, &market, &pool, &owner))
            .ok_or(TDError::OverOrUnderflow)?;

        events::withdraw(&e, owner.clone(), receiver, owner, received, burned);

        Ok(received)
    }

    // -- Borrowing --

    // Borrows `amount` of `borrow_asset` from `borrow_pool` against the caller's shares.
    //
    // The debt sits on the caller's own sub-obligation, so only their collateral is at risk if the
    // position later becomes liquidatable. The market rejects the call outright if the borrow would
    // leave that obligation unhealthy.
    //
    // `borrow_asset` must be the asset `borrow_pool` lends. It is passed explicitly because reading
    // it back from the market would require mirroring the whole `Pool` struct; a caller who passes
    // the wrong address simply observes no balance delta and receives nothing
    pub fn borrow(
        e: Env,
        borrower: Address,
        borrow_pool: Address,
        borrow_asset: Address,
        amount: i128,
    ) -> Result<i128, TDError> {
        borrower.require_auth();
        Self::require_positive(amount)?;
        storage::bump_instance(&e);

        let market = storage::get_market(&e);
        let vault = e.current_contract_address();

        // Borrowed funds are paid to the obligation's owner, which is the vault, so they have to be
        // forwarded on to the participant who actually took the debt
        let asset_client = token::TokenClient::new(&e, &borrow_asset);
        let balance_before = asset_client.balance(&vault);

        MarketClient::new(&e, &market).borrow(
            &market::participant_key(&e, &borrower),
            &borrow_pool,
            &amount,
            &None,
        );

        let received = asset_client
            .balance(&vault)
            .checked_sub(balance_before)
            .ok_or(TDError::OverOrUnderflow)?;
        if received <= 0 {
            return Err(TDError::ZeroAssets);
        }

        asset_client.transfer(&vault, MuxedAddress::from(&borrower), &received);

        Ok(received)
    }

    // Repays `amount` of the caller's debt in `borrow_pool`.
    //
    // The caller must hold `borrow_asset`; it is pulled from them and forwarded to the market
    pub fn repay(
        e: Env,
        borrower: Address,
        borrow_pool: Address,
        borrow_asset: Address,
        amount: i128,
    ) -> Result<(), TDError> {
        borrower.require_auth();
        Self::require_positive(amount)?;
        storage::bump_instance(&e);

        let market = storage::get_market(&e);
        let vault = e.current_contract_address();

        token::TokenClient::new(&e, &borrow_asset).transfer(
            &borrower,
            MuxedAddress::from(&vault),
            &amount,
        );
        Self::authorize_market_pull(&e, &borrow_asset, &market, amount);

        MarketClient::new(&e, &market).repay(
            &market::participant_key(&e, &borrower),
            &borrow_pool,
            &amount,
            &None,
        );

        Ok(())
    }

    // -- Conversions --

    // The underlying value of an account's shares, before withdrawal fees.
    //
    // This is the headline "how much has my deposit grown" figure: it rises purely as supply
    // interest accrues to the pool
    pub fn assets_of(e: Env, account: Address) -> Result<i128, TDError> {
        let (market, pool) = (storage::get_market(&e), storage::get_pool(&e));
        let shares = market::participant_j_tokens(&e, &market, &pool, &account);

        Self::assets_for_shares(&e, &market, &pool, &account, shares)
    }

    // Converts a share count to underlying at the pool's current rate, before withdrawal fees.
    //
    // `reference` must be an account that currently holds shares -- the rate is recovered from a
    // withdrawal simulation, so it needs a funded position to read against. The rate itself is
    // pool-wide, so any funded account yields the same answer
    pub fn convert_to_assets(e: Env, shares: i128, reference: Address) -> Result<i128, TDError> {
        let (market, pool) = (storage::get_market(&e), storage::get_pool(&e));

        Self::assets_for_shares(&e, &market, &pool, &reference, shares)
    }

    // Converts an underlying amount to the shares it would currently buy
    pub fn convert_to_shares(e: Env, assets: i128, reference: Address) -> Result<i128, TDError> {
        Self::require_non_negative(assets)?;
        if assets == 0 {
            return Ok(0);
        }

        let (market, pool) = (storage::get_market(&e), storage::get_pool(&e));
        let (pool_assets, j_tokens) = market::j_token_rate(&e, &market, &pool, &reference)?;

        Ok(assets.fixed_mul_floor(&e, &j_tokens, &pool_assets))
    }

    // -- SEP-41 --

    // The account's share balance, read live from its market sub-obligation.
    //
    // Reading through rather than mirroring means a liquidation is reflected here immediately, with
    // no reconciliation step that could lag or fail
    pub fn balance(e: Env, id: Address) -> i128 {
        let (market, pool) = (storage::get_market(&e), storage::get_pool(&e));

        market::participant_j_tokens(&e, &market, &pool, &id)
    }

    // Moves shares between holders.
    //
    // Implemented as a withdrawal from `from` followed by a deposit to `to`, since the market
    // exposes no direct way to move `jTokens` between obligations. Two consequences: the pool's
    // withdrawal fee is charged, so `to` receives slightly less than `from` gives up; and the market
    // rejects the withdrawal if it would leave `from` unhealthy, which is what stops debt from
    // following the shares
    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) -> Result<(), TDError> {
        from.require_auth();

        Self::do_transfer(&e, &from, &to, amount)
    }

    pub fn transfer_from(
        e: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), TDError> {
        spender.require_auth();
        Self::spend_allowance(&e, &from, &spender, amount)?;

        Self::do_transfer(&e, &from, &to, amount)
    }

    pub fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        storage::get_allowance(&e, &from, &spender).amount
    }

    pub fn approve(
        e: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) -> Result<(), TDError> {
        from.require_auth();
        Self::require_non_negative(amount)?;

        // A live allowance expiring in the past could never be spent, so treat it as a caller error
        if amount > 0 && expiration_ledger < e.ledger().sequence() {
            return Err(TDError::AllowanceExpired);
        }
        storage::bump_instance(&e);
        storage::set_allowance(&e, &from, &spender, &AllowanceValue { amount, expiration_ledger });

        events::approve(&e, from, spender, amount, expiration_ledger);

        Ok(())
    }

    pub fn decimals(e: Env) -> u32 {
        storage::get_metadata(&e).decimals
    }

    pub fn name(e: Env) -> String {
        storage::get_metadata(&e).name
    }

    pub fn symbol(e: Env) -> String {
        storage::get_metadata(&e).symbol
    }

    // -- Wiring --

    pub fn query_asset(e: Env) -> Address {
        storage::get_asset(&e)
    }

    pub fn query_market(e: Env) -> Address {
        storage::get_market(&e)
    }

    pub fn query_pool(e: Env) -> Address {
        storage::get_pool(&e)
    }

    // The market obligation key backing an account's position. Exposed so that liquidators can
    // locate and act on a specific participant's position directly on the market
    pub fn obligation_key_of(e: Env, account: Address) -> market::ObligationKey {
        market::participant_key(&e, &account)
    }

    // -- Admin --

    pub fn set_deposits_paused(e: Env, paused: bool) -> Result<(), TDError> {
        storage::require_admin(&e)?;
        storage::set_deposits_paused(&e, paused);

        Ok(())
    }

    pub fn propose_new_admin(e: Env, new_admin: Address) -> Result<(), TDError> {
        storage::require_admin(&e)?;
        storage::set_pending_admin(&e, &new_admin);

        Ok(())
    }

    pub fn accept_proposed_admin(e: Env) -> Result<(), TDError> {
        let pending = storage::get_pending_admin(&e).ok_or(TDError::NoPendingAdmin)?;
        pending.require_auth();

        storage::set_admin(&e, &pending);
        storage::clear_pending_admin(&e);

        Ok(())
    }

    // -- Internals --

    fn assets_for_shares(
        e: &Env,
        market: &Address,
        pool: &Address,
        reference: &Address,
        shares: i128,
    ) -> Result<i128, TDError> {
        Self::require_non_negative(shares)?;
        if shares == 0 {
            return Ok(0);
        }

        let (assets, j_tokens) = market::j_token_rate(e, market, pool, reference)?;

        Ok(shares.fixed_mul_floor(e, &assets, &j_tokens))
    }

    fn do_transfer(e: &Env, from: &Address, to: &Address, amount: i128) -> Result<(), TDError> {
        Self::require_positive(amount)?;
        storage::bump_instance(e);

        let (asset, market, pool) =
            (storage::get_asset(e), storage::get_market(e), storage::get_pool(e));
        let vault = e.current_contract_address();

        let held = market::participant_j_tokens(e, &market, &pool, from);
        if amount > held {
            return Err(TDError::InsufficientBalance);
        }

        let assets = Self::assets_for_shares(e, &market, &pool, from, amount)?;
        if assets <= 0 {
            return Err(TDError::ZeroAssets);
        }

        // The market silently *caps* an oversized withdrawal to whatever keeps the position healthy
        // rather than rejecting it. Left unchecked that would quietly move less than asked while
        // still reporting success, so the cap is detected up front and turned into a hard failure.
        // This is what stops a borrower's debt from being partially shed onto a recipient
        let withdrawable = market::max_withdrawable(e, &market, &pool, from);
        if withdrawable < assets {
            return Err(TDError::TransferWouldBeUnhealthy);
        }

        let asset_client = token::TokenClient::new(e, &asset);
        let balance_before = asset_client.balance(&vault);

        MarketClient::new(e, &market).withdraw(
            &market::participant_key(e, from),
            &pool,
            &assets,
            &None,
        );

        let received = asset_client
            .balance(&vault)
            .checked_sub(balance_before)
            .ok_or(TDError::OverOrUnderflow)?;
        if received <= 0 {
            return Err(TDError::ZeroAssets);
        }

        // Re-deposit whatever actually came out. The recipient absorbs the withdrawal fee, because
        // crediting the full requested amount would mean covering the gap from other participants'
        // liquidity
        Self::authorize_market_pull(e, &asset, &market, received);
        MarketClient::new(e, &market).deposit(&market::participant_key(e, to), &pool, &received, &None);

        let burned = held
            .checked_sub(market::participant_j_tokens(e, &market, &pool, from))
            .ok_or(TDError::OverOrUnderflow)?;

        events::transfer(e, from.clone(), to.clone(), burned);

        Ok(())
    }

    // Pre-authorizes the market to pull exactly `amount` of `asset` from the vault.
    //
    // The market moves funds via a nested token transfer, and a contract cannot sign for its own
    // sub-invocations. Scoping the grant to this one contract, function and amount means a
    // compromised market cannot reuse it to pull anything beyond the transfer in flight
    fn authorize_market_pull(e: &Env, asset: &Address, market: &Address, amount: i128) {
        let vault = e.current_contract_address();

        e.authorize_as_current_contract(svec![
            e,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: asset.clone(),
                    fn_name: Symbol::new(e, "transfer"),
                    args: svec![e, vault.to_val(), market.to_val(), amount.into_val(e)],
                },
                sub_invocations: svec![e],
            })
        ]);
    }

    fn spend_allowance(
        e: &Env,
        from: &Address,
        spender: &Address,
        amount: i128,
    ) -> Result<(), TDError> {
        let allowance = storage::get_allowance(e, from, spender);
        if allowance.amount < amount {
            return Err(TDError::InsufficientAllowance);
        }

        storage::set_allowance(
            e,
            from,
            spender,
            &AllowanceValue {
                amount: allowance.amount - amount, // safe: checked above
                expiration_ledger: allowance.expiration_ledger,
            },
        );

        Ok(())
    }

    fn require_positive(amount: i128) -> Result<(), TDError> {
        if amount <= 0 {
            return Err(TDError::NegativeAmount);
        }

        Ok(())
    }

    fn require_non_negative(amount: i128) -> Result<(), TDError> {
        if amount < 0 {
            return Err(TDError::NegativeAmount);
        }

        Ok(())
    }
}
