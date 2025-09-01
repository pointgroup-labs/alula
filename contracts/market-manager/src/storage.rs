use soroban_sdk::{contracttype, panic_with_error, Address, Env};

use crate::MMError;

#[derive(Debug)]
#[contracttype]
pub enum DataKey {
    ManagerAdmin, // DAO address?
}

pub fn set_manager_admin(e: &Env, admin: &Address) {
    let key = DataKey::ManagerAdmin;

    e.storage().instance().set(&key, admin);
}

pub fn get_manager_admin(e: &Env) -> Address {
    let key = DataKey::ManagerAdmin;
    let admin = e
        .storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| panic_with_error!(e, MMError::InternalError));

    admin
}

// TTL bumpers
