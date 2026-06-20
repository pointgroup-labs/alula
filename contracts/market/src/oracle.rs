// use aggregated_oracle::PriceFeedClient;
use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{Address, Env, I256};

use crate::{
    constants::*,
    error::MCError,
    math_utils::MathUtils,
    storage::{self},
};

// Fetches the latest price for a given asset from the oracle contract.
// Validates that the price is non-negative and not stale.
// The price is expected to be in the format defined by the oracle (e.g., scaled by 10^decimals).
//
// # Arguments
// * `e` - The Soroban environment.
// * `token_address` - The address of the token/asset for which the price is requested.
//
// # Returns
// * `Ok(i128)` - The latest price of the asset if available and valid.
// * `Err(MCError)` - An error if the price is not available, stale
pub fn get_asset_price(e: &Env, token_address: &Address) -> Result<i128, MCError> {
    let current_timestamp = e.ledger().timestamp();

    if let Some((price, cached_timestamp)) = storage::get_cached_price(e, token_address)
        && cached_timestamp == current_timestamp {
            return Ok(price);
        }

    let oracle = storage::get_oracle(e);
    let oracle_contract = PriceFeedClient::new(e, &oracle);

    let asset = Asset::Stellar(token_address.clone());

    let price_data =
        oracle_contract.lastprice(&asset).ok_or(MCError::OracleDoesNotKnowAssetPrice)?;
    if price_data.price <= 0 {
        return Err(MCError::NonPositiveOraclePrice);
    }

    // Validate price is not too old and not from the future
    let now = e.ledger().timestamp();
    let age = now.saturating_sub(price_data.timestamp);
    if age > MAX_ORACLE_PRICE_AGE_SECONDS || price_data.timestamp > now {
        return Err(MCError::OracleStalePrice);
    }

    storage::set_cached_price(e, token_address, price_data.price, current_timestamp);

    Ok(price_data.price)
}

// -- Amount => Value conversions --

/// Returns the value of a given asset amount, scaled by the oracle's price decimals, floored
///
/// The calculation performed is: (amount * price) / 10^(token_decimals)
///
/// # Arguments
/// * `e` - The Soroban environment.
/// * `amount` - The amount of the asset, scaled by 10^(token_decimals)
/// * `token_address` - The address of the token/asset
/// * `token_decimals` - The decimal precision of the token
///
/// # Returns
/// * `Ok(i128)` - The calculated value of the asset amount, scaled by the oracle's decimals
/// * `Err(MCError)` - An error if the price is unavailable, stale, or if a math overflow occurs
pub fn get_asset_value_floor(
    e: &Env,
    amount: i128,
    token_address: &Address,
    token_decimals: u32,
) -> Result<i128, MCError> {
    let price = I256::from_i128(e, get_asset_price(e, token_address)?);
    let amount = I256::from_i128(e, amount);

    let numerator = price.mul(&amount);
    let denominator = I256::from_i128(e, 10_i128.pow(token_decimals));

    let value = numerator.div(&denominator);

    value.to_i128().map_over_or_underflow()
}

/// Returns the value of a given asset amount, scaled by the oracle's price decimals, ceiled
///
/// The calculation performed is: (amount * price) / 10^(token_decimals)
///
/// # Arguments
/// * `e` - The Soroban environment.
/// * `amount` - The amount of the asset, scaled by 10^(token_decimals)
/// * `token_address` - The address of the token/asset
/// * `token_decimals` - The decimal precision of the token
///
/// # Returns
/// * `Ok(i128)` - The calculated value of the asset amount, scaled by the oracle's decimals
/// * `Err(MCError)` - An error if the price is unavailable, stale, or if a math overflow occurs
pub fn get_asset_value_ceil(
    e: &Env,
    amount: i128,
    token_address: &Address,
    token_decimals: u32,
) -> Result<i128, MCError> {
    let price = I256::from_i128(e, get_asset_price(e, token_address)?);
    let amount = I256::from_i128(e, amount);

    let numerator = price.mul(&amount);
    let denominator = I256::from_i128(e, 10_i128.pow(token_decimals));

    let value = numerator.fixed_div_ceil(e, &denominator, &I256::from_i128(e, 1));

    value.to_i128().map_over_or_underflow()
}

// -- Value => Amount conversions --

/// Returns the amount of a given asset for a specific value, floored
///
/// The calculation performed is: (value * 10^token_decimals) / price
///
/// # Arguments
/// * `e` - The Soroban environment
/// * `value` - The value of the asset, scaled by the oracle's price decimals
/// * `token_address` - The address of the token/asset
/// * `token_decimals` - The decimal precision of the token
///
/// # Returns
/// * `Ok(i128)` - The calculated amount of the asset, scaled by 10^(token_decimals)
/// * `Err(MCError)` - An error if the price is unavailable, stale, or if a math overflow occurs
pub fn get_asset_amount_floor(
    e: &Env,
    value: i128,
    token_address: &Address,
    token_decimals: u32,
) -> Result<i128, MCError> {
    let price = I256::from_i128(e, get_asset_price(e, token_address)?);
    let value_i256 = I256::from_i128(e, value);

    let token_scale = I256::from_i128(e, 10_i128.pow(token_decimals));
    let numerator = value_i256.mul(&token_scale);

    let amount = numerator.div(&price);

    amount.to_i128().map_over_or_underflow()
}

/// Retrieves the decimal precision used by the oracle for price values
///
/// # Arguments
/// * `e` - The Soroban environment
///
/// # Returns
/// * `u32` - The number of decimal places used by the price feed (e.g., 7, 9, or 18)
///
/// # Panics
/// * Panics if the oracle contract call fails, as the protocol cannot safely
///   perform valuations without knowing the price scale
pub fn get_oracle_price_decimals(e: &Env) -> u32 {
    let oracle_address = storage::get_oracle(e);
    let oracle_contract = PriceFeedClient::new(e, &oracle_address);

    oracle_contract.decimals()
}
