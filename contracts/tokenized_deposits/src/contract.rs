use soroban_sdk::{
    Address, BytesN, Env, IntoVal, MuxedAddress, String, Symbol,
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contract, contractimpl, panic_with_error, token, vec as svec,
};

use crate::{
    constants::*,
    error::TDError,
    events,
    market::{MarketClient, vault_obligation_key, vault_withdrawable},
    math_utils::*,
    storage::{self, AllowanceValue, ShareTokenMetadata},
    vault::TokenizedVault,
};

#[contract]
// A SEP-56 tokenized vault over a lending market pool.
//
// Users deposit the underlying asset and receive a fixed, pre-determined share token amount. The vault
// forwards the deposited assets into the market, where they accrue interest; as interest accrues, the
// vault's claim on the market grows while the share count stays put, so each share becomes
// redeemable for more of the underlying. Holders see their yield as a rising share *price* on
// redeem/withdraw.
//
// # Relationship to the market
//
// The vault is a single, ordinary depositor from the market's point of view: all vault liquidity
// sits in one obligation keyed by the vault's own address.
pub struct TokenizedDepositsContract;

impl TokenizedDepositsContract {
    // Reads the vault's redeemable claim on the market.
    fn total_assets_internal(e: &Env) -> i128 {
        let market = storage::get_market(e);
        let pool = storage::get_pool(e);

        vault_withdrawable(e, &market, &pool)
    }

    // Builds the conversion rate from live totals. Re-derived per operation so that interest
    // accrued since the last call is always reflected
    fn rate(e: &Env) -> Result<Rate, TDError> {
        Rate::new(
            storage::get_total_supply(e),
            Self::total_assets_internal(e),
            storage::get_decimals_offset(e),
        )
    }

    fn mint_shares(e: &Env, to: &Address, shares: i128) -> Result<(), TDError> {
        let balance = storage::get_balance(e, to).checked_add(shares).map_over_or_underflow()?;
        let total = storage::get_total_supply(e).checked_add(shares).map_over_or_underflow()?;

        storage::set_balance(e, to, &balance);
        storage::set_total_supply(e, &total);

        Ok(())
    }

    fn burn_shares(e: &Env, from: &Address, shares: i128) -> Result<(), TDError> {
        let balance = storage::get_balance(e, from);
        if balance < shares {
            return Err(TDError::InsufficientBalance);
        }

        let total = storage::get_total_supply(e).checked_sub(shares).map_over_or_underflow()?;
        if total.is_negative() {
            return Err(TDError::InternalError);
        }

        storage::set_balance(e, from, &(balance - shares)); // safe
        storage::set_total_supply(e, &total);

        Ok(())
    }

    fn spend_allowance(
        e: &Env,
        from: &Address,
        spender: &Address,
        amount: i128,
    ) -> Result<(), TDError> {
        // Acting on your own shares never consumes an allowance | why?
        if from == spender {
            return Ok(());
        }

        let allowance = storage::get_allowance(e, from, spender);
        if allowance.amount < amount {
            return Err(TDError::InsufficientAllowance);
        }

        storage::set_allowance(
            e,
            from,
            spender,
            &AllowanceValue {
                amount: allowance.amount - amount, // safe
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

    // Shared entry path for `deposit` and `mint`.
    //
    // Pulls `assets` from `from`, supplies them to the market, and mints `shares` to `receiver`
    fn enter(
        e: &Env,
        assets: i128,
        shares: i128,
        receiver: &Address,
        from: &Address,
        operator: &Address,
    ) -> Result<(), TDError> {
        // TODO: Is it better to put this check here or at the step above??
        if storage::get_deposits_paused(e) {
            return Err(TDError::DepositsPaused);
        }

        Self::require_positive(assets)?;
        if shares <= 0 {
            // WARN: Weird
            return Err(TDError::ZeroShares);
        }

        let pool = storage::get_pool(e);
        let asset = storage::get_asset(e);
        let market = storage::get_market(e);

        // Pull the assets in first. `from` has authorized the vault via `require_auth` below, so
        // this transfer is the point at which the user actually parts with their funds
        let vault = e.current_contract_address();
        token::TokenClient::new(e, &asset).transfer(from, MuxedAddress::from(&vault), &assets);
        // TODO: Address instead of Mu

        // Supply into the market under the vault's own obligation. The vault authorizes this
        // call as itself, which is why the market sees one depositor rather than many.
        //
        // The market will pull the assets from the vault via the token contract. That nested
        // transfer needs the vault's authorization, and since the vault is a contract it cannot
        // sign -- it must pre-declare the exact sub-invocation it consents to. Scoping the
        // authorization to this specific call and amount means a compromised or malicious market
        // cannot use it to drain anything beyond what is being deposited right now
        e.authorize_as_current_contract(svec![
            e, // WARN: Again, why do we demand this explicit authorization here?
            //
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: asset.clone(),
                    fn_name: Symbol::new(e, "transfer"),
                    args: svec![e, vault.to_val(), market.to_val(), assets.into_val(e)],
                },
                sub_invocations: svec![e],
            })
        ]);

        MarketClient::new(e, &market).deposit(&vault_obligation_key(e), &pool, &assets, &None);

        Self::mint_shares(e, receiver, shares)?;
        storage::bump_instance(e);

        events::mint(e, receiver.clone(), shares);
        events::deposit(e, operator.clone(), from.clone(), receiver.clone(), assets, shares);

        Ok(())
    }

    // Shared exit path for `withdraw` and `redeem`.
    //
    // Burns `shares` from `owner`, pulls `assets` out of the market and forwards them to
    // `receiver`
    fn exit(
        e: &Env,
        assets: i128,
        shares: i128,
        receiver: &Address,
        owner: &Address,
        operator: &Address,
    ) -> Result<(), TDError> {
        Self::require_positive(assets)?;
        if shares <= 0 {
            return Err(TDError::ZeroShares);
        }

        let pool = storage::get_pool(e);
        let asset = storage::get_asset(e);
        let market = storage::get_market(e);

        // An operator acting for someone else must hold an allowance over their shares
        Self::spend_allowance(e, owner, operator, shares)?;
        // WARN: Is this by the standard?

        // Burn before the external calls so the accounting is settled regardless of what the
        // market does afterwards
        Self::burn_shares(e, owner, shares)?;

        let asset_client = token::TokenClient::new(e, &asset);
        let vault = e.current_contract_address();
        let balance_before = asset_client.balance(&vault);

        MarketClient::new(e, &market).withdraw(&vault_obligation_key(e), &pool, &assets, &None);

        // The market caps withdrawals by available liquidity and by the position's LTV, so it may
        // legitimately return less than requested. Paying `receiver` the requested amount in that
        // case would silently draw on other holders' liquidity, so treat a shortfall as fatal.
        // `max_withdraw` is what callers should consult to avoid ever hitting this
        let received =
            asset_client.balance(&vault).checked_sub(balance_before).map_over_or_underflow()?;

        if received < assets {
            return Err(TDError::MarketReturnedLess);
        }

        asset_client.transfer(&vault, MuxedAddress::from(receiver), &assets);

        storage::bump_instance(e);

        events::burn(e, owner.clone(), shares);
        events::withdraw(e, operator.clone(), receiver.clone(), owner.clone(), assets, shares);

        Ok(())
    }
}

#[contractimpl]
impl TokenizedDepositsContract {
    // Initializes the vault.
    //
    // # Arguments
    // * `admin` - may pause deposits and rotate the admin. Cannot touch user funds
    // * `asset` - the underlying SEP-41 asset
    // * `market` - the lending market contract
    // * `pool` - the market pool (identified by its address) to supply into
    // * `name`, `symbol` - share token metadata
    //
    // Share decimals are the underlying asset's decimals plus a virtual offset (see
    // [`PREFERRED_DECIMALS_OFFSET`]), which is what gives the inflation mitigation its headroom
    pub fn __constructor(
        e: Env,
        admin: Address,
        asset: Address,
        market: Address,
        pool: Address,
        name: String,
        symbol: String,
    ) -> Result<(), TDError> {
        if name.is_empty()
            || name.len() > MAX_NAME_LENGTH
            || symbol.is_empty()
            || symbol.len() > MAX_SYMBOL_LENGTH
        {
            return Err(TDError::InvalidInitialization);
        }

        let asset_decimals = token::TokenClient::new(&e, &asset).decimals();
        if asset_decimals > MAX_DECIMALS {
            return Err(TDError::InvalidInitialization);
        }

        // Take as much virtual offset as the share token's decimal budget allows. High-precision
        // underlying assets get a smaller offset rather than an unrepresentable share decimal
        let offset = PREFERRED_DECIMALS_OFFSET.min(MAX_DECIMALS - asset_decimals); // safe
        let decimals = asset_decimals + offset; // safe: bounded by MAX_DECIMALS above

        storage::set_admin(&e, &admin);
        storage::set_asset(&e, &asset);
        storage::set_market(&e, &market);
        storage::set_pool(&e, &pool);
        storage::set_metadata(&e, &ShareTokenMetadata { name, symbol, decimals });
        storage::set_decimals_offset(&e, &offset);
        storage::bump_instance(&e);

        Ok(())
    }

    // -- Administration --

    pub fn admin(e: Env) -> Address {
        storage::get_admin(&e)
    }

    pub fn market(e: Env) -> Address {
        storage::get_market(&e)
    }

    pub fn pool(e: Env) -> Address {
        storage::get_pool(&e)
    }

    pub fn deposits_paused(e: Env) -> bool {
        storage::get_deposits_paused(&e)
    }

    // Halts deposits. Withdrawals are intentionally left open: a pause must never be able to
    // strand user funds inside the vault
    pub fn set_deposits_paused(e: Env, paused: bool) -> Result<(), TDError> {
        let admin = storage::require_admin(&e)?;

        storage::set_deposits_paused(&e, paused);
        storage::bump_instance(&e);

        events::deposits_pause_set(&e, admin, paused);

        Ok(())
    }

    // Two-step rotation, so the role can never be handed to an address that cannot sign for it
    pub fn propose_new_admin(e: Env, new_admin: Address) -> Result<(), TDError> {
        let admin = storage::require_admin(&e)?;

        storage::set_pending_admin(&e, &new_admin);
        storage::bump_instance(&e);

        events::admin_proposed(&e, admin, new_admin);

        Ok(())
    }

    pub fn accept_proposed_admin(e: Env) -> Result<(), TDError> {
        let pending = storage::get_pending_admin(&e).ok_or(TDError::NoPendingAdmin)?;
        pending.require_auth();

        let old_admin = storage::get_admin(&e);

        storage::set_admin(&e, &pending);
        storage::clear_pending_admin(&e);
        storage::bump_instance(&e);

        events::admin_updated(&e, old_admin, pending);

        Ok(())
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), TDError> {
        storage::require_admin(&e)?;
        e.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }
}

#[contractimpl]
impl TokenizedVault for TokenizedDepositsContract {
    fn total_supply(e: Env) -> i128 {
        storage::get_total_supply(&e)
    }

    fn query_asset(e: Env) -> Address {
        storage::get_asset(&e)
    }

    fn total_assets(e: Env) -> i128 {
        Self::total_assets_internal(&e)
    }

    fn convert_to_shares(e: Env, assets: i128) -> i128 {
        Self::rate(&e)
            .unwrap_or_else(|err| panic_with_error!(&e, err))
            .assets_to_shares_floor(&e, assets)
    }

    fn convert_to_assets(e: Env, shares: i128) -> i128 {
        Self::rate(&e)
            .unwrap_or_else(|err| panic_with_error!(&e, err))
            .shares_to_assets_floor(&e, shares)
    }

    // NB: this reports "no limit" rather than reading the market's supply cap. Querying the cap
    // would require a second cross-contract call on a purely informational path; a deposit that
    // exceeds it still fails atomically inside the market
    fn max_deposit(e: Env, _receiver: Address) -> i128 {
        // TODO: We must read supply cap for sure + the deposit can be frozen on the market as well
        //

        if storage::get_deposits_paused(&e) { 0 } else { i128::MAX }
    }

    fn preview_deposit(e: Env, assets: i128) -> i128 {
        // TODO: This must also check for market's constraints
        Self::convert_to_shares(e, assets)
    }

    // TODO: Should this panic when minting zero shares?
    // I doubt it
    fn deposit(e: Env, assets: i128, receiver: Address, from: Address, operator: Address) -> i128 {
        operator.require_auth();
        // The asset provider must consent independently: an operator with an allowance over
        // *shares* has no claim on the provider's underlying balance. Guarded because
        // authorizing the same frame twice is rejected by the host
        if from != operator {
            from.require_auth();
        }

        let shares = Self::rate(&e)
            .unwrap_or_else(|err| panic_with_error!(&e, err))
            .assets_to_shares_floor(&e, assets);

        Self::enter(&e, assets, shares, &receiver, &from, &operator)
            .unwrap_or_else(|err| panic_with_error!(&e, err));

        shares
    }

    fn max_mint(e: Env, receiver: Address) -> i128 {
        if storage::get_deposits_paused(&e) {
            return 0;
        }

        let _ = receiver;
        i128::MAX
    }

    fn preview_mint(e: Env, shares: i128) -> i128 {
        Self::rate(&e)
            .unwrap_or_else(|err| panic_with_error!(&e, err))
            .shares_to_assets_ceil(&e, shares)
    }

    fn mint(e: Env, shares: i128, receiver: Address, from: Address, operator: Address) -> i128 {
        operator.require_auth();
        if from != operator {
            from.require_auth();
        }

        // Round the cost up, so minting an exact share amount can never be underpaid
        let assets = Self::rate(&e)
            .unwrap_or_else(|err| panic_with_error!(&e, err))
            .shares_to_assets_ceil(&e, shares);

        Self::enter(&e, assets, shares, &receiver, &from, &operator)
            .unwrap_or_else(|err| panic_with_error!(&e, err));

        assets
    }

    // Bounded by both the owner's shares and what the market can currently honor. The second
    // bound is what makes this differ from a plain vault: the underlying is lent out, so at high
    // utilization the vault's own claim may exceed the liquidity available to satisfy it
    fn max_withdraw(e: Env, owner: Address) -> i128 {
        let total_assets = Self::total_assets_internal(&e);

        let owner_assets = Self::rate(&e)
            .unwrap_or_else(|err| panic_with_error!(&e, err))
            .shares_to_assets_floor(&e, storage::get_balance(&e, &owner));

        // `total_assets` is already the market's own cap on what it would pay out right now
        i128::min(owner_assets, total_assets)
    }

    fn preview_withdraw(e: Env, assets: i128) -> i128 {
        Self::rate(&e)
            .unwrap_or_else(|err| panic_with_error!(&e, err))
            .assets_to_shares_ceil(&e, assets)
    }

    fn withdraw(
        e: Env,
        assets: i128,
        receiver: Address,
        owner: Address,
        operator: Address,
    ) -> i128 {
        operator.require_auth();

        // Round the share cost up: the owner always pays at least what they take out
        let shares = Self::rate(&e)
            .unwrap_or_else(|err| panic_with_error!(&e, err))
            .assets_to_shares_ceil(&e, assets);

        Self::exit(&e, assets, shares, &receiver, &owner, &operator)
            .unwrap_or_else(|err| panic_with_error!(&e, err));

        shares
    }

    fn max_redeem(e: Env, owner: Address) -> i128 {
        let balance = storage::get_balance(&e, &owner);

        let total_assets = Self::total_assets_internal(&e);

        // Cap the owner's balance by the shares the market's current liquidity can actually
        // cover, so redeeming `max_redeem` never fails on a shortfall
        let redeemable_by_liquidity = Self::rate(&e)
            .unwrap_or_else(|err| panic_with_error!(&e, err))
            .assets_to_shares_floor(&e, total_assets);

        i128::min(balance, redeemable_by_liquidity)
    }

    fn preview_redeem(e: Env, shares: i128) -> i128 {
        Self::convert_to_assets(e, shares)
    }

    fn redeem(e: Env, shares: i128, receiver: Address, owner: Address, operator: Address) -> i128 {
        operator.require_auth();

        // Round the payout down: a redeemer never receives more than their shares are worth
        let assets = Self::rate(&e)
            .unwrap_or_else(|err| panic_with_error!(&e, err))
            .shares_to_assets_floor(&e, shares);

        if assets <= 0 {
            panic_with_error!(&e, TDError::ZeroAssets);
        }

        Self::exit(&e, assets, shares, &receiver, &owner, &operator)
            .unwrap_or_else(|err| panic_with_error!(&e, err));

        assets
    }
}

#[contractimpl]
impl token::TokenInterface for TokenizedDepositsContract {
    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        storage::get_allowance(&e, &from, &spender).amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        if amount.is_negative() {
            panic_with_error!(&e, TDError::NegativeAmount);
        }
        // A non-zero allowance that has already expired can never be spent
        if amount > 0 && expiration_ledger < e.ledger().sequence() {
            panic_with_error!(&e, TDError::AllowanceExpired);
        }

        storage::set_allowance(&e, &from, &spender, &AllowanceValue { amount, expiration_ledger });
        storage::bump_instance(&e);

        events::approve(&e, from, spender, amount, expiration_ledger);
    }

    // The share balance. Fixed: it changes only on explicit transfers, deposits and withdrawals.
    // Yield shows up in `convert_to_assets(balance)`, not here
    fn balance(e: Env, id: Address) -> i128 {
        storage::get_balance(&e, &id)
    }

    fn transfer(e: Env, from: Address, to: MuxedAddress, amount: i128) {
        from.require_auth();

        if amount.is_negative() {
            panic_with_error!(&e, TDError::NegativeAmount);
        }

        let to = to.address();

        let from_balance = storage::get_balance(&e, &from);
        if from_balance < amount {
            panic_with_error!(&e, TDError::InsufficientBalance);
        }

        let to_balance = storage::get_balance(&e, &to)
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(&e, TDError::OverOrUnderflow));

        storage::set_balance(&e, &from, &(from_balance - amount)); // safe
        storage::set_balance(&e, &to, &to_balance);
        storage::bump_instance(&e);

        events::transfer(&e, from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        if amount.is_negative() {
            panic_with_error!(&e, TDError::NegativeAmount);
        }

        Self::spend_allowance(&e, &from, &spender, amount)
            .unwrap_or_else(|err| panic_with_error!(&e, err));

        let from_balance = storage::get_balance(&e, &from);
        if from_balance < amount {
            panic_with_error!(&e, TDError::InsufficientBalance);
        }

        let to_balance = storage::get_balance(&e, &to)
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(&e, TDError::OverOrUnderflow));

        storage::set_balance(&e, &from, &(from_balance - amount)); // safe
        storage::set_balance(&e, &to, &to_balance);
        storage::bump_instance(&e);

        events::transfer(&e, from, to, amount);
    }

    // NB: burning shares destroys the claim without redeeming it, donating the underlying to the
    // remaining holders. `redeem` is what users want; this exists because SEP-41 requires it
    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();

        if amount.is_negative() {
            panic_with_error!(&e, TDError::NegativeAmount);
        }

        Self::burn_shares(&e, &from, amount).unwrap_or_else(|err| panic_with_error!(&e, err));
        storage::bump_instance(&e);

        events::burn(&e, from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        if amount.is_negative() {
            panic_with_error!(&e, TDError::NegativeAmount);
        }

        Self::spend_allowance(&e, &from, &spender, amount)
            .unwrap_or_else(|err| panic_with_error!(&e, err));
        Self::burn_shares(&e, &from, amount).unwrap_or_else(|err| panic_with_error!(&e, err));
        storage::bump_instance(&e);

        events::burn(&e, from, amount);
    }

    fn decimals(e: Env) -> u32 {
        storage::get_metadata(&e).decimals
    }

    fn name(e: Env) -> String {
        storage::get_metadata(&e).name
    }

    fn symbol(e: Env) -> String {
        storage::get_metadata(&e).symbol
    }
}
