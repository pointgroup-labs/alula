// ---- General ----

/// Number of basis points (bps) in one percent.
/// 1% = 100 bps
pub const BPS_IN_PERCENT: i128 = 100;

/// Basis points denominator for fractional calculations.
/// 100% = 10_000 bps
pub const BPS_FACTOR: i128 = 10_000;

// ---- Time Units ----

/// Seconds in a minute
pub const SECONDS_PER_MINUTE: u32 = 60;

/// Seconds in an hour
pub const SECONDS_PER_HOUR: u32 = SECONDS_PER_MINUTE * 60;

/// Seconds in a day
pub const SECONDS_PER_DAY: u32 = SECONDS_PER_HOUR * 24;

/// Average number of seconds in a year (365.2422 days).
/// Used for interest accrual scaling.
pub const SECONDS_IN_YEAR: u64 = 31_556_926;

// ---- Storage TTL ----

/// Average ledger close time on Stellar
pub const SECONDS_PER_LEDGER: u32 = 6;

/// Number of ledgers in a day
pub const LEDGERS_PER_DAY: u32 = SECONDS_PER_DAY / SECONDS_PER_LEDGER;

/// Instance storage extension is spread among all users, so it must be cheap and paid regularly.
pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = INSTANCE_THRESHOLD + LEDGERS_PER_DAY;

/// Shared persistent storage extension is spread among all shared resource users, so it must be
/// cheap and paid regularly.
pub const SHARED_THRESHOLD: u32 = 50 * LEDGERS_PER_DAY;
pub const SHARED_BUMP: u32 = SHARED_THRESHOLD + LEDGERS_PER_DAY;

/// Individual persistent storage extension is usually paid by the data owners. It should neither be
/// paid very often (to reduce extension operation costs) nor very rarely (to minimize archival
/// risk).
pub const INDIVIDUAL_THRESHOLD: u32 = 160 * LEDGERS_PER_DAY;
pub const INDIVIDUAL_BUMP: u32 = 180 * LEDGERS_PER_DAY;

// ---- Interest Rate and Accrual ----

/// Default base rate per second in bps (scaled by BPS_FACTOR semantics)
pub const DEFAULT_BASE_RATE_PER_SECOND: i128 = 100;

/// Default optimal utilization ratio in percent (0..=100)
pub const DEFAULT_OPTIMAL_UTILIZATION_RATIO: i128 = 80;

/// Default reserve ratio in percent (0..=100)
pub const DEFAULT_RESERVE_RATIO: i128 = 10;

/// Slope 1 (pre-optimal utilization) in bps per utilization unit
pub const DEFAULT_SLOPE1: i128 = 1;

/// Slope 2 (post-optimal utilization) in bps per utilization unit
pub const DEFAULT_SLOPE2: i128 = 10;

/// Initial accrual scaler (fixed-point anchor)
pub const ACCRUAL_INIT: i128 = 1_000_000_000_000;

// ---- Deposit ----

/// Default supply limit (0 means "no limit")
pub const DEFAULT_SUPPLY_LIMIT: i128 = 0;

// ---- Borrow ----

/// Default utilization ratio limit in percent (0..=100)
pub const DEFAULT_UTILIZATION_RATIO_LIMIT: i128 = 100;

// ---- Liquidation ----

/// Max portion of a position that can be liquidated in one go (percent)
pub const DEFAULT_CLOSE_FACTOR: i128 = 50;

/// Health threshold at which positions become eligible for liquidation (percent)
pub const DEFAULT_LIQUIDATION_THRESHOLD: i128 = 80;

/// Additional spread taken during liquidation (percent)
pub const DEFAULT_LIQUIDATION_SPREAD: i128 = 10;

/// Health factor threshold expressed in bps (100% = 10_000 bps)
pub const HEALTH_FACTOR_THRESHOLD_BPS: i128 = 100 * BPS_IN_PERCENT;

// ---- Swap ----

/// Default max slippage in bps
pub const DEFAULT_MAX_SLIPPAGE_BPS: i128 = 1;

/// Default max swap fee in bps
pub const DEFAULT_MAX_SWAP_FEE_BPS: i128 = 1;

// ---- Flash Loan ----

/// Default flash loan fee in bps
pub const DEFAULT_FLASH_LOAN_FEE_BPS: i128 = 1;

// ---- Deposit with leverage ----

/// Scale to represent leverage multipliers (e.g., 100 = 1.0x)
pub const LEVERAGE_SCALE: u32 = 100;

/// Minimum leverage multiplier (scaled by LEVERAGE_SCALE)
pub const MIN_LEVERAGE_MULTIPLIER: u32 = LEVERAGE_SCALE;

// ---- Oracle ----

/// Maximum acceptable oracle price age in seconds
/// TODO: How to properly pick this value?
pub const MAX_ORACLE_PRICE_AGE_SECONDS: u64 = (15 * SECONDS_PER_MINUTE) as u64;

// ---- Contract Addresses ----

/// SEP-40 oracle contract address
pub const ORACLE_ADDRESS: &str = "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";

pub const ROUTER_ADDRESS: &str = "CCMAPXWVZD4USEKDWRYS7DA4Y3D7E2SDMGBFJUCEXTC7VN6CUBGWPFUS";
