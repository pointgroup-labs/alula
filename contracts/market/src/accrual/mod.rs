use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::contracttype;

use crate::{
    constants::{BPS_FACTOR, SECONDS_IN_YEAR},
    error::MCError,
    interest_rate::SCALED_ONE,
    math_utils::{self, MathUtils},
};

pub trait Accrual {
    fn calculate_multiplier(&self, apr: i128, seconds_passed: u64) -> Result<i128, MCError>;
}

#[derive(Debug, Eq, PartialEq)]
#[contracttype]
pub enum AccrualModel {
    Compounded,
}

impl Accrual for AccrualModel {
    fn calculate_multiplier(&self, apr_bps: i128, seconds_passed: u64) -> Result<i128, MCError> {
        match self {
            AccrualModel::Compounded => {
                let scaled_apr = apr_bps
                    .fixed_mul_ceil(SCALED_ONE, BPS_FACTOR)
                    .map_over_or_underflow()?;

                let per_second_rate = scaled_apr / SECONDS_IN_YEAR as i128;
                let growth_factor = SCALED_ONE
                    .checked_add(per_second_rate)
                    .map_over_or_underflow()?;
                let seconds_passed = seconds_passed as u64;

                math_utils::bin_pow(growth_factor, seconds_passed, SCALED_ONE)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Accrual, AccrualModel, SCALED_ONE, SECONDS_IN_YEAR};
    use crate::constants::SECONDS_PER_DAY;

    #[test]
    fn test_zero_seconds_passed() {
        let model = AccrualModel::Compounded;
        let apr = 1000; // 10%
        let seconds_passed = 0;
        let expected_multiplier = SCALED_ONE;

        assert_eq!(
            model.calculate_multiplier(apr, seconds_passed).unwrap(),
            expected_multiplier
        );
    }

    #[test]
    fn test_zero_apr() {
        let model = AccrualModel::Compounded;
        let apr = 0;
        let seconds_passed = SECONDS_IN_YEAR;

        let expected_multiplier = SCALED_ONE; // 1(0%)

        assert_eq!(
            model.calculate_multiplier(apr, seconds_passed).unwrap(),
            expected_multiplier
        );
    }

    #[test]
    fn test_one_year_passed() {
        let model = AccrualModel::Compounded;
        let apr = 1000; // 10%
        let seconds_passed = SECONDS_IN_YEAR;

        let expected_multiplier: i128 = 110517068512967; // ~1.11 (11%)

        assert_eq!(
            model.calculate_multiplier(apr, seconds_passed).unwrap(),
            expected_multiplier
        );
    }

    #[test]
    fn test_high_apr_one_year() {
        let model = AccrualModel::Compounded;
        let apr = 9000; // 90%
        let seconds_passed = SECONDS_IN_YEAR;

        let expected_multiplier = 245960228433805; // ~2.4 (240%)

        assert_eq!(
            model.calculate_multiplier(apr, seconds_passed).unwrap(),
            expected_multiplier
        );
    }

    #[test]
    fn test_half_year_passed() {
        let model = AccrualModel::Compounded;
        let apr = 1000; // 10%
        let seconds_passed = SECONDS_IN_YEAR / 2;

        let expected_multiplier = 105127098558347; // ~1.051 (5.1%)

        assert_eq!(
            model.calculate_multiplier(apr, seconds_passed).unwrap(),
            expected_multiplier
        );
    }

    #[test]
    fn test_one_day_passed() {
        let model = AccrualModel::Compounded;
        let apr = 1000; // 10%
        let seconds_passed = SECONDS_PER_DAY;

        let expected_multiplier = 100027382783319; // ~1.00027 (0.027%)

        assert_eq!(
            model.calculate_multiplier(apr, seconds_passed).unwrap(),
            expected_multiplier
        );
    }

    #[test]
    fn test_high_apr_over_one_year() {
        let model = AccrualModel::Compounded;
        let apr = 2000; // 20%
        let seconds_passed = SECONDS_IN_YEAR;

        let expected_multiplier = 122140262868925; // ~1.22 (22%)

        assert_eq!(
            model.calculate_multiplier(apr, seconds_passed).unwrap(),
            expected_multiplier
        );
    }

    #[test]
    fn test_low_apr_over_two_years() {
        let model = AccrualModel::Compounded;
        let apr = 100; // 1%
        let seconds_passed = SECONDS_IN_YEAR * 2;

        let expected_multiplier = 102020084536821; // ~1.02 (2%)

        assert_eq!(
            model.calculate_multiplier(apr, seconds_passed).unwrap(),
            expected_multiplier
        );
    }

    #[test]
    fn test_one_second_passed() {
        let model = AccrualModel::Compounded;
        let apr = 1000; // 10%
        let seconds_passed = 1;

        let expected_multiplier = 100000000316887; // ~1.000000003 (0.0000003%)

        assert_eq!(
            model.calculate_multiplier(apr, seconds_passed).unwrap(),
            expected_multiplier
        );
    }

    #[test]
    fn test_high_apr_ten_years() {
        let model = AccrualModel::Compounded;
        let apr = 9000; // 90%
        let seconds_passed = 10 * SECONDS_IN_YEAR;

        let expected_multiplier = 810305668833306508; // ~8103 (810200%)

        assert_eq!(
            model.calculate_multiplier(apr, seconds_passed).unwrap(),
            expected_multiplier
        );
    }
}
