use sep_40_oracle::{Asset, PriceData, PriceFeedTrait};
use soroban_sdk::{Env, Map, Symbol, Vec, contract, contractimpl, contracttype};

#[contracttype]
pub enum DataKey {
    BaseAsset,
    Prices,
    Resolution,
    Decimals,
}

#[contract]
pub struct MockOracleContract;

#[contractimpl]
impl MockOracleContract {
    pub fn __constructor(e: Env, decimals: u32, resolution: u32, base_asset: Asset) {
        e.storage().instance().set(&DataKey::BaseAsset, &base_asset);
        e.storage().instance().set(&DataKey::Decimals, &decimals);
        e.storage().instance().set(&DataKey::Resolution, &resolution);
    }

    pub fn set_price(e: Env, asset: Asset, price: i128, timestamp: u64) {
        let mut prices_map: Map<Asset, PriceData> =
            e.storage().instance().get(&DataKey::Prices).unwrap_or_else(|| Map::new(&e));

        let price_data = PriceData { price, timestamp };
        prices_map.set(asset, price_data);

        e.storage().instance().set(&DataKey::Prices, &prices_map);
    }
}

#[contractimpl]
impl PriceFeedTrait for MockOracleContract {
    fn base(e: Env) -> Asset {
        e.storage().instance().get(&DataKey::BaseAsset).unwrap()
    }

    fn assets(e: Env) -> Vec<Asset> {
        let prices_map: Map<Asset, PriceData> =
            e.storage().instance().get(&DataKey::Prices).unwrap_or_else(|| Map::new(&e));

        prices_map.keys()
    }

    fn decimals(e: Env) -> u32 {
        e.storage().instance().get(&DataKey::Decimals).unwrap()
    }

    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        let prices_map: Map<Asset, PriceData> =
            e.storage().instance().get(&DataKey::Prices).unwrap_or_else(|| Map::new(&e));

        prices_map.get(asset)
    }

    fn resolution(e: Env) -> u32 {
        e.storage().instance().get(&DataKey::Resolution).unwrap()
    }

    fn price(_e: Env, _asset: Asset, _timestamp: u64) -> Option<PriceData> {
        unimplemented!()
    }

    fn prices(_e: Env, _asset: Asset, _records: u32) -> Option<Vec<PriceData>> {
        unimplemented!()
    }
}

// ---------------------------------------------------------------------------
// MockPushOracleContract — simulates a Push/heartbeat oracle (e.g. Redstone).
//
// Key behavioural difference from MockOracleContract:
//   - `lastprice` returns the real *feed signing timestamp* stored at
//     `set_price` time — it does NOT update to ledger time on each call.
//   - The internal staleness guard mirrors the adapter's own check:
//     returns `None` if `ledger_now - feed_ts > resolution`, so the
//     aggregated oracle's push-oracle branch (which skips the cache and
//     re-fetches on every call) sees a `None` when the feed is stale.
// ---------------------------------------------------------------------------

#[contracttype]
pub enum PushDataKey {
    BaseAsset,
    Price,
    Resolution,
    Decimals,
}

#[contract]
pub struct MockPushOracleContract;

#[contractimpl]
impl MockPushOracleContract {
    pub fn __constructor(e: Env, decimals: u32, resolution: u32, base_asset: Asset) {
        e.storage().instance().set(&PushDataKey::BaseAsset, &base_asset);
        e.storage().instance().set(&PushDataKey::Decimals, &decimals);
        e.storage().instance().set(&PushDataKey::Resolution, &resolution);
    }

    /// Sets the price and the signing timestamp (simulates a relayer pushing
    /// a signed Redstone price update on-chain).
    pub fn set_price(e: Env, asset: Asset, price: i128, feed_timestamp: u64) {
        e.storage()
            .instance()
            .set(&PushDataKey::Price, &PriceData { price, timestamp: feed_timestamp });
        // Assets list is not tracked for simplicity — the aggregated oracle
        // only calls `lastprice`, `resolution`, and `decimals`.
        let _ = asset;
    }
}

#[contractimpl]
impl PriceFeedTrait for MockPushOracleContract {
    fn base(e: Env) -> Asset {
        e.storage().instance().get(&PushDataKey::BaseAsset).unwrap()
    }

    fn assets(_e: Env) -> Vec<Asset> {
        unimplemented!()
    }

    fn decimals(e: Env) -> u32 {
        e.storage().instance().get(&PushDataKey::Decimals).unwrap()
    }

    fn resolution(e: Env) -> u32 {
        e.storage().instance().get(&PushDataKey::Resolution).unwrap()
    }

    fn lastprice(e: Env, _asset: Asset) -> Option<PriceData> {
        let price_data: PriceData = e.storage().instance().get(&PushDataKey::Price)?;
        let resolution: u32 = e.storage().instance().get(&PushDataKey::Resolution).unwrap();
        let now = e.ledger().timestamp();

        // Mirror the adapter's own staleness check: reject future timestamps
        // and prices whose signing time is older than the heartbeat window.
        if price_data.timestamp > now || now - price_data.timestamp > resolution as u64 {
            return None;
        }

        Some(price_data)
    }

    fn price(_e: Env, _asset: Asset, _timestamp: u64) -> Option<PriceData> {
        unimplemented!()
    }

    fn prices(_e: Env, _asset: Asset, _records: u32) -> Option<Vec<PriceData>> {
        unimplemented!()
    }
}
