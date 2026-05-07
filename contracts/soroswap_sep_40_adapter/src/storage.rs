use soroban_sdk::{Address, Env, contracttype};

use crate::constants::*;

#[contracttype]
pub enum DataKey {
    Admin,
    Tokens,
}

pub fn set_admin(e: &Env, admin: &Address) {
    let key = DataKey::Admin;

    e.storage().instance().set(&key, admin);
}

pub fn get_admin(e: &Env) -> Option<Address> {
    extend_instance(e);
    let key = DataKey::Admin;

    e.storage().instance().get(&key)
}

/// Instance storage bumper
fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
