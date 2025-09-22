// ---- General ----

/// Number of basis points (bps) in one percent.
/// 1% = 100 bps
pub const BPS_IN_PERCENT: i128 = 100;

/// Basis points denominator for fractional calculations.
/// 100% = 10_000 bps
pub const BPS_FACTOR: i128 = 10_000;

// ---- Time Units ----

/// Seconds in a minute
pub const SECONDS_PER_MINUTE: u64 = 60;

/// Seconds in an hour
pub const SECONDS_PER_HOUR: u64 = SECONDS_PER_MINUTE * 60;

/// Seconds in a day
pub const SECONDS_PER_DAY: u64 = SECONDS_PER_HOUR * 24;

/// Average number of seconds in a year (365.2422 days).
/// Used for interest accrual scaling.
pub const SECONDS_IN_YEAR: u64 = 31_556_926;

// ---- Storage TTL ----

/// Average ledger close time on Stellar
pub const SECONDS_PER_LEDGER: u64 = 6;

/// Number of ledgers in a day
pub const LEDGERS_PER_DAY: u32 = (SECONDS_PER_DAY / SECONDS_PER_LEDGER) as u32;

/// Instance storage extension is spread among all users, so it must be cheap and paid regularly
pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = INSTANCE_THRESHOLD + LEDGERS_PER_DAY;

/// Shared persistent storage extension is spread among all shared resource users, so it must be
/// cheap and paid regularly
pub const SHARED_THRESHOLD: u32 = 50 * LEDGERS_PER_DAY;
pub const SHARED_BUMP: u32 = SHARED_THRESHOLD + LEDGERS_PER_DAY;

/// Individual persistent storage extension is usually paid by the data owners. It should neither be
/// paid very often (to reduce extension operation costs) nor very rarely (to minimize archival
/// risk)(TODO: Though, is it really a sound argument?)
pub const INDIVIDUAL_THRESHOLD: u32 = 160 * LEDGERS_PER_DAY;
pub const INDIVIDUAL_BUMP: u32 = 180 * LEDGERS_PER_DAY;

// ---- Interest Rate and Accrual ----
/// A denominator that is primarily used for compounded accrual calculation when numbers in fixed
/// point representation are raised to the natural number power. The bigger the denominator, the
/// better the multiplication precision, the better the binary exponentiation precision
pub const SCALED_FIXED_POINT_DENOMINATOR: i128 = 1_000_000_000_000_000_000;
pub const DEFAULT_RESERVE_RATIO: i128 = 10;
pub const DEFAULT_BASE_APR_BPS: i128 = 1; // 0.01%
pub const DEFAULT_KINK1_UTILIZATION_RATIO_BPS: i128 = 7_000; // 70%
pub const DEFAULT_KINK2_UTILIZATION_RATIO_BPS: i128 = 8_000; // 80%

pub const DEFAULT_KINK1_APR_BPS: i128 = 3_000; // 30%
pub const DEFAULT_KINK2_APR_BPS: i128 = 6_000; // 60%
pub const DEFAULT_MAX_APR_BPS: i128 = 40_000; // 400%

// ---- Deposit ----

/// Default supply limit (0 means "no limit")
pub const DEFAULT_SUPPLY_LIMIT: i128 = 0;

// ---- Borrow ----

/// Default utilization ratio limit in percent (0..=100)
pub const DEFAULT_UTILIZATION_RATIO_LIMIT: i128 = 100;

// ---- Liquidation ----

/// Max portion of a position that can be liquidated in one go (percent)
pub const DEFAULT_CLOSE_FACTOR: i128 = 50;

pub const DEFAULT_OPEN_LTV: i128 = 70;
pub const DEFAULT_CLOSE_LTV: i128 = 80;

/// Additional spread taken during liquidation (percent)
pub const DEFAULT_LIQUIDATION_SPREAD: i128 = 10;

/// Health factor threshold expressed in bps (100% = 10_000 bps)
pub const HEALTH_FACTOR_THRESHOLD_BPS: i128 = 100 * BPS_IN_PERCENT;

pub const DEFAULT_LIABILITY_FACTOR: i128 = 100; // 100% is equivalent to a liability factor to not make any difference
pub const MAX_LIABILITY_FACTOR: i128 = 200;

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
pub const MAX_ORACLE_PRICE_AGE_SECONDS: u64 = 15 * SECONDS_PER_MINUTE;

// ---- Contract Addresses ----

// pub const ORACLE_ADDRESS: &str = "CCMRMA3P4AJ4T4CBHUBYXBFX7TNZLPVWNUVCR2775OH7KLDJJZXLI32P";

pub const ROUTER_ADDRESS: &str = "CCMAPXWVZD4USEKDWRYS7DA4Y3D7E2SDMGBFJUCEXTC7VN6CUBGWPFUS";
