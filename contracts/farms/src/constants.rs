pub const MAX_CURVE_POINTS: u32 = 20;
pub const MAX_ALLOWED_FARMS: u32 = 10;
pub const MAX_FARM_NUM_REWARDS: u32 = 10;
pub const MAX_TREASURY_FEE_BPS: i128 = 2_000;
pub const MAX_HARVEST_DELAY: u64 = 24 * 60 * 60;
pub const MAX_DEPOSIT_WARMUP_PERIOD: u64 = 24 * 60 * 60;
pub const MAX_LOCKING_DURATION: u64 = 365 * 24 * 60 * 60;
pub const MAX_WITHDRAWAL_COOLDOWN_PERIOD: u64 = 24 * 60 * 60;

pub const BPS_FACTOR: i128 = 10_000;
pub const SCALE_FACTOR: i128 = 10_i128.pow(18);
