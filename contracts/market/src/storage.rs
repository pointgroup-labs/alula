use soroban_sdk::{Address, Env, String, Vec, contracttype};

use crate::{
    constants::*,
    multiply_pair::MultiplyPair,
    obligation::{Obligation, ObligationKey},
    pool::Pool,
};

#[contracttype]
pub struct GlobalState {
    pub status: bool,
    pub admin: Address,
    pub name: String,
}

#[contracttype]
pub enum DataKey {
    GlobalState,
    Pool(Address),
    Obligation(ObligationKey), // NB: What's better Bytes or BytesN here?
    MultiplyPair((Address, Address)), // (deposit_pool_address, borrow_pool_address)
    Accrual,
    AllPools,
    AllObligations,
    AllMultiplyPairs,
    OracleAddress,
}

/// Instance bumper
pub fn extend_instance_storage(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

/// Persistent individual resource bumper
pub fn extend_individual_storage(e: &Env, key: &DataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, INDIVIDUAL_THRESHOLD, INDIVIDUAL_BUMP);
}

/// Persistent shared resource bumper
pub fn extend_shared_storage(e: &Env, key: &DataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, SHARED_THRESHOLD, SHARED_BUMP);
}

/// Gets the oracle address of the contract
pub fn get_oracle_address(e: &Env) -> Address {
    extend_instance_storage(e);
    e.storage()
        .instance()
        .get(&DataKey::OracleAddress)
        .expect("Oracle address must be instantiated at this point")
}

/// Sets the oracle address of the contract
pub fn set_oracle_address(e: &Env, address: &Address) {
    e.storage().instance().set(&DataKey::OracleAddress, address);
    extend_instance_storage(e);
}

/// Gets the global state of the contract
pub fn get_global_state(e: &Env) -> GlobalState {
    extend_instance_storage(e);
    e.storage()
        .instance()
        .get(&DataKey::GlobalState)
        .expect("Global State must be instantiated at this point")
}

/// Sets the global state of the contract
pub fn set_global_state(e: &Env, global_state: &GlobalState) {
    e.storage()
        .instance()
        .set(&DataKey::GlobalState, global_state);
    extend_instance_storage(e);
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
    pools.len() + 1
}

/// Sets a pool by its address
/// NB: Overwrites existing pool if it exists
pub fn set_pool(e: &Env, pool_address: &Address, pool: &Pool) {
    let key = DataKey::Pool(pool_address.clone());
    e.storage().persistent().set(&key, pool);
    extend_shared_storage(e, &key); // TODO: Should we do this, though?
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
    e.storage()
        .persistent()
        .set(&DataKey::AllMultiplyPairs, &pairs);
    extend_shared_storage(e, &DataKey::AllMultiplyPairs);
    pairs.len() + 1
}

/// Sets a multiply pair by its key (deposit and borrow pool addresses)
pub fn set_multiply_pair(
    e: &Env,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    pair: &MultiplyPair,
) {
    let key = DataKey::MultiplyPair((deposit_pool_address.clone(), borrow_pool_address.clone()));
    e.storage()
        .persistent()
        // NB: Should we allow multiple pairs with the same pools?
        .set(&key, pair);
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
    if e.storage().persistent().has(&key) {
        extend_shared_storage(e, &key);
    }
    res
}

/// Registers a new obligation key in the contract storage and returns its index
pub fn register_obligation(e: &Env, obligation_key: &ObligationKey) -> u32 {
    let storage = e.storage().persistent();
    let mut obligations = get_all_obligations(e);
    obligations.push_back(obligation_key.clone());
    storage.set(&DataKey::AllObligations, &obligations);
    extend_shared_storage(e, &DataKey::AllObligations);
    obligations.len() + 1
}

/// Gets all obligation keys stored in the contract
pub fn get_all_obligations(e: &Env) -> Vec<ObligationKey> {
    let storage = e.storage().persistent();
    if let Some(obligations) = storage.get(&DataKey::AllObligations) {
        extend_shared_storage(e, &DataKey::AllObligations);
        obligations
    } else {
        Vec::new(e)
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
    if let Some(idx) = obligations.last_index_of(obligation_key) {
        obligations.remove(idx);
        storage.set(&DataKey::AllObligations, &obligations);
    }
}

/// Removes all obligations from the contract storage
/// Also clears the list of all obligations
pub fn remove_all_obligations(e: &Env) {
    let storage = e.storage().persistent();
    for key in get_all_obligations(e) {
        storage.remove(&DataKey::Obligation(key));
    }
    storage.remove(&DataKey::AllObligations);
}
