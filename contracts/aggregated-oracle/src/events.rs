use sep_40_oracle::{Asset, PriceData};
use soroban_sdk::{Address, Env, String, Symbol, contractevent};

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
pub struct OraclePriceTimestampInvalid {
    #[topic]
    pub oracle_address: Address,
    #[topic]
    pub token_address: Address,
    pub price_data: PriceData,
    pub max_age: u64,
}

#[contractevent]
pub struct AssetIsNotRegistered {
    #[topic]
    pub token_address: Address,
}
