use soroban_sdk::{Address, BytesN, Env, contracttype};

use crate::{
    constants::{INSTANCE_BUMP, INSTANCE_THRESHOLD},
    error::MMCError,
};

#[derive(Debug)]
#[contracttype]
pub enum DataKey {
    Admin,
    PendingAdmin,
    QueuedInManagerUpgrade,
    DeployedMarket(Address),
    QueuedInMarketUpgrade(Address),
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct QueuedInUpgrade {
    pub wasm_hash: BytesN<32>,
    pub queued_in_timestamp: u64,
}

// -- Admin --

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}
pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).expect("Admin must exist")
}

// -- QueuedInMarketUpgrade --

pub fn get_queued_in_market_upgrade(e: &Env, market_address: &Address) -> Option<QueuedInUpgrade> {
    e.storage().instance().get(&DataKey::QueuedInMarketUpgrade(market_address.clone()))
}
pub fn set_queued_in_market_upgrade(e: &Env, market_address: &Address, upgrade: &QueuedInUpgrade) {
    e.storage().instance().set(&DataKey::QueuedInMarketUpgrade(market_address.clone()), upgrade)
}
pub fn remove_queued_in_market_upgrade(e: &Env, market_address: &Address) {
    e.storage().instance().remove(&DataKey::QueuedInMarketUpgrade(market_address.clone()));
}

// -- QueuedInManagerUpgrade --

pub fn get_queued_in_manager_upgrade(e: &Env) -> Option<QueuedInUpgrade> {
    e.storage().instance().get(&DataKey::QueuedInManagerUpgrade)
}
pub fn set_queued_in_manager_upgrade(e: &Env, upgrade: &QueuedInUpgrade) {
    e.storage().instance().set(&DataKey::QueuedInManagerUpgrade, upgrade)
}
pub fn remove_queued_in_manager_upgrade(e: &Env) {
    e.storage().instance().remove(&DataKey::QueuedInManagerUpgrade);
}

// -- PendingAdmin --

pub fn set_pending_admin(e: &Env, pending_admin: &Address) {
    e.storage().instance().set(&DataKey::PendingAdmin, pending_admin);
}
pub fn get_pending_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::PendingAdmin)
}
pub fn remove_pending_admin(e: &Env) {
    e.storage().instance().remove(&DataKey::PendingAdmin);
}

pub fn register_market(
    e: &Env,
    market_address: &Address,
    upgrade_in_queue_period: u64,
) -> Result<(), MMCError> {
    let key = DataKey::DeployedMarket(market_address.clone());
    if e.storage().instance().has(&key) {
        return Err(MMCError::MarketAlreadyExists);
    }

    e.storage().instance().set(&key, &upgrade_in_queue_period);

    Ok(())
}

pub fn is_market_deployed(e: &Env, market_address: &Address) -> bool {
    e.storage().instance().has(&DataKey::DeployedMarket(market_address.clone()))
}

pub fn get_market_upgrade_in_queue_period(e: &Env, market_address: &Address) -> u64 {
    e.storage()
        .instance()
        .get(&DataKey::DeployedMarket(market_address.clone()))
        .expect("UpgradeInQueuePeriod must exist")
}

/// Instance storage bumper
pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}