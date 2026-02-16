use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_sdk::{Address, Env, Map, Vec};

use crate::{
    events,
    storage::{self, OracleConfig},
};

/// Computes the median of `lastprice().price` received from the oracles.
/// In the case of a specific oracle that is not aware of the price, its price doesn't get included in the computation
pub fn compute_median(e: &Env, token_address: &Address) -> Option<i128> {
    let prices = get_last_prices(e, token_address);

    if prices.is_empty() {
        events::AllOraclesUnawareOfPrice { token_address: token_address.clone() }.publish(e);

        return None;
    }

    let sorted_prices = tree_sort(e, prices);

    let n = sorted_prices.len();
    let median = if n.is_multiple_of(2) {
        let left = sorted_prices.get((n / 2) - 1).unwrap().0; // safe
        let right = sorted_prices.get(n / 2).unwrap().0; // safe

        left.checked_add(right)?.checked_div(2)?
    } else {
        sorted_prices.get(n / 2).unwrap().0 // safe
    };

    Some(median)
}

/// Sorts prices via the Tree sort algorithm, handling duplicates by using a counter in the key
fn tree_sort(e: &Env, vec: Vec<i128>) -> Vec<(i128, u32)> {
    // See: <https://en.wikipedia.org/wiki/Tree_sort>
    // See: <https://docs.rs/soroban-sdk/latest/soroban_sdk/struct.Map.html>
    let mut vec_map: Map<(i128, u32), ()> = Map::new(e);
    for v in vec {
        let mut cnt: u32 = 0;

        while vec_map.contains_key((v, cnt)) {
            cnt += 1;
        }

        vec_map.set((v, cnt), ());
    }

    vec_map.keys()
}

/// Gets `lastprice().price` data from the protocol's oracles. If a `lastprice().price` call for a
/// specific oracle doesn't return the price, or the price's timestamp is incorrect/outdated, it
/// doesn't get included in the resulting list
pub fn get_last_prices(e: &Env, token_address: &Address) -> Vec<i128> {
    let mut prices = Vec::new(e);

    for oracle_config in storage::get_oracles(e) {
        if let Some(price) = get_last_price(e, token_address, &oracle_config) {
            prices.push_back(price);
        }
    }

    prices
}

/// # Returns
///
/// Oracle's `lastprice.price` from the cache, or,
/// if cache is expired or not present, takes the data from `lastprice()` oracle's endpoint
/// and updates the cache with it
fn get_last_price(e: &Env, token_address: &Address, oracle_config: &OracleConfig) -> Option<i128> {
    let current_timestamp = e.ledger().timestamp();
    let mut oracle_cache =
        storage::get_oracle_price_data_cache(e, &oracle_config.address).unwrap_or(Map::new(e));

    if let Some(lastprice) = oracle_cache.get(token_address.clone())
        && lastprice.timestamp + (oracle_config.resolution as u64) > current_timestamp
    {
        // No need to fetch the price if it hasn't been updated
        return Some(lastprice.price);
    }

    let asset = if oracle_config.is_stellar_data_based {
        Asset::Stellar(token_address.clone())
    } else {
        let token_ticker = storage::get_token_ticker(e, token_address);

        Asset::Other(token_ticker)
    };

    let oracle_client = PriceFeedClient::new(e, &oracle_config.address);
    let price_data = oracle_client.lastprice(&asset);

    let mut price_data = if let Some(price_data) = price_data {
        price_data
    } else {
        // NB: It's rather unexpected not to obtain a price from one of the protocol's oracles
        // in the first try. The same holds for the second try
        events::OracleUnawareOfAssetVariant {
            asset: asset.clone(),
            oracle_address: oracle_config.address.clone(),
            token_address: token_address.clone(),
        }
        .publish(e);

        // NB: It might be possible that an oracle contains information about the asset's price as
        // another [`Asset`] variant
        let another_variant_asset = match &asset {
            Asset::Other(_symbol) => Asset::Stellar(token_address.clone()),
            Asset::Stellar(token_address) => {
                let token_ticker = storage::get_token_ticker(e, token_address);

                Asset::Other(token_ticker)
            }
        };

        let Some(price_data) = oracle_client.lastprice(&another_variant_asset) else {
            {
                events::OracleUnawareOfPrice {
                    oracle_address: oracle_config.address.clone(),
                    token_address: token_address.clone(),
                }
                .publish(e);
            }

            return None;
        };

        price_data
    };

    let max_age = storage::get_max_age(e);
    if current_timestamp < price_data.timestamp
        || (current_timestamp - price_data.timestamp) > max_age
    {
        events::InvalidOraclePriceTimestamp {
            asset,
            max_age,
            price_data,
            token_address: token_address.clone(),
            oracle_address: oracle_config.address.clone(),
        }
        .publish(e);

        None
    } else {
        if price_data.price <= 0 {
            events::NonPositiveOraclePrice {
                token_address: token_address.clone(),
                oracle_address: oracle_config.address.clone(),
                price_data: price_data.clone(),
            }
            .publish(e);

            return None;
        }

        let protocol_decimals = storage::get_decimals(e);
        let normalized_price =
            normalize_price(price_data.price, oracle_config.decimals, protocol_decimals)?;
        price_data.price = normalized_price;

        // Update the cache
        // TODO: Create a cache per (oracle_address, asset_address) pair?
        oracle_cache.set(token_address.clone(), price_data);
        storage::set_oracle_price_data_cache(e, &oracle_config.address, &oracle_cache);

        Some(normalized_price)
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

#[cfg(test)]
mod tests {
    use soroban_sdk::{Env, Vec, vec as svec};

    use super::tree_sort;

    #[test]
    fn test_tree_sort_odd_no_duplicates() {
        let e = Env::default();
        let prices = svec![&e, 100, 300, 200];

        let sorted = tree_sort(&e, prices);
        let expected = svec![&e, (100, 0), (200, 0), (300, 0)];

        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_tree_sort_even_no_duplicates() {
        let e = Env::default();
        let prices = svec![&e, 100, 400, 200, 300];

        let sorted = tree_sort(&e, prices);
        let expected = svec![&e, (100, 0), (200, 0), (300, 0), (400, 0)];

        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_tree_sort_odd_with_duplicates() {
        let e = Env::default();
        let prices = svec![&e, 100, 200, 100];

        let sorted = tree_sort(&e, prices);
        let expected = svec![&e, (100, 0), (100, 1), (200, 0)];

        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_tree_sort_even_with_duplicates() {
        let e = Env::default();
        let prices = svec![&e, 100, 300, 100, 200];

        let sorted = tree_sort(&e, prices);
        let expected = svec![&e, (100, 0), (100, 1), (200, 0), (300, 0)];

        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_tree_sort_single_price() {
        let e = Env::default();
        let prices = svec![&e, 500];

        let sorted = tree_sort(&e, prices);
        let expected = svec![&e, (500, 0)];

        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_tree_sort_empty_prices() {
        let e = Env::default();
        let prices = svec![&e];

        let sorted = tree_sort(&e, prices);
        let expected = Vec::new(&e);

        assert_eq!(sorted, expected);
    }
}
