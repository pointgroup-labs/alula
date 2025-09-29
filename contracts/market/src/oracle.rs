// use aggregated_oracle::PriceFeedClient;
use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_sdk::{Address, Env};

use crate::{
    constants::MAX_ORACLE_PRICE_AGE_SECONDS,
    error::MCError,
    helpers::require_nonnegative,
    storage::{self},
};

// TODO: move to oracle module?
pub fn get_asset_price(e: &Env, token_address: &Address) -> Result<i128, MCError> {
    let oracle_address = storage::get_oracle_address(e);
    let oracle_contract = PriceFeedClient::new(e, &oracle_address);

    let asset = Asset::Stellar(token_address.clone());

    let price_data = oracle_contract
        .lastprice(&asset)
        .ok_or(MCError::OracleDoesNotKnowAssetPrice)?;

    // TODO: Add sanity checks? I.e., a price range for the asset to be considered adequate
    require_nonnegative(price_data.price)?;

    // Validate price is not too old and not from the future
    let now = e.ledger().timestamp();
    let age = now.saturating_sub(price_data.timestamp);
    if age > MAX_ORACLE_PRICE_AGE_SECONDS || price_data.timestamp > now {
        return Err(MCError::OracleStalePrice);
    }

    Ok(price_data.price)
}

pub fn get_oracle_price_decimals(e: &Env) -> u32 {
    let oracle_address = storage::get_oracle_address(e);
    let oracle_contract = PriceFeedClient::new(e, &oracle_address);

    oracle_contract.decimals()
}
