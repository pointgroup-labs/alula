use soroban_sdk::{Address, BytesN, Env, Map, String, Vec, contracttype};

use crate::{
    constants::*,
    error::MCError,
    obligation::{Obligation, ObligationKey},
    pool::{Pool, PoolConfig},
};

#[contracttype]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MarketInitParams {
    pub max_positions: u32,
    pub min_collateral_value_cents: i128,
    pub insolvency_ltv_bps: i128,
    pub update_in_queue_period: u64,
    pub is_owned: bool,
}

#[contracttype]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct GlobalState {
    pub status: u32,
    pub name: String,
    pub is_owned: bool,
    pub admin: Address,
    pub oracle: Address,
    pub insurance_fund: Address,
    pub deployer: Address,
    pub max_positions: u32,
    pub insolvency_ltv_bps: i128,
    pub min_collateral_value_cents: i128,
    pub update_in_queue_period: u64,
}

#[contracttype]
pub enum MarketStatus {
    // All operations are allowed
    Active,
    // Borrow operations are prohibited
    BorrowFrozen,
    // Borrow operations are prohibited and IF cannot over-write
    BorrowFrozenByAdmin,
    // Borrowing and depositing operations on the market are prohibited
    DepositFrozen,
    // Borrowing and depositing operations on the market are prohibited and IF cannot over-write
    DepositFrozenByAdmin,
    // All operations on the market are prohibited
    Frozen,
    // All operations on the market are prohibited and IF cannot over-write
    FrozenByAdmin,
}

impl MarketStatus {
    // Returns [`true`] if status is a "hard lock" that only Market Admin can move from
    pub fn is_admin_protected(&self) -> bool {
        matches!(
            self,
            MarketStatus::BorrowFrozenByAdmin
                | MarketStatus::DepositFrozenByAdmin
                | MarketStatus::FrozenByAdmin
        )
    }
}

impl TryFrom<u32> for MarketStatus {
    type Error = MCError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let market_status = match value {
            0 => MarketStatus::Active,
            1 => MarketStatus::BorrowFrozen,
            2 => MarketStatus::BorrowFrozenByAdmin,
            3 => MarketStatus::DepositFrozen,
            4 => MarketStatus::DepositFrozenByAdmin,
            5 => MarketStatus::Frozen,
            6 => MarketStatus::FrozenByAdmin,
            _ => return Err(MCError::InvalidMarketConfigOrUpdate),
        };

        Ok(market_status)
    }
}

#[contracttype]
#[derive(Debug, Eq, PartialEq)]
pub struct QueuedPoolSet {
    pub new_config: PoolConfig,
    pub queued_in_timestamp: u64,
}

#[contracttype]
#[derive(Debug, Eq, PartialEq)]
pub struct MarketUpdate {
    pub new_max_positions: u32,
    pub new_min_collateral_value_cents: i128,
    pub queued_in_timestamp: u64,
}

#[contracttype]
pub enum DataKey {
    Name,
    Admin,
    Oracle,
    Accrual,
    IsOwned,
    AllPools,
    GlobalState,
    DeployerHost,
    MaxPositions,
    MarketStatus,
    FarmsContract,
    QueuedPoolSet(Address),
    Pool(Address),
    InsuranceFund,
    AllObligations,
    InsolvencyLtvBps,
    EarnObligationSeed,
    MinCollateralValueCents,
    UpdateInQueuePeriod,
    MarketConfigUpdate,
    Obligation(ObligationKey),
    ProposedAdmin,
}

// -- TTL Bumpers --

// Instance bumper
pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

// Persistent individual resource bumper
pub fn extend_individual(e: &Env, key: &DataKey) {
    e.storage().persistent().extend_ttl(key, INDIVIDUAL_THRESHOLD, INDIVIDUAL_BUMP);
}

// Persistent shared resource bumper
pub fn extend_shared(e: &Env, key: &DataKey) {
    e.storage().persistent().extend_ttl(key, SHARED_THRESHOLD, SHARED_BUMP);
}

// -- Storage getters & setters --

// - Oracle address -
pub fn set_oracle(e: &Env, oracle: &Address) {
    e.storage().instance().set(&DataKey::Oracle, oracle);
}
pub fn get_oracle(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Oracle).expect("Oracle must be set")
}

// - InsuranceFund -
pub fn set_insurance_fund(e: &Env, insurance_fund: &Address) {
    e.storage().instance().set(&DataKey::InsuranceFund, &insurance_fund)
}
pub fn get_insurance_fund(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::InsuranceFund).expect("InsuranceFund must be set")
}

// - UpdateInQueuePeriod -
pub fn set_update_in_queue_period(e: &Env, update_in_queue_period: u64) {
    e.storage().instance().set(&DataKey::UpdateInQueuePeriod, &update_in_queue_period)
}
pub fn get_update_in_queue_period(e: &Env) -> u64 {
    e.storage()
        .instance()
        .get(&DataKey::UpdateInQueuePeriod)
        .expect("UpdateInQueuePeriod must be set")
}

// - IsOwned -
pub fn set_is_owned(e: &Env, is_owned: bool) {
    e.storage().instance().set(&DataKey::IsOwned, &is_owned)
}
pub fn get_is_owned(e: &Env) -> bool {
    e.storage().instance().get(&DataKey::IsOwned).expect("IsOwned must be set")
}
// - MaxPositions -
pub fn set_max_positions(e: &Env, max_positions: u32) {
    e.storage().instance().set(&DataKey::MaxPositions, &max_positions);
}
pub fn get_max_positions(e: &Env) -> u32 {
    e.storage().instance().get(&DataKey::MaxPositions).expect("MaxPositions must be set")
}

// - MinCollateralValueCents -
pub fn set_min_collateral_value_cents(e: &Env, min_collateral_value_cents: i128) {
    e.storage().instance().set(&DataKey::MinCollateralValueCents, &min_collateral_value_cents);
}
pub fn get_min_collateral_value_cents(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::MinCollateralValueCents)
        .expect("MinCollateralValueCents must be set")
}

// - InsolvencyLtvBps -
pub fn set_insolvency_ltv_bps(e: &Env, insolvency_ltv_bps: i128) {
    e.storage().instance().set(&DataKey::InsolvencyLtvBps, &insolvency_ltv_bps);
}
pub fn get_insolvency_ltv_bps(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::InsolvencyLtvBps).expect("InsolvencyLtvBps must be set")
}

// - MarketStatus -
pub fn set_market_status(e: &Env, market_status: &MarketStatus) {
    e.storage().instance().set(&DataKey::MarketStatus, &market_status)
}
pub fn get_market_status(e: &Env) -> MarketStatus {
    e.storage().instance().get(&DataKey::MarketStatus).expect("MarketStatus must be set")
}

// - Admin -
pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, &admin)
}
pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).expect("Admin must be set")
}

// - Name -
pub fn set_name(e: &Env, name: &String) {
    e.storage().instance().set(&DataKey::Name, &name)
}
pub fn get_name(e: &Env) -> String {
    e.storage().instance().get(&DataKey::Name).expect("Name must be set")
}

// - Deployer -
pub fn set_deployer(e: &Env, address: &Address) {
    e.storage().instance().set(&DataKey::DeployerHost, &address)
}
pub fn get_deployer(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::DeployerHost).expect("Deployer must be set")
}

// - EarnObligationSeed -
pub fn set_earn_obligation_seed(e: &Env, seed: &BytesN<32>) {
    e.storage().instance().set(&DataKey::EarnObligationSeed, &seed)
}
pub fn get_earn_obligation_seed(e: &Env) -> Option<BytesN<32>> {
    e.storage().instance().get(&DataKey::EarnObligationSeed)
}

// - ProposedAdmin -
pub fn set_proposed_admin(e: &Env, proposed_admin: &Address) {
    e.storage().instance().set(&DataKey::ProposedAdmin, &proposed_admin);
}
pub fn get_proposed_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::ProposedAdmin)
}
pub fn remove_proposed_admin(e: &Env) {
    e.storage().instance().remove(&DataKey::ProposedAdmin);
}

// - FarmsContract -
pub fn set_farms_contract(e: &Env, farms: &Address) {
    e.storage().instance().set(&DataKey::FarmsContract, farms)
}
pub fn get_farms_contract(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::FarmsContract)
}
pub fn clear_farms_contract(e: &Env) {
    e.storage().instance().remove(&DataKey::FarmsContract)
}

// ---- Pool ----

// Gets all pools stored in the contract
pub fn get_all_pools(e: &Env) -> Vec<Address> {
    let res = e.storage().persistent().get(&DataKey::AllPools);
    if let Some(pools) = res {
        extend_shared(e, &DataKey::AllPools);
        pools
    } else {
        Vec::new(e)
    }
}

// Registers a new pool in the contract storage and returns its index
// NB: Does not check for existing pools, use `pool_exists` before calling this
// if you want to avoid duplicates
pub fn register_pool(e: &Env, pool_address: &Address) -> u32 {
    let mut pools = get_all_pools(e);
    pools.push_back(pool_address.clone());
    e.storage().persistent().set(&DataKey::AllPools, &pools);
    extend_shared(e, &DataKey::AllPools);
    pools.len() - 1
}

// Sets a pool by its address
// NB: Overwrites existing pool if it exists
pub fn set_pool(e: &Env, pool_address: &Address, pool: &Pool) {
    let key = DataKey::Pool(pool_address.clone());

    e.storage().persistent().set(&key, pool);
    extend_shared(e, &key);
}

// Checks whether a pool with the given address exists
pub fn pool_exists(e: &Env, pool_address: &Address) -> bool {
    let key = DataKey::Pool(pool_address.clone());
    let res = e.storage().persistent().has(&key);
    if res {
        extend_shared(e, &key);
    }
    res
}

// Gets a pool by its address
pub fn get_pool(e: &Env, pool_address: &Address) -> Option<Pool> {
    let key = DataKey::Pool(pool_address.clone());
    let res = e.storage().persistent().get(&key);
    if res.is_some() {
        extend_shared(e, &key);
    }
    res
}

// Queues in a pool set
pub fn queue_in_pool_set(
    e: &Env,
    pool_address: &Address,
    config: &PoolConfig,
) -> Result<(), MCError> {
    let key = DataKey::QueuedPoolSet(pool_address.clone());
    if e.storage().persistent().has(&key) {
        return Err(MCError::PoolAlreadyContainsQueuedPoolSet);
    }

    let queued_pool_set =
        QueuedPoolSet { new_config: config.clone(), queued_in_timestamp: e.ledger().timestamp() };
    e.storage().persistent().set(&key, &queued_pool_set);

    Ok(())
}

// Removes a queued pool set from the queue
pub fn remove_queued_pool_set(e: &Env, pool_address: &Address) -> Result<(), MCError> {
    let key = DataKey::QueuedPoolSet(pool_address.clone());

    if !e.storage().persistent().has(&key) {
        return Err(MCError::PoolDoesNotHaveQueuedPoolSet);
    }
    e.storage().persistent().remove(&key);

    Ok(())
}

// Gets a queued pool set from the storage
pub fn get_queued_pool_set(e: &Env, pool_address: &Address) -> Option<QueuedPoolSet> {
    let config_update = e.storage().persistent().get(&DataKey::QueuedPoolSet(pool_address.clone()));
    if config_update.is_some() {
        extend_shared(e, &DataKey::QueuedPoolSet(pool_address.clone()));
    }

    config_update
}

// Queues in a market config update
pub fn queue_in_market_config_update(
    e: &Env,
    new_max_positions: u32,
    new_min_collateral_value_cents: i128,
) -> Result<(), MCError> {
    let key = DataKey::MarketConfigUpdate;
    if e.storage().instance().has(&key) {
        return Err(MCError::MarketAlreadyContainsQueuedInConfigUpdate);
    }

    let market_update = MarketUpdate {
        new_max_positions,
        new_min_collateral_value_cents,
        queued_in_timestamp: e.ledger().timestamp(),
    };
    e.storage().instance().set(&key, &market_update);

    Ok(())
}

// Removes market config update from the queue
pub fn remove_market_config_update(e: &Env) -> Result<(), MCError> {
    let key = DataKey::MarketConfigUpdate;

    if !e.storage().instance().has(&key) {
        return Err(MCError::MarketDoesNotHaveQueuedInConfigUpdate);
    }
    e.storage().instance().remove(&key);

    Ok(())
}

// Gets market config update from storage
pub fn get_market_config_update(e: &Env) -> Option<MarketUpdate> {
    e.storage().instance().get(&DataKey::MarketConfigUpdate)
}

// ---- Obligation ----

// Sets an obligation by its key
pub fn set_obligation(e: &Env, obligation_key: &ObligationKey, obligation: &Obligation) {
    let key = DataKey::Obligation(obligation_key.clone());
    e.storage().persistent().set(&key, obligation);
    extend_individual(e, &key);
}

// Gets an obligation by its key, if it exists
pub fn get_obligation(e: &Env, obligation_key: &ObligationKey) -> Option<Obligation> {
    let key = DataKey::Obligation(obligation_key.clone());
    let res = e.storage().persistent().get(&key);
    if res.is_some() {
        extend_individual(e, &key);
    }
    res
}

// Registers a new obligation key in the contract storage
pub fn register_obligation(e: &Env, obligation_key: &ObligationKey) {
    let storage = e.storage().persistent();
    let mut obligations = get_all_obligations(e);
    obligations.set(obligation_key.clone(), ());
    storage.set(&DataKey::AllObligations, &obligations);
    extend_shared(e, &DataKey::AllObligations);
}

// Gets all obligation keys stored in the contract
pub fn get_all_obligations(e: &Env) -> Map<ObligationKey, ()> {
    let storage = e.storage().persistent();
    if let Some(obligations) = storage.get(&DataKey::AllObligations) {
        extend_shared(e, &DataKey::AllObligations);
        obligations
    } else {
        Map::new(e)
    }
}

// Removes an obligation from the contract storage by its key
// Also removes the obligation key from the list of all obligations
pub fn remove_obligation(e: &Env, obligation_key: &ObligationKey) {
    let storage = e.storage().persistent();
    storage.remove(&DataKey::Obligation(obligation_key.clone()));
    let mut obligations = get_all_obligations(e);

    obligations.remove(obligation_key.clone());
    storage.set(&DataKey::AllObligations, &obligations);
}
