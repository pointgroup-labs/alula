use insurance_fund_interface::CoverageStatus;
use soroban_sdk::{Address, Env, contracttype, panic_with_error};

use crate::error::ContractError;

// -- TTL Configuration --

pub const LEDGERS_PER_DAY: u32 = (24 * 60 * 60) / 6; // NB: Assuming 6 seconds per ledger

pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = 41 * LEDGERS_PER_DAY;

pub const PERSISTENT_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const PERSISTENT_BUMP: u32 = 41 * LEDGERS_PER_DAY;

#[contracttype]
pub enum DataKey {
    Admin,
    Market,
    Request(u64),
    RequestsCounter,
    LockedAmount(Address),
}

#[contracttype]
pub struct Request {
    pub token: Address,
    pub amount: i128,
    pub status: CoverageStatus,
}

impl Request {
    pub fn new(token: Address, amount: i128) -> Self {
        Self { token, amount, status: CoverageStatus::Pending }
    }
}

// -- Market --

pub fn set_market(e: &Env, market: Address) {
    e.storage().instance().set(&DataKey::Market, &market);
}

pub fn get_market(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Market).expect("Market must be set upon usage")
}

// -- Admin --

pub fn set_admin(e: &Env, admin: Address) {
    e.storage().instance().set(&DataKey::Admin, &admin);
}

pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).expect("Admin must be set")
}

// -- RequestsCounter --

pub fn init_requests_counter(e: &Env) {
    e.storage().instance().set(&DataKey::RequestsCounter, &0u64);
}

fn next_requests_counter(e: &Env) -> u64 {
    let counter: u64 =
        e.storage().instance().get(&DataKey::RequestsCounter).expect("RequestsCounter must be set");

    e.storage().instance().set(&DataKey::RequestsCounter, &(counter + 1));

    counter
}

// -- Request --

pub fn set_request(e: &Env, request: Request) -> u64 {
    let request_id = next_requests_counter(e);
    let key = DataKey::Request(request_id);

    e.storage().persistent().set(&key, &request);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_THRESHOLD, PERSISTENT_BUMP);

    request_id
}

pub fn get_request(e: &Env, request_id: u64) -> Option<Request> {
    let key = DataKey::Request(request_id);
    let result = e.storage().persistent().get::<DataKey, Request>(&key);

    if result.is_some() {
        e.storage().persistent().extend_ttl(&key, PERSISTENT_THRESHOLD, PERSISTENT_BUMP);
    }

    result
}

pub fn update_request(e: &Env, request_id: u64, request: Request) {
    let key = DataKey::Request(request_id);

    if e.storage().persistent().has(&key) {
        e.storage().persistent().set(&key, &request);
        e.storage().persistent().extend_ttl(&key, PERSISTENT_THRESHOLD, PERSISTENT_BUMP);
    } else {
        panic_with_error!(e, ContractError::RequestDoesNotExist);
    }
}

pub fn remove_request(e: &Env, request_id: u64) {
    let key = DataKey::Request(request_id);

    if e.storage().persistent().has(&key) {
        e.storage().persistent().remove(&key);
    } else {
        panic_with_error!(e, ContractError::RequestDoesNotExist);
    }
}

// -- LockedAmount --

pub fn get_locked_amount(e: &Env, token: &Address) -> i128 {
    let key = DataKey::LockedAmount(token.clone());

    match e.storage().persistent().get::<DataKey, i128>(&key) {
        Some(amount) => {
            e.storage().persistent().extend_ttl(&key, PERSISTENT_THRESHOLD, PERSISTENT_BUMP);

            amount
        }
        None => 0,
    }
}

pub fn set_locked_amount(e: &Env, token: &Address, amount: i128) {
    let key = DataKey::LockedAmount(token.clone());

    if amount == 0 {
        e.storage().persistent().remove(&key);
    } else {
        e.storage().persistent().set(&key, &amount);
        e.storage().persistent().extend_ttl(&key, PERSISTENT_THRESHOLD, PERSISTENT_BUMP);
    }
}

// ---- TTL Bumpers ----

pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
