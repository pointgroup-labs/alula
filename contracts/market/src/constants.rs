// ---- General ----

/// Basis points denominator for fractional calculations: 100% = 10,000 bps
pub const BPS_FACTOR: i128 = 10_000;

/// Denominator for fractional calculations with 9 decimal places
pub const SCALAR_9: i128 = i128::pow(10, 9);

/// Denominator for fixed-point compounded interest calculations.
/// Higher precision (10^18) ensures accurate results when raising to integer powers.
pub const SCALED_FIXED_POINT_DENOMINATOR: i128 = i128::pow(10, 18);

// ---- Time Units ----

/// Seconds in a minute
pub const SECONDS_PER_MINUTE: u64 = 60;

/// Seconds in an hour
pub const SECONDS_PER_HOUR: u64 = SECONDS_PER_MINUTE * 60;

/// Seconds in a day
pub const SECONDS_PER_DAY: u64 = SECONDS_PER_HOUR * 24;

/// Average number of seconds in a year (365.2422 days).
/// Used for interest rate annualization.
pub const SECONDS_IN_YEAR: u64 = 31_556_926;

// ---- Storage TTL ----

/// Average ledger close time on Stellar (~6 seconds)
pub const SECONDS_PER_LEDGER: u64 = 6;

/// Number of ledgers in a day
pub const LEDGERS_PER_DAY: u32 = (SECONDS_PER_DAY / SECONDS_PER_LEDGER) as u32;

/// Instance storage TTL threshold (~40 days).
/// Cost is shared among all users, so extensions are frequent and cheap.
pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = INSTANCE_THRESHOLD + LEDGERS_PER_DAY;

/// Shared persistent storage TTL threshold (~50 days).
/// Cost is shared among resource users, so extensions are frequent and cheap.
pub const SHARED_THRESHOLD: u32 = 50 * LEDGERS_PER_DAY;
pub const SHARED_BUMP: u32 = SHARED_THRESHOLD + LEDGERS_PER_DAY;

/// Individual persistent storage TTL threshold (~160 days).
/// Paid by data owners; balanced between extension frequency and archival risk.
pub const INDIVIDUAL_THRESHOLD: u32 = 160 * LEDGERS_PER_DAY;
pub const INDIVIDUAL_BUMP: u32 = 180 * LEDGERS_PER_DAY;

// ---- Interest Rate and Accrual ----

// -- Kinked(with 2 kink points) interest rate model --

pub const DEFAULT_BASE_APR_BPS: i128 = 1; // 0.01%
pub const DEFAULT_KINK1_UTILIZATION_RATIO_BPS: i128 = 7_000;
pub const DEFAULT_KINK2_UTILIZATION_RATIO_BPS: i128 = 8_000;

pub const DEFAULT_KINK1_APR_BPS: i128 = 3_000;
pub const DEFAULT_KINK2_APR_BPS: i128 = 6_000;
pub const DEFAULT_MAX_APR_BPS: i128 = 40_000;

// -- Interest Rate reactivity --

pub const DEFAULT_TARGET_UTILIZATION_RATIO_BPS: i128 = 6_500;
pub const MIN_REACTIVITY_CONSTANT: u32 = 0;
pub const MAX_REACTIVITY_CONSTANT: u32 = 100; // 0.01
pub const MIN_IR_MODIFIER: i128 = BPS_FACTOR / 10; // x0.1
pub const MAX_IR_MODIFIER: i128 = BPS_FACTOR * 10; // x10

// ---- Deposit ----

// Default supply limit (0 means "no limit")
pub const DEFAULT_SUPPLY_LIMIT: i128 = 0;

// ---- Borrow ----

// Default utilization ratio limit
pub const DEFAULT_UTILIZATION_RATIO_LIMIT_BPS: i128 = 9000;

// ---- Liquidation ----

// Max portion of a position that can be liquidated in one go
pub const DEFAULT_CLOSE_FACTOR_BPS: i128 = 5_000;
// Additional spread taken during liquidation
pub const DEFAULT_LIQUIDATION_INCENTIVE_BPS: i128 = 1_000;
pub const DEFAULT_OPEN_LTV_BPS: i128 = 7_000;
pub const DEFAULT_CLOSE_LTV_BPS: i128 = 8_000;
pub const DEFAULT_LIABILITY_FACTOR_BPS: i128 = BPS_FACTOR; // 100% (equivalent to a liability factor to make no difference)
pub const MAX_LIABILITY_FACTOR_BPS: i128 = 2 * BPS_FACTOR;

pub const DEFAULT_INSOLVENCY_LTV_BPS: i128 = 9_850;
pub const MIN_INSOLVENCY_LTV_BPS: i128 = 9_500;
pub const MAX_INSOLVENCY_LTV_BPS: i128 = BPS_FACTOR; // 100%

// ---- Fees ----

pub const DEFAULT_REPAY_FEE_BPS: u32 = 0;
pub const DEFAULT_BORROW_FEE_BPS: u32 = 0;
pub const DEFAULT_DEPOSIT_FEE_BPS: u32 = 0;
pub const DEFAULT_WITHDRAW_FEE_BPS: u32 = 0;
pub const DEFAULT_MAX_WITHDRAW_SCARCITY_FEE_BPS: u32 = 500; // 5%
pub const DEFAULT_FLASH_LOAN_FEE_BPS: u32 = 1; // 0.01%
pub const DEFAULT_ADD_COLLATERAL_FEE_BPS: u32 = 0;
pub const DEFAULT_REMOVE_COLLATERAL_FEE_BPS: u32 = 0;

pub const DEFAULT_TAKE_RATE_BPS: u32 = 1_000; // 10%

// ---- Pool Status ----

pub const POOL_STATUS_DEPOSIT_ENABLED: u32 = 1 << 0;
pub const POOL_STATUS_BORROW_ENABLED: u32 = 1 << 1;
pub const POOL_STATUS_ADD_COLLATERAL_ENABLED: u32 = 1 << 2;
pub const POOL_STATUS_FLASH_LOAN_ENABLED: u32 = 1 << 3;
pub const POOL_STATUS_ALL_ENABLED: u32 = u32::MAX;

// ---- Oracle ----

// Maximum acceptable oracle price age in seconds
pub const MAX_ORACLE_PRICE_AGE_SECONDS: u64 = 6 * SECONDS_PER_MINUTE; // NB: Relies on 'Reflector' resolution being 5 minutes

// ---- MISC ----

pub const DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS: u64 = 24 * 60 * 60;
pub const DEFAULT_WITHDRAW_SCARCITY_LIMIT_BPS: i128 = BPS_FACTOR; // 100%
pub const DEFAULT_WITHDRAW_SCARCITY_COOLDOWN_SECS: u64 = 0;
pub const MAX_WITHDRAW_SCARCITY_COOLDOWN_SECS: u64 = 24 * 60 * 60;
pub const DEFAULT_MIN_COLLATERAL_VALUE_CENTS: i128 = 500; // 5$ 
pub const MIN_COLLATERAL_VALUE_CENTS: i128 = 0; // 0$ 
pub const MAX_COLLATERAL_VALUE_CENTS: i128 = 10_000; // 100$
pub const DEFAULT_MAX_POSITIONS: u32 = 20;

pub const MAX_RESERVES: u32 = 25; // Max reserves per a lending market

pub const DEFAULT_BAD_DEBT_LOCK_D: u64 = 12 * SECONDS_PER_HOUR; // 12 hours
pub const MAX_BAD_DEBT_LOCK_D: u64 = 5 * SECONDS_PER_DAY; // 5 days
pub const MIN_BAD_DEBT_LOCK_D: u64 = 0; // no lock
