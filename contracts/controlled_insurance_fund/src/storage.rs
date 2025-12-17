use insurance_fund_trait::CoverageStatus;
use soroban_sdk::{Address, Env, contracttype, panic_with_error};

use crate::error::ContractError;

pub const LEDGERS_PER_DAY: u32 = (24 * 60 * 60) / 6; // NB: Assuming 6 seconds per ledger
pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = 41 * LEDGERS_PER_DAY;

#[contracttype]
pub enum DataKey {
    Admin,
    Market,
    Request(u64),
    RequestsCounter,
    MustClaim,
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
    e.storage().instance().set(&DataKey::RequestsCounter, &0);
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

    e.storage().persistent().set(&DataKey::Request(request_id), &request);

    request_id
}

pub fn get_request(e: &Env, request_id: u64) -> Option<Request> {
    e.storage().persistent().get(&DataKey::Request(request_id))
}

pub fn update_request(e: &Env, request_id: u64, request: Request) {
    let request_key = DataKey::Request(request_id);

    if e.storage().persistent().has(&request_key) {
        e.storage().persistent().set(&request_key, &request);
    } else {
        panic_with_error!(e, ContractError::RequestDoesNotExist);
    }
}

pub fn remove_request(e: &Env, request_id: u64) {
    let request_key = DataKey::Request(request_id);

    if e.storage().persistent().has(&request_key) {
        e.storage().persistent().remove(&request_key);
    } else {
        panic_with_error!(e, ContractError::RequestDoesNotExist);
    }
}

// -- MustClaim --

pub fn set_must_claim(e: &Env, value: bool) {
    e.storage().instance().set(&DataKey::MustClaim, &value);
}

pub fn get_must_claim(e: &Env) -> bool {
    e.storage().instance().get(&DataKey::MustClaim).expect("MustClaim must be set")
}

// ---- TTL Bumpers ----

pub fn extend_instance_storage(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
