use sep_40_oracle::Asset;
use soroban_sdk::{contracttype, Address, Env};

/// Average ledger close time on Stellar
pub const SECONDS_PER_LEDGER: u32 = 6;
pub const SECONDS_PER_DAY: u32 = 24 * 60 * 60;
pub const LEDGERS_PER_DAY: u32 = SECONDS_PER_DAY / SECONDS_PER_LEDGER;

pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = INSTANCE_THRESHOLD + LEDGERS_PER_DAY;

#[contracttype]
enum DataKey {
    Admin,
    Assets, // Must be a HashSet, no? HashSet of symbols
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(e: &Env) -> Option<Address> {
    extend_instance_storage(e);

    e.storage().instance().get(&DataKey::Admin)
}

pub fn add_asset(e: &Env, asset: &Asset) {
    extend_instance_storage(e);

    // let assets:
}

pub fn remove_asset() {}

/// Instance bumper
pub fn extend_instance_storage(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
