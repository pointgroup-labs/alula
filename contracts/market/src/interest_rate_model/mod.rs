use enum_dispatch::enum_dispatch;
use soroban_sdk::contracttype;

use crate::{error::MCError, interest_rate_model::kinked::KinkedIRConfig};

#[enum_dispatch]
pub trait InterestRate {
    /// Computes annual percentage rate in basis points
    ///
    /// # Arguments
    /// * `utilization_ratio_bps` - utilization ratio of a pool
    /// # Returns
    /// `Ok(borrow_apr)` if no overflow occurs. `Err(MCError::OverOrUnderflow)`
    /// otherwise
    fn compute_borrow_apr(&self, utilization_ratio_bps: i128) -> Result<i128, MCError>;

    /// Validates the model's parameters. Accrual runs inside every money operation, so a model that
    /// cannot produce a usable APR takes the pool out of service until a new config is applied.
    fn validate(&self) -> Result<(), &'static str>;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[enum_dispatch(InterestRate)]
#[contracttype]
pub enum InterestRateModel {
    Kinked(KinkedIRConfig),
}

impl Default for InterestRateModel {
    // NB: Rust doesn't support `#[default]` for `#[derive(Default)]` on non-unit variants for now
    fn default() -> Self {
        Self::Kinked(Default::default())
    }
}

pub mod kinked;
