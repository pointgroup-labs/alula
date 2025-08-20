use sep_40_oracle::Asset;
use soroban_sdk::{Address, Env, Map, Vec, contracttype, vec as svec};

use crate::constants::{INSTANCE_BUMP, INSTANCE_THRESHOLD};

#[contracttype]
pub enum DataKey {
    Admin,
    Assets,
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(e: &Env) -> Option<Address> {
    extend_instance_storage(e);

    e.storage().instance().get(&DataKey::Admin)
}

/// Adds asset to the storage
pub fn add_asset(e: &Env, asset: &Asset, token_address: &Address) {
    extend_instance_storage(e);

    let mut assets: Map<Asset, Address> = e
        .storage()
        .instance()
        .get(&DataKey::Assets)
        .unwrap_or(Map::new(e));
    assets.set(asset.clone(), token_address.clone());

    e.storage().instance().set(&DataKey::Assets, &assets);
}

/// Removes asset from the storage
pub fn remove_asset(e: &Env, asset: &Asset) {
    extend_instance_storage(e);

    if let Some(assets) = e.storage().instance().get(&DataKey::Assets) {
        let mut assets: Map<Asset, Address> = assets;
        assets.remove(asset.clone());

        e.storage().instance().set(&DataKey::Assets, &assets);
    };
}

/// # Returns
/// [`Vec`] of all added assets
pub fn get_assets(e: &Env) -> Vec<Asset> {
    extend_instance_storage(e);

    if let Some(assets) = e.storage().instance().get(&DataKey::Assets) {
        let assets: Map<Asset, Address> = assets;

        assets.keys()
    } else {
        svec![e]
    }
}

/// # Returns
/// `Some(Address)` if asset exists in the storage. `None` otherwise
pub fn get_token_address(e: &Env, asset: &Asset) -> Option<Address> {
    extend_instance_storage(e);

    let assets: Map<Asset, Address> = e.storage().instance().get(&DataKey::Assets)?;

    assets.get(asset.clone())
}

/// Instance bumper
pub fn extend_instance_storage(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
