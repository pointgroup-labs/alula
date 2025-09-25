use soroban_sdk::{Address, BytesN, Env, Vec, contracttype, panic_with_error};

use crate::error::MMCError;

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

    e.storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| panic_with_error!(e, MMCError::InternalError))
}

pub fn register_market(e: &Env, market_address: &Address) -> Result<(), MMCError> {
    extend_instance_storage(e);

    let key = DataKey::MarketList;
    let mut markets: Vec<Address> = e.storage().instance().get(&key).unwrap_or(Vec::new(e));

    // TODO: Consider using set instead of vec?
    if markets.contains(market_address) {
        // TODO: Should this be Internal Error?
        return Err(MMCError::MarketAlreadyExists);
    } else {
        markets.push_back(market_address.clone());
    }
    e.storage().instance().set(&key, &markets);

    Ok(())
}

pub fn get_markets(e: &Env) -> Option<Vec<Address>> {
    extend_instance_storage(e);

    let key = DataKey::MarketList;

    e.storage().instance().get(&key)
}

const SECONDS_PER_DAY: u32 = 24 * 60 * 60;
const SECONDS_PER_LEDGER: u32 = 6;
const LEDGERS_PER_DAY: u32 = SECONDS_PER_DAY / SECONDS_PER_LEDGER;
const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
const INSTANCE_BUMP: u32 = INSTANCE_THRESHOLD + LEDGERS_PER_DAY;

/// Instance storage bumper
pub fn extend_instance_storage(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
