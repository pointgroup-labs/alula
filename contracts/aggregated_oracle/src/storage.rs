use sep_40_oracle::Asset;
use soroban_sdk::{contracttype, vec as svec, Address, Env, Vec};

use crate::helpers::AssetsSet;

/// Average ledger close time on Stellar
pub const SECONDS_PER_LEDGER: u32 = 6;
pub const SECONDS_PER_DAY: u32 = 24 * 60 * 60;
pub const LEDGERS_PER_DAY: u32 = SECONDS_PER_DAY / SECONDS_PER_LEDGER;

pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = INSTANCE_THRESHOLD + LEDGERS_PER_DAY;

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

/// Adds asset to the set of assets
pub fn add_asset(e: &Env, asset: &Asset) {
    extend_instance_storage(e);

    let mut assets: AssetsSet = e
        .storage()
        .instance()
        .get(&DataKey::Assets)
        .unwrap_or(AssetsSet::new(&e));
    assets.insert(asset.clone());
}

/// Removes asset from the set of assets
pub fn remove_asset(e: &Env, asset: &Asset) {
    extend_instance_storage(e);

    let Some(mut assets): Option<AssetsSet> = e.storage().instance().get(&DataKey::Assets) else {
        return;
    };
    assets.remove(asset.clone());
}

/// Returns a [`Vec`] of all added assets
pub fn get_assets(e: &Env) -> Vec<Asset> {
    extend_instance_storage(e);

    if let Some(assets) = e.storage().instance().get(&DataKey::Assets) {
        let assets: AssetsSet = assets;
        assets.entries()
    } else {
        svec![e]
    }
}

/// Instance bumper
pub fn extend_instance_storage(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
