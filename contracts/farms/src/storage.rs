use farms_interface::FarmingKey;
use soroban_sdk::{Address, Env, Map, contracttype};

use crate::{
    constants::MAX_ALLOWED_FARMS,
    error::FCError,
    state::{Farm, FarmingPosition, RewardInfo},
    utils::MathUtils,
};

#[contracttype]
pub enum DataKey {
    Admin,
    AllFarms,
    Farm(u64),
    FarmsCounter,
    ProposedAdmin,
    TreasuryFeeBps,
    RewardInfo(u64, Address),
    FarmingPosition(u64, FarmingKey),
}

// Admin

pub fn get_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::Admin)
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin)
}

// TreasuryFeeBps
// Note: this global value is stored but not read by harvest.
// Each farm uses its own `FarmConfig.treasury_fee_bps` set at initialization.

#[allow(dead_code)]
pub fn get_treasury_fee_bps(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::TreasuryFeeBps).unwrap_or(0)
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
    e.storage().instance().remove(&DataKey::ProposedAdmin);
}

// FarmsCounter

pub fn get_farms_counter(e: &Env) -> Option<u64> {
    e.storage().instance().get(&DataKey::FarmsCounter)
}

pub fn increment_farms_counter(e: &Env) -> Result<(), FCError> {
    let counter = get_farms_counter(e).unwrap_or(0);
    e.storage()
        .instance()
        .set(&DataKey::FarmsCounter, &counter.checked_add(1).map_over_or_underflow()?);

    Ok(())
}

// Farm

pub fn get_farm(e: &Env, farm_id: u64) -> Option<Farm> {
    let data_key = DataKey::Farm(farm_id);
    let result = e.storage().persistent().get(&data_key);
    if result.is_some() {
        extend_persistent(e, &data_key);
    }
    result
}

pub fn set_farm(e: &Env, farm: &Farm) {
    let data_key = DataKey::Farm(farm.id);
    e.storage().persistent().set(&data_key, farm);
    extend_persistent(e, &data_key);
}

// AllFarms

pub fn get_all_farms(e: &Env) -> Option<Map<u64, ()>> {
    let data_key = DataKey::AllFarms;
    let result = e.storage().persistent().get(&data_key);
    if result.is_some() {
        extend_persistent(e, &data_key);
    }
    result
}

pub fn register_farm(e: &Env, farm_id: u64) -> Result<(), FCError> {
    let mut all_farms_set = get_all_farms(e).unwrap_or_else(|| Map::new(e));

    if all_farms_set.len() >= MAX_ALLOWED_FARMS {
        return Err(FCError::MaxAllowedFarmsReached);
    }

    if all_farms_set.contains_key(farm_id) {
        return Err(FCError::InternalError);
    }

    all_farms_set.set(farm_id, ());
    let data_key = DataKey::AllFarms;
    e.storage().persistent().set(&data_key, &all_farms_set);
    extend_persistent(e, &data_key);

    Ok(())
}

// RewardInfo

pub fn get_reward_info(e: &Env, farm_id: u64, reward_token: &Address) -> Option<RewardInfo> {
    let data_key = DataKey::RewardInfo(farm_id, reward_token.clone());
    let result = e.storage().persistent().get(&data_key);
    if result.is_some() {
        extend_persistent(e, &data_key);
    }
    result
}

pub fn set_reward_info(e: &Env, farm_id: u64, reward_token: &Address, reward_info: &RewardInfo) {
    let data_key = DataKey::RewardInfo(farm_id, reward_token.clone());
    e.storage().persistent().set(&data_key, reward_info);
    extend_persistent(e, &data_key);
}

// FarmingPosition

pub fn get_user(e: &Env, farm_id: u64, farming_key: &FarmingKey) -> Option<FarmingPosition> {
    let data_key = DataKey::FarmingPosition(farm_id, farming_key.clone());
    let result = e.storage().persistent().get(&data_key);
    if result.is_some() {
        extend_persistent(e, &data_key);
    }
    result
}

pub fn set_user(e: &Env, farm_id: u64, farming_key: &FarmingKey, user: &FarmingPosition) {
    let data_key = DataKey::FarmingPosition(farm_id, farming_key.clone());
    e.storage().persistent().set(&data_key, user);
    extend_persistent(e, &data_key);
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
