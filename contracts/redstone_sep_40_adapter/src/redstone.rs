use soroban_sdk::{Env, U256, contractclient, contracttype};

/// RedStone's on-chain price data returned by `read_price_data`.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RSPriceData {
    pub package_timestamp: u64,
    pub price: U256,
    pub write_timestamp: u64,
}

/// Per-feed RedStone price feed contract interface.
///
/// Each deployed instance is bound to a single feed (set at `init`).
/// All read functions return data for that feed without requiring a feed ID argument.
#[allow(dead_code)]
#[contractclient(name = "RedStonePriceFeedClient")]
pub trait RedStonePriceFeed {
    fn read_price_data(e: Env) -> RSPriceData;
    fn read_price(e: Env) -> U256;
    fn read_timestamp(e: Env) -> u64;
    fn read_price_and_timestamp(e: Env) -> (U256, u64);
    fn decimals(e: Env) -> u64;
}
