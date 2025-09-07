//! Kinked Interest Rate model implementation.
//! For more details, see: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]

use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Env, contracttype, panic_with_error};

use crate::{
    constants::{
        BPS_FACTOR, DEFAULT_BASE_APR_BPS, DEFAULT_KINK1_APR_BPS,
        DEFAULT_KINK1_UTILIZATION_RATIO_BPS, DEFAULT_KINK2_APR_BPS,
        DEFAULT_KINK2_UTILIZATION_RATIO_BPS, DEFAULT_MAX_APR_BPS,
    },
    error::MCError,
    interest_rate_model::InterestRate,
    math_utils::MathUtils,
};

#[contracttype]
#[derive(Debug, Eq, PartialEq)]
pub struct KinkedIRConfig {
    /// Base APR that is accrued regardless of the utilization ratio of a pool
    pub base_apr_bps: u64,
    /// Kink 1 utilization ratio
    pub kink1_ur_bps: u64,
    /// APR that is accrued when the utilization ratio is at the kink 1 value
    pub kink1_apr_bps: u64,
    /// Kink 2 utilization ratio
    pub kink2_ur_bps: u64,
    /// APR that is accrued when the utilization ratio is at the kink 2 value
    pub kink2_apr_bps: u64,
    /// APR that is accrued when the utilization ratio is at 100%
    pub max_apr_bps: u64,
}

impl Default for KinkedIRConfig {
    fn default() -> Self {
        Self {
            base_apr_bps: DEFAULT_BASE_APR_BPS,
            kink1_ur_bps: DEFAULT_KINK1_UTILIZATION_RATIO_BPS,
            kink1_apr_bps: DEFAULT_KINK1_APR_BPS,
            kink2_ur_bps: DEFAULT_KINK2_UTILIZATION_RATIO_BPS,
            kink2_apr_bps: DEFAULT_KINK2_APR_BPS,
            max_apr_bps: DEFAULT_MAX_APR_BPS,
        }
    }
}

impl InterestRate for KinkedIRConfig {
    fn compute_borrow_apr(&self, utilization_ratio_bps: u64) -> Result<u64, MCError> {
        if utilization_ratio_bps < self.kink1_ur_bps {
            self.calculate_pre_kink1_apr(utilization_ratio_bps)
        } else if utilization_ratio_bps < self.kink2_ur_bps {
            self.calculate_pre_kink2_apr(utilization_ratio_bps)
        } else {
            self.calculate_post_kink2_apr(utilization_ratio_bps)
        }
    }
}

impl KinkedIRConfig {
    pub fn new(
        e: &Env,
        base_apr_bps: u64,
        kink1_ur_bps: u64,
        kink1_apr_bps: u64,
        kink2_ur_bps: u64,
        kink2_apr_bps: u64,
        max_apr_bps: u64,
    ) -> Self {
        let config = Self {
            base_apr_bps,
            kink1_ur_bps,
            kink1_apr_bps,
            kink2_ur_bps,
            kink2_apr_bps,
            max_apr_bps,
        };

        if config.validate().is_err() {
            panic_with_error!(e, MCError::InvalidLoanPoolConfig);
        }

        config
    }

    // TODO: Add some meaningful validation here..
    fn validate(&self) -> Result<(), &str> {
        let &Self { .. } = self;

        Ok(())
    }

    /// Computes borrow `APR` if the utilization ratio precedes the first kink utilization ratio
    fn calculate_pre_kink1_apr(&self, utilization_ratio_bps: u64) -> Result<u64, MCError> {
        // 'borrow_APR' = base_apr + (utilization_ratio_bps/kink1_ur_bps) * (kink1_apr_bps
        // - base_apr_bps)
        let kink1_base_diff_apr_bps = self.kink1_apr_bps - self.base_apr_bps; // safe

        let product_term = kink1_base_diff_apr_bps
            .fixed_mul_floor(utilization_ratio_bps, self.kink1_ur_bps)
            .map_over_or_underflow()?;
        let res = self
            .base_apr_bps
            .checked_add(product_term)
            .map_over_or_underflow()?;

        Ok(res)
    }

    /// Computes borrow `APR` if the utilization ratio precedes the second kink utilization ratio
    fn calculate_pre_kink2_apr(&self, utilization_ratio_bps: u64) -> Result<u64, MCError> {
        // 'borrow_APR' = target_kink_apr + [(utilization_ratio_bps -
        // kink1_ur_bps)/(kink2_ur_bps - kink1_ur_bps)]*(kink2_apr - target_kink_apr)

        let ur_diff = utilization_ratio_bps - self.kink1_ur_bps; // safe
        let max_ur_diff = self.kink2_ur_bps - self.kink1_ur_bps; // safe
        let kink2_target_diff_apr = self.kink2_apr_bps - self.kink1_apr_bps; // safe

        let second_term = kink2_target_diff_apr
            .fixed_mul_floor(ur_diff, max_ur_diff)
            .map_over_or_underflow()?;
        let res = self
            .kink1_apr_bps
            .checked_add(second_term)
            .map_over_or_underflow()?;

        Ok(res)
    }

    fn calculate_post_kink2_apr(&self, utilization_ratio_bps: u64) -> Result<u64, MCError> {
        // `borrow_APR` = kink2_apr + [(utilization_ratio_bps - kink2_ur_bps)/(10_000 -
        // kink2_ur_bps)]*(max_apr - kink2_apr)

        let ur_diff = utilization_ratio_bps - self.kink2_ur_bps; // safe
        let max_ur_diff = (BPS_FACTOR as u64) - self.kink2_ur_bps; // safe
        let max_kink2_diff_apr = self.max_apr_bps - self.kink2_apr_bps; // safe

        let second_term = max_kink2_diff_apr
            .fixed_mul_floor(ur_diff, max_ur_diff)
            .map_over_or_underflow()?;

        let res = self
            .kink2_apr_bps
            .checked_add(second_term)
            .map_over_or_underflow()?;

        Ok(res)
    }
}
