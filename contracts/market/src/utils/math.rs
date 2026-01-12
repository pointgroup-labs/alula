use soroban_fixed_point_math::FixedPoint;

use crate::error::MCError;

pub trait MathUtils<T> {
    fn map_over_or_underflow(self) -> Result<T, MCError>;
}

impl<T> MathUtils<T> for Option<T> {
    fn map_over_or_underflow(self) -> Result<T, MCError> {
        self.ok_or(MCError::OverOrUnderflow)
    }
}

/// `O(log(n))` algorithm for quick exponentiation of fixed-point decimal number representations
/// with multiplication flooring
///
/// # Arguments
/// * `base` - The base value in fixed-point representation (scaled by denominator)
/// * `exp` - The exponent (power) to raise the base to
/// * `denominator` - The scaling factor for fixed-point arithmetic
///
/// # Returns
/// * `Result<i128, MCError>` - The result of base^exp in fixed-point representation, or an error if
///   overflow occurs
pub fn bin_pow(mut base: i128, mut exp: u64, denominator: i128) -> Result<i128, MCError> {
    if exp == 0 {
        return Ok(denominator);
    }
    if exp == 1 {
        return fixed_mul(denominator, base, denominator);
    }

    let mut result = denominator;
    while exp != 0 {
        if exp % 2 == 1 {
            result = fixed_mul(result, base, denominator)?;
        }

        base = fixed_mul(base, base, denominator)?;
        exp >>= 1;
    }

    Ok(result)
}

/// `O(log(n))` algorithm for quick exponentiation of fixed-point decimal number representations
/// with multiplication ceiling
///
/// # Arguments
/// * `base` - The base value in fixed-point representation (scaled by denominator)
/// * `exp` - The exponent (power) to raise the base to
/// * `denominator` - The scaling factor for fixed-point arithmetic
///
/// # Returns
/// * `Result<i128, MCError>` - The result of base^exp in fixed-point representation, or an error if
///   overflow occurs
pub fn bin_pow_ceil(mut base: i128, mut exp: u64, denominator: i128) -> Result<i128, MCError> {
    if exp == 0 {
        return Ok(denominator);
    }
    if exp == 1 {
        return fixed_mul_ceil(denominator, base, denominator);
    }

    let mut result = denominator;
    while exp != 0 {
        if exp % 2 == 1 {
            result = fixed_mul_ceil(result, base, denominator)?;
        }

        base = fixed_mul_ceil(base, base, denominator)?;
        exp >>= 1;
    }

    Ok(result)
}

/// Helper function for fixed-point floor multiplication
#[inline]
fn fixed_mul(x: i128, y: i128, denominator: i128) -> Result<i128, MCError> {
    x.fixed_mul_floor(y, denominator).map_over_or_underflow()
}

/// Helper function for fixed-point ceiling multiplication
#[inline]
pub fn fixed_mul_ceil(x: i128, y: i128, denominator: i128) -> Result<i128, MCError> {
    x.fixed_mul_ceil(y, denominator).map_over_or_underflow()
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec::Vec;

    use soroban_sdk::testutils::arbitrary::std::println;

    use super::*;
    use crate::error::MCError;

    #[test]
    fn test_fixed_mul_ceil_vs_floor() {
        let test_cases =
            [(1, 1, 2), (1, 3, 2), (1_000_000, 1, 1_000_001), (1_000_000, 1_000_000, 1_000_001)];

        for (x, y, denominator) in test_cases {
            let floor_result = fixed_mul(x, y, denominator).unwrap();
            let ceil_result = fixed_mul_ceil(x, y, denominator).unwrap();

            if (x * y) % denominator != 0 {
                assert!(
                    ceil_result > floor_result,
                    "For non-exact division, ceiling should be > floor: {} vs {}",
                    ceil_result,
                    floor_result
                );
            } else {
                assert_eq!(ceil_result, floor_result);
            }
        }
    }

    #[test]
    fn test_bin_pow_ceil_vs_floor() {
        let test_cases = [(1, 2, 2), (3, 2, 2), (999_999, 2, 1_000_000), (1_500_000, 2, 1_000_000)];

        for (base, exponent, denominator) in test_cases {
            let floor_result = bin_pow(base, exponent, denominator).unwrap();
            let ceil_result = bin_pow_ceil(base, exponent, denominator).unwrap();

            assert!(
                ceil_result >= floor_result,
                "Ceiling result should be >= floor result: {} vs {}",
                ceil_result,
                floor_result
            );

            println!(
                "base={}, exp={}, denom={}: floor={}, ceil={}",
                base, exponent, denominator, floor_result, ceil_result
            );
        }
    }

    #[test]
    fn test_bin_pow_zero_exponent() {
        let denominators = [1, 10, 1_000, 1_000_000, 1_000_000_000];

        for denominator in denominators {
            let bases = [0, 1, denominator, denominator * 2, -denominator];
            for base in bases {
                let result = bin_pow(base, 0, denominator).unwrap();
                assert_eq!(result, denominator, "Base {}, Denominator {}", base, denominator);
            }
        }
    }

    #[test]
    fn test_bin_pow_one_exponent() {
        let denominators = [1_i128, 10, 1_000, 1_000_000];

        for denominator in denominators {
            let bases = [0, 1, denominator, denominator * 2, -denominator];
            for base in bases {
                let result = bin_pow(base, 1, denominator).unwrap();
                assert_eq!(
                    result,
                    fixed_mul(denominator, base, denominator).unwrap(),
                    "Base {}, Denominator {}",
                    base,
                    denominator
                );
            }
        }
    }

    #[test]
    fn test_bin_pow_integer_powers() {
        let test_cases = [
            (2_000_000, 2, 1_000_000, 4_000_000),
            (2_000_000, 3, 1_000_000, 8_000_000),
            (2_000_000, 4, 1_000_000, 16_000_000),
            (3_000_000, 2, 1_000_000, 9_000_000),
            (10_000_000, 2, 1_000_000, 100_000_000),
        ];

        for (base, exponent, denominator, expected) in test_cases {
            let result = bin_pow(base, exponent, denominator).unwrap();
            assert_eq!(
                result, expected,
                "Base {}, Exponent {}, Denominator {}",
                base, exponent, denominator
            );
        }
    }

    #[test]
    fn test_bin_pow_fractional_base() {
        let test_cases = [
            (500_000, 2, 1_000_000, 250_000),
            (500_000, 3, 1_000_000, 125_000),
            (1_500_000, 2, 1_000_000, 2_250_000),
        ];

        for (base, exponent, denominator, expected) in test_cases {
            let result = bin_pow(base, exponent, denominator).unwrap();
            assert_eq!(
                result, expected,
                "Base {}, Exponent {}, Denominator {}",
                base, exponent, denominator
            );
        }
    }

    #[test]
    fn test_bin_pow_negative_base() {
        let test_cases = [
            (-2_000_000, 2, 1_000_000, 4_000_000),
            (-2_000_000, 3, 1_000_000, -8_000_000),
            (-1_500_000, 2, 1_000_000, 2_250_000),
        ];

        for (base, exponent, denominator, expected) in test_cases {
            let result = bin_pow(base, exponent, denominator).unwrap();
            assert_eq!(
                result, expected,
                "Base {}, Exponent {}, Denominator {}",
                base, exponent, denominator
            );
        }
    }

    #[test]
    fn test_bin_pow_identity_base() {
        let denominators = [1_i128, 10, 1_000, 1_000_000];

        for &denominator in &denominators {
            for exponent in [0, 1, 2, 10, 50, 100] {
                let result = bin_pow(denominator, exponent, denominator).unwrap();
                assert_eq!(
                    result, denominator,
                    "Denominator {}, Exponent {}",
                    denominator, exponent
                );
            }
        }
    }

    #[test]
    fn test_bin_pow_zero_base() {
        let denominators = [1_i128, 10, 1_000, 1_000_000];

        for &denominator in &denominators {
            let result = bin_pow(0, 0, denominator).unwrap();
            assert_eq!(result, denominator);

            for exponent in [1, 2, 10, 100] {
                let result = bin_pow(0, exponent, denominator).unwrap();
                assert_eq!(result, 0);
            }
        }
    }

    #[test]
    fn test_bin_pow_precision_at_boundaries() {
        let denominator = 1_000_000;

        let base = 999_999;
        let result = bin_pow(base, 100, denominator).unwrap();
        assert!(result > 990_000 && result < 999_990);

        let base = 1_000_001;
        let result = bin_pow(base, 100, denominator).unwrap();
        assert!(result > 1_000_010 && result < 1_010_000);

        let base = 1_000_000 + 1;
        let result = bin_pow(base, 1_000_000, denominator).unwrap();
        assert!(result > 2_715_000 && result < 2_717_000);
    }

    #[test]
    fn test_euler_big_exponents() {
        let base: i128 = 1_000_000_001;
        let denominator = 1_000_000_000;
        let exponent = 1_000_000_000;

        let res = bin_pow(base, exponent, denominator).unwrap();
        assert!(res > 2_718_220_000 && res < 2_718_230_000);
    }

    #[test]
    fn test_bin_pow_error_cases() {
        let base = i128::MAX / 2;
        let denominator = 1_000_000;
        let result = bin_pow(base, 2, denominator);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MCError::OverOrUnderflow);

        let result = bin_pow(1_000_000, 2, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_bin_pow_alternative_algorithm_comparison() {
        let denominators = [1_000, 1_000_000];
        let bases = [500, 1_000, 2_000, 5_000];

        for &denominator in &denominators {
            for &base in &bases {
                for exponent in 1..=5 {
                    let mut direct_result = denominator;
                    for _ in 0..exponent {
                        direct_result = fixed_mul(direct_result, base, denominator).unwrap();
                    }

                    let bin_result = bin_pow(base, exponent, denominator).unwrap();
                    assert_eq!(direct_result, bin_result);
                }
            }
        }
    }

    #[test]
    fn test_bin_pow_denominator_scaling() {
        let data = [(1.5, 3), (0.8, 5), (1.1, 10)];

        for (base_value, exponent) in data {
            let mut results = Vec::new();

            for &denom_scale in &[1_000, 1_000_000, 1_000_000_000] {
                let base = (base_value * denom_scale as f64) as i128;
                let result = bin_pow(base, exponent, denom_scale).unwrap();
                results.push(result as f64 / denom_scale as f64);
            }

            let expected_value = base_value.powi(exponent as i32);
            let highest_precision_result = results.last().unwrap();
            let tolerance = 0.001;
            assert!((highest_precision_result - expected_value).abs() < tolerance);
        }
    }
}
