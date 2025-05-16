use crate::constants::{
    BPS_IN_PERCENT, DEFAULT_BASE_RATE, DEFAULT_OPTIMAL_UTILIZATION_RATIO, DEFAULT_RESERVE_RATIO,
    DEFAULT_SLOPE1, DEFAULT_SLOPE2,
};
use {
    crate::error::LendingContractError,
    soroban_sdk::{contracttype, Address, Env, Map, Symbol},
};

pub type PoolAddress = Address;
pub type UserAddress = Address;

#[contracttype]
pub struct GlobalState {
    pub admin: Address,
    pub status: bool,
    pub liquidation_threshold_bps: i128,
    // TODO: Oracle address + ....
}

#[contracttype]
pub enum DataKey {
    GlobalState,
    Pool(PoolAddress),
    Obligation(UserAddress),
    Accrual,
    // TODO: We also must be able to retrieve all pools and all user addresses
}

#[contracttype]
#[derive(Debug)]
pub struct Pool {
    pub token_address: Address,
    pub token_ticker: Symbol,
    pub borrowed: i128,
    pub supply: i128,
    pub config: PoolConfig,
    // TODO: add available_supply for collateral deposits?
}

#[contracttype]
#[derive(Debug)]
pub struct PoolConfig {
    /// Positive Base Rate percentage
    pub base_rate_bps: i128,
    /// Positive Optimal Utilization Ratio percentage
    pub optimal_utilization_ratio_bps: i128,
    pub slope1: i128,
    pub slope2: i128,
    /// Non-negative Reserve Ration percentage (< 100)
    pub reserve_ratio_bps: i128,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            slope1: DEFAULT_SLOPE1,
            slope2: DEFAULT_SLOPE2,
            base_rate_bps: DEFAULT_BASE_RATE * BPS_IN_PERCENT,
            reserve_ratio_bps: DEFAULT_RESERVE_RATIO * BPS_IN_PERCENT,
            optimal_utilization_ratio_bps: DEFAULT_OPTIMAL_UTILIZATION_RATIO * BPS_IN_PERCENT,
        }
    }
}

#[contracttype]
pub struct Obligation {
    pub deposits: Map<PoolAddress, i128>,
    pub borrows: Map<PoolAddress, i128>,
}

#[contracttype]
#[derive(Debug)]
pub struct Accrual {
    pub timestamp: u64,
    pub borrow_accrual: i128,
    pub supply_accrual: i128,
}

#[allow(unused)]
pub fn read_global_state(e: &Env) -> GlobalState {
    e.storage()
        .instance()
        .get(&DataKey::GlobalState)
        .expect("Global State must be instantiated at this point")
}

pub fn write_global_state(e: &Env, global_state: &GlobalState) {
    e.storage()
        .instance()
        .set(&DataKey::GlobalState, global_state);
}

// --- Pool ---
pub fn set_pool(
    e: &Env,
    pool_address: &PoolAddress,
    token_address: &Address,
    token_ticker: &Symbol,
    config: PoolConfig,
) -> Result<(), LendingContractError> {
    e.storage().instance().set(
        &DataKey::Pool(pool_address.clone()),
        &Pool {
            token_address: token_address.clone(),
            token_ticker: token_ticker.clone(),
            supply: 0,
            borrowed: 0,
            config,
        },
    );

    Ok(())
}

// TODO
// pub fn set_pool_config(e: &Env, pool_address: &Address, interest_rate_config: PoolConfig) {

//     // Maybe, store interest rate config separately???
// }

pub fn pool_exists(e: &Env, pool_address: &PoolAddress) -> bool {
    e.storage()
        .instance()
        .has(&DataKey::Pool(pool_address.clone()))
}

pub fn get_pool(e: &Env, pool_address: &PoolAddress) -> Option<Pool> {
    e.storage()
        .instance()
        .get(&DataKey::Pool(pool_address.clone()))
}

pub fn get_pool_ticker(
    e: &Env,
    pool_address: &PoolAddress,
) -> Result<Symbol, LendingContractError> {
    let pool = get_pool(e, pool_address).ok_or(LendingContractError::PoolDoesNotExist)?;

    Ok(pool.token_ticker)
}

pub fn set_pool_data(e: &Env, pool_address: &Address, pool_data: &Pool) {
    e.storage()
        .instance()
        .set(&DataKey::Pool(pool_address.clone()), pool_data);
}

pub(crate) fn adjust_pool_borrowed(
    e: &Env,
    pool_address: &PoolAddress,
    amount: i128,
) -> Result<(), LendingContractError> {
    let mut pool = get_pool(e, pool_address).ok_or(LendingContractError::PoolDoesNotExist)?;
    pool.borrowed = pool
        .borrowed
        .checked_add(amount)
        .ok_or(LendingContractError::OverOrUnderflow)?;
    set_pool_data(e, pool_address, &pool);

    Ok(())
}

pub(crate) fn adjust_pool_supply(
    e: &Env,
    pool_address: &PoolAddress,
    amount: i128,
) -> Result<(), LendingContractError> {
    let mut pool = get_pool(e, pool_address).ok_or(LendingContractError::PoolDoesNotExist)?;
    pool.supply = pool
        .supply
        .checked_add(amount)
        .ok_or(LendingContractError::OverOrUnderflow)?;
    set_pool_data(e, pool_address, &pool);

    Ok(())
}

// --- Obligation ---
pub fn set_obligation(e: &Env, user: &Address, obligation: &Obligation) {
    e.storage()
        .instance()
        .set(&DataKey::Obligation(user.clone()), obligation);
}

pub fn get_obligation(e: &Env, user: &Address) -> Option<Obligation> {
    e.storage()
        .instance()
        .get(&DataKey::Obligation(user.clone()))
}

pub fn adjust_deposit(
    e: &Env,
    user: &Address,
    pool_address: &PoolAddress,
    amount: i128,
) -> Result<i128, LendingContractError> {
    let mut obligation = get_obligation(e, user).unwrap_or(Obligation {
        deposits: Map::new(e),
        borrows: Map::new(e),
    });
    let pool_obligation_deposit = obligation.deposits.get(pool_address.clone()).unwrap_or(0);

    let new_deposit_amount = pool_obligation_deposit
        .checked_add(amount)
        .ok_or(LendingContractError::OverOrUnderflow)?;
    obligation
        .deposits
        .set(pool_address.clone(), new_deposit_amount);
    set_obligation(e, user, &obligation);

    Ok(new_deposit_amount)
}

pub fn adjust_borrow(
    e: &Env,
    user: &Address,
    pool_address: &PoolAddress,
    amount: i128,
) -> Result<i128, LendingContractError> {
    let mut obligation = get_obligation(e, user).unwrap_or(Obligation {
        deposits: Map::new(e),
        borrows: Map::new(e),
    });
    let pool_obligation_borrow = obligation.borrows.get(pool_address.clone()).unwrap_or(0);

    let new_borrow_amount = pool_obligation_borrow
        .checked_add(amount)
        .ok_or(LendingContractError::OverOrUnderflow)?;
    obligation
        .borrows
        .set(pool_address.clone(), new_borrow_amount);
    set_obligation(e, user, &obligation);

    Ok(new_borrow_amount)
}

pub fn deposit_exists(
    e: &Env,
    user: &Address,
    pool_address: &PoolAddress,
) -> Result<bool, LendingContractError> {
    let Obligation {
        deposits,
        borrows: _,
    } = get_obligation(e, user).ok_or(LendingContractError::ObligationDoesNotExist)?;

    Ok(deposits.contains_key(pool_address.clone()))
}

pub fn set_accrual(e: &Env, accrual: &Accrual) {
    e.storage().persistent().set(&DataKey::Accrual, accrual)
}

pub fn get_accrual(e: &Env) -> Option<Accrual> {
    e.storage().persistent().get(&DataKey::Accrual)
}
