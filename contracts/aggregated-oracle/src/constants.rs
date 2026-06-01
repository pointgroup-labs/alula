// -- TTL extension --
pub const LEDGERS_PER_DAY: u32 = (24 * 60 * 60) / 6; // NB: Assuming 6 seconds per ledger
pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = 41 * LEDGERS_PER_DAY;

/// -- Periodic oracles price max age bounds --
pub const MIN_PERIODIC_ORACLES_PRICE_MAX_AGE: u64 = 60;
pub const MAX_PERIODIC_ORACLES_PRICE_MAX_AGE: u64 = 12 * 60 * 60;

/// -- Misc --
pub const BPS_FACTOR: i128 = 10_000;

pub const MIN_ORACLES_LEN: u32 = 1;
pub const MAX_ORACLES_LEN: u32 = 10;
