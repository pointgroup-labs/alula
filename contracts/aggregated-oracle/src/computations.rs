use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_sdk::{Address, Env, Map, Vec};

use crate::storage::{self, OracleConfig};

/// Computes the median of `lastprices` received from the oracles. In the case of a specific oracle
/// not aware of the price it doesn't get included in computation
pub fn compute_median(e: &Env, token_address: &Address) -> Option<i128> {
    let prices = get_lastprices(e, token_address);

    if prices.is_empty() {
        let topics = ("None of the oracles are aware of the recent price",);
        let data = (token_address,);

        e.events().publish(topics, data);

        return None;
    }

    // Sorting prices via the Tree sort algorithm.
    // See: <https://en.wikipedia.org/wiki/Tree_sort>
    // See: <https://docs.rs/soroban-sdk/latest/soroban_sdk/struct.Map.html>
    let mut prices_map: Map<i128, ()> = Map::new(e);
    for price in prices {
        prices_map.set(price, ());
    }
    let sorted_prices = prices_map.keys();

    let n = sorted_prices.len();
    let median = if n % 2 == 0 {
        let left = sorted_prices.get(n / 2).unwrap(); // safe
        let right = sorted_prices.get((n / 2) + 1).unwrap(); // safe

        let mean = left.checked_add(right)?.checked_div(2)?;

        mean
    } else {
        sorted_prices.get(n / 2).unwrap() // safe
    };

    Some(median)
}

/// Gets `lastprice()` data from the protocol's oracles. If a `lastprice()` call for a specific
/// oracle doesn't return the price, or the price's timestamp is incorrect/outdated, it doesn't get
/// included in the resulting list
pub fn get_lastprices(e: &Env, token_address: &Address) -> Vec<i128> {
    let mut prices = Vec::new(e);

    for oracle_config in storage::get_oracles(e) {
        if let Some(price) = get_lastprice(e, token_address, &oracle_config) {
            prices.push_back(price);
        }
    }

    prices
}

fn get_lastprice(e: &Env, token_address: &Address, oracle_config: &OracleConfig) -> Option<i128> {
    let current_timestamp = e.ledger().timestamp();
    let OracleConfig {
        address: oracle_address,
        decimals: oracle_decimals,
        resolution,
        last_twap_price_data,
        is_stellar_data_based,
    } = oracle_config;

    if last_twap_price_data.timestamp + (*resolution as u64) > current_timestamp {
        // No need to fetch the price if it hasn't been updated
        return None;
    }

    let asset = if *is_stellar_data_based {
        Asset::Stellar(token_address.clone())
    } else {
        let token_ticker = storage::get_token_ticker(e, token_address);

        Asset::Other(token_ticker)
    };

    let oracle_client = PriceFeedClient::new(e, &oracle_address);
    let price_data = oracle_client.lastprice(&asset);

    let price_data = if let Some(price_data) = price_data {
        price_data
    } else {
        {
            // NB: Not obtaining a price from a protocol's oracle is unexpected
            let topics = ("Oracle isn't aware of the asset variant",);
            let data = (asset.clone(), token_address.clone(), oracle_address.clone());

            e.events().publish(topics, data);
        }

        // NB: It might be possible that oracle contains information about the asset's price as
        // another `[Asset]` variant
        let another_variant_asset = match &asset {
            Asset::Other(_symbol) => Asset::Stellar(token_address.clone()),
            Asset::Stellar(token_address) => {
                let token_ticker = storage::get_token_ticker(e, &token_address);

                Asset::Other(token_ticker)
            }
        };

        let Some(price_data) = oracle_client.lastprice(&another_variant_asset) else {
            {
                let topics = ("Oracle is fully unaware of the asset's price",);
                let data = ();

                e.events().publish(topics, data);
            }

            return None;
        };

        price_data
    };

    let max_age = storage::get_max_age(e);

    if current_timestamp < price_data.timestamp
        || (current_timestamp - price_data.timestamp) > max_age
    {
        let topics = ("Oracle price's timestamp is invalid",);
        let data = (
            asset,
            oracle_address.clone(),
            token_address.clone(),
            price_data,
            max_age,
        );

        e.events().publish(topics, data);

        None
    } else {
        let protocol_decimals = storage::get_decimals(e);

        normalize_price(price_data.price, *oracle_decimals, protocol_decimals)
    }
}

/// Normalizes oracle's price to match the protocol's `decimal` configuration
fn normalize_price(price: i128, oracle_decimals: u32, protocol_decimals: u32) -> Option<i128> {
    if oracle_decimals >= protocol_decimals {
        // Oracle has more or equal decimals, so we need to divide to scale down
        let diff = oracle_decimals - protocol_decimals;
        let magnitude = i128::checked_pow(10, diff)?;

        price.checked_div(magnitude)
    } else {
        // Oracle has fewer decimals, so we need to multiply to scale up
        let diff = protocol_decimals - oracle_decimals;
        let magnitude = i128::checked_pow(10, diff)?;

        price.checked_mul(magnitude)
    }
}
