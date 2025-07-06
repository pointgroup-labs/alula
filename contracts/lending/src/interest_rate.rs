//! `JLend` for now uses kinked interest rates.
//! See: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]

use {
    crate::{
        constants::{ACCRUAL_INIT, BPS_FACTOR, SECONDS_IN_YEAR},
        math_utils::{self, MathUtils},
        pool::Pool,
        LCError,
    },
    soroban_fixed_point_math::FixedPoint,
    soroban_sdk::{contracttype, Env},
};

pub const SCALED_ONE: i128 = ACCRUAL_INIT;

/// Interest rate multipliers presented as (1 + xxx) where `xxx` is a compound interest rate.
/// The real multiplier(e.g. 1.32, 2.53, etc) is scaled up with [`SCALED_ONE`] value.
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

    fn try_from(value: CompoundRateMultipliers) -> Result<Self, Self::Error> {
        const SCALE_DIVISOR: i128 = SCALED_ONE / BPS_FACTOR;

        let borrow_multiplier_bps =
            u32::try_from(value.borrow / SCALE_DIVISOR).map_err(|_| LCError::OverOrUnderflow)?;
        let supply_multiplier_bps =
            u32::try_from(value.supply / SCALE_DIVISOR).map_err(|_| LCError::OverOrUnderflow)?;

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

        if current_timestamp < self.last_accrual_timestamp {
            return Err(LCError::InvalidTimestamp);
        }

        let seconds_passed = current_timestamp - self.last_accrual_timestamp;
        if seconds_passed == 0 {
            return Ok(()); // No time passed, no interest to accrue
        }

        let borrow_multiplier = self.get_compound_rate_multipliers(seconds_passed)?.borrow;
        self.update_accruals(borrow_multiplier, current_timestamp)?;

        Ok(())
    }

    fn update_accruals(&mut self, borrow_multiplier: i128, timestamp: u64) -> Result<(), LCError> {
        let new_accrual = self
            .last_accrual
            .fixed_mul_ceil(borrow_multiplier, SCALED_ONE)
            .map_over_or_underflow()?;

        let new_total_borrowed = self
            .total_borrowed
            .checked_mul(new_accrual)
            .map_over_or_underflow()?
            .checked_div(self.last_accrual)
            .map_over_or_underflow()?;

        self.total_borrowed = new_total_borrowed;
        self.last_accrual = new_accrual;
        self.last_accrual_timestamp = timestamp;

        Ok(())
    }

    pub fn get_apy(&self) -> Result<CompoundRates, LCError> {
        self.get_compound_rates(SECONDS_IN_YEAR)
    }

    fn get_compound_rates(&self, seconds_passed: u64) -> Result<CompoundRates, LCError> {
        self.get_compound_rate_multipliers(seconds_passed)?
            .try_into()
    }

    /// Calculates the compound rate multipliers for borrowing and supplying based on the time passed.
    ///
    /// # Arguments
    ///
    /// * `seconds_passed` - The number of seconds that have passed since the last calculation.
    ///
    /// # Returns
    ///
    /// On success, returns a `CompoundRateMultipliers` struct containing:
    /// - `borrow`: The compound multiplier for the borrow interest rate.
    /// - `supply`: The compound multiplier for the supply interest rate.
    ///
    /// # Errors
    ///
    /// Returns `LCError` in case of any errors during numerical operations
    pub fn get_compound_rate_multipliers(
        &self,
        seconds_passed: u64,
    ) -> Result<CompoundRateMultipliers, LCError> {
        let borrow_interest_rate = self.get_borrow_rate_per_second()?;
        let borrow = self.calculate_borrow_multiplier(borrow_interest_rate, seconds_passed)?;
        let supply = self.calculate_supply_multiplier(borrow)?;

        Ok(CompoundRateMultipliers { borrow, supply })
    }

    fn calculate_borrow_multiplier(
        &self,
        interest_rate: i128,
        seconds_passed: u64,
    ) -> Result<i128, LCError> {
        let growth_factor = SCALED_ONE + interest_rate;

        math_utils::bin_pow(growth_factor, seconds_passed, SCALED_ONE)
    }

    fn calculate_supply_multiplier(&self, borrow_multiplier: i128) -> Result<i128, LCError> {
        let total = self
            .total_borrowed
            .checked_add(self.available)
            .map_over_or_underflow()?;

        if total == 0 {
            // Is [`SCALED_ONE`], since if a pool doesn't yet have deposits, its next APY update must be
            // as a `deposit` which implies that its compound deposit interest will be set to 0 regardless.
            return Ok(SCALED_ONE);
        }

        let utilization_ratio_scaled = self
            .total_borrowed
            .fixed_div_floor(total, SCALED_ONE)
            .map_over_or_underflow()?;

        let interest_earned = borrow_multiplier
            .checked_sub(SCALED_ONE)
            .map_over_or_underflow()?;

        let supply_interest = interest_earned
            .fixed_mul_ceil(utilization_ratio_scaled, SCALED_ONE)
            .map_over_or_underflow()?;

        supply_interest
            .checked_add(SCALED_ONE)
            .map_over_or_underflow()
    }

    /// Calculates the borrow interest rate per second based on the kinked interest rate model.
    ///
    /// The rate is determined by:
    /// - Pool utilization ratio (borrowed / total liquidity)
    /// - Base rate and slope parameters from pool configuration
    ///
    /// # Rate Calculation
    /// - **Below optimal utilization**: `base_rate + (utilization_ratio * slope1)`
    /// - **Above optimal utilization**: `base_rate + (optimal_ur * slope1) + ((utilization_ratio - optimal_ur) * slope2)`
    ///
    /// # Returns
    /// Interest rate scaled by [`SCALED_ONE`] (e.g., `1000000000000` = 0.1% per second)
    ///
    /// # Errors
    /// Returns [`LCError::OverOrUnderflow`] if any arithmetic operation overflows
    pub fn get_borrow_rate_per_second(&self) -> Result<i128, LCError> {
        let total = self.total_liquidity()?;

        if total == 0 {
            return Ok(self.config.base_rate_per_second);
        }

        let utilization_ratio_bps = self.calculate_utilization_ratio_bps(total)?;
        self.calculate_interest_rate(utilization_ratio_bps)
    }

    fn calculate_utilization_ratio_bps(&self, total: i128) -> Result<i128, LCError> {
        self.total_borrowed
            .fixed_div_ceil(total, BPS_FACTOR)
            .map_over_or_underflow()
    }

    fn calculate_interest_rate(&self, utilization_ratio_bps: i128) -> Result<i128, LCError> {
        if utilization_ratio_bps < self.config.optimal_utilization_ratio_bps {
            self.calculate_pre_threshold_rate(utilization_ratio_bps)
        } else {
            self.calculate_post_threshold_rate(utilization_ratio_bps)
        }
    }

    fn calculate_pre_threshold_rate(&self, utilization_ratio_bps: i128) -> Result<i128, LCError> {
        self.config
            .base_rate_per_second
            .checked_add(
                self.config
                    .slope1
                    .checked_mul(utilization_ratio_bps)
                    .map_over_or_underflow()?,
            )
            .map_over_or_underflow()
    }

    fn calculate_post_threshold_rate(&self, utilization_ratio_bps: i128) -> Result<i128, LCError> {
        let pre_threshold_rate =
            self.calculate_pre_threshold_rate(self.config.optimal_utilization_ratio_bps)?;

        let excess_utilization = utilization_ratio_bps
            .checked_sub(self.config.optimal_utilization_ratio_bps)
            .map_over_or_underflow()?;

        let post_threshold_rate = excess_utilization
            .checked_mul(self.config.slope2)
            .map_over_or_underflow()?;

        pre_threshold_rate
            .checked_add(post_threshold_rate)
            .map_over_or_underflow()
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::pool::PoolConfig,
        soroban_sdk::{
            symbol_short,
            testutils::{Address as _, Ledger},
            Address, Env,
        },
    };

    fn create_test_pool(env: &Env) -> Pool {
        let token_address = Address::generate(env);

        Pool {
            token_address: token_address.clone(),
            pool_address: token_address,
            token_ticker: symbol_short!("TEST"),
            total_borrowed: 0,
            total_shares: 0,
            available: 1_000_000,
            total_collateral: 0,
            config: PoolConfig::default(),
            last_accrual: ACCRUAL_INIT,
            last_accrual_timestamp: 0,
        }
    }

    #[test]
    fn test_compound_rate_multipliers_to_compound_rates() {
        let multipliers = CompoundRateMultipliers {
            borrow: 1320700048000, // 1.3207 multiplier
            supply: 1000000000000, // 1.0 multiplier
        };

        let rates: CompoundRates = multipliers.try_into().unwrap();

        assert_eq!(rates.borrow_bps, 3207); // 32.07%
        assert_eq!(rates.supply_bps, 0); // 0%
    }

    #[test]
    fn test_compound_rate_multipliers_conversion_small_values() {
        let multipliers = CompoundRateMultipliers {
            borrow: 1001000000000, // 1.001 multiplier (0.1% rate)
            supply: 1000500000000, // 1.0005 multiplier (0.05% rate)
        };

        let rates: CompoundRates = multipliers.try_into().unwrap();

        assert_eq!(rates.borrow_bps, 10); // 0.1% = 10 bps
        assert_eq!(rates.supply_bps, 5); // 0.05% = 5 bps
    }

    #[test]
    fn test_compound_rate_multipliers_conversion_overflow() {
        let multipliers = CompoundRateMultipliers {
            borrow: i128::MAX,
            supply: SCALED_ONE,
        };

        let result: Result<CompoundRates, _> = multipliers.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_compound_rate_multipliers_conversion_underflow() {
        let multipliers = CompoundRateMultipliers {
            borrow: (SCALED_ONE * 9) / 10, // Less than SCALED_ONE
            supply: SCALED_ONE,
        };

        let result: Result<CompoundRates, _> = multipliers.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_accrue_interest_no_time_passed() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.last_accrual_timestamp = 100;

        env.ledger().with_mut(|li| li.timestamp = 100);

        let result = pool.accrue_interest(&env);
        assert!(result.is_ok());

        // Values should remain unchanged
        assert_eq!(pool.last_accrual, ACCRUAL_INIT);
        assert_eq!(pool.last_accrual_timestamp, 100);
    }

    #[test]
    fn test_accrue_interest_invalid_timestamp() {
        let env = Env::default();

        let mut pool = create_test_pool(&env);
        pool.last_accrual_timestamp = 200;

        env.ledger().with_mut(|li| li.timestamp = 100);

        let result = pool.accrue_interest(&env);
        assert!(result.is_err());
    }

    #[test]
    fn test_accrue_interest_with_time_passed() {
        let env = Env::default();

        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 1_000_000; // Larger borrowed amount for noticeable interest
        pool.available = 9_000_000;
        pool.last_accrual_timestamp = 0;
        pool.last_accrual = ACCRUAL_INIT;

        let time_passed = 24 * 60 * 60; // 1 day (24 hours) for meaningful interest accrual
        env.ledger().with_mut(|li| li.timestamp = time_passed);

        let initial_total_borrowed = pool.total_borrowed;
        let initial_accrual = pool.last_accrual;

        // Capture the expected multiplier BEFORE calling accrue_interest
        // since accrue_interest will change the pool state
        let expected_multipliers = pool.get_compound_rate_multipliers(time_passed).unwrap();
        let expected_new_accrual = initial_accrual
            .fixed_mul_ceil(expected_multipliers.borrow, SCALED_ONE)
            .unwrap();
        let expected_new_total_borrowed = initial_total_borrowed
            .checked_mul(expected_new_accrual)
            .unwrap()
            .checked_div(initial_accrual)
            .unwrap();

        let result = pool.accrue_interest(&env);
        assert!(result.is_ok());

        // Verify that interest was accrued
        assert!(
            pool.total_borrowed > initial_total_borrowed,
            "Expected total_borrowed {} to be greater than initial {}",
            pool.total_borrowed,
            initial_total_borrowed
        );
        assert!(
            pool.last_accrual > initial_accrual,
            "Expected last_accrual {} to be greater than initial {}",
            pool.last_accrual,
            initial_accrual
        );
        assert_eq!(pool.last_accrual_timestamp, time_passed);

        // Verify the calculations match our expectations
        assert_eq!(pool.last_accrual, expected_new_accrual);
        assert_eq!(pool.total_borrowed, expected_new_total_borrowed);

        // Verify that the interest accrued is reasonable (should be a small percentage)
        let interest_accrued = pool.total_borrowed - initial_total_borrowed;
        assert!(interest_accrued > 0, "Interest accrued should be positive");
        assert!(
            interest_accrued < initial_total_borrowed / 10,
            "Interest accrued shouldn't be more than 10% for one day"
        );
    }

    #[test]
    fn test_get_borrow_rate_per_second_zero_total() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 0;
        pool.available = 0;

        let rate = pool.get_borrow_rate_per_second().unwrap();
        assert_eq!(rate, pool.config.base_rate_per_second);
    }

    #[test]
    fn test_get_borrow_rate_per_second_below_optimal() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 300_000; // 30% utilization
        pool.available = 700_000;

        let rate = pool.get_borrow_rate_per_second().unwrap();

        // Should be base rate + slope1 * utilization
        let expected = pool.config.base_rate_per_second + (pool.config.slope1 * 3000); // 30% in bps
        assert_eq!(rate, expected);
    }

    #[test]
    fn test_get_borrow_rate_per_second_above_optimal() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 900_000; // 90% utilization (above 80% optimal)
        pool.available = 100_000;

        let rate = pool.get_borrow_rate_per_second().unwrap();

        // Should use both slopes
        let pre_threshold = pool.config.base_rate_per_second
            + (pool.config.slope1 * pool.config.optimal_utilization_ratio_bps);
        let excess_rate = (9000 - pool.config.optimal_utilization_ratio_bps) * pool.config.slope2;
        let expected = pre_threshold + excess_rate;

        assert_eq!(rate, expected);
    }

    #[test]
    fn test_get_compound_rate_multipliers() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 500_000;
        pool.available = 500_000;

        let multipliers = pool.get_compound_rate_multipliers(24 * 60 * 60).unwrap();

        assert!(multipliers.borrow > SCALED_ONE);
        assert!(multipliers.supply > SCALED_ONE);
        assert!(multipliers.borrow > multipliers.supply);
    }

    #[test]
    fn test_calculate_supply_multiplier_zero_total() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 0;
        pool.available = 0;

        let supply_multiplier = pool
            .calculate_supply_multiplier(SCALED_ONE + 100000)
            .unwrap();
        assert_eq!(supply_multiplier, SCALED_ONE);
    }

    #[test]
    fn test_calculate_supply_multiplier_with_utilization() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 800000;
        pool.available = 200000;

        let borrow_multiplier = (SCALED_ONE * 105) / 100; // 5% increase
        let supply_multiplier = pool.calculate_supply_multiplier(borrow_multiplier).unwrap();

        assert!(supply_multiplier > SCALED_ONE);
        assert!(supply_multiplier < borrow_multiplier); // Supply rate should be lower
    }

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

    #[test]
    fn test_update_accruals() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 100000;
        pool.last_accrual = ACCRUAL_INIT;

        let borrow_multiplier = (101 * SCALED_ONE) / 100; // 1% increase
        let timestamp = 3600;

        let initial_total_borrowed = pool.total_borrowed;
        let initial_accrual = pool.last_accrual;

        let result = pool.update_accruals(borrow_multiplier, timestamp);
        assert!(result.is_ok());

        assert!(pool.total_borrowed > initial_total_borrowed);
        assert!(pool.last_accrual > initial_accrual);
        assert_eq!(pool.last_accrual_timestamp, timestamp);
    }

    #[test]
    fn test_calculate_utilization_ratio_bps() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 300000;

        let total = 1000000;
        let ratio = pool.calculate_utilization_ratio_bps(total).unwrap();

        assert_eq!(ratio, 3000); // 30% in basis points
    }

    #[test]
    fn test_calculate_pre_threshold_rate() {
        let env = Env::default();
        let pool = create_test_pool(&env);

        let utilization_bps = 5000; // 50%
        let rate = pool.calculate_pre_threshold_rate(utilization_bps).unwrap();

        let expected = pool.config.base_rate_per_second + (pool.config.slope1 * utilization_bps);
        assert_eq!(rate, expected);
    }

    #[test]
    fn test_calculate_post_threshold_rate() {
        let env = Env::default();
        let pool = create_test_pool(&env);

        let utilization_bps = 9000; // 90%
        let rate = pool.calculate_post_threshold_rate(utilization_bps).unwrap();

        let pre_threshold = pool.config.base_rate_per_second
            + (pool.config.slope1 * pool.config.optimal_utilization_ratio_bps);
        let excess_rate =
            (utilization_bps - pool.config.optimal_utilization_ratio_bps) * pool.config.slope2;
        let expected = pre_threshold + excess_rate;

        assert_eq!(rate, expected);
    }

    #[test]
    fn test_edge_case_max_utilization() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 1000000;
        pool.available = 0; // 100% utilization

        let rate = pool.get_borrow_rate_per_second().unwrap();
        assert!(rate > pool.config.base_rate_per_second);
    }

    #[test]
    fn test_compound_rate_multipliers_precision() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 1;
        pool.available = 999999;

        let multipliers = pool.get_compound_rate_multipliers(1).unwrap();

        // With very low utilization and short time, multipliers should be very close to SCALED_ONE
        assert!(multipliers.borrow > SCALED_ONE);
        assert!(multipliers.supply >= SCALED_ONE);
    }

    #[test]
    fn test_interest_rate_consistency() {
        let env = Env::default();
        let mut pool = create_test_pool(&env);
        pool.total_borrowed = 500000;
        pool.available = 500000;

        // Test that longer periods result in higher multipliers
        let short_period = pool.get_compound_rate_multipliers(60 * 60).unwrap(); // 1 hour
        let long_period = pool.get_compound_rate_multipliers(24 * 60 * 60).unwrap(); // 1 day

        assert!(long_period.borrow > short_period.borrow);
        assert!(long_period.supply > short_period.supply);
    }
}
