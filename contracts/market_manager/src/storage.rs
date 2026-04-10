use soroban_sdk::{Address, BytesN, Env, Map, contracttype};

use crate::{
    constants::{INSTANCE_BUMP, INSTANCE_THRESHOLD},
    error::MMCError,
};

#[derive(Debug)]
#[contracttype]
pub enum DataKey {
    Admin,
    MarketsList,
    MarketWasmHash,
    QueuedInMarketUpgrade,
    QueuedInManagerUpgrade,
}

#[contracttype]
pub struct Config {
    pub admin: Address,
    pub market_wasm_hash: BytesN<32>,
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

// -- MarketWasmHash --

pub fn get_market_wasm_hash(e: &Env) -> BytesN<32> {
    e.storage()
        .instance()
        .get(&DataKey::MarketWasmHash)
        .expect("Market contract WASM hash must exist")
}
pub fn set_market_wasm_hash(e: &Env, hash: &BytesN<32>) {
    e.storage().instance().set(&DataKey::MarketWasmHash, hash);
}

// -- QueuedInMarketUpgrade --

pub fn get_queued_in_market_upgrade(e: &Env) -> Option<QueuedInUpgrade> {
    e.storage().instance().get(&DataKey::QueuedInMarketUpgrade)
}
pub fn set_queued_in_market_upgrade(e: &Env, upgrade: &QueuedInUpgrade) {
    e.storage().instance().set(&DataKey::QueuedInMarketUpgrade, upgrade)
}
pub fn remove_queued_in_market_upgrade(e: &Env) {
    e.storage().instance().remove(&DataKey::QueuedInMarketUpgrade);
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

// -- Config --
pub fn get_config(e: &Env) -> Config {
    let admin = get_admin(e);
    let market_wasm_hash = get_market_wasm_hash(e);

    Config { admin, market_wasm_hash }
}

pub fn register_market(e: &Env, market_address: &Address) -> Result<(), MMCError> {
    let mut markets: Map<Address, ()> =
        e.storage().instance().get(&DataKey::MarketsList).unwrap_or(Map::new(e));

    if markets.contains_key(market_address.clone()) {
        return Err(MMCError::MarketAlreadyExists);
    } else {
        markets.set(market_address.clone(), ());
    }
    e.storage().instance().set(&DataKey::MarketsList, &markets);

    Ok(())
}

pub fn get_markets(e: &Env) -> Option<Map<Address, ()>> {
    e.storage().instance().get(&DataKey::MarketsList)?
}

/// Instance storage bumper
pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
