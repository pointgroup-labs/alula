use sep_40_oracle::{Asset, PriceData};
use soroban_sdk::{Address, Env, Map, Symbol, Vec, contracttype, panic_with_error, vec as svec};

use crate::{constants::*, error::AOCError};

#[contracttype]
pub enum DataKey {
    Admin,
    ProposedAdmin,
    PeriodicUpdateMaxAge,
    Decimals,
    BaseAsset,
    Assets,
    Oracles,
    PreviousMedianLastprice(Address), // Address - asset's token address
    OraclePriceDataCached(Address),   // Address - oracle's contract address
}

#[contracttype]
pub struct AssetData {
    /// Asset's ticker
    pub symbol: Symbol,
    /// Max available deviation between 2 consecutive price retrievals in basis points
    pub max_dev_bps: u32,
    /// Max time diff in seconds between retrievals when max deviation check is performed.
    /// Any more extended period between 2 consecutive price retrievals implies that no max deviation check
    /// is performed
    pub max_dev_consecutive_diff_secs: u64,
}

// ---- Storage Setters & Getters ----

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, &admin);
}
pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).expect("Admin must've been set")
}

pub fn set_proposed_admin(e: &Env, proposed_admin: &Address) {
    e.storage().instance().set(&DataKey::ProposedAdmin, &proposed_admin);
}
pub fn get_proposed_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::ProposedAdmin)
}
pub fn remove_proposed_admin(e: &Env) {
    e.storage().instance().remove(&DataKey::ProposedAdmin);
}

pub fn set_periodic_oracles_price_max_age(e: &Env, periodic_oracles_price_max_age: u64) {
    e.storage().instance().set(&DataKey::PeriodicUpdateMaxAge, &periodic_oracles_price_max_age);
}
pub fn get_periodic_oracles_price_max_age(e: &Env) -> u64 {
    e.storage()
        .instance()
        .get(&DataKey::PeriodicUpdateMaxAge)
        .expect("Periodic update max age must've been set")
}

pub fn set_decimals(e: &Env, decimals: u32) {
    e.storage().instance().set(&DataKey::Decimals, &decimals);
}
pub fn get_decimals(e: &Env) -> u32 {
    e.storage().instance().get(&DataKey::Decimals).expect("Decimals must've been set")
}

pub fn set_base_asset(e: &Env, base_asset: Asset) {
    e.storage().instance().set(&DataKey::BaseAsset, &base_asset);
}
pub fn get_base_asset(e: &Env) -> Asset {
    e.storage().instance().get(&DataKey::BaseAsset).expect("Base asset must've been set")
}

pub fn set_previous_median_lastprice(
    e: &Env,
    token_address: &Address,
    median_lastprice: &PriceData,
) {
    e.storage()
        .instance()
        .set(&DataKey::PreviousMedianLastprice(token_address.clone()), median_lastprice)
}
pub fn get_previous_median_lastprice(e: &Env, token_address: &Address) -> Option<PriceData> {
    e.storage().instance().get(&DataKey::PreviousMedianLastprice(token_address.clone()))
}

pub fn add_asset(
    e: &Env,
    symbol: Symbol,
    address: Address,
    max_dev_bps: u32,
    max_dev_consecutive_diff_secs: u64,
) -> Result<(), AOCError> {
    let mut assets: Map<Address, AssetData> =
        e.storage().instance().get(&DataKey::Assets).unwrap_or_else(|| Map::new(e));

    if assets.contains_key(address.clone()) {
        return Err(AOCError::AssetAlreadyRegistered);
    }
    let asset_data = AssetData { symbol, max_dev_bps, max_dev_consecutive_diff_secs };

    assets.set(address, asset_data);
    e.storage().instance().set(&DataKey::Assets, &assets);

    Ok(())
}
pub fn get_asset(e: &Env, token_address: &Address) -> Option<AssetData> {
    let assets: Map<Address, AssetData> = e.storage().instance().get(&DataKey::Assets)?;

    assets.get(token_address.clone())
}

pub fn get_assets(e: &Env) -> Vec<Asset> {
    let assets_map: Map<Address, AssetData> =
        e.storage().instance().get(&DataKey::Assets).unwrap_or_else(|| Map::new(e));
    let mut assets_vec = svec![e];

    for address in assets_map.keys() {
        let asset = Asset::Stellar(address);
        assets_vec.push_back(asset);
    }

    assets_vec
}

pub fn is_asset_registered(e: &Env, token_address: &Address) -> bool {
    let Some(assets_map) =
        e.storage().instance().get::<_, Map<Address, AssetData>>(&DataKey::Assets)
    else {
        return false;
    };

    assets_map.contains_key(token_address.clone())
}

pub fn get_token_ticker(e: &Env, token_address: &Address) -> Symbol {
    let assets: Map<Address, AssetData> =
        e.storage().instance().get(&DataKey::Assets).expect("Assets must've been set");

    assets.get(token_address.clone()).expect("Asset must've been set").symbol
}

pub fn get_oracle_price_data_cache(
    e: &Env,
    oracle_address: &Address,
) -> Option<Map<Address, PriceData>> {
    e.storage().instance().get(&DataKey::OraclePriceDataCached(oracle_address.clone()))
}

pub fn set_oracle_price_data_cache(
    e: &Env,
    oracle_address: &Address,
    oracle_cache: &Map<Address, PriceData>,
) {
    e.storage()
        .instance()
        .set(&DataKey::OraclePriceDataCached(oracle_address.clone()), oracle_cache);
}

pub fn set_oracles(e: &Env, oracles: Vec<OracleConfig>) {
    let mut known_addresses = Map::<Address, ()>::new(e);
    for oracle in oracles.iter() {
        if known_addresses.contains_key(oracle.address.clone()) {
            panic_with_error!(e, AOCError::NonUniqueOraclesWhileDeploying);
        }

        known_addresses.set(oracle.address.clone(), ());
    }

    e.storage().instance().set(&DataKey::Oracles, &oracles)
}

pub fn get_oracles(e: &Env) -> Vec<OracleConfig> {
    e.storage().instance().get(&DataKey::Oracles).expect("Oracles must've been set")
}

// ---- Storage Types ----

/// `SEP-40` compliant oracle contract's registration input configuration
#[derive(Clone)]
#[contracttype]
pub struct OracleConfigInput {
    /// Oracle contract's address on the ledger
    pub address: Address,
    /// Indicator of whether the oracle gets the data from or out of the `Stellar` ledger.
    /// Oracles that have this set to `true` will receive a request with [`Asset::Stellar`] asset
    /// parameter first, and only if it returns [`None`] will receive [`Asset::Other`] afterwards.
    /// The opposite behavior takes place otherwise
    pub is_stellar_data_based: bool,
    /// Indicator of whether the oracle updates the price once every `resolution()` seconds or
    /// when either one of the following two conditions meets:
    /// 1. The price has diverged for more than a specified percentage
    /// from the old one;
    /// 2. The maximum time period (aka heartbeat) has passed since the previous update.
    ///
    /// Having `true` here indicates the `resolution()` based update oracles and `false` the `deviation\heartbeat` based oracles
    pub is_price_update_periodic: bool,
}

/// `SEP-40` compliant oracle contract's configuration
#[derive(Clone)]
#[contracttype]
pub struct OracleConfig {
    /// Oracle contract's address on the ledger
    pub address: Address,
    /// Number of decimals representing a fractional part of a price
    pub decimals: u32,
    /// Indicator of whether the oracle gets the data from or out of the `Stellar` ledger.
    /// Oracles that have this set to `true` will receive a request with [`Asset::Stellar`] asset
    /// parameter first, and only if it returns [`None`] will receive [`Asset::Other`] afterwards.
    /// The opposite behavior takes place otherwise
    pub is_stellar_data_based: bool,
    /// Indicator of whether the oracle updates the price once every `resolution()` seconds or
    /// when either one of the following two conditions meets:
    /// 1. The price has diverged for more than a specified percentage
    /// from the old one;
    /// 2. The maximum time period (aka heartbeat) has passed since the previous update.
    ///
    /// Having `true` here indicates the `resolution()` based update oracles and `false` the `deviation\heartbeat` based oracles
    pub is_price_update_periodic: bool,
}

// ---- TTL Bumper ----

pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
