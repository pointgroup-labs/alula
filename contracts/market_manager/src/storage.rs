use soroban_sdk::{Address, BytesN, Env, contracttype};

use crate::{
    constants::{INSTANCE_BUMP, INSTANCE_THRESHOLD, PERSISTENT_BUMP, PERSISTENT_THRESHOLD},
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

// -- QueuedInMarketUpgrade (persistent, per-market) --

pub fn get_queued_in_market_upgrade(e: &Env, market_address: &Address) -> Option<QueuedInUpgrade> {
    let key = DataKey::QueuedInMarketUpgrade(market_address.clone());

    let upgrade: Option<QueuedInUpgrade> = e.storage().persistent().get(&key);
    if upgrade.is_some() {
        extend_persistent(e, &key);
    }

    upgrade
}
pub fn set_queued_in_market_upgrade(e: &Env, market_address: &Address, upgrade: &QueuedInUpgrade) {
    let key = DataKey::QueuedInMarketUpgrade(market_address.clone());
    e.storage().persistent().set(&key, upgrade);
    extend_persistent(e, &key);
}
pub fn remove_queued_in_market_upgrade(e: &Env, market_address: &Address) {
    e.storage().persistent().remove(&DataKey::QueuedInMarketUpgrade(market_address.clone()));
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
    if e.storage().persistent().has(&key) {
        return Err(MMCError::MarketAlreadyExists);
    }

    e.storage().persistent().set(&key, &upgrade_in_queue_period);
    extend_persistent(e, &key);

    Ok(())
}

pub fn is_market_deployed(e: &Env, market_address: &Address) -> bool {
    let key = DataKey::DeployedMarket(market_address.clone());

    let exists = e.storage().persistent().has(&key);
    if exists {
        extend_persistent(e, &key);
    }

    exists
}

pub fn get_market_upgrade_in_queue_period(e: &Env, market_address: &Address) -> u64 {
    let key = DataKey::DeployedMarket(market_address.clone());

    let period = e.storage().persistent().get(&key).expect("UpgradeInQueuePeriod must exist");
    extend_persistent(e, &key);

    period
}

/// Instance storage bumper
pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

/// Persistent per-market storage bumper
pub fn extend_persistent(e: &Env, key: &DataKey) {
    e.storage().persistent().extend_ttl(key, PERSISTENT_THRESHOLD, PERSISTENT_BUMP);
}
