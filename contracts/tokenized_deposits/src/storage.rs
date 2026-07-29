use soroban_sdk::{Address, Env, String, contracttype};

use crate::{constants::*, error::TDError};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    // Address allowed to pause the vault and rotate itself. Not able to touch user funds
    Admin,
    // Pending admin set by `propose_new_admin`, claimed via `accept_proposed_admin`
    PendingAdmin,
    // The underlying SEP-41 asset the vault accepts and pays out
    Asset,
    // The lending market the vault supplies into
    Market,
    // The market pool (keyed by its address) the vault's deposits live in
    Pool,
    // Share token metadata
    ShareTokenMetadata,
    // The virtual decimals offset chosen at construction, used by every conversion
    DecimalsOffset,
    // Total shares in circulation
    TotalSupply,
    // Per-holder share balance
    Balance(Address),
    // Allowance over shares, denominated in shares
    Allowance(AllowanceKey),
    // When set, deposits are rejected. Withdrawals always remain open so that a paused vault can
    // never trap user funds
    DepositsPaused,
}

#[derive(Clone)]
#[contracttype]
pub struct AllowanceKey {
    pub from: Address,
    pub spender: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct ShareTokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

// -- TTL bumpers --

pub fn bump_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

// No bump_shared/individual?????
// I guess no...

// So, what if there are plenty of users?????
// Why shou

// I think we must have a persistant storage used here...

// -- Admin --

pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).expect("Admin must be set")
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_pending_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::PendingAdmin)
}

pub fn set_pending_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::PendingAdmin, admin);
}

pub fn clear_pending_admin(e: &Env) {
    e.storage().instance().remove(&DataKey::PendingAdmin);
}

pub fn require_admin(e: &Env) -> Result<Address, TDError> {
    let admin = get_admin(e);
    admin.require_auth();

    Ok(admin)
}

// -- Wiring --

pub fn get_asset(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Asset).unwrap()
}

pub fn set_asset(e: &Env, asset: &Address) {
    e.storage().instance().set(&DataKey::Asset, asset);
}

pub fn get_market(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Market).unwrap()
}

pub fn set_market(e: &Env, market: &Address) {
    e.storage().instance().set(&DataKey::Market, market);
}

pub fn get_pool(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Pool).unwrap()
}

pub fn set_pool(e: &Env, pool: &Address) {
    e.storage().instance().set(&DataKey::Pool, pool);
}

pub fn get_metadata(e: &Env) -> ShareTokenMetadata {
    e.storage().instance().get(&DataKey::ShareTokenMetadata).unwrap()
}

pub fn set_metadata(e: &Env, metadata: &ShareTokenMetadata) {
    e.storage().instance().set(&DataKey::ShareTokenMetadata, metadata);
}

pub fn get_decimals_offset(e: &Env) -> u32 {
    e.storage().instance().get(&DataKey::DecimalsOffset).unwrap()
}

pub fn set_decimals_offset(e: &Env, offset: &u32) {
    e.storage().instance().set(&DataKey::DecimalsOffset, offset);
}

// -- Pause --

pub fn get_deposits_paused(e: &Env) -> bool {
    e.storage().instance().get(&DataKey::DepositsPaused).unwrap_or(false)
}

pub fn set_deposits_paused(e: &Env, paused: bool) {
    e.storage().instance().set(&DataKey::DepositsPaused, &paused);
}

// -- Shares --

pub fn get_total_supply(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::TotalSupply).unwrap_or(0)
}

pub fn set_total_supply(e: &Env, total: &i128) {
    e.storage().instance().set(&DataKey::TotalSupply, total);
}

pub fn get_balance(e: &Env, address: &Address) -> i128 {
    e.storage().persistent().get(&DataKey::Balance(address.clone())).unwrap_or(0)
}

pub fn set_balance(e: &Env, address: &Address, balance: &i128) {
    let key = DataKey::Balance(address.clone());

    if *balance == 0 {
        e.storage().persistent().remove(&key);

        return;
    }

    e.storage().persistent().set(&key, balance);
    e.storage().persistent().extend_ttl(&key, BALANCE_THRESHOLD, BALANCE_BUMP);
}

// -- Allowance --

pub fn get_allowance(e: &Env, from: &Address, spender: &Address) -> AllowanceValue {
    let key = DataKey::Allowance(AllowanceKey { from: from.clone(), spender: spender.clone() });

    match e.storage().temporary().get::<_, AllowanceValue>(&key) {
        // An allowance past its expiration ledger is indistinguishable from no allowance | is this by the standard?
        // I guess
        Some(a) if a.expiration_ledger >= e.ledger().sequence() => a,
        _ => AllowanceValue { amount: 0, expiration_ledger: 0 },
    }
}

pub fn set_allowance(e: &Env, from: &Address, spender: &Address, allowance: &AllowanceValue) {
    let key = DataKey::Allowance(AllowanceKey { from: from.clone(), spender: spender.clone() });

    e.storage().temporary().set(&key, allowance);

    // Keep the entry alive exactly as long as the allowance is valid, and no longer
    if allowance.amount > 0 && allowance.expiration_ledger > e.ledger().sequence() {
        let live_for = allowance.expiration_ledger - e.ledger().sequence(); // safe
        e.storage().temporary().extend_ttl(&key, live_for, live_for); // I assume it works this way
    }
}
