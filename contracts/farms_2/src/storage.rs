use farms_interface::FarmingKey;
use soroban_sdk::{Address, Env, Map, contracttype};

use crate::{
    constants::MAX_ALLOWED_FARMS,
    error::FCError,
    state::{Farm, RewardInfo, User},
};

#[contracttype]
pub enum DataKey {
    Admin,
    AllFarms,
    Farm(u64), // farm_id
    FarmsCounter,
    ProposedAdmin,
    TreasuryFeeBps,
    User(u64, FarmingKey),    // (farm_id, farming_key)
    RewardInfo(u64, Address), // (farm_id, reward_token_address)
}

// Admin
pub fn get_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::Admin)
}
pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin)
}

// TreasuryFeeBps
pub fn get_treasury_fee_bps(e: &Env) -> Option<i128> {
    e.storage().instance().get(&DataKey::TreasuryFeeBps)
}
pub fn set_treasury_fee_bps(e: &Env, treasury_fee_bps: i128) {
    e.storage().instance().set(&DataKey::TreasuryFeeBps, &treasury_fee_bps)
}

// ProposedAdmin
pub fn get_proposed_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::ProposedAdmin)
}
pub fn set_proposed_admin(e: &Env, proposed_admin: &Address) {
    e.storage().instance().set(&DataKey::ProposedAdmin, proposed_admin)
}
pub fn remove_proposed_admin(e: &Env) {
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

// AllFarms
pub fn get_all_farms(e: &Env) -> Option<Map<u64, ()>> {
    let data_key = DataKey::AllFarms;

    let res = e.storage().persistent().get(&data_key);
    if res.is_some() {
        extend_persistent(e, &data_key);
    }

    res
}
pub fn register_farm(e: &Env, farm_id: u64) -> Result<(), FCError> {
    let mut all_farms_set = get_all_farms(e).unwrap_or_else(|| Map::new(&e));

    let all_farms_len = all_farms_set.len();
    if all_farms_len > MAX_ALLOWED_FARMS {
        return Err(FCError::InternalError);
    } else if all_farms_len == MAX_ALLOWED_FARMS {
        return Err(FCError::MaxAllowedFarmsReached);
    }

    if all_farms_set.contains_key(farm_id) {
        return Err(FCError::InternalError);
    }

    all_farms_set.set(farm_id, ());
    e.storage().persistent().set(&DataKey::AllFarms, &all_farms_set);

    Ok(())
}
pub fn unregister_farm(e: &Env, farm_id: u64) -> Result<(), FCError> {
    let mut all_farms_set = get_all_farms(e).ok_or(FCError::InternalError)?;
    if all_farms_set.is_empty() || !all_farms_set.contains_key(farm_id) {
        return Err(FCError::InternalError);
    }

    all_farms_set.remove(farm_id);
    e.storage().persistent().set(&DataKey::AllFarms, &all_farms_set);

    Ok(())
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

// User
pub fn get_user(e: &Env, farm_id: u64, farming_key: &FarmingKey) -> Option<User> {
    let data_key = DataKey::User(farm_id, farming_key.clone());
    extend_persistent(e, &data_key);

    e.storage().persistent().get(&data_key)
}
pub fn set_user(e: &Env, farm_id: u64, farming_key: &FarmingKey, user: &User) {
    e.storage().persistent().set(&DataKey::User(farm_id, farming_key.clone()), user);
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
