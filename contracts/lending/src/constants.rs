pub const REFLECTOR_TESTNET_ADDRESS: &str =
    "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";

pub const BPS_IN_PERCENT: i128 = 100;

// TODO: think about extension
// pub trait BpsExtension {
//     fn to_bps(self) -> i128;
// }
// impl BpsExtension for i128 {
//     fn to_bps(self) -> i128 {
//         self * BPS_IN_PERCENT
//     }
// }

pub const DEFAULT_LIQUIDATION_THRESHOLD: i128 = 80;

pub const DEFAULT_BASE_RATE: i128 = 1;
pub const DEFAULT_OPTIMAL_UTILIZATION_RATIO: i128 = 80;
pub const DEFAULT_RESERVE_RATIO: i128 = 10;
pub const DEFAULT_SLOPE1: i128 = 1;
pub const DEFAULT_SLOPE2: i128 = 10;

#[allow(clippy::inconsistent_digit_grouping)]
pub const ACCRUAL_INIT_VALUE: i128 = 1__00_000_000_000;
pub const SECONDS_IN_YEAR: u64 = 31_556_926;

pub type LCError = crate::error::LendingContractError;
