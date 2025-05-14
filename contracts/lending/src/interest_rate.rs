//! `JLending` for now uses kinked interest rates.
//! See: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]

use crate::constants::BPS_IN_PERCENT;
use {
    crate::{
        error::LendingContractError,
        storage::{Pool, PoolConfig},
    },
    soroban_sdk::contracttype,
};

#[derive(Debug)]
#[contracttype]
pub struct InterestRates {
    pub borrow_rate_bps: i128,
    pub supply_rate_bps: i128,
}

impl Pool {
    pub fn get_interest_rates(&self) -> Result<InterestRates, LendingContractError> {
        let &Pool {
            borrowed,
            supply,
            config:
                PoolConfig {
                    base_rate_bps,
                    optimal_utilization_ratio_bps,
                    slope1,
                    slope2,
                    reserve_ratio_bps,
                    ..
                },
            ..
        } = self;

        if borrowed >= supply {
            return Err(LendingContractError::InconsistentPoolState);
        }

        // TODO: think of prettifying this somehow
        let optimal_utilization_ratio = optimal_utilization_ratio_bps / 10;

        // UR is within [0; 10_000]
        let utilization_ratio = borrowed
            .checked_mul(1_000)
            .ok_or(LendingContractError::OverOrUnderflow)?
            .checked_div(supply)
            .ok_or(LendingContractError::OverOrUnderflow)?;

        let borrow_rate_bps = if utilization_ratio <= optimal_utilization_ratio {
            // IR = BR + (UR * 1_000) * Slope1
            base_rate_bps
                .checked_add(
                    slope1
                        .checked_mul(utilization_ratio)
                        .ok_or(LendingContractError::OverOrUnderflow)?,
                )
                .ok_or(LendingContractError::OverOrUnderflow)?
        } else {
            // IR = BR + (OUR * 1_000) * Slope1 + (UR - OUR) * 10_000 * Slope2
            let pre_threshold_rate_bps = base_rate_bps
                .checked_add(
                    slope1
                        .checked_mul(optimal_utilization_ratio)
                        .ok_or(LendingContractError::OverOrUnderflow)?,
                )
                .ok_or(LendingContractError::OverOrUnderflow)?;
            let post_threshold_rate_bps = utilization_ratio
                .checked_sub(optimal_utilization_ratio)
                .ok_or(LendingContractError::OverOrUnderflow)?
                .checked_mul(slope2)
                .ok_or(LendingContractError::OverOrUnderflow)?;

            pre_threshold_rate_bps
                .checked_add(post_threshold_rate_bps)
                .ok_or(LendingContractError::OverOrUnderflow)?
        };
        let supply_rate_bps = base_rate_bps
            .checked_mul(
                utilization_ratio
                    .checked_mul(10_000 - reserve_ratio_bps)
                    .ok_or(LendingContractError::OverOrUnderflow)?,
            )
            .ok_or(LendingContractError::OverOrUnderflow)?
            / 100_000;

        Ok(InterestRates {
            borrow_rate_bps,
            supply_rate_bps,
        })
    }
}

impl PoolConfig {
    pub fn is_valid(&self) -> bool {
        let &PoolConfig {
            base_rate_bps,
            optimal_utilization_ratio_bps,
            slope1,
            slope2,
            reserve_ratio_bps,
            ..
        } = self;

        (base_rate_bps > 0) // BR must be > 0%
        && (optimal_utilization_ratio_bps > 0) // OUR must be > 0%
        && (0..100*BPS_IN_PERCENT).contains(&reserve_ratio_bps) // RR must be [0%; 100%)
        && (slope1 < slope2) // (slope1 < slope2) is necessary for kinked model to work
    }
}
