use sep_40_oracle::{Asset, PriceData};
use soroban_sdk::{contracttype, panic_with_error, vec as svec, Address, Env, Map, Symbol, Vec};

use crate::error::AOCError;

const LEDGERS_PER_DAY: u32 = (24 * 60 * 60) / 6; // NB: Assuming 6 seconds per ledger
const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
const INSTANCE_BUMP: u32 = 41 * LEDGERS_PER_DAY;

#[contracttype]
pub enum DataKey {
    Admin,
    MaxAge,
    Decimals,
    BaseAsset,
    Assets,
    Oracles,
}

// ---- Storage Setters & Getters ----

pub fn set_admin(e: &Env, admin: Address) {
    let key = DataKey::Admin;

    e.storage().instance().set(&key, &admin);
}

pub fn get_admin(e: &Env) -> Address {
    let key = DataKey::Admin;

    e.storage()
        .instance()
        .get(&key)
        .expect("Admin must've been set")
}

pub fn set_max_age(e: &Env, max_age: u64) {
    let key = DataKey::MaxAge;

    e.storage().instance().set(&key, &max_age);
}

pub fn get_max_age(e: &Env) -> u64 {
    let key = DataKey::MaxAge;

    e.storage()
        .instance()
        .get(&key)
        .expect("Max age must've been set")
}

pub fn set_decimals(e: &Env, decimals: u32) {
    let key = DataKey::Decimals;

    e.storage().instance().set(&key, &decimals);
}

pub fn get_decimals(e: &Env) -> u32 {
    let key = DataKey::Decimals;

    e.storage()
        .instance()
        .get(&key)
        .expect("Decimals must've been set")
}

pub fn set_base_asset(e: &Env, base_asset: Asset) {
    let key = DataKey::BaseAsset;

    e.storage().instance().set(&key, &base_asset);
}

pub fn get_base_asset(e: &Env) -> Asset {
    let key = DataKey::BaseAsset;

    e.storage()
        .instance()
        .get(&key)
        .expect("Base asset must've been set")
}

pub fn add_asset(e: &Env, symbol: Symbol, address: Address) -> Result<(), AOCError> {
    let key = DataKey::Assets;

    let mut assets: Map<Address, Symbol> = e
        .storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Map::new(e));

    if assets.contains_key(address.clone()) {
        return Err(AOCError::AssetAlreadyRegistered);
    }

    assets.set(address, symbol);
    e.storage().instance().set(&key, &assets);

    Ok(())
}

pub fn get_assets(e: &Env) -> Vec<Asset> {
    let key = DataKey::Assets;

    let assets_map: Map<Address, Symbol> = e.storage().instance().get(&key).unwrap_or(Map::new(e));
    let mut assets_vec = svec![e];

    for address in assets_map.keys() {
        assets_vec.push_back(address);
    }

    assets_vec
}

pub fn is_asset_registered(e: &Env, token_address: &Address) -> bool {
    let key = DataKey::Assets;

    let Some(assets_map) = e.storage().instance().get::<_, Map<Address, Symbol>>(&key) else {
        return false;
    };

    assets_map.contains_key(token_address.clone())
}

pub fn get_token_ticker(e: &Env, token_address: &Address) -> Symbol {
    let key = DataKey::Assets;

    let assets: Map<Address, Symbol> = e
        .storage()
        .instance()
        .get(&key)
        .expect("Assets must've been set");

    assets
        .get(token_address.clone())
        .expect("Asset must've been set")
}

pub fn set_oracles(e: &Env, oracles: Vec<OracleConfig>) {
    let key = DataKey::Oracles;

    let mut known_addresses = Map::<Address, ()>::new(e);
    for oracle in oracles.iter() {
        if known_addresses.contains_key(oracle.address.clone()) {
            panic_with_error!(e, AOCError::NonUniqueOraclesRegistered);
        }

        known_addresses.set(oracle.address.clone(), ());
    }

    e.storage().instance().set(&key, &oracles)
}

pub fn get_oracles(e: &Env) -> Vec<OracleConfig> {
    extend_instance_storage(e);
    let key = DataKey::Oracles;

    e.storage()
        .instance()
        .get(&key)
        .expect("Oracles must've been set")
}

// ---- Storage Types ----

/// `SEP-40` compliant oracle contract's configuration
#[derive(Clone)]
#[contracttype]
pub struct OracleConfig {
    /// Oracle contract's address on the ledger
    pub address: Address,
    /// Number of decimals representing a fractional part of a price
    pub decimals: u32,
    /// Default tick period timeframe
    pub resolution: u32,
    /// [`PriceData`] of the last time-weighted average price computation
    pub last_twap_price_data: PriceData,
    /// Indicator of whether the oracle gets the data from or out of the `Stellar` ledger.
    /// Oracles that have this set to `true` will receive a request with [`Asset::Stellar`] asset
    /// parameter first, and only if it returns [`None`] will receive [`Asset::Other`] afterwards.
    /// The opposite behavior takes place otherwise
    pub is_stellar_data_based: bool,
}

// ---- TTL Bumper ----

pub fn extend_instance_storage(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
