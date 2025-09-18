use sep_40_oracle::{Asset, PriceData, PriceFeedClient};
use soroban_sdk::{Address, Env, Map, Vec};

use crate::{
    error::AOCError,
    storage::{self, get_oracles, OracleConfig},
};

pub(crate) fn compute_median(e: &Env, token_address: &Address) -> Option<i128> {
    let prices = get_lastprices(e, token_address).unwrap();
    if prices.is_empty() {
        e.events().publish(
            (asset.clone(), "None of the oracles are aware of the price"),
            (),
        );

        return None;
    }

    let mut twaps_map: Map<u64, i128> = Map::new(e);

    for twap in twaps {
        let PriceData { price, timestamp } = twap;
        twaps_map.set(timestamp, price);
    }

    // See: <https://docs.rs/soroban-sdk/latest/soroban_sdk/struct.Map.html>
    let sorted_twaps = twaps_map.values();
    let n = sorted_twaps.len();

    let median = if n % 2 == 0 {
        let left = sorted_twaps.get(n / 2).unwrap(); // safe
        let right = sorted_twaps.get((n / 2) + 1).unwrap(); // safe
        let mean = left.checked_add(right)?.checked_div(2)?;

        mean
    } else {
        sorted_twaps.get((n / 2) + 1).unwrap() // safe
    };

    Some(median)
}

pub(crate) fn get_lastprices(e: &Env, token_address: &Address) -> Result<Vec<i128>, AOCError> {
    let mut prices = Vec::new(e);
    let current_timestamp = e.ledger().timestamp();

    let oracles_configs = get_oracles(e);
    for oracle_config in oracles_configs {
        let OracleConfig {
            address,
            decimals,
            resolution,
            last_twap_price_data,
            is_stellar_data_based,
        } = oracle_config;

        if last_twap_price_data.timestamp + (resolution as u64) > current_timestamp {
            // No need to fetch the price if it hasn't been updated
            prices.push_back(last_twap_price_data.price);
        } else {
            let oracle_client = PriceFeedClient::new(e, &address);

            let asset = if is_stellar_data_based {
                Asset::Stellar(token_address.clone())
            } else {
                let token_ticker = storage::get_token_ticker(e, token_address);

                Asset::Other(token_ticker)
            };

            let price_data = oracle_client.lastprice(&asset);

            let price_data = if let Some(price_data) = price_data {
                price_data
            } else {
                let other_variant_asset = match asset {
                    Asset::Other(_symbol) => Asset::Stellar(token_address.clone()),
                    Asset::Stellar(token_address) => {
                        let token_ticker = storage::get_token_ticker(e, &token_address);

                        Asset::Other(token_ticker)
                    }
                };

                let Some(price_data) = oracle_client.lastprice(&other_variant_asset) else {
                    // Event?

                    continue;
                };

                price_data
            };

            if price_data.timestamp > current_timestamp {
                // Event?

                continue;
            } else if current_timestamp - price_data.timestamp < storage::get_max_age(e) {
                // Event?

                continue;
            } else {
                // Also update the cache here?

                prices.push_back(price_data.price);
            }
        }
    }

    Ok(prices)
}

pub(crate) fn compute_twap(
    e: &Env,
    asset: &Asset,
    oracle_config: &OracleConfig,
) -> Result<i128, AOCError> {
    todo!()
}
