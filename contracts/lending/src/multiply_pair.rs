use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::{
    constants::{BPS_FACTOR, DEFAULT_FLASH_LOAN_FEE_BPS, DEFAULT_MAX_SLIPPAGE_BPS, LEVERAGE_SCALE},
    math_utils::MathUtils,
    storage, LCError,
};

#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MultiplyPair {
    /// Address of a pool in a pair for a leveraged deposit
    pub deposit_pool: Address,
    /// Address of a pool in a pair for a leveraged borrow
    pub borrow_pool: Address,
    /// Maximum leverage multiplier based on borrow pool openLTV value. Scaled with [`LEVERAGE_SCALE`]
    pub max_leverage_multiplier: u32,
}

impl MultiplyPair {
    pub fn new(
        deposit_pool_address: &Address,
        borrow_pool_address: &Address,
        borrow_pool_open_ltv_bps: i128,
    ) -> Self {
        let max_leverage_multiplier = Self::compute_max_leverage_multiplier(
            DEFAULT_FLASH_LOAN_FEE_BPS,
            DEFAULT_MAX_SLIPPAGE_BPS,
            borrow_pool_open_ltv_bps,
        );

        Self {
            deposit_pool: deposit_pool_address.clone(),
            borrow_pool: borrow_pool_address.clone(),
            max_leverage_multiplier,
        }
    }

    /// Tries to get the multiply pair from the contract's storage
    ///
    /// # Returns
    /// - [`Ok(MultiplyPair)`] if a multiply pair for the given deposit and pools addresses exists in the contract's storage
    /// - [`Err(LCError::MultiplyPairDoesNotExist)`] otherwise
    pub fn try_get(
        e: &Env,
        deposit_pool_address: &Address,
        borrow_pool_address: &Address,
    ) -> Result<Self, LCError> {
        // storage::get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)
        storage::get_multiply_pair(e, deposit_pool_address, borrow_pool_address)
            .ok_or(LCError::MultiplyPairDoesNotExist)
    }

    /// Registers a multiply pair in the pairs list
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn register(&self, e: &Env) -> u32 {
        storage::register_multiply_pair(e, self.clone())
    }

    pub fn exists(e: &Env, deposit_pool_address: &Address, borrow_pool_address: &Address) -> bool {
        storage::multiply_pair_exists(e, deposit_pool_address, borrow_pool_address)
    }

    /// Saves\updates multiply pair in the contract's storage
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn set(&self, e: &Env) {
        storage::set_multiply_pair(e, &self.deposit_pool, &self.borrow_pool, self);
    }

    pub fn get_all(e: &Env) -> Vec<MultiplyPair> {
        storage::get_all_multiply_pairs(e)
    }

    // TODO: Likely 'deposit as margin' will contain a different value
    /// Computes the maximum leverage multiplier (for 'borrow as margin' case):
    /// `max_multiplier = (1 + flash_loan_fee) / ((1 + flash_loan_fee) - (1 - max_swap_fee_bps) * openLTV)`.
    /// Since flash loan fee and swap fee are a part of the final 'borrow' position, accounting them makes
    /// maximum multiplier smaller, so this is a must have
    fn compute_max_leverage_multiplier(
        flash_loan_fee_bps: i128,
        max_swap_fee_bps: i128,
        borrow_pool_open_ltv_bps: i128,
    ) -> u32 {
        // compile-time assertion, hence, no error is returned
        const _: () = assert!(
            (LEVERAGE_SCALE as i128) < BPS_FACTOR,
            "leverage_scale_is_too_big"
        );

        const SCALE: i128 = BPS_FACTOR / (LEVERAGE_SCALE as i128);

        // Numerator: (1 + flash_loan_fee)
        let numerator_term = BPS_FACTOR + flash_loan_fee_bps;

        // Denominator term 1: (1 - swap_fee) * openLTV
        let one_minus_swap_fee = BPS_FACTOR - max_swap_fee_bps; // safe
        let denominator_term_one = one_minus_swap_fee
            .fixed_mul_floor(borrow_pool_open_ltv_bps, BPS_FACTOR)
            .map_over_or_underflow()
            .unwrap(); // safe

        // Denominator: (1 + flash_loan_fee) - (1 - swap_fee) * openLTV
        let denominator = numerator_term - denominator_term_one; // safe

        // Full calculation: multiplier = numerator / denominator
        let max_multiplier_bps = numerator_term
            .fixed_div_ceil(denominator, BPS_FACTOR)
            .map_over_or_underflow()
            .unwrap(); // safe

        // Scale the result for the final output
        (max_multiplier_bps / SCALE) as u32 // safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_fees() {
        let open_ltv_bps = 7500;
        let flash_loan_fee_bps = 0;
        let max_swap_fee_bps = 0;

        // The formula simplifies to 1/(1 - LTV)
        let expected_multiplier_bps = BPS_FACTOR
            .fixed_div_ceil(BPS_FACTOR - open_ltv_bps, BPS_FACTOR)
            .map_over_or_underflow()
            .unwrap();
        let expected_result =
            (expected_multiplier_bps / (BPS_FACTOR / (LEVERAGE_SCALE as i128))) as u32;

        let result = MultiplyPair::compute_max_leverage_multiplier(
            flash_loan_fee_bps,
            max_swap_fee_bps,
            open_ltv_bps,
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_multiplier_is_always_smaller_than_theoretical_max_with_fees() {
        let fixed_open_ltv_bps = 8500;

        // Calculate the theoretical maximum multiplier for the fixed LTV
        let denominator = BPS_FACTOR - fixed_open_ltv_bps;
        let theoretical_max_multiplier_bps = (BPS_FACTOR
            .fixed_div_ceil(denominator, BPS_FACTOR)
            .map_over_or_underflow()
            .unwrap() as u32)
            / (BPS_FACTOR as u32 / (LEVERAGE_SCALE)); // safe

        // Iterate over a range of possible fees
        for flash_loan_fee_bps in (1..100).step_by(10) {
            for max_swap_fee_bps in (1..100).step_by(10) {
                let calculated_multiplier = MultiplyPair::compute_max_leverage_multiplier(
                    flash_loan_fee_bps,
                    max_swap_fee_bps,
                    fixed_open_ltv_bps,
                );

                assert!(
                    calculated_multiplier < theoretical_max_multiplier_bps,
                    "Multiplier should be smaller than theoretical max for fees: flash_loan_fee_bps={}, max_swap_fee_bps={}",
                    flash_loan_fee_bps,
                    max_swap_fee_bps
                );
            }
        }
    }
}
