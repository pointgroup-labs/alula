use soroban_sdk::Env;

use crate::{
    error::MCError,
    storage::{self, MarketStatus},
};

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

#[inline(always)]
pub fn require_insurance_fund(e: &Env) -> Result<(), MCError> {
    let fund = storage::get_insurance_fund(e);
    fund.require_auth();

    Ok(())
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
