use soroban_sdk::contracttype;

use crate::{
    LCError,
    constants::SECONDS_IN_YEAR,
    interest_rate::SCALED_ONE,
    math_utils::{self, MathUtils},
};

pub trait Accrual {
    fn calculate_multiplier(&self, apr: i128, seconds_passed: u32) -> Result<i128, LCError>;
}

#[derive(Debug, Eq, PartialEq)]
#[contracttype]
pub enum AccrualModel {
    Compounded,
}

impl Accrual for AccrualModel {
    fn calculate_multiplier(&self, apr: i128, seconds_passed: u32) -> Result<i128, LCError> {
        match self {
            AccrualModel::Compounded => {
                const NEW_SCALED_ONE: i128 = 10 * SCALED_ONE;

                let scaled_apr = apr.checked_mul(NEW_SCALED_ONE).map_over_or_underflow()?;
                let per_second_rate = scaled_apr / SECONDS_IN_YEAR as i128;

                let growth_factor = NEW_SCALED_ONE
                    .checked_add(per_second_rate)
                    .map_over_or_underflow()?;

                let seconds_passed = seconds_passed as u64;

                math_utils::bin_pow(growth_factor, seconds_passed, NEW_SCALED_ONE)
            }
        }
    }
}
