use soroban_sdk::{Address, BytesN, Env, Map, contractclient, contracttype};

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

    fn get_user_obligation(e: Env, user: ObligationKey) -> Obligation;

    fn refresh_pool(e: Env, pool_address: Address);
}

// Returns the vault's own obligation key in the market. The vault deposits under its own
// contract address with no seed, so all vault-held liquidity sits in a single position
pub fn vault_obligation_key(e: &Env) -> ObligationKey {
    ObligationKey { user: e.current_contract_address(), seed: None }
}

// Reads how much the vault could withdraw from the market right now, net of fees.
//
// This is the authoritative definition of `total_assets`: it is what the market would actually
// pay the vault, which is the only figure the shares can safely be priced against
pub fn vault_withdrawable(e: &Env, market: &Address, pool: &Address) -> i128 {
    let client = MarketClient::new(e, market);
    client.refresh_pool(pool);

    let result = client.try_simulate_withdraw(
        &vault_obligation_key(e),
        pool,
        // `i128::MAX` is the market's documented sentinel for "everything available"
        &i128::MAX,
        &None,
    );

    match result {
        // The market caps the result to what is actually withdrawable given liquidity and LTV
        Ok(Ok(withdraw_result)) => withdraw_result.withdrawer_to_receive,
        // No position yet: a vault that has never deposited holds nothing
        _ => 0,
    }
}
