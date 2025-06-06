// ---- General ----
pub const BPS_IN_PERCENT: i128 = 100;
pub const BPS_FACTOR: i128 = 10_000;

// ---- Storage TTL ----
pub const SECONDS_PER_LEDGER: u32 = 6;
pub const SECONDS_PER_DAY: u32 = 60 * 60 * 24;
pub const LEDGERS_PER_DAY: u32 = SECONDS_PER_DAY / SECONDS_PER_LEDGER;

// Instance storage extension must be spread among all users, so it must be cheap and paid regularly
pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = INSTANCE_THRESHOLD + LEDGERS_PER_DAY;

// Shared persistent storage extension must be spread among all shared resource users, so it must be cheap and paid regularly
pub const SHARED_THRESHOLD: u32 = 50 * LEDGERS_PER_DAY;
pub const SHARED_BUMP: u32 = SHARED_THRESHOLD + LEDGERS_PER_DAY;

// Individual persistent storage extension is usually paid by the data owners. It should neither be paid very
// often(in order to not pay for extension operation) nor very rare(to minimize the risk of archival)
pub const INDIVIDUAL_THRESHOLD: u32 = 160 * LEDGERS_PER_DAY;
pub const INDIVIDUAL_BUMP: u32 = 180 * LEDGERS_PER_DAY;

// ---- Interest Rate and Accrual ----
pub const DEFAULT_BASE_RATE_PER_SECOND: i128 = 100;
pub const DEFAULT_OPTIMAL_UTILIZATION_RATIO: i128 = 80;
pub const DEFAULT_RESERVE_RATIO: i128 = 10;
pub const DEFAULT_SLOPE1: i128 = 1;
pub const DEFAULT_SLOPE2: i128 = 10;
pub const ACCRUAL_INIT: i128 = 1_000_000_000_000;
pub const SECONDS_IN_YEAR: u64 = 31_556_926;

// ---- Liquidation ----
pub const DEFAULT_CLOSE_FACTOR: i128 = 50;
pub const DEFAULT_LIQUIDATION_THRESHOLD: i128 = 80;
pub const DEFAULT_LIQUIDATION_SPREAD: i128 = 10;
pub const HEALTH_FACTOR_THRESHOLD_BPS: i128 = 100 * BPS_IN_PERCENT;

// ---- Contract Addresses ----
pub const REFLECTOR_TESTNET_ADDRESS: &str =
    "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";

// ---- Aliases ----
pub type LCError = crate::error::LendingContractError;

// TODO: think about extension
// pub trait BpsExtension {
//     fn to_bps(self) -> i128;
// }
// impl BpsExtension for i128 {
//     fn to_bps(self) -> i128 {
//         self * BPS_IN_PERCENT
//     }
// }
