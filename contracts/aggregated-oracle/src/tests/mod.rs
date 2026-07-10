#![allow(unused)]

use sep_40_oracle::{Asset, PriceData};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, Env, FromVal, IntoVal, Map, Symbol, Vec as SVec, symbol_short,
    testutils::{Address as _, Events, Ledger},
    vec as svec,
};

use crate::{
    constants::{BPS_FACTOR, PERIODIC_UPDATE_GRACE_PERIOD},
    contract::{AggregatedOracleContract, AggregatedOracleContractClient},
    storage::{DataKey, OracleConfigInput},
    tests::mock_oracle::{
        MockOracleContract, MockOracleContractClient, MockPushOracleContract,
        MockPushOracleContractClient,
    },
};

extern crate std;
use std::vec::Vec;

#[test]
fn test_median_price_with_odd_number_of_reported_prices() {
    let TestFixture { e, oracle_clients, oracle_config_inputs, contract_client, .. } =
        TestFixture::new();

    // Set XLM prices on the mock oracles: [100, 200, 300]
    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");

    let xlm_asset_other = Asset::Other(xlm_ticker.clone());
    let xlm_asset_stellar = Asset::Stellar(xlm_address.clone());

    for (idx, (oracle_client, oracle_config_input)) in
        oracle_clients.iter().zip(oracle_config_inputs.iter()).enumerate()
    {
        let asset = if oracle_config_input.is_stellar_data_based {
            xlm_asset_stellar.clone()
        } else {
            xlm_asset_other.clone()
        };
        // Oracles report:
        // 100, 200 and 300
        let price = 100 * (idx as i128 + 1) * i128::pow(10, ORACLES_DECIMALS);
        let timestamp = 1_000_000_600;

        oracle_client.set_price(&asset, &price, &timestamp);
    }

    contract_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    let lastprice = contract_client.lastprice(&xlm_asset_stellar).unwrap();

    assert_eq!(lastprice.price, 200 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS));
    // The timestamp should be the oldest source timestamp, not the current ledger time
    assert_eq!(lastprice.timestamp, 1_000_000_600);
}

#[test]
fn test_median_price_with_even_number_of_reported_prices() {
    let TestFixture { e, oracle_clients, oracle_config_inputs, contract_client, .. } =
        TestFixture::new();

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");

    let xlm_asset_other = Asset::Other(xlm_ticker.clone());
    let xlm_asset_stellar = Asset::Stellar(xlm_address.clone());

    for (idx, (oracle_client, oracle_config_input)) in
        oracle_clients.iter().skip(1).zip(oracle_config_inputs.iter().skip(1)).enumerate()
    {
        let asset = if oracle_config_input.is_stellar_data_based {
            xlm_asset_stellar.clone()
        } else {
            xlm_asset_other.clone()
        };
        let price = 100 * (idx as i128 + 1) * i128::pow(10, ORACLES_DECIMALS);
        let allowed_timestamp = (e.ledger().timestamp() - AGGREGATED_ORACLE_MAX_AGE);

        oracle_client.set_price(&asset, &price, &allowed_timestamp);
    }

    let asset = if oracle_config_inputs.get(0).unwrap().is_stellar_data_based {
        xlm_asset_stellar.clone()
    } else {
        xlm_asset_other.clone()
    };
    let new_price = 200 * i128::pow(10, ORACLES_DECIMALS);
    let expired_timestamp = e.ledger().timestamp() - AGGREGATED_ORACLE_MAX_AGE - 1;
    oracle_clients[0].set_price(&asset, &new_price, &expired_timestamp);

    contract_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    let lastprice = contract_client.lastprice(&xlm_asset_stellar).unwrap();

    assert_eq!(
        lastprice.price,
        150 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS) // (100 + 200) / 2
    );
    // The timestamp should be the oldest valid source timestamp
    // In this test, 2 oracles report at (ledger_time - periodic_oracles_price_max_age)
    let expected_timestamp = e.ledger().timestamp() - AGGREGATED_ORACLE_MAX_AGE;
    assert_eq!(lastprice.timestamp, expected_timestamp);
}

#[test]
fn test_median_price_with_all_expired_prices() {
    let TestFixture { e, oracle_clients, contract_client, oracle_config_inputs, .. } =
        TestFixture::new();

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");

    let xlm_asset_other = Asset::Other(xlm_ticker.clone());
    let xlm_asset_stellar = Asset::Stellar(xlm_address.clone());

    // Set prices for all oracles to be expired
    for (idx, (oracle_client, oracle_config_input)) in
        oracle_clients.iter().zip(oracle_config_inputs).enumerate()
    {
        let asset = if oracle_config_input.is_stellar_data_based {
            xlm_asset_stellar.clone()
        } else {
            xlm_asset_other.clone()
        };
        let price = 100 * (idx as i128 + 1) * i128::pow(10, ORACLES_DECIMALS);
        let expired_timestamp = e.ledger().timestamp() - AGGREGATED_ORACLE_MAX_AGE - 1;

        oracle_client.set_price(&asset, &price, &expired_timestamp);
    }
    contract_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    let lastprice = contract_client.lastprice(&xlm_asset_stellar);

    assert!(lastprice.is_none());
}

#[test]
fn test_max_deviation_check() {
    const MAX_DEV_BPS: u32 = 100; // 10%
    const MAX_DEV_CONSECUTIVE_DIFF_SECS: u64 = 10000; // NB: Must exceed the resolution on the oracles to notice the effect

    let TestFixture {
        e, oracle_clients, contract_client, oracle_config_inputs, contract_id, ..
    } = TestFixture::new();

    // -- Set prices on oracles --

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");

    let xlm_asset_other = Asset::Other(xlm_ticker.clone());
    let xlm_asset_stellar = Asset::Stellar(xlm_address.clone());

    contract_client.add_asset(
        &xlm_ticker,
        &xlm_address,
        &MAX_DEV_BPS,
        &MAX_DEV_CONSECUTIVE_DIFF_SECS,
    );

    for (idx, (oracle_client, oracle_config_input)) in
        oracle_clients.iter().zip(oracle_config_inputs.iter()).enumerate()
    {
        let asset = if oracle_config_input.is_stellar_data_based {
            xlm_asset_stellar.clone()
        } else {
            xlm_asset_other.clone()
        };
        let price = 100 * (idx as i128 + 1) * i128::pow(10, ORACLES_DECIMALS);
        let timestamp = e.ledger().timestamp();

        oracle_client.set_price(&asset, &price, &timestamp);
    }

    // -- Check that the price is calculated when no previous lastprice is recorded in the contract's storage --

    e.as_contract(&contract_id, || {
        assert!(
            e.storage()
                .instance()
                .get::<DataKey, PriceData>(&DataKey::PreviousMedianLastprice(xlm_address.clone()))
                .is_none()
        );

        // No oracle cache must exist as well
        for oracle_config_input in &oracle_config_inputs {
            let address = oracle_config_input.address;

            assert!(
                e.storage()
                    .instance()
                    .get::<DataKey, Map<Address, PriceData>>(&DataKey::OraclePriceDataCached(
                        address.clone()
                    ))
                    .is_none()
            );
        }
    });

    let lastprice_1 = contract_client.lastprice(&xlm_asset_stellar).unwrap();

    assert_eq!(lastprice_1.price, 200 * 10_i128.pow(AGGREGATED_ORACLE_DECIMALS));
    assert_eq!(lastprice_1.timestamp, e.ledger().timestamp());

    // -- Verify that the previous lastprice is set --

    e.as_contract(&contract_id, || {
        let previous_median = e
            .storage()
            .instance()
            .get::<DataKey, PriceData>(&DataKey::PreviousMedianLastprice(xlm_address))
            .unwrap();

        assert_eq!(previous_median.price, lastprice_1.price);
        assert_eq!(previous_median.timestamp, lastprice_1.timestamp);

        for oracle_config_input in &oracle_config_inputs {
            let address = oracle_config_input.address;

            assert!(
                e.storage()
                    .instance()
                    .get::<DataKey, Map<Address, PriceData>>(&DataKey::OraclePriceDataCached(
                        address.clone()
                    ))
                    .is_some()
            );
        }
    });

    // -- Move time --

    e.ledger().with_mut(|li| li.timestamp += ORACLES_RESOLUTION as u64);

    // -- Set prices that exceed the max allowed deviation --

    for (idx, (oracle_client, oracle_config_input)) in
        oracle_clients.iter().zip(oracle_config_inputs.iter()).enumerate()
    {
        let asset = if oracle_config_input.is_stellar_data_based {
            xlm_asset_stellar.clone()
        } else {
            xlm_asset_other.clone()
        };

        let price = 100 * (idx as i128 + 1) * i128::pow(10, ORACLES_DECIMALS);
        let price_with_deviation =
            price + price.fixed_mul_ceil(MAX_DEV_BPS as i128, BPS_FACTOR).unwrap() + 1;
        let timestamp = e.ledger().timestamp();

        oracle_client.set_price(&asset, &price_with_deviation, &timestamp);
    }

    // -- Verify that deviation check has failed --

    assert!(e.events().all().is_empty());

    assert!(contract_client.lastprice(&xlm_asset_stellar).is_none());

    let events = e.events().all();
    let (contract_address, topics, _data) = events.get(0).unwrap();
    assert_eq!(contract_address, contract_id);
    let first_topic_symbol = Symbol::from_val(&e, &topics.get(0).unwrap());
    assert_eq!(first_topic_symbol, Symbol::new(&e, "price_deviation_exceeds_max"));

    // -- Set prices that don't exceed max deviation check --

    e.ledger().with_mut(|li| li.timestamp += ORACLES_RESOLUTION as u64);

    for (idx, (oracle_client, oracle_config_input)) in
        oracle_clients.iter().zip(oracle_config_inputs.iter()).enumerate()
    {
        let asset = if oracle_config_input.is_stellar_data_based {
            xlm_asset_stellar.clone()
        } else {
            xlm_asset_other.clone()
        };

        let price = 100 * (idx as i128 + 1) * i128::pow(10, ORACLES_DECIMALS);
        let price_with_deviation =
            price + price.fixed_mul_ceil(MAX_DEV_BPS as i128, BPS_FACTOR).unwrap();
        let timestamp = e.ledger().timestamp();

        oracle_client.set_price(&asset, &price_with_deviation, &timestamp);
    }

    let lastprice_2 = contract_client.lastprice(&xlm_asset_stellar).unwrap();

    assert_eq!(
        lastprice_1.price
            + lastprice_1.price.fixed_mul_ceil(MAX_DEV_BPS as i128, BPS_FACTOR).unwrap(),
        lastprice_2.price,
    );
    assert_eq!(lastprice_2.timestamp, e.ledger().timestamp());

    // -- Set prices that deviates again --

    e.ledger().with_mut(|li| li.timestamp += ORACLES_RESOLUTION as u64);

    for (idx, (oracle_client, oracle_config_input)) in
        oracle_clients.iter().zip(oracle_config_inputs.iter()).enumerate()
    {
        let asset = if oracle_config_input.is_stellar_data_based {
            xlm_asset_stellar.clone()
        } else {
            xlm_asset_other.clone()
        };

        let obviously_deviated_price = i64::MAX as i128;
        let timestamp = e.ledger().timestamp();

        oracle_client.set_price(&asset, &obviously_deviated_price, &timestamp);
    }

    assert_eq!(e.events().all().len(), 0);

    assert!(contract_client.lastprice(&xlm_asset_stellar).is_none());

    let events = e.events().all();
    let (contract_address, topics, _data) = events.get(0).unwrap();
    assert_eq!(contract_address, contract_id);
    let first_topic_symbol = Symbol::from_val(&e, &topics.get(0).unwrap());
    assert_eq!(first_topic_symbol, Symbol::new(&e, "price_deviation_exceeds_max"));

    // -- Wait for deviation to expire --

    e.ledger().with_mut(|li| {
        li.timestamp += MAX_DEV_CONSECUTIVE_DIFF_SECS - (ORACLES_RESOLUTION as u64) + 1
    });

    // -- Set prices that deviate again --

    for (idx, (oracle_client, oracle_config_input)) in
        oracle_clients.iter().zip(oracle_config_inputs.iter()).enumerate()
    {
        let asset = if oracle_config_input.is_stellar_data_based {
            xlm_asset_stellar.clone()
        } else {
            xlm_asset_other.clone()
        };

        let obviously_deviated_price = i64::MAX as i128;
        let timestamp = e.ledger().timestamp();

        oracle_client.set_price(&asset, &obviously_deviated_price, &timestamp);
    }

    assert!(contract_client.lastprice(&xlm_asset_stellar).is_some());
}

#[test]
fn test_non_positive_oracle_price_is_excluded_from_the_median() {
    const MAX_DEV_BPS: u32 = 100; // 10%
    const MAX_DEV_CONSECUTIVE_DIFF_SECS: u64 = 10000;
    const ORACLE_POSITIVE_PRICE: i128 = 100;

    let TestFixture {
        e, oracle_clients, contract_client, oracle_config_inputs, contract_id, ..
    } = TestFixture::new();

    // -- Set prices on oracles --

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");

    let xlm_asset_other = Asset::Other(xlm_ticker.clone());
    let xlm_asset_stellar = Asset::Stellar(xlm_address.clone());

    contract_client.add_asset(
        &xlm_ticker,
        &xlm_address,
        &MAX_DEV_BPS,
        &MAX_DEV_CONSECUTIVE_DIFF_SECS,
    );

    let get_client_and_asset = |idx| {
        let oracle_asset = if oracle_config_inputs.get(idx).unwrap().is_stellar_data_based {
            xlm_asset_stellar.clone()
        } else {
            xlm_asset_other.clone()
        };

        (&oracle_clients[idx as usize], oracle_asset)
    };

    let (good_oracle_client, good_oracle_asset) = get_client_and_asset(0);
    let (bad_oracle_client, bad_oracle_asset) = get_client_and_asset(1);

    // - Verify zero price is excluded -

    good_oracle_client.set_price(
        &good_oracle_asset,
        &(100 * i128::pow(10, ORACLES_DECIMALS)),
        &e.ledger().timestamp(),
    );
    bad_oracle_client.set_price(&bad_oracle_asset, &0, &e.ledger().timestamp());

    let lastprice_1 = contract_client.lastprice(&xlm_asset_stellar).unwrap();
    assert_eq!(lastprice_1.price, 100 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS));

    // - Verify negative price is excluded -

    bad_oracle_client.set_price(&bad_oracle_asset, &-1, &e.ledger().timestamp());

    let lastprice_1 = contract_client.lastprice(&xlm_asset_stellar).unwrap();
    assert_eq!(lastprice_1.price, 100 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS));

    // - Verify positive prices are included -

    bad_oracle_client.set_price(
        &bad_oracle_asset,
        &(98 * i128::pow(10, ORACLES_DECIMALS)),
        &e.ledger().timestamp(),
    );

    let lastprice_2 = contract_client.lastprice(&xlm_asset_stellar).unwrap();
    assert_eq!(lastprice_2.price, 99 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS));
}

#[test]
fn test_propose_new_admin_and_accept() {
    let TestFixture {
        e,
        oracle_clients,
        contract_client,
        oracle_config_inputs,
        contract_id,
        contract_admin,
        ..
    } = TestFixture::new();
    let proposed_admin = Address::generate(&e);

    e.as_contract(&contract_id, || {
        assert!(e.storage().instance().get::<_, Address>(&DataKey::ProposedAdmin).is_none());
    });

    contract_client.propose_new_admin(&proposed_admin);

    e.as_contract(&contract_id, || {
        let stored_proposed_admin =
            e.storage().instance().get::<_, Address>(&DataKey::ProposedAdmin).unwrap();

        assert_eq!(stored_proposed_admin, proposed_admin);
        assert_ne!(stored_proposed_admin, contract_admin);
    });

    contract_client.accept_proposed_admin();

    e.as_contract(&contract_id, || {
        let contract_admin = e.storage().instance().get::<_, Address>(&DataKey::Admin).unwrap();
        assert_eq!(contract_admin, proposed_admin);

        assert!(e.storage().instance().get::<_, Address>(&DataKey::ProposedAdmin).is_none());
    });
}

// ---- Push-oracle integration tests ----
//
// The tests below exercise the aggregated oracle with a mix of periodic and
// Push/heartbeat oracles.  Key invariants being validated:
//
//   1. A Push oracle whose signing timestamp is within its heartbeat window
//      contributes to the median even when it hasn't updated since the
//      previous aggregated-oracle call.
//   2. A Push oracle whose signing timestamp exceeds `resolution` (i.e. the
//      heartbeat has passed without an update) is excluded from the median.
//   3. `periodic_update_max_age` does NOT apply to Push oracles — a Push oracle
//      whose signing timestamp is older than `max_age` but within `resolution`
//      still contributes.
//   4. The deviation circuit-breaker uses wall-clock time (ledger timestamp of
//      the previous call), not the source-derived median timestamp, so a Push
//      oracle's old signing timestamp does not disable the check.
//   5. The aggregated oracle always re-fetches from Push oracles on every call
//      (no cache hit), so a price update pushed between two calls is immediately
//      reflected in the next median.

const PUSH_ORACLE_RESOLUTION: u32 = 24 * 60 * 60; // 24 h heartbeat
const PUSH_ORACLE_DECIMALS: u32 = ORACLES_DECIMALS;

/// Deploys a mock Push oracle
fn deploy_push_oracle<'a>(
    e: &Env,
    base_asset: &Asset,
) -> (Address, MockPushOracleContractClient<'a>, OracleConfigInput) {
    let address = e.register(
        MockPushOracleContract,
        (PUSH_ORACLE_DECIMALS, PUSH_ORACLE_RESOLUTION, base_asset.clone()),
    );
    let client = MockPushOracleContractClient::new(e, &address);
    let config = OracleConfigInput {
        address: address.clone(),
        is_stellar_data_based: true,
        is_price_update_periodic: false, // Push oracle
    };

    (address, client, config)
}

// ---------------------------------------------------------------------------
// 1. Push oracle within heartbeat window contributes to median
// ---------------------------------------------------------------------------
#[test]
fn test_push_oracle_within_resolution_contributes_to_median() {
    let e = get_default_env();
    // ledger time = 1_000_000_700 (from get_default_env)
    let now = e.ledger().timestamp();
    let base_asset = Asset::Other(Symbol::new(&e, "USD"));

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");
    let xlm_asset = Asset::Stellar(xlm_address.clone());

    // One periodic oracle reporting 100, one push oracle reporting 300.
    // Median of [100, 300] = 200.
    let (_, periodic_client, periodic_config) =
        deploy_mock_oracle(&e, ORACLES_DECIMALS, ORACLES_RESOLUTION, &base_asset, true, true);
    let (_, push_client, push_config) = deploy_push_oracle(&e, &base_asset);

    periodic_client.set_price(&xlm_asset, &(100 * i128::pow(10, ORACLES_DECIMALS)), &now);
    // Push signing timestamp is 23 h in the past — within the 24 h heartbeat.
    let feed_ts = now - 23 * 60 * 60;
    push_client.set_price(&xlm_asset, &(300 * i128::pow(10, PUSH_ORACLE_DECIMALS)), &feed_ts);

    let (_, contract_client) = deploy_aggregated_oracle(
        &e,
        &Address::generate(&e),
        &base_asset,
        AGGREGATED_ORACLE_DECIMALS,
        AGGREGATED_ORACLE_MAX_AGE,
        svec![&e, periodic_config, push_config],
    );
    contract_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    let lastprice = contract_client.lastprice(&xlm_asset).unwrap();
    assert_eq!(lastprice.price, 200 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS));

    // Only periodic-source timestamps feed the published timestamp.
    // Push (e.g. Redstone) signing times are intentionally ignored
    // here because they can sit `resolution` seconds in the past *by
    // design* — blending them in would systematically misrepresent the
    // aggregated median as stale even when every contributor just
    // passed its own per-source freshness check. The periodic oracle
    // reported at `now`, so the aggregated timestamp is `now`.
    assert_eq!(lastprice.timestamp, now);
}

// ---------------------------------------------------------------------------
// 2. Push oracle past heartbeat is excluded; median falls back to periodic only
// ---------------------------------------------------------------------------
#[test]
fn test_push_oracle_past_resolution_is_excluded() {
    let e = get_default_env();
    let now = e.ledger().timestamp();
    let base_asset = Asset::Other(Symbol::new(&e, "USD"));

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");
    let xlm_asset = Asset::Stellar(xlm_address.clone());

    let (_, periodic_client, periodic_config) =
        deploy_mock_oracle(&e, ORACLES_DECIMALS, ORACLES_RESOLUTION, &base_asset, true, true);
    let (_, push_client, push_config) = deploy_push_oracle(&e, &base_asset);

    periodic_client.set_price(&xlm_asset, &(100 * i128::pow(10, ORACLES_DECIMALS)), &now);
    // Push signing timestamp is 25 h in the past — heartbeat has expired.
    let stale_feed_ts = now - 25 * 60 * 60;
    push_client.set_price(&xlm_asset, &(300 * i128::pow(10, PUSH_ORACLE_DECIMALS)), &stale_feed_ts);

    let (_, contract_client) = deploy_aggregated_oracle(
        &e,
        &Address::generate(&e),
        &base_asset,
        AGGREGATED_ORACLE_DECIMALS,
        AGGREGATED_ORACLE_MAX_AGE,
        svec![&e, periodic_config, push_config],
    );
    contract_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    let lastprice = contract_client.lastprice(&xlm_asset).unwrap();
    // Only the periodic oracle contributes — median of [100] = 100.
    assert_eq!(lastprice.price, 100 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS));
    assert_eq!(lastprice.timestamp, now);
}

// ---------------------------------------------------------------------------
// 3. periodic_update_max_age does NOT gate Push oracles
//    Push oracle signing ts older than max_age but within resolution → accepted
// ---------------------------------------------------------------------------
#[test]
fn test_push_oracle_not_gated_by_periodic_max_age() {
    let e = get_default_env();
    let now = e.ledger().timestamp();
    let base_asset = Asset::Other(Symbol::new(&e, "USD"));

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");
    let xlm_asset = Asset::Stellar(xlm_address.clone());

    // Use a tight max_age for periodic oracles (10 minutes).
    // Push oracle resolution is 24 h — its signing ts is 12 h old, which is:
    //   - older than max_age (10 min) → would be rejected if max_age applied
    //   - within resolution (24 h)   → should be accepted
    const TIGHT_MAX_AGE: u64 = 10 * 60;

    let (_, periodic_client, periodic_config) =
        deploy_mock_oracle(&e, ORACLES_DECIMALS, ORACLES_RESOLUTION, &base_asset, true, true);
    let (_, push_client, push_config) = deploy_push_oracle(&e, &base_asset);

    periodic_client.set_price(&xlm_asset, &(100 * i128::pow(10, ORACLES_DECIMALS)), &now);
    let feed_ts = now - 12 * 60 * 60; // 12 h old, within 24 h heartbeat
    push_client.set_price(&xlm_asset, &(300 * i128::pow(10, PUSH_ORACLE_DECIMALS)), &feed_ts);

    let (_, contract_client) = deploy_aggregated_oracle(
        &e,
        &Address::generate(&e),
        &base_asset,
        AGGREGATED_ORACLE_DECIMALS,
        TIGHT_MAX_AGE,
        svec![&e, periodic_config, push_config],
    );
    contract_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    let lastprice = contract_client.lastprice(&xlm_asset).unwrap();
    // Both oracles contribute → median of [100, 300] = 200.
    assert_eq!(lastprice.price, 200 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS));
}

// ---------------------------------------------------------------------------
// 4. Deviation circuit-breaker uses wall-clock time — Push oracle's old signing
//    timestamp does not disable the check
// ---------------------------------------------------------------------------
#[test]
fn test_deviation_check_uses_wall_clock_time_not_source_timestamp() {
    const MAX_DEV_BPS: u32 = 500; // 5 %
    // Window is 60 s — any two calls made within a minute will be compared.
    const MAX_DEV_CONSECUTIVE_DIFF_SECS: u64 = 60;

    let e = get_default_env();
    let now = e.ledger().timestamp();
    let base_asset = Asset::Other(Symbol::new(&e, "USD"));

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");
    let xlm_asset = Asset::Stellar(xlm_address.clone());

    // Push oracle signing ts is 20 h old — the source-derived median timestamp
    // will be 20 h in the past.  If the circuit-breaker used that value for
    // time_diff it would get ~72000 s >> MAX_DEV_CONSECUTIVE_DIFF_SECS and skip
    // the deviation check entirely.  With the fix it uses wall-clock delta (~0 s)
    // and the check fires correctly.
    let (_, push_client, push_config) = deploy_push_oracle(&e, &base_asset);
    let feed_ts = now - 20 * 60 * 60;
    push_client.set_price(&xlm_asset, &(100 * i128::pow(10, PUSH_ORACLE_DECIMALS)), &feed_ts);

    let (_, contract_client) = deploy_aggregated_oracle(
        &e,
        &Address::generate(&e),
        &base_asset,
        AGGREGATED_ORACLE_DECIMALS,
        AGGREGATED_ORACLE_MAX_AGE,
        svec![&e, push_config],
    );
    contract_client.add_asset(
        &xlm_ticker,
        &xlm_address,
        &MAX_DEV_BPS,
        &MAX_DEV_CONSECUTIVE_DIFF_SECS,
    );

    // First call — establishes the baseline median (100) and stores wall-clock now.
    let first = contract_client.lastprice(&xlm_asset).unwrap();
    assert_eq!(first.price, 100 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS));

    // Advance ledger by 10 s (within MAX_DEV_CONSECUTIVE_DIFF_SECS = 60 s).
    e.ledger().with_mut(|li| li.timestamp += 10);

    // Push a new price 10 % higher than the baseline — exceeds MAX_DEV_BPS (5 %).
    // The signing timestamp advances with ledger time so the feed stays fresh.
    let new_feed_ts = e.ledger().timestamp() - 20 * 60 * 60;
    push_client.set_price(&xlm_asset, &(110 * i128::pow(10, PUSH_ORACLE_DECIMALS)), &new_feed_ts);

    // Second call — should be rejected by the deviation check.
    let second = contract_client.lastprice(&xlm_asset);
    assert!(
        second.is_none(),
        "Expected deviation check to reject the 10% price jump within the 60 s window"
    );
}

// ---------------------------------------------------------------------------
// 5. Push oracle price update between two calls is immediately visible
//    (no cache hit path for push oracles)
// ---------------------------------------------------------------------------
#[test]
fn test_push_oracle_price_update_is_reflected_immediately() {
    let e = get_default_env();
    let now = e.ledger().timestamp();
    let base_asset = Asset::Other(Symbol::new(&e, "USD"));

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");
    let xlm_asset = Asset::Stellar(xlm_address.clone());

    let (_, push_client, push_config) = deploy_push_oracle(&e, &base_asset);
    push_client.set_price(&xlm_asset, &(100 * i128::pow(10, PUSH_ORACLE_DECIMALS)), &now);

    let (_, contract_client) = deploy_aggregated_oracle(
        &e,
        &Address::generate(&e),
        &base_asset,
        AGGREGATED_ORACLE_DECIMALS,
        AGGREGATED_ORACLE_MAX_AGE,
        svec![&e, push_config],
    );
    contract_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    let first = contract_client.lastprice(&xlm_asset).unwrap();
    assert_eq!(first.price, 100 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS));

    // Advance ledger slightly (well within resolution) and push a new price.
    e.ledger().with_mut(|li| li.timestamp += 30);
    let new_now = e.ledger().timestamp();
    push_client.set_price(&xlm_asset, &(200 * i128::pow(10, PUSH_ORACLE_DECIMALS)), &new_now);

    let second = contract_client.lastprice(&xlm_asset).unwrap();
    // If the cache were consulted, we'd still get 100. The correct answer is 200.
    assert_eq!(second.price, 200 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS));
}

// ---------------------------------------------------------------------------
// Periodic-oracle grace window (PERIODIC_UPDATE_GRACE_PERIOD)
//
// For periodic oracles the staleness ceiling is
//   max_allowed_age = (resolution + PERIODIC_UPDATE_GRACE_PERIOD).min(max_age)
// The grace term only has an effect when `max_age` is wider than
// `resolution + grace`; otherwise the protocol `max_age` ceiling clamps it
// away. The default fixture's AGGREGATED_ORACLE_MAX_AGE (360) equals the
// oracle resolution, so it clamps the grace entirely — the tests below
// therefore deploy with a deliberately wide max_age to exercise the grace
// path itself.
//
// `EXPECTED_GRACE_PERIOD` is an INDEPENDENT literal, intentionally not derived
// from `PERIODIC_UPDATE_GRACE_PERIOD`. The boundary tests below use this literal
// for their arithmetic so that shrinking the production constant makes them
// fail rather than silently track the change. The compile-time assertion ties
// the two together: changing the production value triggers a build error here,
// forcing the new value to be acknowledged deliberately.
// ---------------------------------------------------------------------------

const EXPECTED_GRACE_PERIOD: u64 = 70;
const _: () = assert!(
    PERIODIC_UPDATE_GRACE_PERIOD == EXPECTED_GRACE_PERIOD,
    "PERIODIC_UPDATE_GRACE_PERIOD changed; update EXPECTED_GRACE_PERIOD and the grace-window \
     tests deliberately"
);

/// A periodic price aged exactly `resolution + grace` is still accepted.
/// Because this age already exceeds `resolution`, the price would be rejected
/// without the grace term (or with a smaller one) — so this pins the grace
/// value and guards against it being silently shrunk.
#[test]
fn test_periodic_oracle_accepted_at_grace_window_boundary() {
    let e = get_default_env();
    let now = e.ledger().timestamp();
    let base_asset = Asset::Other(Symbol::new(&e, "USD"));

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");
    let xlm_asset = Asset::Stellar(xlm_address.clone());

    // Wide max_age so the `.min(max_age)` clamp does not swallow the grace.
    const WIDE_MAX_AGE: u64 = 60 * 60; // 1 h, >> resolution + grace
    // Oldest age that must still pass: full resolution + grace window.
    let max_valid_age = ORACLES_RESOLUTION as u64 + EXPECTED_GRACE_PERIOD;

    let (_, periodic_client, periodic_config) =
        deploy_mock_oracle(&e, ORACLES_DECIMALS, ORACLES_RESOLUTION, &base_asset, true, true);

    let feed_ts = now - max_valid_age;
    periodic_client.set_price(&xlm_asset, &(100 * i128::pow(10, ORACLES_DECIMALS)), &feed_ts);

    let (_, contract_client) = deploy_aggregated_oracle(
        &e,
        &Address::generate(&e),
        &base_asset,
        AGGREGATED_ORACLE_DECIMALS,
        WIDE_MAX_AGE,
        svec![&e, periodic_config],
    );
    contract_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    let lastprice = contract_client.lastprice(&xlm_asset).unwrap();
    assert_eq!(lastprice.price, 100 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS));
    // The single periodic source drives the published timestamp.
    assert_eq!(lastprice.timestamp, feed_ts);
}

/// One second past `resolution + grace` the price is stale and dropped. With a
/// single oracle the median has no contributors, so `lastprice` is `None`.
#[test]
fn test_periodic_oracle_rejected_just_past_grace_window() {
    let e = get_default_env();
    let now = e.ledger().timestamp();
    let base_asset = Asset::Other(Symbol::new(&e, "USD"));

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");
    let xlm_asset = Asset::Stellar(xlm_address.clone());

    const WIDE_MAX_AGE: u64 = 60 * 60; // 1 h, >> resolution + grace
    let just_too_old = ORACLES_RESOLUTION as u64 + EXPECTED_GRACE_PERIOD + 1;

    let (_, periodic_client, periodic_config) =
        deploy_mock_oracle(&e, ORACLES_DECIMALS, ORACLES_RESOLUTION, &base_asset, true, true);

    let feed_ts = now - just_too_old;
    periodic_client.set_price(&xlm_asset, &(100 * i128::pow(10, ORACLES_DECIMALS)), &feed_ts);

    let (_, contract_client) = deploy_aggregated_oracle(
        &e,
        &Address::generate(&e),
        &base_asset,
        AGGREGATED_ORACLE_DECIMALS,
        WIDE_MAX_AGE,
        svec![&e, periodic_config],
    );
    contract_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    assert!(
        contract_client.lastprice(&xlm_asset).is_none(),
        "a periodic price older than resolution + grace must be rejected"
    );
}

/// The grace window can never push the ceiling above the protocol `max_age`.
/// With a tight `max_age` (< resolution + grace) a price that the grace term
/// alone would accept is still rejected, proving the `.min(max_age)` clamp is
/// live.
#[test]
fn test_periodic_grace_window_is_clamped_by_max_age() {
    let e = get_default_env();
    let now = e.ledger().timestamp();
    let base_asset = Asset::Other(Symbol::new(&e, "USD"));

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");
    let xlm_asset = Asset::Stellar(xlm_address.clone());

    // resolution (360) < TIGHT_MAX_AGE (400) < resolution + grace (510).
    const TIGHT_MAX_AGE: u64 = 400;
    // Aged inside the grace window but beyond the tight max_age ceiling.
    let age_within_grace_but_past_max_age = TIGHT_MAX_AGE + 1; // 401, still < 510

    let (_, periodic_client, periodic_config) =
        deploy_mock_oracle(&e, ORACLES_DECIMALS, ORACLES_RESOLUTION, &base_asset, true, true);

    let feed_ts = now - age_within_grace_but_past_max_age;
    periodic_client.set_price(&xlm_asset, &(100 * i128::pow(10, ORACLES_DECIMALS)), &feed_ts);

    let (_, contract_client) = deploy_aggregated_oracle(
        &e,
        &Address::generate(&e),
        &base_asset,
        AGGREGATED_ORACLE_DECIMALS,
        TIGHT_MAX_AGE,
        svec![&e, periodic_config],
    );
    contract_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    assert!(
        contract_client.lastprice(&xlm_asset).is_none(),
        "max_age must clamp the grace window — a price past max_age is rejected"
    );
}

// ---- Helpers -----

struct TestFixture<'a> {
    e: Env,
    contract_admin: Address,
    oracle_clients: Vec<MockOracleContractClient<'a>>,
    oracle_config_inputs: SVec<OracleConfigInput>,
    contract_id: Address,
    contract_client: AggregatedOracleContractClient<'a>,
}

const ORACLES_DECIMALS: u32 = 7;
const ORACLES_RESOLUTION: u32 = 360;

const AGGREGATED_ORACLE_DECIMALS: u32 = 14;
const AGGREGATED_ORACLE_MAX_AGE: u64 = 360;

impl<'a> TestFixture<'a> {
    fn new() -> Self {
        let e = get_default_env();
        let base_asset = Asset::Other(Symbol::new(&e, "USD"));

        // -- Deploy mock oracles --

        let (_oracle_1_address, oracle_1_client, oracle_1_config_input) =
            deploy_mock_oracle(&e, ORACLES_DECIMALS, ORACLES_RESOLUTION, &base_asset, true, true);
        let (_oracle_2_address, oracle_2_client, oracle_2_config_input) =
            deploy_mock_oracle(&e, ORACLES_DECIMALS, ORACLES_RESOLUTION, &base_asset, false, true);
        let (_oracle_3_address, oracle_3_client, oracle_3_config_input) =
            deploy_mock_oracle(&e, ORACLES_DECIMALS, ORACLES_RESOLUTION, &base_asset, true, true);

        // -- Deploy the aggregated oracle contract --

        let contract_admin = Address::generate(&e);

        let oracle_config_inputs =
            svec![&e, oracle_1_config_input, oracle_2_config_input, oracle_3_config_input];

        let (contract_id, contract_client) = deploy_aggregated_oracle(
            &e,
            &contract_admin,
            &Asset::Other(symbol_short!("USD")),
            AGGREGATED_ORACLE_DECIMALS,
            AGGREGATED_ORACLE_MAX_AGE,
            oracle_config_inputs.clone(),
        );

        let oracle_clients = std::vec![oracle_1_client, oracle_2_client, oracle_3_client];

        TestFixture {
            e,
            contract_admin,
            oracle_clients,
            oracle_config_inputs,
            contract_id,
            contract_client,
        }
    }
}

/// Deploys aggregated oracle contract
fn deploy_aggregated_oracle<'a>(
    e: &Env,
    admin: &Address,
    base_asset: &Asset,
    decimals: u32,
    periodic_oracles_price_max_age: u64,
    oracles: SVec<OracleConfigInput>,
) -> (Address, AggregatedOracleContractClient<'a>) {
    let address = e.register(
        AggregatedOracleContract,
        (admin, base_asset.clone(), decimals, periodic_oracles_price_max_age, oracles),
    );
    let client = AggregatedOracleContractClient::new(e, &address);

    (address, client)
}

/// Deploys a mock oracle contract
fn deploy_mock_oracle<'a>(
    e: &Env,
    decimals: u32,
    resolution: u32,
    base_asset: &Asset,
    is_stellar_data_based: bool,
    is_price_update_periodic: bool,
) -> (Address, MockOracleContractClient<'a>, OracleConfigInput) {
    let address = e.register(MockOracleContract, (decimals, resolution, base_asset.clone()));
    let client = MockOracleContractClient::new(e, &address);
    let oracle_config_input = OracleConfigInput {
        address: address.clone(),
        is_stellar_data_based,
        is_price_update_periodic,
    };

    (address, client, oracle_config_input)
}

fn get_default_env() -> Env {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set_timestamp(1_000_000_700);

    e
}

mod mock_oracle;
