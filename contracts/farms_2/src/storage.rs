use soroban_sdk::{Address, Env, contracttype};

use crate::state::{Farm, RewardInfo};

#[contracttype]
pub enum DataKey {
    Admin,
    ProposedAdmin,
    FarmsCounter,
    Farm(u64),                // farm_id
    RewardInfo(u64, Address), // (farm_id, reward_token_address)
}

// Admin
pub fn get_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::Admin)
}
pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin)
}

// ProposedAdmin
pub fn get_proposed_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::ProposedAdmin)
}
pub fn set_proposed_admin(e: &Env, proposed_admin: &Address) {
    e.storage().instance().set(&DataKey::ProposedAdmin, proposed_admin)
}
pub fn reset_proposed_admin(e: &Env) {
    // Check if fails when isn't set
    e.storage().instance().remove(&DataKey::ProposedAdmin); // safe?
}

// FarmsCounter
pub fn get_farms_counter(e: &Env) -> Option<u64> {
    e.storage().instance().get(&DataKey::FarmsCounter)
}
pub fn increment_farms_counter(e: &Env) {
    let counter = get_farms_counter(e).unwrap_or(0);
    e.storage().instance().set(&DataKey::FarmsCounter, &counter.checked_add(1).unwrap()); // TODO map unwrap
}

// Farm
pub fn get_farm(e: &Env, farm_id: u64) -> Option<Farm> {
    e.storage().persistent().get(&DataKey::Farm(farm_id))
}
pub fn set_farm(e: &Env, farm: &Farm) {
    e.storage().persistent().set(&DataKey::Farm(farm.id), farm)
}

// Farm
pub fn get_reward_info(e: &Env, farm_id: u64, reward_token: &Address) -> Option<RewardInfo> {
    let data_key = DataKey::RewardInfo(farm_id, reward_token.clone());
    extend_persistent(e, &data_key);

    e.storage().persistent().get(&data_key)
}
pub fn set_reward_info(e: &Env, farm_id: u64, reward_token: &Address, reward_info: &RewardInfo) {
    e.storage().persistent().set(&DataKey::RewardInfo(farm_id, reward_token.clone()), reward_info);
}

// -- TTL --

const DAY_IN_LEDGERS: u32 = 24 * 60 * 60;

const INSTANCE_TTL: u32 = 7 * DAY_IN_LEDGERS;
const INSTANCE_TTL_THRESHOLD: u32 = DAY_IN_LEDGERS;

const PERSISTENT_TTL: u32 = 30 * DAY_IN_LEDGERS;
const PERSISTENT_TTL_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS;

pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL);
}

pub fn extend_persistent(e: &Env, data_key: &DataKey) {
    e.storage().persistent().extend_ttl(data_key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL);
}
