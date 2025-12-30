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
