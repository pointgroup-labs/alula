use {
    crate::constants::{
        LCError, BPS_IN_PERCENT, DEFAULT_BASE_RATE, DEFAULT_OPTIMAL_UTILIZATION_RATIO,
        DEFAULT_RESERVE_RATIO, DEFAULT_SLOPE1, DEFAULT_SLOPE2, INDIVIDUAL_BUMP,
        INDIVIDUAL_THRESHOLD, INSTANCE_BUMP, INSTANCE_THRESHOLD, SHARED_BUMP, SHARED_THRESHOLD,
    },
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
    pub accrual: Accrual,
}

#[contracttype]
#[derive(Debug)]
pub struct PoolConfig {
    /// Positive Base Rate percentage
    pub base_rate_bps: i128, // TODO: Not bps anymore
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
    pub deposits: Map<PoolAddress, ObligationPosition>,
    pub borrows: Map<PoolAddress, ObligationPosition>,
}

#[contracttype]
pub struct ObligationPosition {
    pub amount: i128,
    /// The numerical value that is used to determine the scaling factor required for updating the position amount
    /// with interest i.e. (current_accrual \ last_accrual) * amount = new_amount
    pub last_accrual: i128,
}

#[contracttype]
#[derive(Debug)]
pub struct Accrual {
    pub timestamp: u64,
    pub borrow_accrual: i128,
    pub supply_accrual: i128,
}

/// Instance bumper
pub fn extend_instance_storage(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

/// Persistent individual resource bumper
pub fn extend_individual_storage(e: &Env, key: &DataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, INDIVIDUAL_THRESHOLD, INDIVIDUAL_BUMP);
}

/// Persistent shared resource bumper
pub fn extend_shared_storage(e: &Env, key: &DataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, SHARED_THRESHOLD, SHARED_BUMP);
}

#[allow(unused)]
pub fn get_global_state(e: &Env) -> GlobalState {
    extend_instance_storage(e);

    e.storage()
        .instance()
        .get(&DataKey::GlobalState)
        .expect("Global State must be instantiated at this point")
}

pub fn set_global_state(e: &Env, global_state: &GlobalState) {
    e.storage()
        .instance()
        .set(&DataKey::GlobalState, global_state);

    extend_instance_storage(e);
}

// --- Pool ---
pub fn set_pool(e: &Env, pool_address: &Address, pool: &Pool) -> Result<(), LCError> {
    e.storage()
        .persistent()
        .set(&DataKey::Pool(pool_address.clone()), pool);

    extend_shared_storage(e, &DataKey::Pool(pool_address.clone()));

    Ok(())
}

// TODO
// pub fn set_pool_config(e: &Env, pool_address: &Address, interest_rate_config: PoolConfig) {

//     // Maybe, store interest rate config separately???
// }

pub fn pool_exists(e: &Env, pool_address: &Address) -> bool {
    let res = e
        .storage()
        .persistent()
        .has(&DataKey::Pool(pool_address.clone()));

    if res {
        extend_shared_storage(e, &DataKey::Pool(pool_address.clone()));
    }

    res
}

pub fn get_pool(e: &Env, pool_address: &Address) -> Option<Pool> {
    let res = e
        .storage()
        .persistent()
        .get(&DataKey::Pool(pool_address.clone()));

    if res.is_some() {
        extend_shared_storage(e, &DataKey::Pool(pool_address.clone()));
    }

    res
}

pub fn get_pool_ticker(e: &Env, pool_address: &Address) -> Result<Symbol, LCError> {
    let pool = get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;

    Ok(pool.token_ticker)
}

pub fn set_pool_data(e: &Env, pool_address: &Address, pool_data: &Pool) {
    e.storage()
        .persistent()
        .set(&DataKey::Pool(pool_address.clone()), pool_data);

    extend_shared_storage(e, &DataKey::Pool(pool_address.clone()));
}

pub(crate) fn set_pool_borrowed(
    e: &Env,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    let mut pool = get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;

    pool.borrowed = pool
        .borrowed
        .checked_add(amount)
        .ok_or(LCError::OverOrUnderflow)?;
    set_pool_data(e, pool_address, &pool);

    Ok(())
}

pub(crate) fn set_pool_supply(
    e: &Env,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    let mut pool = get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;

    pool.supply = pool
        .supply
        .checked_add(amount)
        .ok_or(LCError::OverOrUnderflow)?;
    set_pool_data(e, pool_address, &pool);

    Ok(())
}

// --- Obligation ---
pub fn set_obligation(e: &Env, user: &Address, obligation: &Obligation) {
    e.storage()
        .persistent()
        .set(&DataKey::Obligation(user.clone()), obligation);

    extend_individual_storage(e, &DataKey::Obligation(user.clone()));
}

pub fn get_obligation(e: &Env, user: &Address) -> Option<Obligation> {
    let res = e
        .storage()
        .persistent()
        .get(&DataKey::Obligation(user.clone()));

    if res.is_some() {
        extend_individual_storage(e, &DataKey::Obligation(user.clone()));
    }

    res
}

pub fn set_obligation_deposit(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<i128, LCError> {
    let mut obligation = get_obligation(e, user).unwrap_or(Obligation {
        deposits: Map::new(e),
        borrows: Map::new(e),
    });

    // TODO: Refactor
    let pool_accrual = get_pool(e, pool_address)
        .expect("Pool must exist at this point")
        .accrual;
    let mut pool_obligation_deposit =
        obligation
            .deposits
            .get(pool_address.clone())
            .unwrap_or(ObligationPosition {
                amount: 0,
                last_accrual: pool_accrual.supply_accrual,
            });

    let new_deposit_amount = pool_obligation_deposit
        .amount
        .checked_add(amount)
        .ok_or(LCError::OverOrUnderflow)?;
    pool_obligation_deposit.amount = new_deposit_amount;

    obligation
        .deposits
        .set(pool_address.clone(), pool_obligation_deposit);

    set_obligation(e, user, &obligation); // NB: Is it reasonable to have `set_obligation` without `read_obligation` first?

    Ok(new_deposit_amount)
}

pub fn set_obligation_borrow(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<i128, LCError> {
    let mut obligation = get_obligation(e, user).unwrap_or(Obligation {
        deposits: Map::new(e),
        borrows: Map::new(e),
    });

    // TODO: Refactor
    let pool_accrual = get_pool(e, pool_address)
        .expect("Pool must exist at this point")
        .accrual;
    let mut pool_obligation_borrow =
        obligation
            .borrows
            .get(pool_address.clone())
            .unwrap_or(ObligationPosition {
                amount: 0,
                last_accrual: pool_accrual.borrow_accrual,
            });

    let new_borrow_amount = pool_obligation_borrow
        .amount
        .checked_add(amount)
        .ok_or(LCError::OverOrUnderflow)?;

    pool_obligation_borrow.amount = new_borrow_amount;

    obligation
        .borrows
        .set(pool_address.clone(), pool_obligation_borrow);
    set_obligation(e, user, &obligation);

    Ok(new_borrow_amount)
}

pub fn deposit_exists(e: &Env, user: &Address, pool_address: &Address) -> Result<bool, LCError> {
    let Obligation {
        deposits,
        borrows: _,
    } = get_obligation(e, user).ok_or(LCError::ObligationDoesNotExist)?;

    Ok(deposits.contains_key(pool_address.clone()))
}

pub fn accrue_interest(e: &Env, pool_address: &Address) -> Result<Accrual, LCError> {
    let mut pool = get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;
    pool.accrue_interest(e)?;

    set_pool(e, pool_address, &pool)?;

    Ok(pool.accrual)
}
