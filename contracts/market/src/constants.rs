// ---- General ----

/// Number of basis points (bps) in one percent
/// 1% = 100 bps
pub const BPS_IN_PERCENT: i128 = 100;
/// Basis points denominator for fractional calculations
/// 100% = 10_000 bps
pub const BPS_FACTOR: i128 = 10_000;
/// A denominator that is primarily used for compounded accrual calculation when numbers are in
/// fixed point representations are raised to the natural number power. The bigger the denominator,
/// the better the multiplication precision, the better the binary exponentiation precision
pub const SCALED_FIXED_POINT_DENOMINATOR: i128 = i128::pow(10, 18);

// ---- Time Units ----

/// Seconds in a minute
pub const SECONDS_PER_MINUTE: u64 = 60;
/// Seconds in an hour
pub const SECONDS_PER_HOUR: u64 = SECONDS_PER_MINUTE * 60;
/// Seconds in a day
pub const SECONDS_PER_DAY: u64 = SECONDS_PER_HOUR * 24;
/// Average number of seconds in a year (365.2422 days)
/// Used for interest accrual scaling
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

// -- Kinked(wth 2 kink points) interest rate model --

pub const DEFAULT_BASE_APR_BPS: i128 = 1; // 0.01%
pub const DEFAULT_RESERVE_RATIO_BPS: i128 = 1_000; // 10%
pub const DEFAULT_KINK1_UTILIZATION_RATIO_BPS: i128 = 7_000; // 70%
pub const DEFAULT_KINK2_UTILIZATION_RATIO_BPS: i128 = 8_000; // 80%

pub const DEFAULT_KINK1_APR_BPS: i128 = 3_000; // 30%
pub const DEFAULT_KINK2_APR_BPS: i128 = 6_000; // 60%
pub const DEFAULT_MAX_APR_BPS: i128 = 40_000; // 400%

// -- Interest Reactivity --

pub const DEFAULT_TARGET_UTILIZATION_RATIO_BPS: i128 = 7_000; // 70%
pub const DEFAULT_REACTIVITY_CONSTANT: i128 = 0;

pub const MAX_REACTIVITY_CONSTANT: i128 = 100; // 100%(represents 0.0001)

// ---- Deposit ----

/// Default supply limit (0 means "no limit")
pub const DEFAULT_SUPPLY_LIMIT: i128 = 0;

// ---- Borrow ----

/// Default utilization ratio limit
pub const DEFAULT_UTILIZATION_RATIO_LIMIT_BPS: i128 = 9000; // 90%

// ---- Liquidation ----

/// Max portion of a position that can be liquidated in one go
pub const DEFAULT_CLOSE_FACTOR_BPS: i128 = 5_000;
/// Additional spread taken during liquidation
pub const DEFAULT_LIQUIDATION_INCENTIVE_BPS: i128 = 1_000; // 10%
pub const DEFAULT_OPEN_LTV_BPS: i128 = 7_000;
pub const DEFAULT_CLOSE_LTV_BPS: i128 = 8_000;
/// Health factor threshold expressed in bps (100% = 10_000 bps)
pub const HEALTH_FACTOR_THRESHOLD_BPS: i128 = BPS_FACTOR; // 100%
pub const DEFAULT_LIABILITY_FACTOR_BPS: i128 = BPS_FACTOR; // 100% (equivalent to a liability factor to make no difference)
pub const MAX_LIABILITY_FACTOR_BPS: i128 = 2 * BPS_FACTOR; // 200%

pub const DEFAULT_INSOLVENCY_LTV_BPS: i128 = 9_850; // 98.5%
pub const MIN_INSOLVENCY_LTV_BPS: i128 = 9_500; // 95%
pub const MAX_INSOLVENCY_LTV_BPS: i128 = BPS_FACTOR; // 100%

// ---- Swap ----

/// Default max slippage in bps
pub const DEFAULT_MAX_SLIPPAGE_BPS: i128 = 1; // 0.01%
/// Default max swap fee in bps
pub const DEFAULT_MAX_SWAP_FEE_BPS: i128 = 1; // 0.01%

// ---- Deposit with leverage ----

/// Scale to represent leverage multipliers (e.g., with current scale 100 = 1.0x, 224 = 2.24x)
pub const LEVERAGE_SCALE: u32 = 100;
/// Minimum leverage multiplier (scaled by LEVERAGE_SCALE)
pub const MIN_LEVERAGE_MULTIPLIER: u32 = 100; // x1

// ---- Fees ----

pub const DEFAULT_REPAY_FEE_BPS: u32 = 0;
pub const DEFAULT_BORROW_FEE_BPS: u32 = 0;
pub const DEFAULT_DEPOSIT_FEE_BPS: u32 = 0;
pub const DEFAULT_WITHDRAW_FEE_BPS: u32 = 0;
pub const DEFAULT_WITHDRAW_SCARCITY_FEE_SCALAR_BPS: u32 = 20_000; // 200%
pub const DEFAULT_FLASH_LOAN_FEE_BPS: u32 = 1; // 0.01%
pub const DEFAULT_ADD_COLLATERAL_FEE_BPS: u32 = 0;
pub const DEFAULT_REMOVE_COLLATERAL_FEE_BPS: u32 = 0;

pub const DEFAULT_HOST_FEE_BPS: u32 = 2000; // 20%

pub const DEFAULT_TAKE_RATE_BPS: u32 = 1000; // 10%

// ---- Oracle ----

/// Maximum acceptable oracle price age in seconds

// TODO: Make it configurable? | Should we even have something like this?
pub const MAX_ORACLE_PRICE_AGE_SECONDS: u64 = 6 * SECONDS_PER_MINUTE; // NB: Relies on 'Reflector' resolution being 5 minutes

// ---- Dependency Contract Addresses ----

pub const ROUTER_ADDRESS: &str = "CCMAPXWVZD4USEKDWRYS7DA4Y3D7E2SDMGBFJUCEXTC7VN6CUBGWPFUS";

// ---- MISC ----

pub const DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS: u64 = 24 * 60 * 60;
pub const DEFAULT_WITHDRAW_SCARCITY_LIMIT_BPS: i128 = BPS_FACTOR; // 100%
pub const DEFAULT_WITHDRAW_SCARCITY_COOLDOWN_SECS: u64 = 0;
pub const MAX_WITHDRAW_SCARCITY_COOLDOWN_SECS: u64 = 24 * 60 * 60;
pub const DEFAULT_MIN_COLLATERAL_VALUE: i128 = 10i128.pow(5); // 10^5 = 0.01
pub const DEFAULT_MAX_POSITIONS: u32 = 20;

pub const MAX_RESERVES: u32 = 25; // Max reserves per a lending market

pub const INITIAL_SHARES_AMOUNT: i128 = 10_i128.pow(15);
