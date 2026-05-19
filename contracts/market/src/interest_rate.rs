use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Env, I256, contracttype};

use crate::{
    accrual::Accrual, constants::*, error::MCError, events, interest_rate_model::InterestRate,
    math_utils::MathUtils, pool::Pool,
};

// Compound interest rates represented in basis points
#[derive(Debug, Eq, PartialEq, Clone)]
#[contracttype]
pub struct AnnualPercentageYields {
    pub borrow_bps: u32,
    pub supply_bps: u32,
}

impl Pool {
    // Accrues interest on the pool's total borrowed amount based
    // on the time elapsed since the last accrual
    pub fn accrue_interest(&mut self, e: &Env) -> Result<(), MCError> {
        let current_timestamp = e.ledger().timestamp();
        if current_timestamp < self.last_accrual_timestamp {
            events::current_ledger_timestamp_smaller_than_stored_timestamp(
                e,
                current_timestamp,
                self.last_accrual_timestamp,
            );

            return Err(MCError::InternalError);
        }

        // -- Accrue interest on the pool --

        let seconds_passed = current_timestamp - self.last_accrual_timestamp; // safe
        if seconds_passed == 0 {
            // NB: No time passed, no interest to accrue
            return Ok(());
        }

        let utilization_ratio_bps = self.compute_utilization_ratio_bps()?;
        let current_borrow_apr = self
            .config
            .interest_rate_model
            .compute_borrow_apr(utilization_ratio_bps)?
            .fixed_mul_ceil(self.interest_rate_modifier_bps, BPS_FACTOR)
            .map_over_or_underflow()?;
        let accrual_multiplier: i128 =
            self.config.accrual_model.compute_multiplier(e, current_borrow_apr, seconds_passed)?;

        let new_total_borrowed = {
            // Use I256 to avoid overflow when total_borrowed * accrual_multiplier exceeds
            // i128::MAX before the division by SCALED_FIXED_POINT_DENOMINATOR.
            let a = I256::from_i128(e, self.total_borrowed);
            let b = I256::from_i128(e, accrual_multiplier);
            let denom = I256::from_i128(e, SCALED_FIXED_POINT_DENOMINATOR);
            let product = a.mul(&b);
            let remainder = product.rem_euclid(&denom);
            let zero = I256::from_i32(e, 0);
            let result_i256 = if remainder == zero {
                product.div(&denom)
            } else {
                product.div(&denom).add(&I256::from_i32(e, 1))
            };

            result_i256.to_i128().map_over_or_underflow()?
        };
        let accrued =
            new_total_borrowed.checked_sub(self.total_borrowed).map_over_or_underflow()?;
        let take_rate_accrual_part = accrued
            .fixed_mul_ceil(self.config.fee_config.take_rate_bps as i128, BPS_FACTOR)
            .map_over_or_underflow()?;

        let new_take_rate_fees_sum =
            self.take_rate_fees_sum.checked_add(take_rate_accrual_part).map_over_or_underflow()?;

        self.total_borrowed = new_total_borrowed;
        self.take_rate_fees_sum = new_take_rate_fees_sum;

        self.borrow_apr_bps = current_borrow_apr;
        self.supply_apr_bps = current_borrow_apr
            .fixed_mul_floor(utilization_ratio_bps, BPS_FACTOR)
            .map_over_or_underflow()?
            .fixed_mul_floor(BPS_FACTOR - self.config.fee_config.take_rate_bps as i128, BPS_FACTOR)
            .map_over_or_underflow()?;

        self.last_accrual_timestamp = current_timestamp;

        let utilization_diff_bps = utilization_ratio_bps
            .checked_sub(self.config.target_utilization_ratio_bps)
            .map_over_or_underflow()?;
        let utilization_error =
            (seconds_passed as i128).checked_mul(utilization_diff_bps).map_over_or_underflow()?;
        let new_interest_rate_modifier_bps = if utilization_diff_bps >= 0 {
            // Positive diff - modifier decreases
            let rate_diff = utilization_error
                .fixed_mul_floor(self.config.ir_reactivity_constant as i128, BPS_FACTOR)
                .map_over_or_underflow()?;

            i128::max(MIN_IR_MODIFIER, self.interest_rate_modifier_bps.saturating_sub(rate_diff))
        } else {
            // Negative diff - modifier increases
            let rate_diff = utilization_error
                .fixed_mul_ceil(self.config.ir_reactivity_constant as i128, BPS_FACTOR)
                .map_over_or_underflow()?
                .checked_neg()
                .map_over_or_underflow()?;
            i128::min(MAX_IR_MODIFIER, self.interest_rate_modifier_bps.saturating_add(rate_diff))
        };

        self.interest_rate_modifier_bps = new_interest_rate_modifier_bps;

        Ok(())
    }

    // Get current annual percentage yields (APY) for borrowing and supplying
    // based on the pool's utilization ratio, interest rate model, and accrual model
    pub fn get_apy(&self, e: &Env) -> Result<AnnualPercentageYields, MCError> {
        let utilization_ratio_bps = self.compute_utilization_ratio_bps()?;

        let borrow_apr = self
            .config
            .interest_rate_model
            .compute_borrow_apr(utilization_ratio_bps)?
            .fixed_mul_ceil(self.interest_rate_modifier_bps, BPS_FACTOR)
            .map_over_or_underflow()?;
        let supply_apr = borrow_apr
            .fixed_mul_floor(utilization_ratio_bps, BPS_FACTOR)
            .map_over_or_underflow()?
            .fixed_mul_floor(BPS_FACTOR - self.config.fee_config.take_rate_bps as i128, BPS_FACTOR)
            .map_over_or_underflow()?; // safe

        let borrow_apy_multiplier =
            self.config.accrual_model.compute_multiplier(e, borrow_apr, SECONDS_IN_YEAR)?;
        let supply_apy_multiplier =
            self.config.accrual_model.compute_multiplier(e, supply_apr, SECONDS_IN_YEAR)?;

        let borrow_apy_bps = multiplier_to_percentage_increase(borrow_apy_multiplier)?;
        let supply_apy_bps = multiplier_to_percentage_increase(supply_apy_multiplier)?;

        let apy = AnnualPercentageYields { borrow_bps: borrow_apy_bps, supply_bps: supply_apy_bps };

        Ok(apy)
    }

    // Computes the current utilization ratio in basis points (bps)
    pub fn compute_utilization_ratio_bps(&self) -> Result<i128, MCError> {
        let total = self.total_supply()?;

        if total == 0 {
            Ok(0)
        } else {
            self.total_borrowed.fixed_div_ceil(total, BPS_FACTOR).map_over_or_underflow()
        }
    }
}

// -- Helpers --

fn multiplier_to_percentage_increase(multiplier: i128) -> Result<u32, MCError> {
    const SCALE_DIVISOR: i128 = SCALED_FIXED_POINT_DENOMINATOR / BPS_FACTOR;

    let multiplier_bps =
        u32::try_from(multiplier / SCALE_DIVISOR).map_err(|_| MCError::OverOrUnderflow)?;
    let percentage_increase_bps = multiplier_bps.saturating_sub(BPS_FACTOR as u32);

    Ok(percentage_increase_bps)
}
