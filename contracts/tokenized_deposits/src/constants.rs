// ---- Virtual offset (inflation attack mitigation) ----

// Чи є нам сенс відносити jTokens завжди як 1:1?
// Не очевидно, що є...

// Virtual shares and assets are added to both sides of every conversion, making the vault behave
// as if it always held a small phantom deposit. This removes the division-by-zero and the
// "first depositor sets an arbitrary exchange rate" problem in an empty vault.
//
// The offset is a power of ten: `virtual_shares = 10^offset`, `virtual_assets = 1`. A donation
// attacker's inflated shares are diluted against the virtual shares, so the value they can
// capture from a victim's rounding loss is always less than what they must donate to cause it.
// A larger offset shrinks the victim's loss; at offset 9 it is nil for realistic amounts.
//
// The effective offset is clamped at construction so that
// `asset_decimals + offset <= MAX_DECIMALS`, which keeps the share token's decimals
// representable for high-precision underlying assets
pub const VIRTUAL_ASSETS: i128 = 1;
pub const PREFERRED_DECIMALS_OFFSET: u32 = 9;

// Returns the virtual share count for a given offset
pub fn virtual_shares(offset: u32) -> i128 {
    i128::pow(10, offset)
}

// ---- Metadata limits ----

pub const MAX_DECIMALS: u32 = 18;
pub const MAX_NAME_LENGTH: u32 = 64;
pub const MAX_SYMBOL_LENGTH: u32 = 32;

// ---- TTL ----

pub const LEDGERS_PER_DAY: u32 = 17_280; // ~5 seconds per ledger

pub const INSTANCE_BUMP: u32 = 30 * LEDGERS_PER_DAY;
pub const INSTANCE_THRESHOLD: u32 = INSTANCE_BUMP - LEDGERS_PER_DAY;

pub const BALANCE_BUMP: u32 = 120 * LEDGERS_PER_DAY;
pub const BALANCE_THRESHOLD: u32 = BALANCE_BUMP - LEDGERS_PER_DAY;
