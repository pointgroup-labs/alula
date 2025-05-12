//! `JLending` for now uses kinked interest rates. See: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]
use {
    crate::{
        error::LendingContractError,
        storage::{self, PoolConfig, PoolData},
    },
    soroban_sdk::{Address, Env},
};

pub(crate) fn get_interest_rate(
    e: &Env,
    pool_address: &Address,
) -> Result<i128, LendingContractError> {
    let PoolData {
        borrowed, supply, ..
    } = storage::get_pool_data(e, pool_address).ok_or(LendingContractError::PoolDoesNotExist)?;

    if borrowed >= supply {
        return Err(LendingContractError::InconsistentPoolState);
    }
    let PoolConfig {
        base_rate,
        optimal_utilization_ratio,
        slope1,
        slope2,
        ..
    } = storage::get_pool_config(e, pool_address)
        .ok_or(LendingContractError::InconsistentPoolState)?;

    compute_interest_rate(
        borrowed,
        supply,
        base_rate,
        optimal_utilization_ratio,
        slope1,
        slope2,
    )
}

fn compute_interest_rate(
    borrow_amount: i128,
    supply_amount: i128,
    base_rate: i128,
    optimal_utilization_ratio: i128,
    slope1: i128,
    slope2: i128,
) -> Result<i128, LendingContractError> {
    // Utilization ratio mustn't be 0...
    let utiliation_ratio = borrow_amount
        .checked_mul(100)
        .ok_or(LendingContractError::OverOrUnderflow)?
        .checked_div(supply_amount)
        .ok_or(LendingContractError::OverOrUnderflow)?;

    let non_panic_case_rate = base_rate
        .checked_add(
            slope1
                .checked_mul(utiliation_ratio)
                .ok_or(LendingContractError::OverOrUnderflow)?,
        )
        .ok_or(LendingContractError::OverOrUnderflow)?;

    let interest_rate = if utiliation_ratio <= optimal_utilization_ratio {
        non_panic_case_rate
    } else {
        // Panic case
        non_panic_case_rate
            .checked_add(
                utiliation_ratio
                    .checked_sub(optimal_utilization_ratio)
                    .ok_or(LendingContractError::OverOrUnderflow)?,
            )
            .ok_or(LendingContractError::OverOrUnderflow)?
            .checked_mul(slope2)
            .ok_or(LendingContractError::OverOrUnderflow)?
    };

    Ok(interest_rate)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_compute_interest_rate() {}
}
