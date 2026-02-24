pub const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

pub const MAX_CURVE_POINTS: u32 = 20;
pub const MAX_FARM_NUM_REWARDS: u32 = 10;
pub const MAX_TREASURY_FEE_BPS: i128 = 2_000; // 20%
pub const MAX_HARVEST_DELAY: u64 = SECONDS_PER_DAY;
pub const MAX_DEPOSIT_WARMUP_PERIOD: u64 = SECONDS_PER_DAY;
pub const MAX_LOCKING_DURATION: u64 = 365 * SECONDS_PER_DAY;
pub const MAX_WITHDRAWAL_COOLDOWN_PERIOD: u64 = SECONDS_PER_DAY;

/// 100% in basis points
pub const BPS_FACTOR: i128 = 10_000;

/// Fixed-point scaling factor for rewards-per-share accumulator (18 decimals)
pub const SCALE_FACTOR: i128 = 10_i128.pow(18);
