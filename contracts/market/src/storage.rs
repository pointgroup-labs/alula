use soroban_sdk::{Address, BytesN, Env, String, Vec, contracttype};

use crate::{
    constants::{
        INDIVIDUAL_BUMP, INDIVIDUAL_THRESHOLD, INSTANCE_BUMP, INSTANCE_THRESHOLD, SHARED_BUMP,
        SHARED_THRESHOLD,
    },
    multiply_pair::MultiplyPair,
    obligation::Obligation,
    pool::Pool,
};

#[contracttype]
pub struct GlobalState {
    pub status: bool,
    pub admin: Address,
    pub name: String,
    // TODO: Oracle addresses?
}

#[contracttype]
pub enum DataKey {
    GlobalState,
    Pool(Address),
    Obligation((Address, Option<BytesN<32>>)), // NB: What's better Bytes or BytesN here?
    MultiplyPair((Address, Address)),          // (deposit, borrow)
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

pub fn get_oracle_address(e: &Env) -> Address {
    extend_instance_storage(e);

    let key = DataKey::OracleAddress;

    e.storage()
        .instance()
        .get(&key)
        .expect("Oracle address must be instantiated at this point")
}

pub fn set_oracle_address(e: &Env, address: &Address) {
    let key = DataKey::OracleAddress;

    e.storage().instance().set(&key, address);

    extend_instance_storage(e);
}

pub fn get_global_state(e: &Env) -> GlobalState {
    extend_instance_storage(e);

    let key = DataKey::GlobalState;

    e.storage()
        .instance()
        .get(&key)
        .expect("Global State must be instantiated at this point")
}

pub fn set_global_state(e: &Env, global_state: &GlobalState) {
    let key = DataKey::GlobalState;

    e.storage().instance().set(&key, global_state);

    extend_instance_storage(e);
}

// --- Pool ---
pub fn get_all_pools(e: &Env) -> Vec<Address> {
    let res = e.storage().persistent().get(&DataKey::AllPools);

    if let Some(pools) = res {
        extend_shared_storage(e, &DataKey::AllPools);

        pools
    } else {
        Vec::new(e)
    }
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
    let key = DataKey::Pool(pool_address.clone());

    e.storage().persistent().set(&key, pool);

    extend_shared_storage(e, &key);
}

pub fn pool_exists(e: &Env, pool_address: &Address) -> bool {
    let key = DataKey::Pool(pool_address.clone());

    let res = e.storage().persistent().has(&key);

    if res {
        extend_shared_storage(e, &key);
    }

    res
}

pub fn get_pool(e: &Env, pool_address: &Address) -> Option<Pool> {
    let key = DataKey::Pool(pool_address.clone());

    let res = e.storage().persistent().get(&key);

    if res.is_some() {
        extend_shared_storage(e, &key);
    }

    res
}

pub fn remove_pool(e: &Env, pool_address: &Address) {
    let key = DataKey::Pool(pool_address.clone());

    e.storage().persistent().remove(&key);

    let pools = get_all_pools(e);
    let mut new_pools = Vec::new(e);

    // TODO: This doesn't scale well. Start using [`soroban_sdk::Map`]
    for pool_addr in pools.iter() {
        if pool_addr != *pool_address {
            new_pools.push_back(pool_addr);
        }
    }

    e.storage().persistent().set(&DataKey::AllPools, &new_pools);
}

pub fn remove_all_pools(e: &Env) {
    let all_pools: Vec<Address> = get_all_pools(e);

    for pool in all_pools.iter() {
        remove_pool(e, &pool);
    }
}

// --- Multiply Pair ---
pub fn get_all_multiply_pairs(e: &Env) -> Vec<MultiplyPair> {
    let res = e.storage().persistent().get(&DataKey::AllMultiplyPairs);

    if let Some(pairs) = res {
        extend_shared_storage(e, &DataKey::AllMultiplyPairs);

        pairs
    } else {
        Vec::new(e)
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

pub fn multiply_pair_exists(
    e: &Env,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> bool {
    let key = DataKey::MultiplyPair((deposit_pool_address.clone(), borrow_pool_address.clone()));

    let res: bool = e.storage().persistent().has(&key);

    if res {
        extend_shared_storage(e, &key);
    }

    res
}

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

pub fn remove_multiply_pair(e: &Env, pair: &MultiplyPair) {
    let key = DataKey::MultiplyPair((pair.deposit_pool.clone(), pair.borrow_pool.clone()));

    e.storage().persistent().remove(&key);

    let pairs = get_all_multiply_pairs(e);
    let mut new_pairs = Vec::new(e);

    for p in pairs.iter() {
        if p != *pair {
            new_pairs.push_back(p);
        }
    }

    e.storage()
        .persistent()
        .set(&DataKey::AllMultiplyPairs, &new_pairs);
}

pub fn remove_all_multiply_pairs(e: &Env) {
    let all_pairs: Vec<MultiplyPair> = get_all_multiply_pairs(e);

    for pair in all_pairs.iter() {
        remove_multiply_pair(e, &pair);
    }
}

// --- Obligation ---
pub fn set_obligation(e: &Env, user: &Address, seed: &Option<BytesN<32>>, obligation: &Obligation) {
    let key = DataKey::Obligation((user.clone(), seed.clone()));

    e.storage().persistent().set(&key, obligation);

    extend_individual_storage(e, &key);
}

pub fn get_obligation(
    e: &Env,
    user_address: &Address,
    seed: &Option<BytesN<32>>,
) -> Option<Obligation> {
    let key = DataKey::Obligation((user_address.clone(), seed.clone()));

    let res = e.storage().persistent().get(&key);

    if res.is_some() {
        extend_individual_storage(e, &key);
    }

    res
}

pub fn obligation_exists(e: &Env, user_address: &Address, seed: &Option<BytesN<32>>) -> bool {
    let key = DataKey::Obligation((user_address.clone(), seed.clone()));

    let res = e.storage().persistent().has(&key);

    if res {
        extend_shared_storage(e, &key);
    }

    res
}

// TODO: Is this a good way of doing this?
// Doesn't seem to be scaling well
pub fn register_obligation(e: &Env, user_address: &Address, seed: &Option<BytesN<32>>) -> u32 {
    let mut all_obligations = get_all_obligations(e);
    all_obligations.push_back((user_address.clone(), seed.clone()));
    let new_index = all_obligations.len() - 1;

    e.storage()
        .persistent()
        .set(&DataKey::AllObligations, &all_obligations);
    extend_shared_storage(e, &DataKey::AllObligations);

    new_index
}

pub fn remove_obligation(e: &Env, user_address: &Address, seed: &Option<BytesN<32>>) {
    let key = DataKey::Obligation((user_address.clone(), seed.clone()));

    e.storage().persistent().remove(&key);

    let obligations = get_all_obligations(e);
    let mut new_obligations = Vec::new(e);

    // WARN: This doesn't scale well, so it better be rewritten with 'Map'
    for (user_addr, s) in &obligations {
        if user_addr != *user_address && s != *seed {
            new_obligations.push_back((user_addr, s));
        }
    }

    e.storage()
        .persistent()
        .set(&DataKey::AllObligations, &new_obligations);
}

pub fn remove_all_obligations(e: &Env) {
    let all_obligations: Vec<(Address, Option<BytesN<32>>)> = get_all_obligations(e);

    for (user_address, seed) in all_obligations.iter() {
        remove_obligation(e, &user_address, &seed);
    }
}

pub fn get_all_obligations(e: &Env) -> Vec<(Address, Option<BytesN<32>>)> {
    let res = e.storage().persistent().get(&DataKey::AllObligations);
    if let Some(obligations) = res {
        extend_shared_storage(e, &DataKey::AllObligations);
        obligations
    } else {
        Vec::new(e)
    }
}
