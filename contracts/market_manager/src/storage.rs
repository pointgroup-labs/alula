use soroban_sdk::{Address, BytesN, Env, Map, contracttype};

use crate::{
    constants::{INSTANCE_BUMP, INSTANCE_THRESHOLD},
    error::MMCError,
};

#[derive(Debug)]
#[contracttype]
pub enum DataKey {
    Admin,
    MarketContractWasmHash,
    MarketList,
}

#[contracttype]
pub struct Config {
    pub admin: Address,
    pub market_contract_wasm_hash: BytesN<32>,
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}
pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).expect("Admin must exist")
}

pub fn set_market_contract_wasm_hash(e: &Env, hash: &BytesN<32>) {
    e.storage().instance().set(&DataKey::MarketContractWasmHash, hash);
}
pub fn get_market_contract_wasm_hash(e: &Env) -> BytesN<32> {
    e.storage()
        .instance()
        .get(&DataKey::MarketContractWasmHash)
        .expect("Market contract WASM hash must exist")
}

pub fn get_config(e: &Env) -> Config {
    let admin = get_admin(e);
    let market_contract_wasm_hash = get_market_contract_wasm_hash(e);

    Config { admin, market_contract_wasm_hash }
}

pub fn register_market(e: &Env, market_address: &Address) -> Result<(), MMCError> {
    let mut markets: Map<Address, ()> =
        e.storage().instance().get(&DataKey::MarketList).unwrap_or(Map::new(e));

    if markets.contains_key(market_address.clone()) {
        return Err(MMCError::MarketAlreadyExists);
    } else {
        markets.set(market_address.clone(), ());
    }
    e.storage().instance().set(&DataKey::MarketList, &markets);

    Ok(())
}

pub fn get_markets(e: &Env) -> Option<Map<Address, ()>> {
    e.storage().instance().get(&DataKey::MarketList)?
}

/// Instance storage bumper
pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
