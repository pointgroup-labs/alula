use soroban_fixed_point_math::FixedPoint;

use crate::error::FarmsError;

/// Trait for converting Option<T> to Result<T, FarmsError>
pub trait MathUtils<T> {
    fn map_over_or_underflow(self) -> Result<T, FarmsError>;
}

impl<T> MathUtils<T> for Option<T> {
    fn map_over_or_underflow(self) -> Result<T, FarmsError> {
        self.ok_or(FarmsError::Overflow)
    }
}

/// Fixed-point floor multiplication: (x * y) / denominator, rounded down
///
/// # Arguments
/// * `x` - First operand
/// * `y` - Second operand
/// * `denominator` - The scaling factor (e.g., SCALE_FACTOR or BPS_FACTOR)
///
/// # Returns
/// * `Result<i128, FarmsError>` - The result or Overflow error
#[inline]
pub fn fixed_mul_floor(x: i128, y: i128, denominator: i128) -> Result<i128, FarmsError> {
    x.fixed_mul_floor(y, denominator).map_over_or_underflow()
}

/// Fixed-point ceiling multiplication: (x * y) / denominator, rounded up
///
/// # Arguments
/// * `x` - First operand
/// * `y` - Second operand
/// * `denominator` - The scaling factor
///
/// # Returns
/// * `Result<i128, FarmsError>` - The result or Overflow error
#[inline]
#[allow(dead_code)]
pub fn fixed_mul_ceil(x: i128, y: i128, denominator: i128) -> Result<i128, FarmsError> {
    x.fixed_mul_ceil(y, denominator).map_over_or_underflow()
}

/// Fixed-point floor division: (x * denominator) / y, rounded down
///
/// # Arguments
/// * `x` - Numerator
/// * `denominator` - The scaling factor
/// * `y` - Divisor
///
/// # Returns
/// * `Result<i128, FarmsError>` - The result or error
#[inline]
pub fn fixed_div_floor(x: i128, denominator: i128, y: i128) -> Result<i128, FarmsError> {
    if y == 0 {
        return Err(FarmsError::DivisionByZero);
    }
    x.fixed_div_floor(y, denominator).map_over_or_underflow()
}

/// Fixed-point ceiling division: (x * denominator) / y, rounded up
///
/// # Arguments
/// * `x` - Numerator
/// * `denominator` - The scaling factor
/// * `y` - Divisor
///
/// # Returns
/// * `Result<i128, FarmsError>` - The result or error
#[inline]
#[allow(dead_code)]
pub fn fixed_div_ceil(x: i128, denominator: i128, y: i128) -> Result<i128, FarmsError> {
    if y == 0 {
        return Err(FarmsError::DivisionByZero);
    }
    x.fixed_div_ceil(y, denominator).map_over_or_underflow()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{BPS_FACTOR, SCALE_FACTOR};

    #[test]
    fn test_fixed_mul_floor() {
        // 100 tokens * 50% = 50 tokens (using BPS_FACTOR)
        let result = fixed_mul_floor(100, 5_000, BPS_FACTOR).unwrap();
        assert_eq!(result, 50);

        // Test with SCALE_FACTOR
        // 1000 * reward_per_share(scaled) / SCALE_FACTOR
        let rps_scaled = 2 * SCALE_FACTOR; // 2.0 rewards per share
        let stake = 100;
        let result = fixed_mul_floor(stake, rps_scaled, SCALE_FACTOR).unwrap();
        assert_eq!(result, 200);
    }

    #[test]
    fn test_fixed_mul_ceil() {
        // Test ceiling vs floor difference
        // 1 * 1 / 2 = 0.5 -> floor=0, ceil=1
        let floor = fixed_mul_floor(1, 1, 2).unwrap();
        let ceil = fixed_mul_ceil(1, 1, 2).unwrap();
        assert_eq!(floor, 0);
        assert_eq!(ceil, 1);
    }

    #[test]
    fn test_fixed_div_floor() {
        // 100 rewards / 50 staked = 2 rewards per share
        let result = fixed_div_floor(100, SCALE_FACTOR, 50).unwrap();
        assert_eq!(result, 2 * SCALE_FACTOR);
    }

    #[test]
    fn test_fixed_div_by_zero() {
        let result = fixed_div_floor(100, SCALE_FACTOR, 0);
        assert_eq!(result.unwrap_err(), FarmsError::DivisionByZero);
    }

    #[test]
    fn test_bps_calculations() {
        // 5% fee on 1000 tokens
        let fee = fixed_mul_floor(1000, 500, BPS_FACTOR).unwrap();
        assert_eq!(fee, 50);

        // 0.01% fee on 1000 tokens (1 bps)
        let small_fee = fixed_mul_floor(1000, 1, BPS_FACTOR).unwrap();
        assert_eq!(small_fee, 0); // Rounds down to 0

        let small_fee_ceil = fixed_mul_ceil(1000, 1, BPS_FACTOR).unwrap();
        assert_eq!(small_fee_ceil, 1); // Rounds up to 1
    }
}
