use farms_interface::Delegatee;
use soroban_sdk::{Address, BytesN, Env, Vec, contracttype, vec as svec};

use crate::{
    constants::SECONDS_PER_DAY,
    state::{DelegateeState, Farm, GlobalConfig, RewardInfo},
};

#[contracttype]
pub enum DataKey {
    GlobalConfig,
    /// Registry of all farm IDs in creation order.
    Farms,
    Farm(BytesN<32>),
    /// Reward state, keyed by `(farm_id, reward_token)`.
    RewardInfo(BytesN<32>, Address),
    /// Delegatee position, keyed by `(farm_id, delegatee)`.
    DelegateeState(BytesN<32>, Delegatee),
}

const SECONDS_PER_LEDGER: u32 = 6;
const DAY_IN_LEDGERS: u32 = SECONDS_PER_DAY as u32 / SECONDS_PER_LEDGER;

const INSTANCE_TTL: u32 = 30 * DAY_IN_LEDGERS;
const INSTANCE_TTL_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS;

const PERSISTENT_TTL: u32 = 120 * DAY_IN_LEDGERS;
const PERSISTENT_TTL_THRESHOLD: u32 = 30 * DAY_IN_LEDGERS;

pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL);
}

fn get_persistent<T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>>(
    e: &Env,
    key: &DataKey,
) -> Option<T> {
    let result = e.storage().persistent().get(key);
    if result.is_some() {
        e.storage().persistent().extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL);
    }

    result
}

fn set_persistent<T: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(
    e: &Env,
    key: &DataKey,
    value: &T,
) {
    e.storage().persistent().set(key, value);
    e.storage().persistent().extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL);
}

pub fn get_global_config(e: &Env) -> Option<GlobalConfig> {
    e.storage().instance().get(&DataKey::GlobalConfig)
}

pub fn set_global_config(e: &Env, config: &GlobalConfig) {
    e.storage().instance().set(&DataKey::GlobalConfig, config);
}

pub fn get_farms(e: &Env) -> Vec<BytesN<32>> {
    get_persistent(e, &DataKey::Farms).unwrap_or_else(|| svec![e])
}

pub fn add_farm_to_registry(e: &Env, farm_id: &BytesN<32>) {
    let mut farms = get_farms(e);
    farms.push_back(farm_id.clone());
    
    set_persistent(e, &DataKey::Farms, &farms);
}

pub fn has_farm(e: &Env, farm_id: &BytesN<32>) -> bool {
    e.storage().persistent().has(&DataKey::Farm(farm_id.clone()))
}

pub fn get_farm(e: &Env, farm_id: &BytesN<32>) -> Option<Farm> {
    get_persistent(e, &DataKey::Farm(farm_id.clone()))
}

pub fn set_farm(e: &Env, farm: &Farm) {
    set_persistent(e, &DataKey::Farm(farm.id.clone()), farm);
}

pub fn get_reward_info(
    e: &Env,
    farm_id: &BytesN<32>,
    reward_token: &Address,
) -> Option<RewardInfo> {
    get_persistent(e, &DataKey::RewardInfo(farm_id.clone(), reward_token.clone()))
}

pub fn set_reward_info(
    e: &Env,
    farm_id: &BytesN<32>,
    reward_token: &Address,
    reward_info: &RewardInfo,
) {
    set_persistent(e, &DataKey::RewardInfo(farm_id.clone(), reward_token.clone()), reward_info);
}

pub fn get_delegatee_state(
    e: &Env,
    farm_id: &BytesN<32>,
    delegatee: &Delegatee,
) -> Option<DelegateeState> {
    get_persistent(e, &DataKey::DelegateeState(farm_id.clone(), delegatee.clone()))
}

pub fn set_delegatee_state(
    e: &Env,
    farm_id: &BytesN<32>,
    delegatee: &Delegatee,
    state: &DelegateeState,
) {
    set_persistent(e, &DataKey::DelegateeState(farm_id.clone(), delegatee.clone()), state);
}
