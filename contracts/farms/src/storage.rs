use farms_interface::FarmingKey;
use soroban_sdk::{Address, Env, contracttype};

use crate::{
    constants::SECONDS_PER_DAY,
    state::{Farm, FarmingPosition, RewardInfo},
};

#[contracttype]
pub enum DataKey {
    Farm,
    RewardInfo(Address),
    FarmingPosition(FarmingKey),
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

pub fn get_farm(e: &Env) -> Option<Farm> {
    get_persistent(e, &DataKey::Farm)
}

pub fn set_farm(e: &Env, farm: &Farm) {
    set_persistent(e, &DataKey::Farm, farm);
}

pub fn get_reward_info(e: &Env, reward_token: &Address) -> Option<RewardInfo> {
    get_persistent(e, &DataKey::RewardInfo(reward_token.clone()))
}

pub fn set_reward_info(e: &Env, reward_token: &Address, reward_info: &RewardInfo) {
    set_persistent(e, &DataKey::RewardInfo(reward_token.clone()), reward_info);
}

pub fn get_user(e: &Env, farming_key: &FarmingKey) -> Option<FarmingPosition> {
    get_persistent(e, &DataKey::FarmingPosition(farming_key.clone()))
}

pub fn set_user(e: &Env, farming_key: &FarmingKey, user: &FarmingPosition) {
    set_persistent(e, &DataKey::FarmingPosition(farming_key.clone()), user);
}
