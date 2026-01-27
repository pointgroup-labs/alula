use core::cmp::{Ord, Ordering};
use soroban_sdk::{Bytes, BytesN, Env, crypto::Hash, xdr::ToXdr};

use crate::{error::FCError, storage};

pub fn require_admin(e: &Env) {
    let admin = storage::get_admin(e).expect("Admin must be set");
    admin.require_auth();
}

#[inline(always)]
pub fn require_nonnegative(value: i128) -> Result<(), FCError> {
    if value < 0 {
        return Err(FCError::NegativeInputAmount);
    }

    Ok(())
}
