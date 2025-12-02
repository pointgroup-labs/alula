#![allow(unused)]

use sep_40_oracle::{Asset, PriceData};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, Env, FromVal, IntoVal, Map, Symbol, Vec as SVec, symbol_short,
    testutils::{Address as _, Events, Ledger},
    vec as svec,
};

use crate::{
    constants::BPS_FACTOR,
    contract::{AggregatedOracleContract, AggregatedOracleContractClient},
    storage::{DataKey, OracleConfigInput},
    tests::mock_oracle::{MockOracleContract, MockOracleContractClient},
};

extern crate std;
use std::vec::Vec;

#[test]
fn test_median_price_with_odd_number_of_reported_prices() {
    let TestFixture { e, oracle_clients, oracle_config_inputs, aggregated_oracle_client, .. } =
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

    aggregated_oracle_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    let lastprice = aggregated_oracle_client.lastprice(&xlm_asset_stellar).unwrap();

    assert_eq!(lastprice.price, 200 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS));
    assert_eq!(lastprice.timestamp, e.ledger().timestamp());
}

#[test]
fn test_median_price_with_even_number_of_reported_prices() {
    let TestFixture { e, oracle_clients, oracle_config_inputs, aggregated_oracle_client, .. } =
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

    aggregated_oracle_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    let lastprice = aggregated_oracle_client.lastprice(&xlm_asset_stellar).unwrap();

    assert_eq!(
        lastprice.price,
        150 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS) // (100 + 200) / 2
    );
    assert_eq!(lastprice.timestamp, e.ledger().timestamp());
}

#[test]
fn test_median_price_with_all_expired_prices() {
    let TestFixture { e, oracle_clients, aggregated_oracle_client, oracle_config_inputs, .. } =
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
    aggregated_oracle_client.add_asset(&xlm_ticker, &xlm_address, &0, &0);

    let lastprice = aggregated_oracle_client.lastprice(&xlm_asset_stellar);

    assert!(lastprice.is_none());
}

#[test]
fn test_max_deviation_check() {
    const MAX_DEV_BPS: u32 = 100; // 10%
    const MAX_DEV_CONSECUTIVE_DIFF_SECS: u64 = 10000; // NB: Must exceed the resolution on the oracles to notice the effect

    let TestFixture {
        e,
        oracle_clients,
        aggregated_oracle_client,
        oracle_config_inputs,
        aggregated_oracle_address,
        ..
    } = TestFixture::new();

    // -- Set prices on oracles --

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");

    let xlm_asset_other = Asset::Other(xlm_ticker.clone());
    let xlm_asset_stellar = Asset::Stellar(xlm_address.clone());

    aggregated_oracle_client.add_asset(
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

    e.as_contract(&aggregated_oracle_address, || {
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

    let lastprice_1 = aggregated_oracle_client.lastprice(&xlm_asset_stellar).unwrap();

    assert_eq!(lastprice_1.price, 200 * 10_i128.pow(AGGREGATED_ORACLE_DECIMALS));
    assert_eq!(lastprice_1.timestamp, e.ledger().timestamp());

    // -- Verify that the previous lastprice is set --

    e.as_contract(&aggregated_oracle_address, || {
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

    assert!(aggregated_oracle_client.lastprice(&xlm_asset_stellar).is_none());

    let events = e.events().all();
    let (contract_address, topics, _data) = events.get(0).unwrap();
    assert_eq!(contract_address, aggregated_oracle_address);
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

    let lastprice_2 = aggregated_oracle_client.lastprice(&xlm_asset_stellar).unwrap();

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

    assert!(aggregated_oracle_client.lastprice(&xlm_asset_stellar).is_none());

    let events = e.events().all();
    let (contract_address, topics, _data) = events.get(0).unwrap();
    assert_eq!(contract_address, aggregated_oracle_address);
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

    assert!(aggregated_oracle_client.lastprice(&xlm_asset_stellar).is_some());
}

// ---- Helpers -----
struct TestFixture<'a> {
    e: Env,
    oracle_clients: Vec<MockOracleContractClient<'a>>,
    oracle_config_inputs: SVec<OracleConfigInput>,
    aggregated_oracle_address: Address,
    aggregated_oracle_client: AggregatedOracleContractClient<'a>,
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
            deploy_mock_oracle(&e, ORACLES_DECIMALS, ORACLES_RESOLUTION, &base_asset, true);
        let (_oracle_2_address, oracle_2_client, oracle_2_config_input) =
            deploy_mock_oracle(&e, ORACLES_DECIMALS, ORACLES_RESOLUTION, &base_asset, false);
        let (_oracle_3_address, oracle_3_client, oracle_3_config_input) =
            deploy_mock_oracle(&e, ORACLES_DECIMALS, ORACLES_RESOLUTION, &base_asset, true);

        // -- Deploy the aggregated oracle contract --

        let admin = Address::generate(&e);

        let oracle_config_inputs =
            svec![&e, oracle_1_config_input, oracle_2_config_input, oracle_3_config_input];

        let (aggregated_oracle_address, aggregated_oracle_client) = deploy_aggregated_oracle(
            &e,
            &admin,
            &Symbol::new(&e, "USD"),
            AGGREGATED_ORACLE_DECIMALS,
            AGGREGATED_ORACLE_MAX_AGE,
            oracle_config_inputs.clone(),
        );

        let oracle_clients = std::vec![oracle_1_client, oracle_2_client, oracle_3_client];

        TestFixture {
            e,
            oracle_clients,
            oracle_config_inputs,
            aggregated_oracle_address,
            aggregated_oracle_client,
        }
    }
}

/// Deploys aggregated oracle contract
fn deploy_aggregated_oracle<'a>(
    e: &Env,
    admin: &Address,
    base_asset_symbol: &Symbol,
    decimals: u32,
    max_age: u64,
    oracles: SVec<OracleConfigInput>,
) -> (Address, AggregatedOracleContractClient<'a>) {
    let address = e.register(
        AggregatedOracleContract,
        (admin, base_asset_symbol.clone(), decimals, max_age, oracles),
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
) -> (Address, MockOracleContractClient<'a>, OracleConfigInput) {
    let address = e.register(MockOracleContract, (decimals, resolution, base_asset.clone()));
    let client = MockOracleContractClient::new(e, &address);
    let oracle_config_input = OracleConfigInput { address: address.clone(), is_stellar_data_based };

    (address, client, oracle_config_input)
}

fn get_default_env() -> Env {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set_timestamp(1_000_000_700);

    e
}

mod mock_oracle;
