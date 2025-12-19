/// Basis points factor (100% = 10000 bps)
pub const BPS_FACTOR: i128 = 10_000;

/// Scale factor for reward per share calculations (10^18 for precision)
pub const SCALE_FACTOR: i128 = 1_000_000_000_000_000_000; // 10^18

/// Maximum number of reward tokens per farm
pub const MAX_REWARD_TOKENS: u32 = 10;

/// Maximum number of points in a reward curve
pub const MAX_CURVE_POINTS: u32 = 20;

/// Maximum early withdrawal penalty (50%)
pub const MAX_EARLY_WITHDRAWAL_PENALTY_BPS: i128 = 5_000;

/// Maximum treasury fee (10%)
pub const MAX_TREASURY_FEE_BPS: i128 = 1_000;

/// Minimum stake amount to prevent dust attacks
pub const MIN_STAKE_AMOUNT: i128 = 1;
