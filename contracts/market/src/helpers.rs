use soroban_sdk::Env;

use crate::{error::MCError, storage};

#[inline(always)]
pub fn require_nonnegative(amount: i128) -> Result<(), MCError> {
    if amount < 0 {
        return Err(MCError::NegativeAmount);
    }

    Ok(())
}

#[inline(always)]
pub fn require_admin(e: &Env) {
    let admin = storage::get_global_state(e).admin;
    admin.require_auth();
}
