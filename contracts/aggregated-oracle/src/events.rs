use sep_40_oracle::{Asset, PriceData};
use soroban_sdk::{Address, contractevent};

use crate::storage::AssetData;

#[contractevent]
pub struct AllOraclesUnawareOfPrice {
    #[topic]
    pub token_address: Address,
}

#[contractevent]
pub struct OracleUnawareOfAssetVariant {
    #[topic]
    pub asset: Asset,
    #[topic]
    pub oracle_address: Address,
    #[topic]
    pub token_address: Address,
}

#[contractevent]
pub struct OracleUnawareOfPrice {
    #[topic]
    pub oracle_address: Address,
    #[topic]
    pub token_address: Address,
}

#[contractevent]
pub struct InvalidOraclePriceTimestamp {
    #[topic]
    pub oracle_address: Address,
    #[topic]
    pub token_address: Address,
    #[topic]
    pub asset: Asset,
    pub price_data: PriceData,
    pub max_age: u64,
}

#[contractevent]
pub struct NonPositiveOraclePrice {
    #[topic]
    pub token_address: Address,
    #[topic]
    pub oracle_address: Address,
    pub price_data: PriceData,
}

#[contractevent]
pub struct AssetIsNotRegistered {
    #[topic]
    pub token_address: Address,
}

#[contractevent]
pub struct PriceDeviationExceedsMax {
    #[topic]
    pub asset_data: AssetData,
    pub cached_price_data: PriceData,
    pub new_price_data: PriceData,
}
