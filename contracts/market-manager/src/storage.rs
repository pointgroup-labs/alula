use market::constants::{INSTANCE_BUMP, INSTANCE_THRESHOLD};
use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env, Vec};

use crate::MMError;

#[derive(Debug)]
#[contracttype]
pub enum DataKey {
    Config,
    MarketList,
}

#[contracttype]
pub struct Config {
    pub admin: Address,
    pub market_contract_wasm_hash: BytesN<32>,
}

pub fn set_config(e: &Env, config: Config) {
    extend_instance_storage(e);

    let key = DataKey::Config;
    e.storage().instance().set(&key, &config);
}

pub fn get_config(e: &Env) -> Config {
    extend_instance_storage(e);

    let key = DataKey::Config;
    let config = e
        .storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| panic_with_error!(e, MMError::InternalError));

    config
}

pub fn register_market(e: &Env, market_address: &Address) -> Result<(), MMError> {
    extend_instance_storage(e);

    let key = DataKey::MarketList;
    let mut markets: Vec<Address> = e.storage().instance().get(&key).unwrap_or(Vec::new(e));

    // TODO: Should this be a set?
    if markets.contains(market_address) {
        // TODO: Add an event

        return Err(MMError::InternalError);
    } else {
        markets.push_back(market_address.clone());
    }

    Ok(())
}

pub fn get_markets(e: &Env) -> Option<Vec<Address>> {
    extend_instance_storage(e); // will the storage ever be extended here?

    let key = DataKey::MarketList;

    e.storage().instance().get(&key)
}

/// Instance storage bumper
pub fn extend_instance_storage(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
