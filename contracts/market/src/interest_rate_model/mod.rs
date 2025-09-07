use enum_dispatch::enum_dispatch;
use soroban_sdk::contracttype;

use crate::{error::MCError, interest_rate_model::kinked::KinkedIRConfig};

#[enum_dispatch]
pub trait InterestRate {
    // TODO: Add a better doc comment
    /// Computes annual percentage rates scaled with [`SCALED_ONE`] value, where [`SCALED_ONE`]
    /// equals to 1%
    ///
    /// ### Arguments
    /// * `utilization_ratio_bps` - utilization ratio of a pool
    /// # Returns
    /// `Ok(borrow_apr)` if no overflow occurs. `Err(MCError::OverOrUnderflow)`
    /// otherwise
    fn compute_borrow_apr(&self, utilization_ratio_bps: u64) -> Result<u64, MCError>;
}

#[derive(Debug, Eq, PartialEq)]
#[enum_dispatch(InterestRate)]
#[contracttype]
pub enum InterestRateModel {
    Kinked(KinkedIRConfig),
}

pub mod kinked;
