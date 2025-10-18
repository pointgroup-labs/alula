use soroban_sdk::{Env, panic_with_error};

use crate::{error::MCError, storage};

/// Ensures that the provided amount is non-negative.
#[inline(always)]
pub fn require_nonnegative(amount: i128) -> Result<(), MCError> {
    if amount < 0 {
        return Err(MCError::NegativeAmount);
    }

    Ok(())
}

#[inline(always)]
pub fn require_owned(e: &Env) {
    let is_owned = storage::get_is_owned(e);

    if !is_owned {
        panic_with_error!(e, MCError::MarketIsNotOwned);
    }
}

/// Ensures that the caller is the admin of the contract
#[inline(always)]
pub fn require_admin(e: &Env) {
    let admin = storage::get_admin(e);
    admin.require_auth();
}

#[inline(always)]
pub fn require_deployer(e: &Env) {
    let deployer = storage::get_deployer(e);
    deployer.require_auth();
}
