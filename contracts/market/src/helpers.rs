use soroban_sdk::Env;

use crate::{error::MCError, storage};

/// Ensures that the provided amount is non-negative.
#[inline(always)]
pub fn require_nonnegative(amount: i128) -> Result<(), MCError> {
    if amount < 0 {
        return Err(MCError::NegativeAmount);
    }

    Ok(())
}

/// Ensures that the caller is the admin of the contract.
#[inline(always)]
pub fn require_admin(e: &Env) {
    let admin = storage::get_global_state(e).admin;
    admin.require_auth();
}
