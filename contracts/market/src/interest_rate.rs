use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Env, Vec, contracttype, vec as svec};

use crate::{
    accrual::Accrual,
    constants::*,
    error::MCError,
    events,
    interest_rate_model::InterestRate,
    math_utils::MathUtils,
    pool::{Pool, PoolBootstrapPeriod},
};

/// Compound interest rates represented in basis points
#[derive(Debug, Eq, PartialEq, Clone)]
#[contracttype]
pub struct AnnualPercentageYields {
    pub borrow_bps: u32,
    pub supply_bps: u32,
}

impl Pool {
    /// Accrues interest on the pool's total borrowed amount based
    /// on the time elapsed since the last accrual
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
        let current_borrow_apr =
            self.config.interest_rate_model.compute_borrow_apr(utilization_ratio_bps)?;
        let accrual_multiplier =
            self.config.accrual_model.compute_multiplier(current_borrow_apr, seconds_passed)?;

        let new_total_borrowed = self
            .total_borrowed
            .fixed_mul_ceil(accrual_multiplier, SCALED_FIXED_POINT_DENOMINATOR)
            .map_over_or_underflow()?;
        let accrued =
            new_total_borrowed.checked_sub(self.total_borrowed).map_over_or_underflow()?;
        let accrued_to_reserve = accrued
            .fixed_mul_ceil(self.config.fee_config.take_rate_bps as i128, BPS_FACTOR)
            .map_over_or_underflow()?;
        let new_accumulated_reserve_fees = self
            .accumulated_reserve_fees
            .checked_add(accrued_to_reserve)
            .map_over_or_underflow()?;

        self.accumulated_reserve_fees = new_accumulated_reserve_fees;
        self.total_borrowed = new_total_borrowed;
        self.last_accrual_timestamp = current_timestamp;

        self.borrow_apr_bps = current_borrow_apr;
        self.supply_apr_bps = current_borrow_apr
            .fixed_mul_floor(utilization_ratio_bps, BPS_FACTOR)
            .map_over_or_underflow()?
            .fixed_mul_floor(BPS_FACTOR - self.config.fee_config.take_rate_bps as i128, BPS_FACTOR)
            .map_over_or_underflow()?; // safe

        // -- Accrue supply APR bootstraps --

        let mut updated_periods: Vec<((u64, u64), PoolBootstrapPeriod)> = svec![e];
        let mut outdated_periods: Vec<(u64, u64)> = svec![e];

        for ((start_period, end_period), mut pool_bootstrap_period) in self.bootstrap_periods.iter()
        {
            if end_period <= current_timestamp {
                let new_total_available = self
                    .total_available
                    .checked_add(pool_bootstrap_period.remaining_amount)
                    .map_over_or_underflow()?;

                self.total_available = new_total_available;
                outdated_periods.push_back((start_period, end_period));
            } else if current_timestamp > start_period && current_timestamp < end_period {
                let remaining_time_period = end_period - current_timestamp; // safe
                let remaining_time_ratio = remaining_time_period
                    .fixed_div_ceil(end_period - start_period, BPS_FACTOR as u64)
                    .map_over_or_underflow()?; // safe

                let new_remaining_amount = pool_bootstrap_period
                    .total_amount
                    .fixed_mul_floor(remaining_time_ratio as i128, BPS_FACTOR)
                    .map_over_or_underflow()?;
                let diff = pool_bootstrap_period.remaining_amount - new_remaining_amount; // safe

                let new_total_available =
                    self.total_available.checked_add(diff).map_over_or_underflow()?;
                self.total_available = new_total_available;

                pool_bootstrap_period.remaining_amount = new_remaining_amount;
                updated_periods.push_back(((start_period, end_period), pool_bootstrap_period));
            }
        }

        for outdated_period in outdated_periods {
            self.bootstrap_periods.remove(outdated_period);
        }
        for (period, updated_period) in updated_periods {
            self.bootstrap_periods.set(period, updated_period);
        }

        Ok(())
    }

    /// Get current annual percentage yields (APY) for borrowing and supplying
    /// based on the pool's utilization ratio, interest rate model, and accrual model
    pub fn get_apy(&self) -> Result<AnnualPercentageYields, MCError> {
        let utilization_ratio_bps = self.compute_utilization_ratio_bps()?;

        let borrow_apr =
            self.config.interest_rate_model.compute_borrow_apr(utilization_ratio_bps)?;
        let supply_apr = borrow_apr
            .fixed_mul_floor(utilization_ratio_bps, BPS_FACTOR)
            .map_over_or_underflow()?
            .fixed_mul_floor(BPS_FACTOR - self.config.fee_config.take_rate_bps as i128, BPS_FACTOR)
            .map_over_or_underflow()?; // safe

        let borrow_apy_multiplier =
            self.config.accrual_model.compute_multiplier(borrow_apr, SECONDS_IN_YEAR)?;
        let supply_apy_multiplier =
            self.config.accrual_model.compute_multiplier(supply_apr, SECONDS_IN_YEAR)?;

        let borrow_apy_bps = multiplier_to_percentage_increase(borrow_apy_multiplier)?;
        let supply_apy_bps = multiplier_to_percentage_increase(supply_apy_multiplier)?;

        let apy = AnnualPercentageYields { borrow_bps: borrow_apy_bps, supply_bps: supply_apy_bps };

        Ok(apy)
    }

    /// Computes the current utilization ratio in basis points (bps)
    pub fn compute_utilization_ratio_bps(&self) -> Result<i128, MCError> {
        // WARN: Is this a correct way to count UR now, when we have reserves?
        let total = self.total_supply()?;

        if total == 0 {
            Ok(0)
        } else {
            self.total_borrowed
                // TODO: Investigate why using `floor` here breaks fuzzing tests
                .fixed_div_ceil(total, BPS_FACTOR)
                .map_over_or_underflow()
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
