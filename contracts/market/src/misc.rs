use soroban_sdk::{Env, Vec, contracttype};

use crate::{
    error::MCError,
    interest_rate::AnnualPercentageYields,
    pool::Pool,
    storage::{self, GlobalState, MarketStatus},
};
#[contracttype]
#[derive(Debug, PartialEq, Eq, Clone)]

/// Represents the pool's plain data with additionally computed info. Intended to be used as a result of simulated read-only
/// invocations
pub struct PoolData {
    pub pool: Pool,
    pub apy: AnnualPercentageYields,
    pub j_token_rate_floor_bps: i128,
    pub d_token_rate_ceil_bps: i128,
}

/// Represents the entire market's data(for every pool) with additionally computed info. Intended to be used as a result of simulated read-only
/// invocations
#[contracttype]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MarketData {
    pub pools_data: Vec<PoolData>,
    pub global_state: GlobalState,
}

/// Ensures that the provided amount is non-negative.
#[inline(always)]
pub fn require_nonnegative(amount: i128) -> Result<(), MCError> {
    if amount < 0 {
        return Err(MCError::NegativeInputAmount);
    }

    Ok(())
}

#[inline(always)]
pub fn require_owned_and_admin(e: &Env) -> Result<(), MCError> {
    require_admin(e);

    if storage::get_update_in_queue_period(e).is_none() {
        return Err(MCError::MarketIsNotOwned);
    }

    Ok(())
}

/// Ensures that the caller is the admin of the contract
#[inline(always)]
pub fn require_admin(e: &Env) {
    storage::get_admin(e).require_auth();
}

#[inline(always)]
pub fn require_deployer(e: &Env) {
    storage::get_deployer(e).require_auth();
}

#[inline(always)]
pub fn require_borrow_allowed(e: &Env) -> Result<(), MCError> {
    if !matches!(storage::get_market_status(e), MarketStatus::Active) {
        return Err(MCError::BorrowForbiddenOnMarket);
    }

    Ok(())
}

#[inline(always)]
pub fn require_deposit_allowed(e: &Env) -> Result<(), MCError> {
    if !matches!(storage::get_market_status(e), MarketStatus::Active | MarketStatus::BorrowFrozen) {
        return Err(MCError::DepositForbiddenOnMarket);
    }

    Ok(())
}

#[inline(always)]
pub fn require_not_frozen(e: &Env) -> Result<(), MCError> {
    if matches!(storage::get_market_status(e), MarketStatus::Frozen) {
        return Err(MCError::MarketIsFrozen);
    }

    Ok(())
}
