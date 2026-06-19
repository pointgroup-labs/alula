use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_sdk::{Address, Env};

use crate::{error::FCError, state::OracleConfig};

pub fn get_oracle_price(e: &Env, token: &Address, config: &OracleConfig) -> Result<i128, FCError> {
    let client = PriceFeedClient::new(e, &config.oracle_address);
    let asset = Asset::Stellar(token.clone());
    let price_data = client.lastprice(&asset).ok_or(FCError::OraclePriceUnavailable)?;

    let now = e.ledger().timestamp();
    let age = now.saturating_sub(price_data.timestamp);
    if age > config.oracle_max_age || price_data.timestamp > now {
        return Err(FCError::OraclePriceStale);
    }

    if price_data.price <= 0 {
        return Err(FCError::OraclePriceUnavailable);
    }

    Ok(price_data.price)
}

pub fn get_oracle_decimals(e: &Env, config: &OracleConfig) -> Result<u32, FCError> {
    let decimals = PriceFeedClient::new(e, &config.oracle_address).decimals();
    if decimals > 18 {
        return Err(FCError::OraclePriceUnavailable);
    }
    Ok(decimals)
}
