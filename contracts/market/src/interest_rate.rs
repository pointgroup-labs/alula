//! `JLend` for now uses kinked interest rates.
//! See: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]

use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Env, contracttype};

use crate::{
    accrual::Accrual,
    constants::{BPS_FACTOR, SECONDS_IN_YEAR},
    error::MCError,
    events,
    interest_rate_model::InterestRate,
    math_utils::MathUtils,
    pool::Pool,
};

pub const SCALED_ONE: i128 = 100_000_000_000_000;

/// Linear annual interest rates represented in basis points
#[derive(Debug, Eq, PartialEq)]
#[contracttype]
pub struct AnnualPercentageRates {
    pub borrow_bps: u32,
    pub supply_bps: u32,
}

impl AnnualPercentageRates {
    pub fn try_new(borrow_bps: u32, utilization_ratio_bps: u32) -> Result<Self, MCError> {
        let supply_bps = (borrow_bps as u64)
            .fixed_mul_floor(utilization_ratio_bps as u64, BPS_FACTOR as u64)
            .map_over_or_underflow()? as u32;

        Ok(Self {
            borrow_bps,
            supply_bps,
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

fn multiplier_to_percentage_yield(multiplier: i128) -> Result<u32, MCError> {
    const SCALE_DIVISOR: i128 = SCALED_ONE / BPS_FACTOR;

    let multiplier_bps =
        u32::try_from(multiplier / SCALE_DIVISOR).map_err(|_| MCError::OverOrUnderflow)?;
    let multiplier_yield_bps = multiplier_bps.saturating_sub(BPS_FACTOR as u32);

    Ok(multiplier_yield_bps)
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

        let utilization_ratio_bps = self.compute_utilization_ratio_bps()?;

        let current_borrow_apr = self
            .interest_rate_model
            .compute_borrow_apr(utilization_ratio_bps)?;
        let accrual_multiplier = self
            .accrual_model
            .calculate_multiplier(current_borrow_apr, seconds_passed)?;

        let new_total_borrowed = self
            .total_borrowed
            .fixed_mul_ceil(accrual_multiplier, SCALED_ONE)
            .map_over_or_underflow()?;

        self.total_borrowed = new_total_borrowed;
        self.last_accrual_timestamp = current_timestamp;

        Ok(())
    }

    pub fn compute_utilization_ratio_bps(&self) -> Result<i128, MCError> {
        let total_supply = self.total_supply()?;

        let res = if total_supply == 0 {
            0
        } else {
            // 'utilization_ratio_bps' = (total_borrowed * 10_000)/total_supply
            self.total_borrowed
                .fixed_div_ceil(total_supply, BPS_FACTOR)
                .map_over_or_underflow()?
        };

        Ok(res)
    }

    pub fn get_apy(&self) -> Result<AnnualPercentageYields, MCError> {
        let utilization_ratio_bps = self.compute_utilization_ratio_bps()?;

        let borrow_apr = self
            .interest_rate_model
            .compute_borrow_apr(utilization_ratio_bps)?;
        let supply_apr = borrow_apr
            .fixed_mul_floor(utilization_ratio_bps, BPS_FACTOR)
            .map_over_or_underflow()?;

        let borrow_apy_multiplier = self
            .accrual_model
            .calculate_multiplier(borrow_apr as i128, SECONDS_IN_YEAR)?;
        let supply_apy_multiplier = self
            .accrual_model
            .calculate_multiplier(supply_apr as i128, SECONDS_IN_YEAR)?;

        let borrow_apy_bps = multiplier_to_percentage_yield(borrow_apy_multiplier)?;
        let supply_apy_bps = multiplier_to_percentage_yield(supply_apy_multiplier)?;

        let apy = AnnualPercentageYields {
            borrow_bps: borrow_apy_bps,
            supply_bps: supply_apy_bps,
        };

        Ok(apy)
    }

    /// Computes the maximum available amount for borrowing that doesn't exceed the utilization
    /// ratio limit on a pool
    pub fn compute_available_utilization_ratio_cap_borrow(&self, e: &Env) -> Result<i128, MCError> {
        let total_supply = self.total_supply()?;
        let utilization_ratio = self.calculate_utilization_ratio_bps()?;

        if utilization_ratio > self.config.utilization_ratio_limit_bps {
            // NB: This can happen when the `total_borrowed` amount on a pool has accrued over time
            // by itself, so for now, we simply emit an event. We can agree to stop
            // accruing interest on a pool if this happens
            events::utilization_ratio_exceeds_limit(
                e,
                utilization_ratio,
                self.config.utilization_ratio_limit_bps,
            );

            return Ok(0);
        }
        let available_percentage_to_borrow_bps =
            self.config.utilization_ratio_limit_bps - utilization_ratio; // safe

        total_supply
            .fixed_mul_ceil(available_percentage_to_borrow_bps, BPS_FACTOR)
            .map_over_or_underflow()
    }

    fn calculate_utilization_ratio_bps(&self) -> Result<i128, MCError> {
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

#[cfg(test)]
mod tests {
    use soroban_sdk::{
        Address, Env, String, symbol_short,
        testutils::{Address as _, Ledger},
    };

    use super::*;
    use crate::{
        accrual::AccrualModel,
        interest_rate_model::{InterestRateModel, kinked::KinkedIRConfig},
        pool::PoolConfig,
    };

    fn create_test_pool(e: &Env) -> Pool {
        let token_address = Address::generate(e);

        Pool {
            token_address: token_address.clone(),
            pool_address: token_address,
            token_ticker: symbol_short!("TEST"),
            total_borrowed: 0,
            total_d_tokens: 0,
            total_j_tokens: 0,
            total_available: 1_000_000,
            total_collateral: 0,
            config: PoolConfig::default(),
            last_accrual_timestamp: 0,
            interest_rate_model: InterestRateModel::Kinked(KinkedIRConfig::default()),
            accrual_model: AccrualModel::Compounded,
            name: String::from_str(
                e,
                "TEST:CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
            ),
        }
    }

    // #[test]
    // fn test_compound_rate_multipliers_to_compound_rates() {
    //     let multipliers = CompoundRateMultipliers {
    //         borrow: 1320700048000, // 1.3207 multiplier
    //         supply: 1000000000000, // 1.0 multiplier
    //     };

    //     let rates: CompoundRates = multipliers.try_into().unwrap();

    //     assert_eq!(rates.borrow_bps, 3207); // 32.07%
    //     assert_eq!(rates.supply_bps, 0); // 0%
    // }

    // #[test]
    // fn test_compound_rate_multipliers_conversion_small_values() {
    //     let multipliers = CompoundRateMultipliers {
    //         borrow: 1001000000000, // 1.001 multiplier (0.1% rate)
    //         supply: 1000500000000, // 1.0005 multiplier (0.05% rate)
    //     };

    //     let rates: CompoundRates = multipliers.try_into().unwrap();

    //     assert_eq!(rates.borrow_bps, 10); // 0.1% = 10 bps
    //     assert_eq!(rates.supply_bps, 5); // 0.05% = 5 bps
    // }

    // #[test]
    // fn test_compound_rate_multipliers_conversion_overflow() {
    //     let multipliers = CompoundRateMultipliers {
    //         borrow: i128::MAX,
    //         supply: SCALED_ONE,
    //     };

    //     let result: Result<CompoundRates, _> = multipliers.try_into();
    //     assert!(result.is_err());
    // }

    // #[test]
    // fn test_compound_rate_multipliers_conversion_underflow() {
    //     let multipliers = CompoundRateMultipliers {
    //         borrow: (SCALED_ONE * 9) / 10, // Less than SCALED_ONE
    //         supply: SCALED_ONE,
    //     };

    //     let result: Result<CompoundRates, _> = multipliers.try_into();
    //     assert!(result.is_err());
    // }

    #[test]
    fn test_accrue_interest_no_time_passed() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.last_accrual_timestamp = 100;

        env.ledger().with_mut(|li| li.timestamp += 100);

        let result = pool.accrue_interest(&env);
        assert!(result.is_ok());

        // Values should remain unchanged
        // assert_eq!(pool.last_accrual, ACCRUAL_INIT);
        assert_eq!(pool.last_accrual_timestamp, 100);
    }

    #[test]
    fn test_accrue_interest_invalid_timestamp() {
        let env = Env::default();

        let mut pool = create_test_pool(&env);
        pool.last_accrual_timestamp = 200;

        env.ledger().with_mut(|li| li.timestamp += 100);

        let result = pool.accrue_interest(&env);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_apy() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 500000;
        pool.total_available = 500000;

        let apy = pool.get_apy().unwrap();

        assert!(apy.borrow_bps > 0);
        assert!(apy.supply_bps > 0);
        assert!(apy.borrow_bps > apy.supply_bps);
    }
}
