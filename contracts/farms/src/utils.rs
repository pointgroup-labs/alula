use soroban_sdk::{Address, Env, token};

use crate::error::FCError;

pub trait MathUtils<T> {
    fn map_over_or_underflow(self) -> Result<T, FCError>;
}

impl<T> MathUtils<T> for Option<T> {
    fn map_over_or_underflow(self) -> Result<T, FCError> {
        self.ok_or(FCError::OverOrUnderflow)
    }
}

#[inline(always)]
pub fn require_nonnegative(value: i128) -> Result<(), FCError> {
    if value < 0 {
        return Err(FCError::NegativeInputAmount);
    }

    Ok(())
}

#[inline(always)]
pub fn require_positive(value: i128) -> Result<(), FCError> {
    if value <= 0 {
        return Err(FCError::InvalidAmount);
    }

    Ok(())
}

/// Transfers `amount` of `token` from `from` into the contract, then verifies
/// the contract's balance increased by exactly `amount`.
///
/// Rejects fee-on-transfer or rebasing tokens that would cause internal
/// bookkeeping to diverge from actual token holdings.
pub fn transfer_in(e: &Env, token: &Address, from: &Address, amount: i128) -> Result<(), FCError> {
    let client = token::Client::new(e, token);
    let balance_before = client.balance(&e.current_contract_address());
    client.transfer(from, e.current_contract_address(), &amount);
    let received = client
        .balance(&e.current_contract_address())
        .checked_sub(balance_before)
        .map_over_or_underflow()?;
    if received != amount {
        return Err(FCError::TransferAmountMismatch);
    }
    Ok(())
}
