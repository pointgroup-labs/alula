//! `JLend` for now uses kinked interest rates.
//! See: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]

use {
    crate::{
        constants::BPS_IN_PERCENT,
        error::LendingContractError,
        storage::{self, Accrual, Pool, PoolConfig},
    },
    soroban_sdk::{contracttype, Env},
};

#[derive(Debug)]
#[contracttype]
pub struct InterestRates {
    pub borrow_rate_bps: i128,
    pub supply_rate_bps: i128,
}

impl Pool {
    pub fn accrue_interest(&self, e: &Env) -> Result<(), LendingContractError> {
        // TODO: check https://github.com/script3/soroban-fixed-point-math
        const SCALE_FACTOR: i128 = 100_000_000;
        const SECONDS_IN_YEAR: i128 = 31_556_926; // TODO: Each second the accrual must happen
                                                  // what if APY will be changed faster than each second?

        let Accrual {
            borrow_accrual,
            supply_accrual,
            timestamp,
        } = storage::get_accrual(e).expect("Accrual must be set during contract construction");

        let current_timestamp = e.ledger().timestamp();
        assert!(current_timestamp >= timestamp); // NB: > or >= ?

        let seconds_passed = current_timestamp - timestamp;
        let InterestRates {
            borrow_rate_bps,
            supply_rate_bps,
        } = self.get_interest_rates()?;

        let borrow_rate_per_second_scaled = borrow_rate_bps
            .checked_mul(SCALE_FACTOR)
            .ok_or(LendingContractError::OverOrUnderflow)?
            .checked_div(SECONDS_IN_YEAR)
            .ok_or(LendingContractError::OverOrUnderflow)?;

        let compound_borrow_rate = i128::pow(
            SCALE_FACTOR
                .checked_add(borrow_rate_per_second_scaled)
                .ok_or(LendingContractError::OverOrUnderflow)?,
            seconds_passed as u32,
        );

        // A very big number, no??
        let supply_rate_per_second_scaled = supply_rate_bps
            .checked_mul(SCALE_FACTOR)
            .ok_or(LendingContractError::OverOrUnderflow)?
            .checked_div(SECONDS_IN_YEAR)
            .ok_or(LendingContractError::OverOrUnderflow)?;

        let compound_supply_rate = i128::pow(
            SCALE_FACTOR
                .checked_add(supply_rate_per_second_scaled)
                .ok_or(LendingContractError::OverOrUnderflow)?,
            seconds_passed as u32,
        );

        let new_borrow_accrual = borrow_accrual
            .checked_mul(compound_borrow_rate)
            .ok_or(LendingContractError::OverOrUnderflow)?;
        let new_supply_accrual = supply_accrual
            .abs()
            .checked_mul(compound_supply_rate)
            .ok_or(LendingContractError::OverOrUnderflow)?;

        let new_accrual = Accrual {
            timestamp: e.ledger().timestamp(),
            borrow_accrual: new_borrow_accrual,
            supply_accrual: new_supply_accrual,
        };
        storage::set_accrual(e, &new_accrual);

        Ok(())
    }

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

        assert!(
            borrowed < supply,
            "Total borrowed funds cannot be less than supplied funds"
        );
        // TODO: think of, maybe, prettifying the computation somehow
        let optimal_utilization_ratio_scaled = optimal_utilization_ratio_bps / 10;
        // UR is within [0; 1_000]
        let utilization_ratio = borrowed
            .checked_mul(1_000)
            .ok_or(LendingContractError::OverOrUnderflow)?
            .checked_div(supply)
            .ok_or(LendingContractError::OverOrUnderflow)?;

        let borrow_rate_bps = if utilization_ratio < optimal_utilization_ratio_scaled {
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
                        .checked_mul(optimal_utilization_ratio_scaled)
                        .ok_or(LendingContractError::OverOrUnderflow)?,
                )
                .ok_or(LendingContractError::OverOrUnderflow)?;
            let post_threshold_rate_bps = utilization_ratio
                .checked_sub(optimal_utilization_ratio_scaled)
                .ok_or(LendingContractError::OverOrUnderflow)?
                .checked_mul(slope2)
                .ok_or(LendingContractError::OverOrUnderflow)?;

            pre_threshold_rate_bps
                .checked_add(post_threshold_rate_bps)
                .ok_or(LendingContractError::OverOrUnderflow)?
        };
        let supply_rate_bps = borrow_rate_bps
            .checked_mul(
                utilization_ratio
                    .checked_mul(10_000 - reserve_ratio_bps)
                    .ok_or(LendingContractError::OverOrUnderflow)?,
            )
            .ok_or(LendingContractError::OverOrUnderflow)?
            / (10_000 * 1_000); // scaling down to base points after consecutive multiplication

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
