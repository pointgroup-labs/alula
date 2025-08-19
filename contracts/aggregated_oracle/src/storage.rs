use soroban_sdk::{contracttype, Address, Env, Vec};

/// Average ledger close time on Stellar
pub const SECONDS_PER_LEDGER: u32 = 6;
pub const SECONDS_PER_DAY: u32 = 24 * 60 * 60;
pub const LEDGERS_PER_DAY: u32 = SECONDS_PER_DAY / SECONDS_PER_LEDGER;

pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = INSTANCE_THRESHOLD + LEDGERS_PER_DAY;

#[contracttype]
enum DataKey {
    Admin,
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(e: &Env) -> Option<Address> {
    extend_instance_storage(e);

    e.storage().instance().get(&DataKey::Admin)
}

/// Instance bumper
pub fn extend_instance_storage(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
