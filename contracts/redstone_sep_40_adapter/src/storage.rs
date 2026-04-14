use sep_40_oracle::Asset;
use soroban_sdk::{Address, Env, Vec, contracttype};

use crate::constants::*;

#[contracttype]
pub enum DataKey {
    Admin,
    BaseAsset,
    Decimals,
    /// Vec<Asset>: all registered assets, set once at construction
    Assets,
    /// Per-token persistent key: token address → RedStone price feed contract address
    Feed(Address),
}

pub fn set_base_asset(e: &Env, base_asset: &Asset) {
    e.storage().instance().set(&DataKey::BaseAsset, base_asset);
}

pub fn get_base_asset(e: &Env) -> Asset {
    e.storage().instance().get(&DataKey::BaseAsset).expect("Base asset must've been set")
}

pub fn set_decimals(e: &Env, decimals: u32) {
    e.storage().instance().set(&DataKey::Decimals, &decimals);
}

pub fn get_decimals(e: &Env) -> u32 {
    e.storage().instance().get(&DataKey::Decimals).expect("Decimals must've been set")
}

pub fn set_assets(e: &Env, assets: &Vec<Asset>) {
    e.storage().instance().set(&DataKey::Assets, assets);
}

pub fn get_assets(e: &Env) -> Vec<Asset> {
    e.storage().instance().get(&DataKey::Assets).expect("Assets must've been set")
}

pub fn set_feed(e: &Env, token_address: &Address, price_feed: &Address) {
    let key = DataKey::Feed(token_address.clone());
    e.storage().persistent().set(&key, price_feed);
    e.storage().persistent().extend_ttl(&key, INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

pub fn get_feed(e: &Env, token_address: &Address) -> Option<Address> {
    let key = DataKey::Feed(token_address.clone());
    let feed: Option<Address> = e.storage().persistent().get(&key);
    if feed.is_some() {
        e.storage().persistent().extend_ttl(&key, INSTANCE_THRESHOLD, INSTANCE_BUMP);
    }

    feed
}

pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
