use soroban_sdk::{Env, Vec, contracttype};

use crate::{
    error::MCError,
    interest_rate::AnnualPercentageYields,
    multiply_pair::MultiplyPair,
    pool::Pool,
    storage::{self, GlobalState, MarketStatus},
};
#[contracttype]
#[derive(Debug, PartialEq, Eq, Clone)]

// Represents the pool's plain data with additionally computed info. Intended to be used as a result of simulated read-only
// invocations
pub struct PoolData {
    pub pool: Pool,
    pub apy: AnnualPercentageYields,
    pub total_supply: i128,
    pub total_available_adjusted: i128,
    pub j_token_rate_floor_bps: i128,
    pub d_token_rate_ceil_bps: i128,
    pub oracle_asset_price: i128,
}

// Represents the entire market's data(for every pool) with additionally computed info. Intended to be used as a result of simulated read-only
// invocations
#[contracttype]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MarketData {
    pub pools_data: Vec<PoolData>,
    pub multiply_pairs: Vec<MultiplyPair>,
    pub global_state: GlobalState,
    pub asset_decimals: u32,
    pub oracle_price_decimals: u32,
}

// Ensures that the provided amount is non-negative.
#[inline(always)]
pub fn require_nonnegative(amount: i128) -> Result<(), MCError> {
    if amount.is_negative() {
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

#[inline(always)]
pub fn require_insurance_fund(e: &Env) -> Result<(), MCError> {
    let fund = storage::get_insurance_fund(e);
    fund.require_auth();

    Ok(())
}

// Ensures that the caller is the admin of the contract
#[inline(always)]
pub fn require_admin(e: &Env) {
    storage::get_admin(e).require_auth();
}

#[inline(always)]
pub fn require_borrows_on_market_allowed(e: &Env) -> Result<(), MCError> {
    if !matches!(storage::get_market_status(e), MarketStatus::Active) {
        return Err(MCError::BorrowForbiddenOnMarket);
    }

    Ok(())
}

#[inline(always)]
pub fn require_deposits_on_market_allowed(e: &Env) -> Result<(), MCError> {
    if !matches!(storage::get_market_status(e), MarketStatus::Active | MarketStatus::BorrowFrozen) {
        return Err(MCError::DepositForbiddenOnMarket);
    }

    Ok(())
}

#[inline(always)]
pub fn require_market_not_frozen(e: &Env) -> Result<(), MCError> {
    if matches!(storage::get_market_status(e), MarketStatus::Frozen) {
        return Err(MCError::MarketIsFrozen);
    }

    Ok(())
}
