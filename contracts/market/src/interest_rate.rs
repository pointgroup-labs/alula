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
            .fixed_mul_floor(self.config.fee_config.take_rate_bps as i128, BPS_FACTOR)
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
        // Interest accrues over the real interval above; the controller integrates over a clamped one, so
        // an idle pool cannot ask for a step wider than the modifier band.
        let controller_seconds = seconds_passed.min(MAX_IR_INTEGRATION_SECONDS);
        let utilization_error = (controller_seconds as i128)
            .checked_mul(utilization_diff_bps)
            .map_over_or_underflow()?;
        // Above target the rate rises and below it falls, so borrowing and supply pull utilization back
        // (Blend v2's `calc_accrual`). Each step's size rounds toward zero.
        let new_interest_rate_modifier_bps = if utilization_diff_bps >= 0 {
            let rate_diff = utilization_error
                .fixed_mul_floor(self.config.ir_reactivity_constant as i128, BPS_FACTOR)
                .map_over_or_underflow()?;

            i128::min(MAX_IR_MODIFIER, self.interest_rate_modifier_bps.saturating_add(rate_diff))
        } else {
            let rate_diff = utilization_error
                .fixed_mul_ceil(self.config.ir_reactivity_constant as i128, BPS_FACTOR)
                .map_over_or_underflow()?
                .checked_neg()
                .map_over_or_underflow()?;
            i128::max(MIN_IR_MODIFIER, self.interest_rate_modifier_bps.saturating_sub(rate_diff))
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

        let apy_bps = |apr_bps: i128| -> Result<u32, MCError> {
            // Past this APR a year's compounding no longer fits `u32` bps, and well past it the year's
            // exponentiation itself overflows; one stressed pool would otherwise fail `get_market_data`.
            if apr_bps >= APY_SATURATING_APR_BPS {
                return Ok(u32::MAX);
            }
            let multiplier =
                self.config.accrual_model.compute_multiplier(e, apr_bps, SECONDS_IN_YEAR)?;
            multiplier_to_percentage_increase(multiplier)
        };

        Ok(AnnualPercentageYields {
            borrow_bps: apy_bps(borrow_apr)?,
            supply_bps: apy_bps(supply_apr)?,
        })
    }

    // Computes the current utilization ratio in basis points (bps)
    pub fn compute_utilization_ratio_bps(&self) -> Result<i128, MCError> {
        let total = self.total_supply()?;

        if total == 0 {
            // Debt with no net backing left (fees outgrew it after a write-off) is full utilization.
            Ok(if self.total_borrowed > 0 { BPS_FACTOR } else { 0 })
        } else {
            // Above 100 % once fees exceed the cash; the curve is defined only up to full utilization.
            Ok(self
                .total_borrowed
                .fixed_div_ceil(total, BPS_FACTOR)
                .map_over_or_underflow()?
                .min(BPS_FACTOR))
        }
    }
}

// -- Helpers --

/// The compounded APR at which a year's APY first exceeds `u32::MAX` bps is 129 704 bps; the APY saturates
/// from here on.
pub const APY_SATURATING_APR_BPS: i128 = 130_000;

fn multiplier_to_percentage_increase(multiplier: i128) -> Result<u32, MCError> {
    const SCALE_DIVISOR: i128 = SCALED_FIXED_POINT_DENOMINATOR / BPS_FACTOR;

    if multiplier < 0 {
        return Err(MCError::OverOrUnderflow);
    }
    let increase_bps = multiplier / SCALE_DIVISOR - BPS_FACTOR;
    Ok(increase_bps.clamp(0, u32::MAX as i128) as u32)
}
