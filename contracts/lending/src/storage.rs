use soroban_sdk::{contracttype, Address, Env, Symbol};

use crate::{
    constants::{
        INDIVIDUAL_BUMP, INDIVIDUAL_THRESHOLD, INSTANCE_BUMP, INSTANCE_THRESHOLD, SHARED_BUMP,
        SHARED_THRESHOLD,
    },
    obligation::Obligation,
    pool::{MultiplyPair, Pool},
    LCError,
};

pub type PoolAddress = Address;
pub type UserAddress = Address;

#[contracttype]
pub struct GlobalState {
    pub admin: Address,
    pub status: bool,
    pub liquidation_threshold_bps: i128,
    // TODO: Oracle addresses?
}

#[contracttype]
pub enum DataKey {
    GlobalState,
    Pool(PoolAddress),
    Obligation(UserAddress),
    Accrual,
    AllPools,
    AllObligations,
    AllMultiplyPairs,
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

pub fn get_global_state(e: &Env) -> GlobalState {
    extend_instance_storage(e);

    e.storage()
        .instance()
        .get(&DataKey::GlobalState)
        .expect("Global State must be instantiated at this point")
}

pub fn set_global_state(e: &Env, global_state: &GlobalState) {
    e.storage()
        .instance()
        .set(&DataKey::GlobalState, global_state);

    extend_instance_storage(e);
}

// --- Pool ---

pub fn get_all_pools(e: &Env) -> soroban_sdk::Vec<PoolAddress> {
    let res = e.storage().persistent().get(&DataKey::AllPools);
    if let Some(pools) = res {
        extend_shared_storage(e, &DataKey::AllPools);
        pools
    } else {
        soroban_sdk::Vec::new(e)
    }
}

pub fn get_all_multiply_pairs(e: &Env) -> soroban_sdk::Vec<MultiplyPair> {
    let res = e.storage().persistent().get(&DataKey::AllMultiplyPairs);

    if let Some(pairs) = res {
        extend_shared_storage(e, &DataKey::AllMultiplyPairs);
        pairs
    } else {
        soroban_sdk::Vec::new(e)
    }
}

pub fn register_multiply_pair(e: &Env, pair: MultiplyPair) -> u32 {
    let mut all_pairs = get_all_multiply_pairs(e);
    all_pairs.push_back(pair);

    let new_index = all_pairs.len() - 1;

    e.storage()
        .persistent()
        .set(&DataKey::AllMultiplyPairs, &all_pairs);
    extend_shared_storage(e, &DataKey::AllMultiplyPairs);

    new_index
}

pub fn register_pool(e: &Env, pool_address: &Address) -> u32 {
    let mut all_pools = get_all_pools(e);
    all_pools.push_back(pool_address.clone());

    let new_index = all_pools.len() - 1;

    e.storage().persistent().set(&DataKey::AllPools, &all_pools);
    extend_shared_storage(e, &DataKey::AllPools);

    new_index
}

pub fn set_pool(e: &Env, pool_address: &Address, pool: &Pool) {
    e.storage()
        .persistent()
        .set(&DataKey::Pool(pool_address.clone()), pool);

    extend_shared_storage(e, &DataKey::Pool(pool_address.clone()));
}

pub fn pool_exists(e: &Env, pool_address: &Address) -> bool {
    let res = e
        .storage()
        .persistent()
        .has(&DataKey::Pool(pool_address.clone()));

    if res {
        extend_shared_storage(e, &DataKey::Pool(pool_address.clone()));
    }

    res
}

pub fn obligation_exists(e: &Env, user_address: &Address) -> bool {
    let res = e
        .storage()
        .persistent()
        .has(&DataKey::Obligation(user_address.clone()));

    if res {
        extend_shared_storage(e, &DataKey::Obligation(user_address.clone()));
    }

    res
}

pub fn get_pool(e: &Env, pool_address: &Address) -> Option<Pool> {
    let res = e
        .storage()
        .persistent()
        .get(&DataKey::Pool(pool_address.clone()));

    if res.is_some() {
        extend_shared_storage(e, &DataKey::Pool(pool_address.clone()));
    }

    res
}

pub fn get_pool_ticker(e: &Env, pool_address: &Address) -> Result<Symbol, LCError> {
    let pool = get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;

    Ok(pool.token_ticker)
}

pub fn set_pool_data(e: &Env, pool_address: &Address, pool_data: &Pool) {
    e.storage()
        .persistent()
        .set(&DataKey::Pool(pool_address.clone()), pool_data);

    extend_shared_storage(e, &DataKey::Pool(pool_address.clone()));
}

pub fn remove_pool(e: &Env, pool_address: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::Pool(pool_address.clone()));

    // TODO: remove address from `DataKey::AllPools`
}

pub fn remove_all_pools(e: &Env) {
    let all_pools: soroban_sdk::Vec<PoolAddress> = get_all_pools(e);

    for pool in all_pools.iter() {
        remove_pool(e, &pool);
    }

    if !all_pools.is_empty() {
        e.storage().persistent().remove(&DataKey::AllPools);
    }
}

pub fn remove_all_multiply_pairs(e: &Env) {
    let all_pairs: soroban_sdk::Vec<MultiplyPair> = get_all_multiply_pairs(e);

    if !all_pairs.is_empty() {
        e.storage().persistent().remove(&DataKey::AllMultiplyPairs);
    }
}

// --- Obligation ---
pub fn set_obligation(e: &Env, user: &Address, obligation: &Obligation) {
    e.storage()
        .persistent()
        .set(&DataKey::Obligation(user.clone()), obligation);

    extend_individual_storage(e, &DataKey::Obligation(user.clone()));
}

pub fn get_obligation(e: &Env, user: &Address) -> Option<Obligation> {
    let res = e
        .storage()
        .persistent()
        .get(&DataKey::Obligation(user.clone()));

    if res.is_some() {
        extend_individual_storage(e, &DataKey::Obligation(user.clone()));
    }

    res
}

pub fn register_obligation(e: &Env, user_address: &Address) -> u32 {
    let mut all_obligations = get_all_obligations(e);
    all_obligations.push_back(user_address.clone());
    let new_index = all_obligations.len() - 1;

    e.storage()
        .persistent()
        .set(&DataKey::AllObligations, &all_obligations);
    extend_shared_storage(e, &DataKey::AllObligations);

    new_index
}

pub fn remove_obligation(e: &Env, user: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::Obligation(user.clone()));
}

pub fn remove_all_obligations(e: &Env) {
    let all_obligations: soroban_sdk::Vec<Address> = get_all_obligations(e);

    for obligation in all_obligations.iter() {
        // TODO: This is an ad-hoc fix, and it's better to be rewritten well
        if obligation_exists(e, &obligation) {
            remove_obligation(e, &obligation);
        }
    }

    if !all_obligations.is_empty() {
        e.storage().persistent().remove(&DataKey::AllObligations);
    }
}

pub fn get_all_obligations(e: &Env) -> soroban_sdk::Vec<UserAddress> {
    let res = e.storage().persistent().get(&DataKey::AllObligations);
    if let Some(obligations) = res {
        extend_shared_storage(e, &DataKey::AllObligations);
        obligations
    } else {
        soroban_sdk::Vec::new(e)
    }
}
