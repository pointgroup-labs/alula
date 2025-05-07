use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::error::LendingContractError;

#[contracttype]
pub struct GlobalState {
    pub admin: Address,
    pub status: bool,
}

pub type PoolAddress = Address;
pub type UserAddress = Address;

#[contracttype]
pub enum DataKey {
    GlobalState,
    Pool(PoolAddress),
    Obligation(UserAddress),
}

#[contracttype]
pub struct PoolConfig {
    pub token_address: Address,
    pub liquidation_threshold: i128,
}

#[contracttype]
pub struct ObligationBorrow {
    pub pool_address: Address,
    pub amount: i128,
    // @TODO: Should there be some currency info?
}

#[contracttype]
pub struct ObligationDeposit {
    pub pool_address: Address,
    pub amount: i128,
}

#[contracttype]
pub struct Obligation {
    pub deposits: Vec<ObligationDeposit>,
    pub borrows: Vec<ObligationBorrow>,
}

#[allow(unused)]
pub(crate) fn read_global_state(e: &Env) -> GlobalState {
    e.storage().instance().get(&DataKey::GlobalState).unwrap() // @TODO: unwraps
}

pub(crate) fn write_global_state(e: &Env, global_state: &GlobalState) {
    e.storage()
        .instance()
        .set(&DataKey::GlobalState, global_state);
}

pub(crate) fn pool_exists(e: &Env, pool_address: &PoolAddress) -> bool {
    e.storage()
        .instance()
        .has(&DataKey::Pool(pool_address.clone()))
}

pub(crate) fn initialize_pool(
    e: &Env,
    pool_address: &PoolAddress,
    token_address: Address,
    liquidation_threshold: i128,
) {
    e.storage().instance().set(
        &DataKey::Pool(pool_address.clone()),
        &PoolConfig {
            token_address,
            liquidation_threshold,
        },
    )
}

pub(crate) fn get_pool_config(
    e: &Env,
    pool_address: PoolAddress,
) -> Result<PoolConfig, LendingContractError> {
    Ok(e.storage()
        .instance()
        .get(&DataKey::Pool(pool_address))
        .unwrap()) // I don't like these `unwraps()` here
}
