use soroban_sdk::{contracttype, Address, Env, String, Symbol, Vec};

#[contracttype]
pub struct PoolConfig {
    pub admin: Address,
    pub liquidation_threshold: i128,
}

#[contracttype]
pub struct ObligationBorrow {
    pub pool_name: Symbol,
    pub currency: Currency, // should this be duplicated here and in [`PoolConfig`]?
    pub amount: i128,
}

#[contracttype]
pub struct ObligationDeposit {
    pub pool_name: String,
    pub currency: Currency,
    pub amount: i128,
}

#[contracttype]
pub struct Currency {
    pub token_address: Address,
    pub ticker: Symbol,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Pool(Address, Symbol), // should this be `String` or `Symbol`?
    User(Address),
}

#[contracttype]
pub struct Obligation {
    pub owner: Address,
    pub deposits: Vec<ObligationDeposit>,
    pub borrows: Vec<ObligationBorrow>,
}

pub(crate) fn write_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub(crate) fn read_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).unwrap() // safe unwrap, right?
}

pub(crate) fn pool_exists(e: &Env, token_address: Address, pool_name: Symbol) -> bool {
    e.storage()
        .instance()
        .has(&DataKey::Pool(token_address, pool_name))
}

pub(crate) fn initialize_pool(
    e: &Env,
    token_address: Address,
    pool_name: Symbol,
    admin: Address,
    liquidation_threshold: i128,
) {
    e.storage().instance().set(
        &DataKey::Pool(token_address, pool_name),
        &PoolConfig {
            admin,
            liquidation_threshold,
        },
    )
}

pub(crate) fn read_obligation() {
    todo!()
}
