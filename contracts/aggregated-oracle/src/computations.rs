use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_sdk::{Address, Env, Map, Vec};

use crate::storage::{self, OracleConfig};

/// Computes the median of `lastprice().price` received from the oracles. In the case of a specific
/// oracle is not aware of the price, it doesn't get included in the computation
pub fn compute_median(e: &Env, token_address: &Address) -> Option<i128> {
    let prices = get_last_prices(e, token_address);

    if prices.is_empty() {
        let topics = ("None of the oracles is aware of the recent price",);
        let data = (token_address,);

        e.events().publish(topics, data);

        return None;
    }

    let sorted_prices = tree_sort(e, prices);

    let n = sorted_prices.len();
    let median = if n.is_multiple_of(2) {
        let left = sorted_prices.get(n / 2).unwrap().0; // safe
        let right = sorted_prices.get((n / 2) - 1).unwrap().0; // safe

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

fn get_last_price(e: &Env, token_address: &Address, oracle_config: &OracleConfig) -> Option<i128> {
    let current_timestamp = e.ledger().timestamp();
    let OracleConfig {
        address: oracle_address,
        decimals: oracle_decimals,
        resolution,
        lastprices_cached,
        is_stellar_data_based,
    } = oracle_config;

    if let Some(lastprice) = lastprices_cached.get(token_address.clone())
        && lastprice.timestamp + (*resolution as u64) > current_timestamp
    {
        // No need to fetch the price if it hasn't been updated
        return Some(lastprice.price);
    }

    let asset = if *is_stellar_data_based {
        Asset::Stellar(token_address.clone())
    } else {
        let token_ticker = storage::get_token_ticker(e, token_address);

        Asset::Other(token_ticker)
    };

    let oracle_client = PriceFeedClient::new(e, oracle_address);
    let price_data = oracle_client.lastprice(&asset);

    let price_data = if let Some(price_data) = price_data {
        price_data
    } else {
        {
            // NB: It's rather unexpected not to obtain a price from one of the protocol's oracles
            // in the first try, as well as the second try
            let topics = ("Oracle isn't aware of the asset variant",);
            let data = (asset.clone(), token_address.clone(), oracle_address.clone());

            e.events().publish(topics, data);
        }

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
                let topics = ("Oracle is completely unaware of the asset's price",);
                let data = (); // No need to publish context data, since it's already published in the prior event

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
