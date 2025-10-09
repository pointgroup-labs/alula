// use aggregated_oracle::PriceFeedClient;
use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_sdk::{Address, Env};

use crate::{
    constants::*,
    error::MCError,
    helpers::require_nonnegative,
    storage::{self},
};

/// Fetches the latest price for a given asset from the oracle contract.
/// Validates that the price is non-negative and not stale.
/// The price is expected to be in the format defined by the oracle (e.g., scaled by 10^decimals).
///
/// # Arguments
/// * `e` - The Soroban environment.
/// * `token_address` - The address of the token/asset for which the price is requested.
///
/// # Returns
/// * `Ok(i128)` - The latest price of the asset if available and valid.
/// * `Err(MCError)` - An error if the price is not available, stale
pub fn get_asset_price(e: &Env, token_address: &Address) -> Result<i128, MCError> {
    let oracle_address = storage::get_oracle_address(e);
    let oracle_contract = PriceFeedClient::new(e, &oracle_address);

    let asset = Asset::Stellar(token_address.clone());

    let price_data =
        oracle_contract.lastprice(&asset).ok_or(MCError::OracleDoesNotKnowAssetPrice)?;

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
