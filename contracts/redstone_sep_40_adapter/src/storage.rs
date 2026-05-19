use sep_40_oracle::Asset;
use soroban_sdk::{Address, Env, Vec, contracttype, panic_with_error};

use crate::{constants::*, error::RS40ACError};

#[contracttype]
pub enum DataKey {
    Admin,
    PendingAdmin,
    BaseAsset,
    Decimals,
    Resolution,
    /// Vec<Asset>: all registered assets
    Assets,
    /// Forward: token address → RedStone price feed contract address
    Feed(Address),
    /// Reverse: price feed contract address → token address (duplicate-feed guard)
    FeedToken(Address),
}

pub fn set_base_asset(e: &Env, base_asset: &Asset) {
    e.storage().instance().set(&DataKey::BaseAsset, base_asset);
}
pub fn get_base_asset(e: &Env) -> Asset {
    e.storage()
        .instance()
        .get(&DataKey::BaseAsset)
        .unwrap_or_else(|| panic_with_error!(e, RS40ACError::InternalError))
}

pub fn set_decimals(e: &Env, decimals: u32) {
    e.storage().instance().set(&DataKey::Decimals, &decimals);
}
pub fn get_decimals(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&DataKey::Decimals)
        .unwrap_or_else(|| panic_with_error!(e, RS40ACError::InternalError))
}

pub fn set_assets(e: &Env, assets: &Vec<Asset>) {
    e.storage().instance().set(&DataKey::Assets, assets);
}
pub fn get_assets(e: &Env) -> Vec<Asset> {
    e.storage()
        .instance()
        .get(&DataKey::Assets)
        .unwrap_or_else(|| panic_with_error!(e, RS40ACError::InternalError))
}

pub fn set_feed(e: &Env, token_address: &Address, price_feed: &Address) {
    let fwd = DataKey::Feed(token_address.clone());
    e.storage().persistent().set(&fwd, price_feed);
    extend_persistent(e, &fwd);

    let rev = DataKey::FeedToken(price_feed.clone());
    e.storage().persistent().set(&rev, token_address);
    extend_persistent(e, &rev);
}

pub fn get_feed(e: &Env, token_address: &Address) -> Option<Address> {
    let key = DataKey::Feed(token_address.clone());

    let feed: Option<Address> = e.storage().persistent().get(&key);
    if feed.is_some() {
        extend_persistent(e, &key);
    }

    feed
}

pub fn get_feed_token(e: &Env, price_feed: &Address) -> Option<Address> {
    let key = DataKey::FeedToken(price_feed.clone());

    let token: Option<Address> = e.storage().persistent().get(&key);
    if token.is_some() {
        extend_persistent(e, &key);
    }

    token
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| panic_with_error!(e, RS40ACError::InternalError))
}

pub fn set_pending_admin(e: &Env, pending_admin: &Address) {
    e.storage().instance().set(&DataKey::PendingAdmin, pending_admin);
}

pub fn get_pending_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::PendingAdmin)
}

pub fn remove_pending_admin(e: &Env) {
    e.storage().instance().remove(&DataKey::PendingAdmin);
}

pub fn set_resolution(e: &Env, resolution: u32) {
    e.storage().instance().set(&DataKey::Resolution, &resolution);
}

pub fn get_resolution(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&DataKey::Resolution)
        .unwrap_or_else(|| panic_with_error!(e, RS40ACError::InternalError))
}

pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
fn extend_persistent(e: &Env, key: &DataKey) {
    e.storage().persistent().extend_ttl(key, PERSISTENT_THRESHOLD, PERSISTENT_BUMP);
}
