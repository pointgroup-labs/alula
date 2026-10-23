use sep_40_oracle::{Asset, PriceData, PriceFeedTrait};
use soroban_sdk::{
    Address, BytesN, Env, Map, Symbol, Vec, contract, contracterror, contractimpl, contracttype,
    panic_with_error,
};

use crate::chainlink::DataFeedsProxyClient;

/// Chainlink's push feeds update on a 0.5 % move or every 24 h; the aggregated oracle reads `resolution` as the
/// staleness limit, so it carries an hour of slack for a heartbeat that lands late.
const RESOLUTION_S: u32 = 25 * 60 * 60;
const MAX_DECIMALS: u32 = 18;
const LEDGERS_PER_DAY: u32 = 14_400;

#[contracttype]
enum DataKey {
    Proxy,
    Decimals,
    Feeds,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AdapterError {
    InvalidDecimals = 1,
    NoFeeds = 2,
}

#[contract]
pub struct ChainlinkSep40AdapterContract;

#[contractimpl]
impl ChainlinkSep40AdapterContract {
    /// `feeds` maps each token to its Chainlink data id. Everything is fixed here: there is no admin and no
    /// upgrade path, so a change means deploying a new adapter.
    pub fn __constructor(e: Env, proxy: Address, decimals: u32, feeds: Map<Address, BytesN<32>>) {
        if decimals > MAX_DECIMALS {
            panic_with_error!(&e, AdapterError::InvalidDecimals);
        }
        if feeds.is_empty() {
            panic_with_error!(&e, AdapterError::NoFeeds);
        }
        // A wrong id or unsupported decimals fails the deploy instead of the first read.
        let client = DataFeedsProxyClient::new(&e, &proxy);
        for (_, data_id) in feeds.iter() {
            client.latest_round(&data_id, &decimals);
        }
        let storage = e.storage().instance();
        storage.set(&DataKey::Proxy, &proxy);
        storage.set(&DataKey::Decimals, &decimals);
        storage.set(&DataKey::Feeds, &feeds);
        extend_instance(&e);
    }

    pub fn proxy(e: Env) -> Address {
        e.storage().instance().get(&DataKey::Proxy).unwrap()
    }

    pub fn feeds(e: Env) -> Map<Address, BytesN<32>> {
        e.storage().instance().get(&DataKey::Feeds).unwrap()
    }
}

#[contractimpl]
impl PriceFeedTrait for ChainlinkSep40AdapterContract {
    /// Every Chainlink feed on Stellar is quoted in USD.
    fn base(e: Env) -> Asset {
        Asset::Other(Symbol::new(&e, "USD"))
    }

    fn assets(e: Env) -> Vec<Asset> {
        let mut assets = Vec::new(&e);
        for token in Self::feeds(e.clone()).keys() {
            assets.push_back(Asset::Stellar(token));
        }
        assets
    }

    fn decimals(e: Env) -> u32 {
        e.storage().instance().get(&DataKey::Decimals).unwrap()
    }

    fn resolution(_e: Env) -> u32 {
        RESOLUTION_S
    }

    fn price(_e: Env, _asset: Asset, _timestamp: u64) -> Option<PriceData> {
        None
    }

    fn prices(_e: Env, _asset: Asset, _records: u32) -> Option<Vec<PriceData>> {
        None
    }

    /// The latest round, or `None` for an unknown asset, a failing or frozen feed, or a non-positive answer.
    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        extend_instance(&e);
        let Asset::Stellar(token) = asset else {
            return None;
        };
        let data_id = Self::feeds(e.clone()).get(token)?;
        let client = DataFeedsProxyClient::new(&e, &Self::proxy(e.clone()));
        let Ok(Ok(round)) = client.try_latest_round(&data_id, &Self::decimals(e.clone())) else {
            return None;
        };
        let price = round.answer.to_i128()?;
        (price > 0).then_some(PriceData { price, timestamp: round.timestamp })
    }
}

fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(40 * LEDGERS_PER_DAY, 41 * LEDGERS_PER_DAY);
}
