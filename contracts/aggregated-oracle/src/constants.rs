// -- TTL extension --
pub const LEDGERS_PER_DAY: u32 = (24 * 60 * 60) / 6; // NB: Assuming 6 seconds per ledger
pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = 41 * LEDGERS_PER_DAY;

/// -- Misc --
pub const BPS_FACTOR: i128 = 10_000;
