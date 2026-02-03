use soroban_sdk::Env;

use crate::{error::FCError, storage};

pub trait MathUtils<T> {
    fn map_over_or_underflow(self) -> Result<T, FCError>;
}

impl<T> MathUtils<T> for Option<T> {
    fn map_over_or_underflow(self) -> Result<T, FCError> {
        self.ok_or(FCError::OverOrUnderflow)
    }
}

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
