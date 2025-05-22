use {crate::constants::LCError, soroban_fixed_point_math::FixedPoint};

// TODO: Check, what happens with precision compared to O(n) algorithm.
// The issue is that `fixed_mul_floor` divides by the denominator which leads to a precision loss
/// O(log(n)) algorithm for quick exponentiation
pub fn bin_pow(base: i128, mut exponent: u64, denominator: i128) -> Result<i128, LCError> {
    let mut result = denominator;
    let mut temp_base = base;

    while exponent > 0 {
        if exponent % 2 == 1 {
            result = result
                .fixed_mul_floor(temp_base, denominator)
                .ok_or(LCError::OverOrUnderflow)?; // TODO: `floor` or `ceil`?
        }

        temp_base = temp_base
            .fixed_mul_floor(temp_base, denominator)
            .ok_or(LCError::OverOrUnderflow)?;
        exponent /= 2;
    }

    Ok(result)
}
