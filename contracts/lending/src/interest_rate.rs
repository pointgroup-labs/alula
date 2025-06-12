//! `JLend` for now uses kinked interest rates.
//! See: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]

use {
    crate::{
        constants::{LCError, ACCRUAL_INIT, BPS_FACTOR, SECONDS_IN_YEAR},
        math_utils,
        pool::{Pool, PoolConfig},
    },
    soroban_fixed_point_math::FixedPoint,
    soroban_sdk::{contracttype, Env},
};

pub const SCALED_ONE: i128 = ACCRUAL_INIT;

/// Interest rate multipliers presented as (1 + xxx) where `xxx` is a compound interest rate.
/// The real multiplier(e.g. 1.32, 2.53, etc) is scaled up with [`SCALED_ONE`] value.
///
/// # Examples:
/// ```
/// use lending::interest_rate::{CompoundRates, CompoundRateMultipliers};
///
/// let multipliers = CompoundRateMultipliers {
///     borrow: 1320700048000, // x 1.3207
///     supply: 1000000000000  // x 1.0
/// };
///
/// let compound_rates: CompoundRates = multipliers.try_into().unwrap();
///
/// assert_eq!(compound_rates.borrow_bps, 32_07); // 32.07%
/// assert_eq!(compound_rates.supply_bps, 00_00); // 0%
///
/// ```
#[derive(Debug)]
#[contracttype]
pub struct CompoundRateMultipliers {
    pub borrow: i128,
    pub supply: i128,
}

/// Compound interest rates represented in basis points
#[derive(Debug)]
#[contracttype]
pub struct CompoundRates {
    pub borrow_bps: u32,
    pub supply_bps: u32,
}

impl TryFrom<CompoundRateMultipliers> for CompoundRates {
    type Error = LCError;

    fn try_from(val: CompoundRateMultipliers) -> Result<Self, Self::Error> {
        let CompoundRateMultipliers {
            borrow: borrow_multiplier,
            supply: supply_multiplier,
        } = val;

        let borrow_multiplier_bps = (borrow_multiplier / (SCALED_ONE / BPS_FACTOR)) as u32;
        let supply_multiplier_bps = (supply_multiplier / (SCALED_ONE / BPS_FACTOR)) as u32;

        let borrow_bps = borrow_multiplier_bps
            .checked_sub(BPS_FACTOR as u32)
            .ok_or(LCError::OverOrUnderflow)?;
        let supply_bps = supply_multiplier_bps.saturating_sub(BPS_FACTOR as u32);

        Ok(Self {
            borrow_bps,
            supply_bps,
        })
    }
}

impl Pool {
    /// Accrues interest on the pool and updates the `total_borrowed` amount according to it
    pub fn accrue_interest(&mut self, e: &Env) -> Result<(), LCError> {
        let current_timestamp = e.ledger().timestamp();
        assert!(current_timestamp >= self.last_accrual_timestamp);
        let seconds_passed = current_timestamp - self.last_accrual_timestamp;

        let borrow_multiplier = self.get_compound_rate_multipliers(seconds_passed)?.borrow;

        let new_accrual = self
            .last_accrual
            .fixed_mul_ceil(borrow_multiplier, SCALED_ONE)
            .ok_or(LCError::OverOrUnderflow)?;

        let new_total_borrowed = self
            .total_borrowed
            .checked_mul(new_accrual)
            .ok_or(LCError::OverOrUnderflow)?
            .checked_div(self.last_accrual)
            .ok_or(LCError::OverOrUnderflow)?;

        self.total_borrowed = new_total_borrowed;
        self.last_accrual = new_accrual;
        self.last_accrual_timestamp = current_timestamp;

        Ok(())
    }

    pub fn get_apy(&self) -> Result<CompoundRates, LCError> {
        self.get_compound_rates(SECONDS_IN_YEAR)
    }

    fn get_compound_rates(&self, seconds_passed: u64) -> Result<CompoundRates, LCError> {
        self.get_compound_rate_multipliers(seconds_passed)?
            .try_into()
    }

    pub fn get_compound_rate_multipliers(
        &self,
        seconds_passed: u64,
    ) -> Result<CompoundRateMultipliers, LCError> {
        let borrow_interest_rate = self.get_borrow_rate_per_second()?;

        let per_second_growth_factor = SCALED_ONE + borrow_interest_rate; // e.g. 1,00000000xxx, where `xxx` is the interest rate
        let borrow_multiplier =
            math_utils::bin_pow(per_second_growth_factor, seconds_passed, SCALED_ONE)?;

        let &Pool {
            total_borrowed,
            available,
            ..
        } = self;

        let total = total_borrowed
            .checked_add(available)
            .ok_or(LCError::OverOrUnderflow)?;

        let supply_multiplier = if total == 0 {
            /* Is [`SCALED_ONE`], since if a pool doesn't yet have deposits, its next APY update must be
            as a `deposit` which implies that its compound deposit interest will be set to 0 regardless */
            SCALED_ONE
        } else {
            let utilization_ratio_scaled = total_borrowed
                .fixed_div_floor(total, SCALED_ONE)
                .ok_or(LCError::OverOrUnderflow)?;

            // TODO: Start accounting reserve ratio
            (borrow_multiplier - SCALED_ONE)
                .fixed_mul_ceil(utilization_ratio_scaled, SCALED_ONE)
                .ok_or(LCError::OverOrUnderflow)?
                .checked_add(SCALED_ONE)
                .ok_or(LCError::OverOrUnderflow)?
        };

        Ok(CompoundRateMultipliers {
            borrow: borrow_multiplier,
            supply: supply_multiplier,
        })
    }

    /// Calculates `x` * 1/[`SCALED_ONE`] units of the interest rate per second
    pub fn get_borrow_rate_per_second(&self) -> Result<i128, LCError> {
        let &Pool {
            total_borrowed,
            available,
            config:
                PoolConfig {
                    base_rate_per_second,
                    optimal_utilization_ratio_bps,
                    slope1,
                    slope2,
                    ..
                },
            ..
        } = self;

        let total = total_borrowed + available;

        if total == 0 {
            return Ok(base_rate_per_second);
        }

        // UR is within [0; 10_000]
        let utilization_ratio_bps = total_borrowed
            .fixed_div_ceil(total, BPS_FACTOR)
            .ok_or(LCError::OverOrUnderflow)?;

        let borrow_rate_per_second = if utilization_ratio_bps < optimal_utilization_ratio_bps {
            // IR = BR + (UR * 10_000) * Slope1
            base_rate_per_second
                .checked_add(
                    slope1
                        .checked_mul(utilization_ratio_bps)
                        .ok_or(LCError::OverOrUnderflow)?,
                )
                .ok_or(LCError::OverOrUnderflow)?
        } else {
            // IR = BR + (OUR * 10_000) * Slope1 + (UR - OUR) * 10_000 * Slope2
            let pre_threshold_rate_bps = base_rate_per_second
                .checked_add(
                    slope1
                        .checked_mul(optimal_utilization_ratio_bps)
                        .ok_or(LCError::OverOrUnderflow)?,
                )
                .ok_or(LCError::OverOrUnderflow)?;
            let post_threshold_rate_bps = utilization_ratio_bps
                .checked_sub(optimal_utilization_ratio_bps)
                .ok_or(LCError::OverOrUnderflow)?
                .checked_mul(slope2)
                .ok_or(LCError::OverOrUnderflow)?;

            pre_threshold_rate_bps
                .checked_add(post_threshold_rate_bps)
                .ok_or(LCError::OverOrUnderflow)?
        };

        Ok(borrow_rate_per_second)
    }
}
