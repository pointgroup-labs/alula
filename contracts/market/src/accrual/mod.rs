use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::contracttype;

use crate::{
    constants::*,
    error::MCError,
    helpers::require_nonnegative,
    math_utils::{self, MathUtils},
};

pub trait Accrual {
    /// Computes the growth multiplier based on the APR and the time passed.
    /// The multiplier is a fixed-point number with `SCALED_FIXED_POINT_DENOMINATOR` precision.
    ///
    /// # Arguments
    /// * `apr_bps` - The annual percentage rate in basis points (bps).
    /// * `seconds_passed` - The number of seconds that have passed.
    ///
    /// # Returns
    /// * `Ok(i128)` - The growth multiplier as a fixed-point number.
    /// * `Err(MCError)` - If the APR is negative or if there is an overflow/underflow during
    ///   calculations.
    fn compute_multiplier(&self, apr_bps: i128, seconds_passed: u64) -> Result<i128, MCError>;
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AccrualModel {
    #[default]
    Compounded,
}

impl Accrual for AccrualModel {
    fn compute_multiplier(&self, apr_bps: i128, seconds_passed: u64) -> Result<i128, MCError> {
        require_nonnegative(apr_bps)?;

        match self {
            Self::Compounded => {
                let scaled_apr = apr_bps
                    .fixed_mul_ceil(SCALED_FIXED_POINT_DENOMINATOR, BPS_FACTOR)
                    .map_over_or_underflow()?;

                let per_second_rate =
                    scaled_apr.checked_div(SECONDS_IN_YEAR as i128).map_over_or_underflow()?;

                let growth_factor = SCALED_FIXED_POINT_DENOMINATOR
                    .checked_add(per_second_rate)
                    .map_over_or_underflow()?;

                math_utils::bin_pow(growth_factor, seconds_passed, SCALED_FIXED_POINT_DENOMINATOR)
            }
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::inconsistent_digit_grouping)]
    use soroban_fixed_point_math::FixedPoint;

    use super::{Accrual, AccrualModel};
    use crate::{constants::*, error::MCError};

    #[test]
    fn test_zero_seconds_passed() {
        let model = AccrualModel::Compounded;
        let apr = 1000; // 10%
        let seconds_passed = 0;
        let expected_multiplier = SCALED_FIXED_POINT_DENOMINATOR;

        assert_eq!(model.compute_multiplier(apr, seconds_passed).unwrap(), expected_multiplier);
    }

    #[test]
    fn test_zero_apr() {
        let model = AccrualModel::Compounded;
        let apr = 0;
        let seconds_passed = SECONDS_IN_YEAR;

        let expected_multiplier = SCALED_FIXED_POINT_DENOMINATOR; // =1 (0%)

        assert_eq!(model.compute_multiplier(apr, seconds_passed).unwrap(), expected_multiplier);
    }

    #[test]
    fn test_one_year_passed() {
        let model = AccrualModel::Compounded;
        let apr = 1000; // 10%
        let seconds_passed = SECONDS_IN_YEAR;

        let expected_multiplier: i128 = 1_105_170_917_873_740_281; // ~1.105 (10.5%)

        assert_eq!(model.compute_multiplier(apr, seconds_passed).unwrap(), expected_multiplier);
    }

    #[test]
    fn test_high_apr_one_year() {
        let model = AccrualModel::Compounded;
        let apr = 9000; // 90%
        let seconds_passed = SECONDS_IN_YEAR;

        let expected_multiplier = 2_459_603_079_490_413_216; // ~2.4 (240%)

        assert_eq!(model.compute_multiplier(apr, seconds_passed).unwrap(), expected_multiplier);
    }

    #[test]
    fn test_half_year_passed() {
        let model = AccrualModel::Compounded;
        let apr = 1000; // 10%
        let seconds_passed = SECONDS_IN_YEAR / 2;

        let expected_multiplier = 1_051_271_096_279_993_934; // ~1.051 (5.1%)

        assert_eq!(model.compute_multiplier(apr, seconds_passed).unwrap(), expected_multiplier);
    }

    #[test]
    fn test_one_day_passed() {
        let model = AccrualModel::Compounded;
        let apr = 1000; // 10%
        let seconds_passed = SECONDS_PER_DAY;

        let expected_multiplier = 1_000_273_828_409_933_351; // ~1.00027 (0.027%)

        assert_eq!(model.compute_multiplier(apr, seconds_passed).unwrap(), expected_multiplier);
    }

    #[test]
    fn test_high_apr_over_one_year() {
        let model = AccrualModel::Compounded;
        let apr = 2000; // 20%
        let seconds_passed = SECONDS_IN_YEAR;

        let expected_multiplier = 1_221_402_757_366_989_775; // ~1.22 (22%)

        assert_eq!(model.compute_multiplier(apr, seconds_passed).unwrap(), expected_multiplier);
    }

    #[test]
    fn test_low_apr_over_two_years() {
        let model = AccrualModel::Compounded;
        let apr = 100; // 1%
        let seconds_passed = SECONDS_IN_YEAR * 2;

        let expected_multiplier = 1_020_201_339_994_595_008; // ~1.02 (2%)

        assert_eq!(model.compute_multiplier(apr, seconds_passed).unwrap(), expected_multiplier);
    }

    #[test]
    fn test_one_second_passed() {
        let model = AccrualModel::Compounded;
        let apr = 1000; // 10%
        let seconds_passed = 1;

        let expected_multiplier = 1_000_000_003_168_876_461; // ~1.000000003 (0.0000003%)

        assert_eq!(model.compute_multiplier(apr, seconds_passed).unwrap(), expected_multiplier);
    }

    #[test]
    fn test_high_apr_two_years() {
        let model = AccrualModel::Compounded;
        let apr = 15_000; // 150%
        let seconds_passed = 2 * SECONDS_IN_YEAR;

        let expected_multiplier = 20_085_535_490_337_347_880; // ~20.085 (1900.85%)

        assert_eq!(model.compute_multiplier(apr, seconds_passed).unwrap(), expected_multiplier);
    }

    #[test]
    fn test_moderate_apr_ten_years() {
        let model = AccrualModel::Compounded;
        let apr = 2_000; // 20%
        let seconds_passed = 10 * SECONDS_IN_YEAR;

        let expected_multiplier = 7_389_056_050_946_052_968; // ~7.389 (638.9%)

        assert_eq!(model.compute_multiplier(apr, seconds_passed).unwrap(), expected_multiplier);
    }

    #[test]
    fn test_3_years_of_high_apr_breaks_i128() {
        let model = AccrualModel::Compounded;
        let apr = 15_000; // 150%
        let seconds_passed = 3 * SECONDS_IN_YEAR;

        // NB: Switching to [`I256`] extends the computational constraints
        assert_eq!(model.compute_multiplier(apr, seconds_passed), Err(MCError::OverOrUnderflow));
    }

    #[test]
    fn test_extreme_multiplicator_extreme_old_value_works_with_i128() {
        let old_value: i128 = 10_000_000_000_0000000;
        let big_multiplier = 200 * SCALED_FIXED_POINT_DENOMINATOR;

        let new_value =
            old_value.fixed_mul_floor(big_multiplier, SCALED_FIXED_POINT_DENOMINATOR).unwrap();

        assert_eq!(new_value, 2_000_000_000_000_0000000);
    }

    #[test]
    fn test_extreme_multiplicator_very_extreme_old_value_breaks_i128() {
        // NB: This test shows the boundaries of using [`i128`] for intermediate calculations.
        // The boundaries can be extended when switching [`i128`] => [`I256`](See: https://docs.rs/soroban-sdk/latest/soroban_sdk/struct.I256.html).
        // WARN: Consider increased gas cost if switching to [`I256`] as a future-proofing measure
        // and not as a necessity
        let old_value: i128 = 100_000_000_000_0000000;
        let big_multiplier = 200 * SCALED_FIXED_POINT_DENOMINATOR;

        let new_value =
            old_value.fixed_mul_floor(big_multiplier, SCALED_FIXED_POINT_DENOMINATOR * 10);

        assert_eq!(new_value, None);
    }

    #[test]
    fn test_negative_apr_rejected() {
        let model = AccrualModel::Compounded;
        let apr = -1000; // -10%
        let seconds_passed = SECONDS_IN_YEAR;

        assert_eq!(
            model.compute_multiplier(apr, seconds_passed),
            Err(MCError::NegativeInputAmount)
        );
    }
}
