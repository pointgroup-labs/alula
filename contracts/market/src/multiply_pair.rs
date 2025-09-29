use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, Bytes, BytesN, Env, Vec, contracttype, xdr::ToXdr};

use crate::{
    constants::{
        BPS_FACTOR, DEFAULT_FLASH_LOAN_FEE_BPS, DEFAULT_MAX_SWAP_FEE_BPS, LEVERAGE_SCALE,
        MIN_LEVERAGE_MULTIPLIER,
    },
    error::MCError,
    math_utils::MathUtils,
    storage,
};

/// Used to generate a unique seed for a multiply pair obligation
/// See [`MultiplyPair::compute_obligation_seed`]
const MULTIPLY_PAIR_PREFIX: &str = "MP_";

#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MultiplyPair {
    /// Address of a pool in a pair for a leveraged deposit
    pub deposit_pool: Address,
    /// Address of a pool in a pair for a leveraged borrow
    pub borrow_pool: Address,
    /// Maximum leverage multiplier based on borrow pool openLTV value. Scaled with
    /// [`LEVERAGE_SCALE`]
    pub max_leverage_multiplier: u32,
    /// Deterministically computed unique seed per a pair, used to distinguish a user's multiply
    /// pair obligation from other
    pub seed: BytesN<32>,
}

impl MultiplyPair {
    pub fn new(
        e: &Env,
        deposit_pool_address: &Address,
        borrow_pool_address: &Address,
        borrow_pool_open_ltv_bps: i128,
        flash_loan_fee_bps: i128,
        collateral_pool_liability_factor_bps: i128,
    ) -> Self {
        let max_leverage_multiplier = Self::compute_max_leverage_multiplier(
            flash_loan_fee_bps,
            DEFAULT_MAX_SWAP_FEE_BPS,
            borrow_pool_open_ltv_bps,
            collateral_pool_liability_factor_bps,
        );
        let seed = Self::compute_obligation_seed(e, deposit_pool_address, borrow_pool_address);

        Self {
            deposit_pool: deposit_pool_address.clone(),
            borrow_pool: borrow_pool_address.clone(),
            max_leverage_multiplier,
            seed,
        }
    }

    /// Returns a tuple that can be used as a unique key
    pub fn key(&self) -> (Address, Address) {
        (self.deposit_pool.clone(), self.borrow_pool.clone())
    }

    /// Tries to get the multiply pair from the contract's storage
    ///
    /// # Returns
    /// - [`Ok(MultiplyPair)`] if a multiply pair for the given deposit and pools addresses exists
    ///   in the contract's storage
    /// - [`Err(MCError::MultiplyPairDoesNotExist)`] otherwise
    pub fn try_get(
        e: &Env,
        deposit_pool_address: &Address,
        borrow_pool_address: &Address,
    ) -> Result<Self, MCError> {
        storage::get_multiply_pair(e, deposit_pool_address, borrow_pool_address)
            .ok_or(MCError::MultiplyPairDoesNotExist)
    }

    /// Registers a multiply pair in the pairs list
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn register(&self, e: &Env) -> u32 {
        storage::register_multiply_pair(e, self.clone())
    }

    pub fn require_does_not_exists(
        e: &Env,
        deposit_pool_address: &Address,
        borrow_pool_address: &Address,
    ) -> Result<(), MCError> {
        if Self::exists(e, deposit_pool_address, borrow_pool_address) {
            return Err(MCError::MultiplyPairAlreadyExists);
        }

        Ok(())
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

    // TODO: Likely 'deposit as margin' will contain a different value, compared to
    // `borrow as margin` case. Use min?
    /// Computes the maximum leverage multiplier (for 'borrow as margin' case):
    /// `max_multiplier = (1 + flash_loan_fee) / ((1 + flash_loan_fee) - (1 - max_swap_fee_bps) *
    /// openLTV)`. Since flash loan fee and swap fee are a part of the final 'borrow' position,
    /// accounting them makes maximum multiplier smaller, so this is a must have
    fn compute_max_leverage_multiplier(
        flash_loan_fee_bps: i128,
        max_swap_fee_bps: i128,
        borrow_pool_open_ltv_bps: i128,
        _collateral_pool_liability_factor_bps: i128, // TODO: start accounting in calculations
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
            .fixed_div_floor(denominator, BPS_FACTOR)
            .map_over_or_underflow()
            .unwrap(); // safe

        // Scale the result for the final output
        (max_multiplier_bps / SCALE) as u32 // safe
    }

    /// # Returns
    /// - [`Ok(())`] if the provided multiplier is within the valid range
    /// - [`Err(MCError::InvalidLeverageMultiplier)`] otherwise
    pub fn require_valid_leverage_multiplier(
        &self,
        leverage_multiplier: u32,
    ) -> Result<(), MCError> {
        if !(MIN_LEVERAGE_MULTIPLIER..=self.max_leverage_multiplier).contains(&leverage_multiplier)
        {
            return Err(MCError::InvalidLeverageMultiplier);
        }

        Ok(())
    }

    /// # Returns
    /// [`BytesN<32>`] bytes used as an obligation seed to distinguish unique users' obligations
    fn compute_obligation_seed(
        e: &Env,
        deposit_pool_address: &Address,
        borrow_pool_address: &Address,
    ) -> BytesN<32> {
        let mut seed = Bytes::new(e);
        seed.extend_from_slice(MULTIPLY_PAIR_PREFIX.as_bytes());
        seed.extend_from_slice(deposit_pool_address.to_xdr(e).to_buffer::<40>().as_slice());
        seed.extend_from_slice(borrow_pool_address.to_xdr(e).to_buffer::<40>().as_slice());
        e.crypto().keccak256(&seed).into()
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::{Address, BytesN, Env, testutils::Address as _};

    use super::*;

    #[test]
    fn test_computes_obligation_seed_with_valid_addresses() {
        let e = Env::default();

        let deposit_pool = Address::generate(&e);
        let borrow_pool = Address::generate(&e);
        let seed = MultiplyPair::compute_obligation_seed(&e, &deposit_pool, &borrow_pool);

        assert_ne!(seed, BytesN::from_array(&e, &[0; 32]));
    }

    #[test]
    fn test_computes_different_seeds_for_different_addresses() {
        let e = Env::default();

        let deposit_pool = Address::generate(&e);
        let borrow_pool = Address::generate(&e);

        let seed1 = MultiplyPair::compute_obligation_seed(&e, &deposit_pool, &borrow_pool);
        let seed2 = MultiplyPair::compute_obligation_seed(&e, &borrow_pool, &deposit_pool);

        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_computes_deterministic_seed_for_same_inputs() {
        let e = Env::default();

        let deposit_pool = Address::generate(&e);
        let borrow_pool = Address::generate(&e);

        let seed1 = MultiplyPair::compute_obligation_seed(&e, &deposit_pool, &borrow_pool);
        let seed2 = MultiplyPair::compute_obligation_seed(&e, &deposit_pool, &borrow_pool);

        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_computes_different_seeds_when_changing_deposit_address() {
        let e = Env::default();

        let borrow_pool = Address::generate(&e);

        let deposit_pool1 = Address::generate(&e);
        let deposit_pool2 = Address::generate(&e);

        let seed1 = MultiplyPair::compute_obligation_seed(&e, &deposit_pool1, &borrow_pool);
        let seed2 = MultiplyPair::compute_obligation_seed(&e, &deposit_pool2, &borrow_pool);

        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_computes_different_seeds_when_changing_borrow_address() {
        let e = Env::default();

        let deposit_pool = Address::generate(&e);

        let borrow_pool1 = Address::generate(&e);
        let borrow_pool2 = Address::generate(&e);

        let seed1 = MultiplyPair::compute_obligation_seed(&e, &deposit_pool, &borrow_pool1);
        let seed2 = MultiplyPair::compute_obligation_seed(&e, &deposit_pool, &borrow_pool2);

        assert_ne!(seed1, seed2);
    }
}
