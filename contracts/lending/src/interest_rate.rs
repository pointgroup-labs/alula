//! `JLend` for now uses kinked interest rates.
//! See: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]

use {
    crate::{
        constants::{LCError, ACCRUAL_INIT_VALUE, BPS_IN_PERCENT, SECONDS_IN_YEAR},
        storage::{self, Accrual, Pool, PoolConfig},
    },
    soroban_fixed_point_math::FixedPoint,
    soroban_sdk::{contracttype, Env},
};

const SCALED_ONE: i128 = ACCRUAL_INIT_VALUE;

#[derive(Debug)]
#[contracttype]
pub struct CompoundRates {
    pub borrow_rate: i128,
    pub supply_rate: i128,
}

/// O(log(n)) algorithm for quick exponentiation
fn binary_power(base: i128, mut exponent: u64, denominator: i128) -> i128 {
    let mut result = denominator;
    let mut temp_base = base;

    while exponent > 0 {
        if exponent % 2 == 1 {
            result = result.fixed_mul_floor(temp_base, denominator).unwrap(); // TODO: `floor` or `ceil`?
        }

        temp_base = temp_base.fixed_mul_floor(temp_base, denominator).unwrap();
        exponent /= 2;
    }

    result
}
impl Pool {
    pub fn accrue_interest(&self, e: &Env, seconds_passed: u64) -> Result<(), LCError> {
        let Accrual {
            timestamp,
            borrow_accrual,
            supply_accrual,
        } = storage::get_accrual(e).expect("Accrual must be set during contract construction");

        let current_timestamp = e.ledger().timestamp();
        assert!(current_timestamp >= timestamp);
        // let seconds_passed = current_timestamp - timestamp; // TODO: Add after fixing TTL issue
        // let seconds_passed: u64 = SECONDS_IN_YEAR;

        let CompoundRates {
            borrow_rate,
            supply_rate,
        } = self.get_compound_interest_rates(seconds_passed)?;

        let new_borrow_accrual = borrow_accrual
            .fixed_mul_ceil(borrow_rate, SCALED_ONE)
            .ok_or(LCError::OverOrUnderflow)?;
        let new_supply_accrual = supply_accrual
            .fixed_mul_ceil(supply_rate, SCALED_ONE)
            .ok_or(LCError::OverOrUnderflow)?;

        let new_accrual = Accrual {
            timestamp: current_timestamp,
            borrow_accrual: new_borrow_accrual,
            supply_accrual: new_supply_accrual,
        };
        storage::set_accrual(e, &new_accrual);

        Ok(())
    }

    pub fn get_apys(&self) -> Result<CompoundRates, LCError> {
        self.get_compound_interest_rates(SECONDS_IN_YEAR)
    }

    fn get_compound_interest_rates(&self, seconds_passed: u64) -> Result<CompoundRates, LCError> {
        let borrow_interest_rate = self.get_interest_rate()?;

        let per_second_growth_factor = SCALED_ONE + borrow_interest_rate; // e.g. 1,00000000xxx, where `xxx` is the interest rate
        let borrow_compound_interest_rate =
            binary_power(per_second_growth_factor, seconds_passed, SCALED_ONE);

        let &Pool {
            borrowed, supply, ..
        } = self;

        // TODO: Comment
        let utilization_ratio_scaled = borrowed
            .checked_mul(SCALED_ONE)
            .ok_or(LCError::OverOrUnderflow)?
            .checked_div(supply)
            .ok_or(LCError::OverOrUnderflow)?;

        // TODO: Start accounting reserve ratio
        let supply_compound_interest_rate = (borrow_compound_interest_rate - SCALED_ONE)
            .fixed_mul_ceil(utilization_ratio_scaled, SCALED_ONE)
            .ok_or(LCError::OverOrUnderflow)?
            .checked_add(SCALED_ONE)
            .ok_or(LCError::OverOrUnderflow)?;

        Ok(CompoundRates {
            borrow_rate: borrow_compound_interest_rate,
            supply_rate: supply_compound_interest_rate,
        })
    }

    pub fn get_interest_rate(&self) -> Result<i128, LCError> {
        let &Pool {
            borrowed,
            supply,
            config:
                PoolConfig {
                    base_rate_bps,
                    optimal_utilization_ratio_bps,
                    slope1,
                    slope2,
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
            .ok_or(LCError::OverOrUnderflow)?
            .checked_div(supply)
            .ok_or(LCError::OverOrUnderflow)?;

        let borrow_rate_bps = if utilization_ratio < optimal_utilization_ratio_scaled {
            // IR = BR + (UR * 1_000) * Slope1
            base_rate_bps
                .checked_add(
                    slope1
                        .checked_mul(utilization_ratio)
                        .ok_or(LCError::OverOrUnderflow)?,
                )
                .ok_or(LCError::OverOrUnderflow)?
        } else {
            // IR = BR + (OUR * 1_000) * Slope1 + (UR - OUR) * 10_000 * Slope2
            let pre_threshold_rate_bps = base_rate_bps
                .checked_add(
                    slope1
                        .checked_mul(optimal_utilization_ratio_scaled)
                        .ok_or(LCError::OverOrUnderflow)?,
                )
                .ok_or(LCError::OverOrUnderflow)?;
            let post_threshold_rate_bps = utilization_ratio
                .checked_sub(optimal_utilization_ratio_scaled)
                .ok_or(LCError::OverOrUnderflow)?
                .checked_mul(slope2)
                .ok_or(LCError::OverOrUnderflow)?;

            pre_threshold_rate_bps
                .checked_add(post_threshold_rate_bps)
                .ok_or(LCError::OverOrUnderflow)?
        };

        Ok(borrow_rate_bps)
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
