use soroban_sdk::{Address, BytesN, Env, Map, contractclient, contracttype, xdr::ToXdr};

use crate::error::TDError;

// A minimal, structurally-compatible mirror of the parts of the lending market the vault needs.
//
// Only the shapes matter for cross-contract calls, so mirroring them here keeps the vault
// decoupled from the market crate. The fields below must stay in sync with the market's
// definitions -- the integration tests exercise real calls against the real contract and will
// fail loudly if they drift

#[contracttype]
#[derive(Clone)]
pub struct ObligationKey {
    pub user: Address,
    pub seed: Option<BytesN<32>>,
}

#[contracttype]
#[derive(Clone)]
pub struct OperationFees {
    pub fee_sum: i128,
    pub referrer_fee: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct WithdrawResult {
    pub j_tokens_to_burn: i128,
    pub deposit_decrease: i128,
    pub withdrawer_to_receive: i128,
    pub operation_fees: OperationFees,
}

#[contracttype]
#[derive(Clone)]
pub struct DepositPosition {
    pub j_tokens: i128,
    pub collateral: i128,
    pub originally_deposited: i128,
    pub last_scarcity_withdraw_ts: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct BorrowPosition {
    pub d_tokens: i128,
    pub originally_borrowed: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct Obligation {
    pub deposits: Map<Address, DepositPosition>,
    pub borrows: Map<Address, BorrowPosition>,
    pub positions_count: u32,
    pub insurance_fund_requests_ids: Map<(Address, u64), u64>,
}

#[contractclient(name = "MarketClient")]
pub trait MarketInterface {
    fn deposit(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    );

    fn withdraw(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    );

    fn simulate_withdraw(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> WithdrawResult;

    fn borrow(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    );

    fn repay(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    );

    fn get_user_obligation(e: Env, user: ObligationKey) -> Obligation;

    fn refresh_pool(e: Env, pool_address: Address);
}

// Derives the sub-obligation key the vault uses on behalf of a single participant.
//
// Every participant gets their own obligation, owned by the vault but namespaced by a seed derived
// from the participant's address. This is what isolates liquidation risk: a liquidator acting on
// one participant's sub-obligation cannot touch any other participant's collateral
pub fn participant_key(e: &Env, participant: &Address) -> ObligationKey {
    let seed = e.crypto().sha256(&participant.clone().to_xdr(e)).to_bytes();

    ObligationKey { user: e.current_contract_address(), seed: Some(seed) }
}

// Reads a participant's raw `jToken` count -- the quantity the share token reports as `balance`.
//
// Read live from the market on every call rather than mirrored locally. That is what makes
// liquidations self-reporting: when a liquidator seizes `jTokens`, the participant's balance drops
// automatically, with no callback and no reconciliation step that could fall out of sync
pub fn participant_j_tokens(e: &Env, market: &Address, pool: &Address, participant: &Address) -> i128 {
    let client = MarketClient::new(e, market);

    match client.try_get_user_obligation(&participant_key(e, participant)) {
        Ok(Ok(obligation)) => {
            obligation.deposits.get(pool.clone()).map(|position| position.j_tokens).unwrap_or(0)
        }
        // No obligation yet: a participant who has never deposited holds nothing
        _ => 0,
    }
}

// How much a participant could actually withdraw right now, net of fees.
//
// The market caps withdrawals to whatever keeps the obligation healthy and to available pool
// liquidity, and it does so *silently* -- an oversized request succeeds for a smaller amount rather
// than reverting. Callers must therefore compare against this before withdrawing if they need
// all-or-nothing semantics
pub fn max_withdrawable(e: &Env, market: &Address, pool: &Address, participant: &Address) -> i128 {
    let client = MarketClient::new(e, market);
    client.refresh_pool(pool);

    match client.try_simulate_withdraw(&participant_key(e, participant), pool, &i128::MAX, &None) {
        Ok(Ok(result)) => result.withdrawer_to_receive,
        _ => 0,
    }
}

// The pool's current `jToken` exchange rate, expressed as the fraction `(assets, j_tokens)`.
//
// The market exposes its rate only through `Pool`, which is far too large to mirror safely here.
// Instead the rate is recovered from a full-withdrawal simulation against a position that is known
// to hold shares -- the ratio of gross assets to burnt shares *is* the rate, and it is a pool-wide
// property, so any funded position yields the same answer.
//
// `deposit_decrease` is the numerator rather than `withdrawer_to_receive` because the latter is net
// of withdrawal fees. Fees are a cost of exiting, not part of the share price
pub fn j_token_rate(
    e: &Env,
    market: &Address,
    pool: &Address,
    reference: &Address,
) -> Result<(i128, i128), TDError> {
    let client = MarketClient::new(e, market);
    // Interest accrues lazily, so the pool must be brought up to date before its rate is read
    client.refresh_pool(pool);

    match client.try_simulate_withdraw(&participant_key(e, reference), pool, &i128::MAX, &None) {
        Ok(Ok(result)) if result.j_tokens_to_burn > 0 && result.deposit_decrease > 0 => {
            Ok((result.deposit_decrease, result.j_tokens_to_burn))
        }
        // Unavailable when the reference holds nothing, or when a borrow has locked its collateral
        // so completely that no withdrawal is simulatable
        _ => Err(TDError::RateUnavailable),
    }
}

