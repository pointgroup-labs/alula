//! `JLend` for now uses kinked interest rates.
//! See: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]

use {
    crate::{
        constants::{LCError, ACCRUAL_INIT, SECONDS_IN_YEAR},
        math_utils,
        storage::{Accrual, Pool, PoolConfig},
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
///     borrow_multiplier: 1320700048000, // x 1.3207
///     deposit_multiplier: 1000000000000  // x 1.0
/// };
///
/// let compound_rates: CompoundRates = multipliers.try_into().unwrap();
///
/// assert_eq!(compound_rates.borrow_rate_bps, 32_07);
/// assert_eq!(compound_rates.deposit_rate_bps, 00_00);
///
/// ```
#[derive(Debug)]
#[contracttype]
pub struct CompoundRateMultipliers {
    pub borrow_multiplier: i128,
    pub deposit_multiplier: i128,
}

/// Compound interest rates represented in basis points
#[derive(Debug)]
#[contracttype]
pub struct CompoundRates {
    pub borrow_rate_bps: u32,
    pub deposit_rate_bps: u32,
}

impl TryFrom<CompoundRateMultipliers> for CompoundRates {
    type Error = LCError;

    fn try_from(val: CompoundRateMultipliers) -> Result<Self, Self::Error> {
        const BPS_FACTOR: i128 = 10_000;

        let CompoundRateMultipliers {
            borrow_multiplier,
            deposit_multiplier,
        } = val;

        let borrow_multiplier_bps = (borrow_multiplier / (SCALED_ONE / BPS_FACTOR)) as u32;
        let deposit_multiplier_bps = (deposit_multiplier / (SCALED_ONE / BPS_FACTOR)) as u32;

        let borrow_rate_bps = borrow_multiplier_bps
            .checked_sub(BPS_FACTOR as u32)
            .ok_or(LCError::OverOrUnderflow)?;
        let deposit_rate_bps = deposit_multiplier_bps.saturating_sub(BPS_FACTOR as u32);

        Ok(Self {
            borrow_rate_bps,
            deposit_rate_bps,
        })
    }
}

impl Pool {
    pub fn accrue_interest(&mut self, e: &Env) -> Result<(), LCError> {
        let Accrual {
            timestamp,
            borrow_accrual,
            deposit_accrual,
        } = self.accrual;

        let current_timestamp = e.ledger().timestamp();
        assert!(current_timestamp >= timestamp);
        let seconds_passed = current_timestamp - timestamp;

        let CompoundRateMultipliers {
            borrow_multiplier,
            deposit_multiplier,
        } = self.get_compound_rate_multipliers(seconds_passed)?;

        let new_borrow_accrual = borrow_accrual
            .fixed_mul_ceil(borrow_multiplier, SCALED_ONE)
            .ok_or(LCError::OverOrUnderflow)?;
        let new_deposit_accrual = deposit_accrual
            .fixed_mul_ceil(deposit_multiplier, SCALED_ONE)
            .ok_or(LCError::OverOrUnderflow)?;

        let new_accrual = Accrual {
            timestamp: current_timestamp,
            borrow_accrual: new_borrow_accrual,
            deposit_accrual: new_deposit_accrual,
        };
        self.accrual = new_accrual;

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
            borrowed,
            deposited,
            ..
        } = self;

        let deposit_multiplier = if deposited == 0 {
            /* Is zero, since if a pool doesn't yet have a supply, its next APY update must be
            because as a `deposit` which implies that its compound supply interest will be set to 0 regardless */
            SCALED_ONE
        } else {
            // TODO: Comment
            let utilization_ratio_scaled = borrowed
                .checked_mul(SCALED_ONE)
                .ok_or(LCError::OverOrUnderflow)?
                .checked_div(deposited)
                .ok_or(LCError::OverOrUnderflow)?;

            // TODO: Start accounting reserve ratio
            (borrow_multiplier - SCALED_ONE)
                .fixed_mul_ceil(utilization_ratio_scaled, SCALED_ONE)
                .ok_or(LCError::OverOrUnderflow)?
                .checked_add(SCALED_ONE)
                .ok_or(LCError::OverOrUnderflow)?
        };

        Ok(CompoundRateMultipliers {
            borrow_multiplier,
            deposit_multiplier,
        })
    }

    /// Calculates `x` * 1/[`SCALED_ONE`] units of the interest rate per second
    pub fn get_borrow_rate_per_second(&self) -> Result<i128, LCError> {
        let &Pool {
            borrowed,
            deposited,
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

        if deposited == 0 {
            return Ok(base_rate_per_second);
        }

        assert!(
            borrowed <= deposited,
            "Total borrowed funds cannot be less than supplied funds"
        );

        // UR is within [0; 10_000]
        let utilization_ratio_bps = borrowed
            .fixed_div_ceil(deposited, 10_000)
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

impl PoolConfig {
    // by the way, this is not the case for now(((
    pub fn is_valid(&self) -> bool {
        // TODO: Since now interest rate is calculated based on the utilization ration per second
        // this likely has to be changed...

        // let &PoolConfig {
        //     base_rate_bps,
        //     optimal_utilization_ratio_bps,
        //     slope1,
        //     slope2,
        //     reserve_ratio_bps,
        //     ..
        // } = self;

        // (base_rate_bps > 0) // BR must be > 0%
        // && (optimal_utilization_ratio_bps > 0) // OUR must be > 0%
        // && (0..100*BPS_IN_PERCENT).contains(&reserve_ratio_bps) // RR must be [0%; 100%)
        // && (slope1 < slope2) // (slope1 < slope2) is necessary for kinked model to work

        true
    }
}
