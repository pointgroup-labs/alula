// ---- Metadata limits ----

pub const MAX_DECIMALS: u32 = 18;
pub const MAX_NAME_LENGTH: u32 = 64;
pub const MAX_SYMBOL_LENGTH: u32 = 32;

// ---- TTL ----

pub const LEDGERS_PER_DAY: u32 = 17_280; // ~5 seconds per ledger

pub const INSTANCE_BUMP: u32 = 30 * LEDGERS_PER_DAY;
pub const INSTANCE_THRESHOLD: u32 = INSTANCE_BUMP - LEDGERS_PER_DAY;
