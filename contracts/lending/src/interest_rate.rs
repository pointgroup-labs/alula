//! `JLend` for now uses kinked interest rates.
//! See: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]

use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Env, contracttype};

use crate::{
    LCError,
    accrual::Accrual,
    constants::{BPS_FACTOR, SECONDS_IN_YEAR},
    events,
    interest_rate_model::InterestRate,
    math_utils::MathUtils,
    pool::Pool,
};

pub const SCALED_ONE: i128 = 10_000_000_000_000;

/// Interest rate multipliers presented as (1 + xxx) where `xxx` is a compound interest rate.
/// The real multiplier(e.g. 1.32, 2.53, etc) is scaled up with [`SCALED_ONE`] value.
// #[derive(Debug)]
// #[contracttype]
// pub struct CompoundRateMultipliers {
//     pub borrow: i128,
//     pub supply: i128,
// }

/// Compound interest rates represented in basis points
#[derive(Debug)]
#[contracttype]
pub struct AnnualPercentageYields {
    pub borrow_bps: u32,
    pub supply_bps: u32,
}

fn multiplier_to_percentage_yield(multiplier: i128) -> Result<u32, LCError> {
    const SCALE_DIVISOR: i128 = SCALED_ONE / BPS_FACTOR;

    let multiplier_bps =
        u32::try_from(multiplier / SCALE_DIVISOR).map_err(|_| LCError::OverOrUnderflow)?;
    let multiplier_yield_bps = multiplier_bps.saturating_sub(BPS_FACTOR as u32);

    Ok(multiplier_yield_bps)
}

impl Pool {
    pub fn accrue_interest(&mut self, e: &Env) -> Result<(), LCError> {
        let current_timestamp = e.ledger().timestamp();
        if current_timestamp < self.last_accrual_timestamp {
            events::current_ledger_timestamp_smaller_than_stored_timestamp(
                e,
                current_timestamp,
                self.last_accrual_timestamp,
            );

            return Err(LCError::InternalError);
        }

        let seconds_passed = current_timestamp - self.last_accrual_timestamp; // safe
        if seconds_passed == 0 {
            // No time passed, no interest to accrue
            return Ok(());
        }
        let utilization_ratio_bps = self.compute_utilization_ratio_bps()?;

        let current_borrow_apr = self
            .interest_rate_model
            .compute_borrow_apr(utilization_ratio_bps as u64)?;
        let accrual_multiplier = self
            .accrual_model
            .calculate_multiplier(current_borrow_apr as i128, seconds_passed as u32)?;

        let new_total_borrowed = self
            .total_borrowed
            .fixed_mul_ceil(accrual_multiplier, 10 * SCALED_ONE)
            .map_over_or_underflow()?;

        self.total_borrowed = new_total_borrowed;
        self.last_accrual_timestamp = current_timestamp;

        Ok(())
    }

    pub fn compute_utilization_ratio_bps(&self) -> Result<u32, LCError> {
        let total_supply = self.total_supply()?;

        let res = if total_supply == 0 {
            0
        } else {
            // 'utilization_ratio_bps' = (total_borrowed * 10_000)/total_supply
            self.total_borrowed
                .fixed_div_ceil(total_supply, BPS_FACTOR)
                .map_over_or_underflow()?
        } as u32; // safe

        Ok(res)
    }

    pub fn get_apy(&self) -> Result<AnnualPercentageYields, LCError> {
        let utilization_ratio_bps = self.compute_utilization_ratio_bps()?;

        let borrow_apr = self
            .interest_rate_model
            .compute_borrow_apr(utilization_ratio_bps as u64)?;
        let supply_apr = borrow_apr
            .fixed_mul_floor(utilization_ratio_bps as u64, BPS_FACTOR as u64)
            .map_over_or_underflow()?;

        let borrow_apy_multiplier = self
            .accrual_model
            .calculate_multiplier(borrow_apr as i128, SECONDS_IN_YEAR as u32)?;
        let supply_apy_multiplier = self
            .accrual_model
            .calculate_multiplier(supply_apr as i128, SECONDS_IN_YEAR as u32)?;

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
    // TODO: We better pre-compute the max available amount during Pool initialization
    pub fn compute_available_borrow(&self, e: &Env) -> Result<i128, LCError> {
        let total_supply = self.total_supply()?;
        let utilization_ratio = self.calculate_utilization_ratio_for_total_bps(total_supply)?;

        if utilization_ratio > self.config.utilization_ratio_limit_bps {
            // NB: This can happen when the `total_borrowed` amount on a pool has accrued over time
            // by itself, so for now, we simply emit an event. We can agree to stop
            // accruing interest on a pool if this happens
            events::utilization_ratio_exceeds_limit(
                e,
                utilization_ratio,
                self.config.utilization_ratio_limit_bps,
            );
            // return Err(LCError::InternalError);
        }
        let available_percentage_to_borrow_bps =
            self.config.utilization_ratio_limit_bps - utilization_ratio; // safe

        total_supply
            .fixed_mul_ceil(available_percentage_to_borrow_bps, BPS_FACTOR)
            .map_over_or_underflow()
    }

    fn calculate_utilization_ratio_for_total_bps(&self, total: i128) -> Result<i128, LCError> {
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
            total_d_tokens_amount: 0,
            total_j_tokens_amount: 0,
            available: 1_000_000,
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

    // #[test]
    // fn test_accrue_interest_with_time_passed() {
    //     let env = Env::default();

    //     let mut pool = create_test_pool(&env);
    //     pool.total_borrowed = 1_000_000; // Larger borrowed amount for noticeable interest
    //     pool.available = 9_000_000;
    //     pool.last_accrual_timestamp = 0;
    //     // pool.last_accrual = ACCRUAL_INIT;

    //     let time_passed = 24 * 60 * 60; // 1 day (24 hours) for meaningful interest accrual
    //     env.ledger().with_mut(|li| li.timestamp += time_passed);

    //     let initial_total_borrowed = pool.total_borrowed;
    //     // let initial_accrual = pool.last_accrual;

    //     // Capture the expected multiplier BEFORE calling accrue_interest
    //     // since accrue_interest will change the pool state
    //     let expected_multipliers = pool.get_compound_rate_multipliers(time_passed).unwrap();
    //     let expected_new_accrual = initial_accrual
    //         .fixed_mul_ceil(expected_multipliers.borrow, SCALED_ONE)
    //         .unwrap();
    //     let expected_new_total_borrowed = initial_total_borrowed
    //         .fixed_div_floor(initial_accrual, expected_new_accrual)
    //         .map_over_or_underflow()
    //         .unwrap();

    //     let result = pool.accrue_interest(&env);
    //     assert!(result.is_ok());

    //     // Verify that interest was accrued
    //     assert!(
    //         pool.total_borrowed > initial_total_borrowed,
    //         "Expected total_borrowed {} to be greater than initial {}",
    //         pool.total_borrowed,
    //         initial_total_borrowed
    //     );
    //     assert!(
    //         pool.last_accrual > initial_accrual,
    //         "Expected last_accrual {} to be greater than initial {}",
    //         pool.last_accrual,
    //         initial_accrual
    //     );
    //     assert_eq!(pool.last_accrual_timestamp, time_passed);

    //     // Verify the calculations match our expectations
    //     assert_eq!(pool.last_accrual, expected_new_accrual);
    //     assert_eq!(pool.total_borrowed, expected_new_total_borrowed);

    //     // Verify that the interest accrued is reasonable (should be a small percentage)
    //     let interest_accrued = pool.total_borrowed - initial_total_borrowed;
    //     assert!(interest_accrued > 0, "Interest accrued should be positive");
    //     assert!(
    //         interest_accrued < initial_total_borrowed / 10,
    //         "Interest accrued shouldn't be more than 10% for one day"
    //     );
    // }

    // #[test]
    // fn test_get_borrow_rate_per_second_zero_total() {
    //     let env = Env::default();
    //     let mut pool = create_test_pool(&env);
    //     pool.total_borrowed = 0;
    //     pool.available = 0;

    //     let rate = pool.get_borrow_rate_per_second().unwrap();
    //     assert_eq!(rate, pool.config.base_rate_per_second);
    // }

    // #[test]
    // fn test_get_borrow_rate_per_second_below_optimal() {
    //     let env = Env::default();
    //     let mut pool = create_test_pool(&env);
    //     pool.total_borrowed = 300_000; // 30% utilization
    //     pool.available = 700_000;

    //     let rate = pool.get_borrow_rate_per_second().unwrap();

    //     // Should be base rate + slope1 * utilization
    //     let expected = pool.config.base_rate_per_second + (pool.config.slope1 * 3000); // 30% in
    // bps     assert_eq!(rate, expected);
    // }

    // #[test]
    // fn test_get_borrow_rate_per_second_above_optimal() {
    //     let env = Env::default();
    //     let mut pool = create_test_pool(&env);
    //     pool.total_borrowed = 900_000; // 90% utilization (above 80% optimal)
    //     pool.available = 100_000;

    //     let rate = pool.get_borrow_rate_per_second().unwrap();

    //     // Should use both slopes
    //     let pre_threshold = pool.config.base_rate_per_second
    //         + (pool.config.slope1 * pool.config.optimal_utilization_ratio_bps);
    //     let excess_rate = (9000 - pool.config.optimal_utilization_ratio_bps) *
    // pool.config.slope2;     let expected = pre_threshold + excess_rate;

    //     assert_eq!(rate, expected);
    // }

    // #[test]
    // fn test_get_compound_rate_multipliers() {
    //     let env = Env::default();
    //     let mut pool = create_test_pool(&env);
    //     pool.total_borrowed = 500_000;
    //     pool.available = 500_000;

    //     let multipliers = pool.get_compound_rate_multipliers(24 * 60 * 60).unwrap();

    //     assert!(multipliers.borrow > SCALED_ONE);
    //     assert!(multipliers.supply > SCALED_ONE);
    //     assert!(multipliers.borrow > multipliers.supply);
    // }

    // #[test]
    // fn test_calculate_supply_multiplier_zero_total() {
    //     let env = Env::default();
    //     let mut pool = create_test_pool(&env);
    //     pool.total_borrowed = 0;
    //     pool.available = 0;

    //     let supply_multiplier = pool
    //         .calculate_supply_multiplier(SCALED_ONE + 100000)
    //         .unwrap();
    //     assert_eq!(supply_multiplier, SCALED_ONE);
    // }

    // #[test]
    // fn test_calculate_supply_multiplier_with_utilization() {
    //     let env = Env::default();
    //     let mut pool = create_test_pool(&env);
    //     pool.total_borrowed = 800000;
    //     pool.available = 200000;

    //     let borrow_multiplier = (SCALED_ONE * 105) / 100; // 5% increase
    //     let supply_multiplier = pool.calculate_supply_multiplier(borrow_multiplier).unwrap();

    //     assert!(supply_multiplier > SCALED_ONE);
    //     assert!(supply_multiplier < borrow_multiplier); // Supply rate should be lower
    // }

    #[test]
    fn test_get_apy() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 500000;
        pool.available = 500000;

        let apy = pool.get_apy().unwrap();

        assert!(apy.borrow_bps > 0);
        assert!(apy.supply_bps > 0);
        assert!(apy.borrow_bps > apy.supply_bps);
    }

    // #[test]
    // fn test_update_accruals() {
    //     let env = Env::default();
    //     let mut pool = create_test_pool(&env);
    //     pool.total_borrowed = 100000;
    //     pool.last_accrual = ACCRUAL_INIT;

    //     let borrow_multiplier = (101 * SCALED_ONE) / 100; // 1% increase
    //     let timestamp = 3600;

    //     let initial_total_borrowed = pool.total_borrowed;
    //     let initial_accrual = pool.last_accrual;

    //     let result = pool.update_accruals(borrow_multiplier, timestamp);
    //     assert!(result.is_ok());

    //     assert!(pool.total_borrowed > initial_total_borrowed);
    //     assert!(pool.last_accrual > initial_accrual);
    //     assert_eq!(pool.last_accrual_timestamp, timestamp);
    // }

    #[test]
    fn test_calculate_utilization_ratio_for_total_bps() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 300000;

        let total = 1000000;
        let ratio = pool
            .calculate_utilization_ratio_for_total_bps(total)
            .unwrap();

        assert_eq!(ratio, 3000); // 30% in basis points
    }

    // #[test]
    // fn test_calculate_pre_threshold_rate() {
    //     let env = Env::default();
    //     let pool = create_test_pool(&env);

    //     let utilization_bps = 5000; // 50%
    //     let rate = pool.calculate_pre_threshold_rate(utilization_bps).unwrap();

    //     let expected = pool.config.base_rate_per_second + (pool.config.slope1 * utilization_bps);
    //     assert_eq!(rate, expected);
    // }

    // #[test]
    // fn test_calculate_post_threshold_rate() {
    //     let env = Env::default();
    //     let pool = create_test_pool(&env);

    //     let utilization_bps = 9000; // 90%
    //     let rate = pool.calculate_post_threshold_rate(utilization_bps).unwrap();

    //     let pre_threshold = pool.config.base_rate_per_second
    //         + (pool.config.slope1 * pool.config.optimal_utilization_ratio_bps);
    //     let excess_rate =
    //         (utilization_bps - pool.config.optimal_utilization_ratio_bps) * pool.config.slope2;
    //     let expected = pre_threshold + excess_rate;

    //     assert_eq!(rate, expected);
    // }

    // #[test]
    // fn test_edge_case_max_utilization() {
    //     let env = Env::default();
    //     let mut pool = create_test_pool(&env);
    //     pool.total_borrowed = 1000000;
    //     pool.available = 0; // 100% utilization

    //     let rate = pool.get_borrow_rate_per_second().unwrap();
    //     assert!(rate > pool.config.base_rate_per_second);
    // }

    // #[test]
    // fn test_compound_rate_multipliers_precision() {
    //     let env = Env::default();
    //     let mut pool = create_test_pool(&env);
    //     pool.total_borrowed = 1;
    //     pool.available = 999999;

    //     let multipliers = pool.get_compound_rate_multipliers(1).unwrap();

    //     // With very low utilization and short time, multipliers should be very close to SCALED_ONE
    //     assert!(multipliers.borrow > SCALED_ONE);
    //     assert!(multipliers.supply >= SCALED_ONE);
    // }

    // #[test]
    // fn test_interest_rate_consistency() {
    //     let env = Env::default();
    //     let mut pool = create_test_pool(&env);
    //     pool.total_borrowed = 500000;
    //     pool.available = 500000;

    //     // Test that longer periods result in higher multipliers
    //     let short_period = pool.get_compound_rate_multipliers(60 * 60).unwrap(); // 1 hour
    //     let long_period = pool.get_compound_rate_multipliers(24 * 60 * 60).unwrap(); // 1 day

    //     assert!(long_period.borrow > short_period.borrow);
    //     assert!(long_period.supply > short_period.supply);
    // }
}
