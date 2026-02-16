use soroban_sdk::{Address, Env, token};

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

/// Transfers `amount` of `token` from `from` into the contract, then verifies
/// the contract's balance increased by exactly `amount`.
///
/// Rejects fee-on-transfer or rebasing tokens that would cause internal
/// bookkeeping to diverge from actual token holdings.
pub fn transfer_in(
    e: &Env,
    token: &Address,
    from: &Address,
    amount: i128,
) -> Result<(), FCError> {
    let client = token::Client::new(e, token);
    let balance_before = client.balance(&e.current_contract_address());
    client.transfer(from, e.current_contract_address(), &amount);
    if client.balance(&e.current_contract_address()) - balance_before != amount {
        return Err(FCError::TransferAmountMismatch);
    }
    Ok(())
}
