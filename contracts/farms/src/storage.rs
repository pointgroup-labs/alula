use soroban_sdk::{Address, BytesN, Env, Vec, contracttype};

use crate::state::{FarmState, GlobalConfig, UserState};

/// Storage keys for the Farms contract
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Global configuration
    GlobalConfig,
    /// Farm state by farm_id
    Farm(BytesN<32>),
    /// User state by (user, farm_id)
    User(Address, BytesN<32>),
    /// List of all farm IDs
    AllFarms,
    /// Counter for generating unique farm IDs
    FarmCounter,
}

// TTL constants (in ledgers, ~5 seconds per ledger on Stellar)
const DAY_IN_LEDGERS: u32 = 17_280; // 24 * 60 * 60 / 5
const INSTANCE_TTL: u32 = 7 * DAY_IN_LEDGERS; // 7 days
const INSTANCE_TTL_THRESHOLD: u32 = DAY_IN_LEDGERS; // 1 day
const PERSISTENT_TTL: u32 = 30 * DAY_IN_LEDGERS; // 30 days
const PERSISTENT_TTL_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS; // 7 days

// ═══════════════════════════════════════════════════════════════════════════════
// Instance Storage Extension
// ═══════════════════════════════════════════════════════════════════════════════

pub fn extend_instance_storage(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Global Config
// ═══════════════════════════════════════════════════════════════════════════════

pub fn set_global_config(e: &Env, config: &GlobalConfig) {
    e.storage().instance().set(&DataKey::GlobalConfig, config);
}

pub fn get_global_config(e: &Env) -> Option<GlobalConfig> {
    e.storage().instance().get(&DataKey::GlobalConfig)
}

pub fn has_global_config(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::GlobalConfig)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Farms
// ═══════════════════════════════════════════════════════════════════════════════

pub fn set_farm(e: &Env, farm_id: &BytesN<32>, farm: &FarmState) {
    let key = DataKey::Farm(farm_id.clone());
    e.storage().persistent().set(&key, farm);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL);
}

pub fn get_farm(e: &Env, farm_id: &BytesN<32>) -> Option<FarmState> {
    let key = DataKey::Farm(farm_id.clone());
    let farm: Option<FarmState> = e.storage().persistent().get(&key);
    if farm.is_some() {
        e.storage().persistent().extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL);
    }
    farm
}

pub fn get_all_farms(e: &Env) -> Vec<BytesN<32>> {
    e.storage().persistent().get(&DataKey::AllFarms).unwrap_or_else(|| Vec::new(e))
}

pub fn register_farm(e: &Env, farm_id: &BytesN<32>) {
    let key = DataKey::AllFarms;
    let mut farms = get_all_farms(e);
    farms.push_back(farm_id.clone());
    e.storage().persistent().set(&key, &farms);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL);
}

pub fn get_farm_counter(e: &Env) -> u64 {
    e.storage().instance().get(&DataKey::FarmCounter).unwrap_or(0)
}

pub fn increment_farm_counter(e: &Env) -> u64 {
    let counter = get_farm_counter(e) + 1;
    e.storage().instance().set(&DataKey::FarmCounter, &counter);
    counter
}

// ═══════════════════════════════════════════════════════════════════════════════
// Users
// ═══════════════════════════════════════════════════════════════════════════════

pub fn set_user(e: &Env, user: &Address, farm_id: &BytesN<32>, state: &UserState) {
    let key = DataKey::User(user.clone(), farm_id.clone());
    e.storage().persistent().set(&key, state);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL);
}

pub fn get_user(e: &Env, user: &Address, farm_id: &BytesN<32>) -> Option<UserState> {
    let key = DataKey::User(user.clone(), farm_id.clone());
    let state: Option<UserState> = e.storage().persistent().get(&key);
    if state.is_some() {
        e.storage().persistent().extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL);
    }
    state
}

pub fn has_user(e: &Env, user: &Address, farm_id: &BytesN<32>) -> bool {
    e.storage().persistent().has(&DataKey::User(user.clone(), farm_id.clone()))
}
