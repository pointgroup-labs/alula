//! `JLending` for now uses kinked interest rates. See: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]
use {
    crate::{
        error::LendingContractError,
        storage::{Pool, PoolConfig, BPS_IN_PERCENT},
    },
    soroban_sdk::contracttype,
};

#[derive(Debug)]
#[contracttype]
pub struct InterestRates {
    pub borrow_rate: i128,
    pub supply_rate: i128,
}

impl Pool {
    pub fn get_interest_rates(&self) -> Result<InterestRates, LendingContractError> {
        let &Pool {
            borrowed,
            supply,
            config:
                PoolConfig {
                    base_rate,
                    optimal_utilization_ratio,
                    slope1,
                    slope2,
                    reserve_ratio,
                    ..
                },
            ..
        } = self;

        if borrowed >= supply {
            return Err(LendingContractError::InconsistentPoolState);
        }
        // @TODO: think of prettifying this somehow
        let optimal_utilization_ratio = optimal_utilization_ratio / 10;
        // UR is within [0; 10_000]
        let utiliation_ratio = borrowed
            .checked_mul(1_000)
            .ok_or(LendingContractError::OverOrUnderflow)?
            .checked_div(supply)
            .ok_or(LendingContractError::OverOrUnderflow)?;

        let borrow_rate = if utiliation_ratio <= optimal_utilization_ratio {
            // IR = BR + (UR * 1_000) * Slope1
            base_rate
                .checked_add(
                    slope1
                        .checked_mul(utiliation_ratio)
                        .ok_or(LendingContractError::OverOrUnderflow)?,
                )
                .ok_or(LendingContractError::OverOrUnderflow)?
        } else {
            // IR = BR + (OUR * 1_000) * Slope1 + (UR - OUR) * 10_000 * Slope2
            let pre_threshold_rate = base_rate
                .checked_add(
                    slope1
                        .checked_mul(optimal_utilization_ratio)
                        .ok_or(LendingContractError::OverOrUnderflow)?,
                )
                .ok_or(LendingContractError::OverOrUnderflow)?;
            let post_threshold_rate = utiliation_ratio
                .checked_sub(optimal_utilization_ratio)
                .ok_or(LendingContractError::OverOrUnderflow)?
                .checked_mul(slope2)
                .ok_or(LendingContractError::OverOrUnderflow)?;

            pre_threshold_rate
                .checked_add(post_threshold_rate)
                .ok_or(LendingContractError::OverOrUnderflow)?
        };
        let supply_rate = base_rate
            .checked_mul(
                utiliation_ratio
                    .checked_mul(10_000 - reserve_ratio)
                    .ok_or(LendingContractError::OverOrUnderflow)?,
            )
            .ok_or(LendingContractError::OverOrUnderflow)?
            / 100_000;

        Ok(InterestRates {
            borrow_rate,
            supply_rate,
        })
    }
}

impl PoolConfig {
    pub(crate) fn is_valid(&self) -> bool {
        let &PoolConfig {
            base_rate,
            optimal_utilization_ratio,
            slope1,
            slope2,
            reserve_ratio,
            ..
        } = self;

        (base_rate > 0) // BR must be > 0%
        && (optimal_utilization_ratio > 0) // OUR must be > 0%
        && (0..(100*BPS_IN_PERCENT)).contains(&reserve_ratio) // RR must be [0%; 100%)
        && (slope1 < slope2) // (slope1 < slope2) is necessary for kinked model to work
    }
}
