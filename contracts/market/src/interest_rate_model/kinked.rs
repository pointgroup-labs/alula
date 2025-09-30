//! Kinked Interest Rate model implementation.
//! For more details, see: [`https://berkeley-defi.github.io/assets/material/DeFi%20Protocols%20for%20Loanable%20Funds.pdf`]

use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Env, contracttype, panic_with_error};

use crate::{
    constants::*, error::MCError, interest_rate_model::InterestRate, math_utils::MathUtils,
};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KinkedIRConfig {
    /// Base APR that is accrued regardless of the utilization ratio of a pool
    pub base_apr_bps: i128,
    /// Kink 1 utilization ratio
    pub kink1_ur_bps: i128,
    /// APR that is accrued when the utilization ratio is at the kink 1 value
    pub kink1_apr_bps: i128,
    /// Kink 2 utilization ratio
    pub kink2_ur_bps: i128,
    /// APR that is accrued when the utilization ratio is at the kink 2 value
    pub kink2_apr_bps: i128,
    /// APR that is accrued when the utilization ratio is at 100%
    pub max_apr_bps: i128,
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
    fn compute_borrow_apr(&self, utilization_ratio_bps: i128) -> Result<i128, MCError> {
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
        base_apr_bps: i128,
        kink1_ur_bps: i128,
        kink1_apr_bps: i128,
        kink2_ur_bps: i128,
        kink2_apr_bps: i128,
        max_apr_bps: i128,
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

    fn validate(&self) -> Result<(), &str> {
        let &Self {
            base_apr_bps,
            kink1_ur_bps,
            kink1_apr_bps,
            kink2_ur_bps,
            kink2_apr_bps,
            max_apr_bps,
        } = self;

        if 0 > kink1_ur_bps || kink1_ur_bps > kink2_ur_bps || kink2_ur_bps > BPS_FACTOR {
            return Err(
                "Utilization ratios for kink points must be non-negative and in non-descending
                 order and not exceeding 100%",
            );
        }

        if 0 > base_apr_bps
            || base_apr_bps > kink1_apr_bps
            || kink1_apr_bps > kink2_apr_bps
            || kink2_apr_bps > max_apr_bps
        {
            return Err("APRs for kink points must be non-negative and in non-descending order");
        }

        Ok(())
    }

    /// Computes borrow `APR` if the utilization ratio precedes the first kink utilization ratio
    fn calculate_pre_kink1_apr(&self, utilization_ratio_bps: i128) -> Result<i128, MCError> {
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
    fn calculate_pre_kink2_apr(&self, utilization_ratio_bps: i128) -> Result<i128, MCError> {
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

    fn calculate_post_kink2_apr(&self, utilization_ratio_bps: i128) -> Result<i128, MCError> {
        // `borrow_APR` = kink2_apr + [(utilization_ratio_bps - kink2_ur_bps)/(10_000 -
        // kink2_ur_bps)]*(max_apr - kink2_apr)
        let ur_diff = utilization_ratio_bps - self.kink2_ur_bps; // safe
        let max_ur_diff = BPS_FACTOR - self.kink2_ur_bps; // safe
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

#[cfg(test)]
mod test {
    use soroban_sdk::Env;

    use super::{BPS_FACTOR, InterestRate, KinkedIRConfig};

    #[test]
    fn test_apr_at_zero_utilization() {
        let config = KinkedIRConfig::default();
        let utilization_ratio = 0;
        let expected_apr_bps = config.base_apr_bps;

        assert_eq!(
            config.compute_borrow_apr(utilization_ratio).unwrap(),
            expected_apr_bps
        );
    }

    #[test]
    fn test_apr_pre_kink1() {
        let config = KinkedIRConfig::default();
        let utilization_ratio = 2_500; // 25% UR
        // Calculation: 1 + (2500/7000) * (3000 - 1) = 1072
        let expected_apr_bps = 1072;

        assert_eq!(
            config.compute_borrow_apr(utilization_ratio).unwrap(),
            expected_apr_bps
        );
    }

    #[test]
    fn test_apr_at_kink1() {
        let config = KinkedIRConfig::default();
        let utilization_ratio = config.kink1_ur_bps;
        let expected_apr_bps = config.kink1_apr_bps;

        assert_eq!(
            config.compute_borrow_apr(utilization_ratio).unwrap(),
            expected_apr_bps
        );
    }

    #[test]
    fn test_apr_between_kinks() {
        let config = KinkedIRConfig::default();
        let utilization_ratio = 7_500; // 75% UR
        // Calculation: 3000 + [(7500 - 7000)/(8000 - 7000)] * (6000 - 3000)
        let expected_apr_bps = 4500;

        assert_eq!(
            config.compute_borrow_apr(utilization_ratio).unwrap(),
            expected_apr_bps
        );
    }

    #[test]
    fn test_apr_at_kink2() {
        let config = KinkedIRConfig::default();
        let utilization_ratio = config.kink2_ur_bps;
        let expected_apr_bps = config.kink2_apr_bps;

        assert_eq!(
            config.compute_borrow_apr(utilization_ratio).unwrap(),
            expected_apr_bps
        );
    }

    #[test]
    fn test_apr_post_kink2() {
        let config = KinkedIRConfig::default();
        let utilization_ratio = 9_000; // 90% UR
        // Calculation: 6000 + [(9000 - 8000)/(10000 - 8000)] * (40000 - 6000)
        let expected_apr_bps = 23000;

        assert_eq!(
            config.compute_borrow_apr(utilization_ratio).unwrap(),
            expected_apr_bps
        );
    }

    #[test]
    fn test_apr_at_max_utilization() {
        let config = KinkedIRConfig::default();
        let utilization_ratio = BPS_FACTOR;
        let expected_apr_bps = config.max_apr_bps;

        assert_eq!(
            config.compute_borrow_apr(utilization_ratio).unwrap(),
            expected_apr_bps
        );
    }

    #[test]
    fn test_apr_with_custom_config() {
        let e = Env::default();
        let custom_config = KinkedIRConfig::new(
            &e, 100,   // base_apr_bps
            4_000, // kink1_ur_bps
            400,   // kink1_apr_bps
            9_000, // kink2_ur_bps
            1_500, // kink2_apr_bps
            5_000, // max_apr_bps
        );

        let utilization_ratio = 5_000; // 50% UR
        // Calculation: 400 + [(5000 - 4000)/(9000 - 4000)] * (1500 - 400)
        let expected_apr_bps = 620;

        assert_eq!(
            custom_config.compute_borrow_apr(utilization_ratio).unwrap(),
            expected_apr_bps
        );
    }
}
