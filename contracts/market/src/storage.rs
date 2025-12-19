use soroban_sdk::{Address, BytesN, Env, Map, String, Vec, contracttype};

use crate::{
    constants::*,
    error::MCError,
    multiply_pair::MultiplyPair,
    obligation::{Obligation, ObligationKey},
    pool::{Pool, PoolConfig},
};

#[contracttype]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct GlobalState {
    pub status: u32,
    pub name: String,
    pub is_owned: bool,
    pub admin: Address,
    pub oracle: Address,
    pub deployer: Address,
    pub max_positions: u32,
    pub insolvency_ltv_bps: i128,
    pub min_collateral_value_cents: i128,
    pub update_in_queue_period: Option<u64>,
}

#[contracttype]
pub enum MarketStatus {
    /// All operations are allowed
    Active,
    /// Borrow operations are prohibited
    BorrowFrozen,
    /// Borrowing and depositing operations on the market are prohibited
    DepositFrozen,
    /// All operations on the market are prohibited
    Frozen,
}

impl TryFrom<u32> for MarketStatus {
    type Error = MCError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let market_status = match value {
            0 => MarketStatus::Active,
            1 => MarketStatus::BorrowFrozen,
            2 => MarketStatus::DepositFrozen,
            3 => MarketStatus::Frozen,
            _ => return Err(MCError::InvalidMarketStatusUpdate),
        };

        Ok(market_status)
    }
}

#[contracttype]
#[derive(Debug, Eq, PartialEq)]
pub struct PoolUpdate {
    pub new_config: PoolConfig,
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
    Pool(Address),
    InsuranceFund,
    AllObligations,
    InsolvencyLtvBps,
    AllMultiplyPairs,
    EarnObligationSeed,
    MinCollateralValueCents,
    UpdateInQueuePeriod,
    ConfigUpdate(Address),
    Obligation(ObligationKey),
    MultiplyPair((Address, Address)),
}

// -- TTL Bumpers --

/// Instance bumper
pub fn extend_instance_storage(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

/// Persistent individual resource bumper
pub fn extend_individual_storage(e: &Env, key: &DataKey) {
    e.storage().persistent().extend_ttl(key, INDIVIDUAL_THRESHOLD, INDIVIDUAL_BUMP);
}

/// Persistent shared resource bumper
pub fn extend_shared_storage(e: &Env, key: &DataKey) {
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
pub fn set_update_in_queue_period(e: &Env, update_in_queue_period: Option<u64>) {
    e.storage().instance().set(&DataKey::UpdateInQueuePeriod, &update_in_queue_period)
}
pub fn get_update_in_queue_period(e: &Env) -> Option<u64> {
    e.storage()
        .instance()
        .get(&DataKey::UpdateInQueuePeriod)
        .expect("UpdateInQueuePeriod must be set")
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
// ---- Pool ----

/// Gets all pools stored in the contract
pub fn get_all_pools(e: &Env) -> Vec<Address> {
    let res = e.storage().persistent().get(&DataKey::AllPools);
    if let Some(pools) = res {
        extend_shared_storage(e, &DataKey::AllPools);
        pools
    } else {
        Vec::new(e)
    }
}

/// Registers a new pool in the contract storage and returns its index
/// NB: Does not check for existing pools, use `pool_exists` before calling this
/// if you want to avoid duplicates
pub fn register_pool(e: &Env, pool_address: &Address) -> u32 {
    let mut pools = get_all_pools(e);
    pools.push_back(pool_address.clone());
    e.storage().persistent().set(&DataKey::AllPools, &pools);
    extend_shared_storage(e, &DataKey::AllPools);
    pools.len() - 1
}

/// Sets a pool by its address
/// NB: Overwrites existing pool if it exists
pub fn set_pool(e: &Env, pool_address: &Address, pool: &Pool) {
    let key = DataKey::Pool(pool_address.clone());

    e.storage().persistent().set(&key, pool);
    extend_shared_storage(e, &key);
}

/// Checks whether a pool with the given address exists
pub fn pool_exists(e: &Env, pool_address: &Address) -> bool {
    let key = DataKey::Pool(pool_address.clone());
    let res = e.storage().persistent().has(&key);
    if res {
        extend_shared_storage(e, &key);
    }
    res
}

/// Gets a pool by its address
pub fn get_pool(e: &Env, pool_address: &Address) -> Option<Pool> {
    let key = DataKey::Pool(pool_address.clone());
    let res = e.storage().persistent().get(&key);
    if res.is_some() {
        extend_shared_storage(e, &key);
    }
    res
}

/// Queues in pool's config update
pub fn queue_in_pool_config_update(
    e: &Env,
    pool_address: &Address,
    config: &PoolConfig,
) -> Result<(), MCError> {
    let key = DataKey::ConfigUpdate(pool_address.clone());
    if e.storage().persistent().has(&key) {
        return Err(MCError::PoolAlreadyContainsQueuedInConfigUpdate);
    }

    let pool_update =
        PoolUpdate { new_config: config.clone(), queued_in_timestamp: e.ledger().timestamp() };
    e.storage().persistent().set(&key, &pool_update);

    Ok(())
}

/// Removes pool's config update from the queue
pub fn remove_pool_config_update(e: &Env, pool_address: &Address) -> Result<(), MCError> {
    let key = DataKey::ConfigUpdate(pool_address.clone());

    if !e.storage().persistent().has(&key) {
        return Err(MCError::PoolDoesNotHaveQueuedInConfigUpdate);
    }
    e.storage().persistent().remove(&key);

    Ok(())
}

/// Gets pool's config update from the storage
pub fn get_pool_config_update(e: &Env, pool_address: &Address) -> Option<PoolUpdate> {
    let config_update = e.storage().persistent().get(&DataKey::ConfigUpdate(pool_address.clone()));
    if config_update.is_some() {
        extend_shared_storage(e, &DataKey::ConfigUpdate(pool_address.clone()));
    }

    config_update
}

// ---- Multiply Pair ----

/// Gets all multiply pairs stored in the contract
pub fn get_all_multiply_pairs(e: &Env) -> Vec<MultiplyPair> {
    let storage = e.storage().persistent();
    if let Some(pairs) = storage.get(&DataKey::AllMultiplyPairs) {
        extend_shared_storage(e, &DataKey::AllMultiplyPairs);
        pairs
    } else {
        Vec::new(e)
    }
}

/// Registers a new multiply pair in the contract storage and returns its index
/// NB: Does not check for existing pairs, use `multiply_pair_exists` before calling this
/// if you want to avoid duplicates
pub fn register_multiply_pair(e: &Env, pair: MultiplyPair) -> u32 {
    let mut pairs = get_all_multiply_pairs(e);
    pairs.push_back(pair);
    e.storage().persistent().set(&DataKey::AllMultiplyPairs, &pairs);
    extend_shared_storage(e, &DataKey::AllMultiplyPairs);
    pairs.len() - 1
}

/// Sets a multiply pair by its key (deposit and borrow pool addresses)
pub fn set_multiply_pair(
    e: &Env,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    pair: &MultiplyPair,
) {
    let key = DataKey::MultiplyPair((deposit_pool_address.clone(), borrow_pool_address.clone()));
    e.storage().persistent().set(&key, pair);
    extend_shared_storage(e, &key);
}

/// Checks whether a multiply pair with the given deposit and borrow pool addresses exists
pub fn multiply_pair_exists(
    e: &Env,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> bool {
    let key = DataKey::MultiplyPair((deposit_pool_address.clone(), borrow_pool_address.clone()));
    let res = e.storage().persistent().has(&key);
    if res {
        extend_shared_storage(e, &key);
    }

    res
}

/// Gets a multiply pair by its key (deposit and borrow pool addresses) if it exists
pub fn get_multiply_pair(
    e: &Env,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> Option<MultiplyPair> {
    let key = DataKey::MultiplyPair((deposit_pool_address.clone(), borrow_pool_address.clone()));
    let res = e.storage().persistent().get(&key);
    if res.is_some() {
        extend_shared_storage(e, &key);
    }
    res
}

// ---- Obligation ----

/// Sets an obligation by its key
pub fn set_obligation(e: &Env, obligation_key: &ObligationKey, obligation: &Obligation) {
    let key = DataKey::Obligation(obligation_key.clone());
    e.storage().persistent().set(&key, obligation);
    extend_individual_storage(e, &key);
}

/// Gets an obligation by its key, if it exists
pub fn get_obligation(e: &Env, obligation_key: &ObligationKey) -> Option<Obligation> {
    let key = DataKey::Obligation(obligation_key.clone());
    let res = e.storage().persistent().get(&key);
    if res.is_some() {
        extend_individual_storage(e, &key);
    }
    res
}

/// # Returns
/// `true` if an obligation with the given key exists,
/// `false` otherwise
pub fn obligation_exists(e: &Env, obligation_key: &ObligationKey) -> bool {
    let key = DataKey::Obligation(obligation_key.clone());
    let res = e.storage().persistent().has(&key);
    if res {
        extend_individual_storage(e, &key);
    }

    res
}

/// Registers a new obligation key in the contract storage
pub fn register_obligation(e: &Env, obligation_key: &ObligationKey) {
    let storage = e.storage().persistent();
    let mut obligations = get_all_obligations(e);
    obligations.set(obligation_key.clone(), ());
    storage.set(&DataKey::AllObligations, &obligations);
    extend_shared_storage(e, &DataKey::AllObligations);
}

/// Gets all obligation keys stored in the contract
pub fn get_all_obligations(e: &Env) -> Map<ObligationKey, ()> {
    let storage = e.storage().persistent();
    if let Some(obligations) = storage.get(&DataKey::AllObligations) {
        extend_shared_storage(e, &DataKey::AllObligations);
        obligations
    } else {
        Map::new(e)
    }
}

// ---- State Removal(useful only for state resetting on testnet) ----

/// Removes a pool from the contract storage by its address
/// Also removes the pool from the list of all pools
pub fn remove_pool(e: &Env, pool_address: &Address) {
    let storage = e.storage().persistent();
    storage.remove(&DataKey::Pool(pool_address.clone()));
    let mut pools = get_all_pools(e);
    if let Some(idx) = pools.last_index_of(pool_address) {
        pools.remove(idx);
        storage.set(&DataKey::AllPools, &pools);
    }
}

/// Removes all pools from the contract storage
/// Also clears the list of all pools
pub fn remove_all_pools(e: &Env) {
    let storage = e.storage().persistent();
    for pool in get_all_pools(e) {
        storage.remove(&DataKey::Pool(pool));
    }
    storage.remove(&DataKey::AllPools);
}

/// Removes a multiply pair from the contract storage by its key
/// Also removes the multiply pair from the list of all multiply pairs
pub fn remove_multiply_pair(e: &Env, pair: &MultiplyPair) {
    let storage = e.storage().persistent();
    storage.remove(&DataKey::MultiplyPair(pair.key()));
    let mut pairs = get_all_multiply_pairs(e);
    if let Some(idx) = pairs.last_index_of(pair) {
        pairs.remove(idx);
        storage.set(&DataKey::AllMultiplyPairs, &pairs);
    }
}

/// Removes all multiply pairs from the contract storage
pub fn remove_all_multiply_pairs(e: &Env) {
    let storage = e.storage().persistent();
    for pair in get_all_multiply_pairs(e) {
        storage.remove(&DataKey::MultiplyPair(pair.key()));
    }
    storage.remove(&DataKey::AllMultiplyPairs);
}

/// Removes an obligation from the contract storage by its key
/// Also removes the obligation key from the list of all obligations
pub fn remove_obligation(e: &Env, obligation_key: &ObligationKey) {
    let storage = e.storage().persistent();
    storage.remove(&DataKey::Obligation(obligation_key.clone()));
    let mut obligations = get_all_obligations(e);

    obligations.remove(obligation_key.clone());
    storage.set(&DataKey::AllObligations, &obligations);
}

/// Removes all obligations from the contract storage
/// Also clears the list of all obligations
pub fn remove_all_obligations(e: &Env) {
    let storage = e.storage().persistent();
    for (key, _) in get_all_obligations(e) {
        storage.remove(&DataKey::Obligation(key));
    }
    storage.remove(&DataKey::AllObligations);
}
