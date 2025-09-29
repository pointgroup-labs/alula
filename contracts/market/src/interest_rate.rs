//! `JLend` for now uses kinked interest rates.
//! See: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]

use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Env, contracttype};

use crate::{
    accrual::Accrual,
    constants::{BPS_FACTOR, SCALED_FIXED_POINT_DENOMINATOR, SECONDS_IN_YEAR},
    error::MCError,
    events,
    interest_rate_model::InterestRate,
    math_utils::MathUtils,
    pool::Pool,
};

/// Linear annual interest rates represented in basis points
#[derive(Debug, Eq, PartialEq)]
#[contracttype]
pub struct AnnualPercentageRates {
    pub borrow_bps: u64,
    pub supply_bps: u64,
}

impl AnnualPercentageRates {
    pub fn try_new(borrow_bps: i128, utilization_ratio_bps: i128) -> Result<Self, MCError> {
        let supply_bps = borrow_bps
            .fixed_mul_floor(utilization_ratio_bps, BPS_FACTOR)
            .map_over_or_underflow()?;

        Ok(Self {
            borrow_bps: borrow_bps as u64,
            supply_bps: supply_bps as u64,
        })
    }
}

/// Compound interest rates represented in basis points
#[derive(Debug, Eq, PartialEq)]
#[contracttype]
pub struct AnnualPercentageYields {
    pub borrow_bps: u32,
    pub supply_bps: u32,
}

impl Pool {
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

        let seconds_passed = current_timestamp - self.last_accrual_timestamp; // safe
        if seconds_passed == 0 {
            // NB: No time passed, no interest to accrue
            return Ok(());
        }

        let utilization_ratio_bps = self.calculate_utilization_ratio_bps()?;

        let current_borrow_apr = self
            .interest_rate_model
            .compute_borrow_apr(utilization_ratio_bps)?;
        let accrual_multiplier = self
            .accrual_model
            .calculate_multiplier(current_borrow_apr, seconds_passed)?;

        let new_total_borrowed = self
            .total_borrowed
            .fixed_mul_ceil(accrual_multiplier, SCALED_FIXED_POINT_DENOMINATOR)
            .map_over_or_underflow()?;

        let accrued = new_total_borrowed
            .checked_sub(self.total_borrowed)
            .map_over_or_underflow()?;

        let accrued_to_reserve = accrued
            .fixed_mul_ceil(self.fee_config.take_rate_bps as i128, BPS_FACTOR)
            .map_over_or_underflow()?;
        self.accumulated_reserve_fee = self
            .accumulated_reserve_fee
            .checked_add(accrued_to_reserve)
            .map_over_or_underflow()?;

        self.total_borrowed = new_total_borrowed;
        self.last_accrual_timestamp = current_timestamp;

        Ok(())
    }

    pub fn get_apr(&self) -> Result<AnnualPercentageRates, MCError> {
        let utilization_ratio_bps = self.calculate_utilization_ratio_bps()?;

        let borrow_apr_bps = self
            .interest_rate_model
            .compute_borrow_apr(utilization_ratio_bps)?;
        let res = AnnualPercentageRates::try_new(borrow_apr_bps, utilization_ratio_bps)?;

        Ok(res)
    }

    pub fn get_apy(&self) -> Result<AnnualPercentageYields, MCError> {
        let utilization_ratio_bps = self.calculate_utilization_ratio_bps()?;

        let borrow_apr = self
            .interest_rate_model
            .compute_borrow_apr(utilization_ratio_bps)?;
        let supply_apr = borrow_apr
            .fixed_mul_floor(utilization_ratio_bps, BPS_FACTOR)
            .map_over_or_underflow()?;

        let borrow_apy_multiplier = self
            .accrual_model
            .calculate_multiplier(borrow_apr, SECONDS_IN_YEAR)?;
        let supply_apy_multiplier = self
            .accrual_model
            .calculate_multiplier(supply_apr, SECONDS_IN_YEAR)?;

        let borrow_apy_bps = multiplier_to_percentage_increase(borrow_apy_multiplier)?;
        let supply_apy_bps = multiplier_to_percentage_increase(supply_apy_multiplier)?;

        let apy = AnnualPercentageYields {
            borrow_bps: borrow_apy_bps,
            supply_bps: supply_apy_bps,
        };

        Ok(apy)
    }

    pub fn calculate_utilization_ratio_bps(&self) -> Result<i128, MCError> {
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
