use soroban_sdk::{Address, Env, contractevent};

// -- SEP-41 events (shape must stay byte-compatible with the Stellar Asset Contract) --

#[contractevent(topics = ["transfer"], data_format = "single-value")]
pub struct TransferEvent {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

#[contractevent(topics = ["approve"], data_format = "vec")]
pub struct ApproveEvent {
    #[topic]
    pub from: Address,
    #[topic]
    pub spender: Address,
    pub amount: i128,
    pub expiration_ledger: u32,
}

// -- Vault lifecycle events --
//
// Modelled on SEP-56's `Deposit` and `Withdraw` for familiarity. The vault does not claim SEP-56
// conformance, but there is no reason to invent a different shape for the same two operations

#[contractevent]
pub struct Deposit {
    #[topic]
    pub operator: Address,
    #[topic]
    pub from: Address,
    #[topic]
    pub receiver: Address,
    pub assets: i128,
    pub shares: i128,
}

#[contractevent]
pub struct Withdraw {
    #[topic]
    pub operator: Address,
    #[topic]
    pub receiver: Address,
    #[topic]
    pub owner: Address,
    pub assets: i128,
    pub shares: i128,
}

// -- Administrative events --

#[contractevent]
pub struct DepositsPauseSet {
    #[topic]
    pub admin: Address,
    pub paused: bool,
}

#[contractevent]
pub struct AdminProposed {
    #[topic]
    pub current_admin: Address,
    #[topic]
    pub proposed_admin: Address,
}

#[contractevent]
pub struct AdminUpdated {
    #[topic]
    pub old_admin: Address,
    #[topic]
    pub new_admin: Address,
}

pub fn transfer(e: &Env, from: Address, to: Address, amount: i128) {
    TransferEvent { from, to, amount }.publish(e);
}

pub fn approve(e: &Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
    ApproveEvent { from, spender, amount, expiration_ledger }.publish(e);
}

pub fn deposit(
    e: &Env,
    operator: Address,
    from: Address,
    receiver: Address,
    assets: i128,
    shares: i128,
) {
    Deposit { operator, from, receiver, assets, shares }.publish(e);
}

pub fn withdraw(
    e: &Env,
    operator: Address,
    receiver: Address,
    owner: Address,
    assets: i128,
    shares: i128,
) {
    Withdraw { operator, receiver, owner, assets, shares }.publish(e);
}

pub fn deposits_pause_set(e: &Env, admin: Address, paused: bool) {
    DepositsPauseSet { admin, paused }.publish(e);
}

pub fn admin_proposed(e: &Env, current_admin: Address, proposed_admin: Address) {
    AdminProposed { current_admin, proposed_admin }.publish(e);
}

pub fn admin_updated(e: &Env, old_admin: Address, new_admin: Address) {
    AdminUpdated { old_admin, new_admin }.publish(e);
}
