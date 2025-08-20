use soroban_sdk::{Symbol, symbol_short};

/// Standard SEP-40 parameters(same as in https://stellar.expert/explorer/testnet/contract/CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63)
pub const DECIMALS: u32 = 14;
pub const RESOLUTION: u32 = 300;

pub const PRICE_SCALING_FACTOR: i128 = i128::pow(10, DECIMALS);

/// TTL extension
pub const SECONDS_PER_LEDGER: u32 = 6;
pub const SECONDS_PER_DAY: u32 = 24 * 60 * 60;
pub const LEDGERS_PER_DAY: u32 = SECONDS_PER_DAY / SECONDS_PER_LEDGER;

pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = INSTANCE_THRESHOLD + LEDGERS_PER_DAY;

/// Dependency contracts
pub const ROUTER_ADDRESS: &str = "CCMAPXWVZD4USEKDWRYS7DA4Y3D7E2SDMGBFJUCEXTC7VN6CUBGWPFUS";
pub const USDC_SAC_ADDRESS: &str = "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA";

/// Misc
pub const USD_SYMBOL: Symbol = symbol_short!("USD");
