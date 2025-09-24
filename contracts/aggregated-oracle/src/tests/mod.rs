#![allow(unused)]

use sep_40_oracle::Asset;
use soroban_sdk::{
    Address, Env, Symbol, Vec as SVec,
    testutils::{Address as _, Ledger},
    vec as svec,
};

use crate::{
    contract::{AggregatedOracleContract, AggregatedOracleContractClient},
    storage::OracleConfigInput,
    tests::mock_oracle::{MockOracleContract, MockOracleContractClient},
};

extern crate std;
use std::vec::Vec;

#[test]
fn test_median_price_with_odd_number_of_reported_prices() {
    let TestFixture {
        e,
        oracle_clients,
        oracle_config_inputs,
        aggregated_oracle_client,
        ..
    } = TestFixture::new();

    // Set XLM prices on the mock oracles: [100, 200, 300]
    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");

    let xlm_asset_other = Asset::Other(xlm_ticker.clone());
    let xlm_asset_stellar = Asset::Stellar(xlm_address.clone());

    for (idx, (oracle_client, oracle_config_input)) in oracle_clients
        .iter()
        .zip(oracle_config_inputs.iter())
        .enumerate()
    {
        oracle_client.set_price(
            &if oracle_config_input.is_stellar_data_based {
                xlm_asset_stellar.clone()
            } else {
                xlm_asset_other.clone()
            },
            &(100 * (idx as i128 + 1) * i128::pow(10, ORACLES_DECIMALS)), // Oracles reports: 100, 200 and 300
            &1_000_000_600,
        );
    }

    aggregated_oracle_client.add_asset(&xlm_ticker, &xlm_address);

    let lastprice = aggregated_oracle_client
        .lastprice(&xlm_asset_stellar)
        .unwrap();

    assert_eq!(
        lastprice.price,
        200 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS)
    );
    assert_eq!(lastprice.timestamp, e.ledger().timestamp());
}

#[test]
fn test_median_price_with_even_number_of_reported_prices() {
    let TestFixture {
        e,
        oracle_clients,
        oracle_config_inputs,
        aggregated_oracle_client,
        ..
    } = TestFixture::new();

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");

    let xlm_asset_other = Asset::Other(xlm_ticker.clone());
    let xlm_asset_stellar = Asset::Stellar(xlm_address.clone());

    for (idx, (oracle_client, oracle_config_input)) in oracle_clients
        .iter()
        .skip(1)
        .zip(oracle_config_inputs.iter().skip(1))
        .enumerate()
    {
        oracle_client.set_price(
            &if oracle_config_input.is_stellar_data_based {
                xlm_asset_stellar.clone()
            } else {
                xlm_asset_other.clone()
            },
            &(100 * (idx as i128 + 1) * i128::pow(10, ORACLES_DECIMALS)), // Oracles reports: 100, 200 and 300
            (&(e.ledger().timestamp() - AGGREGATED_ORACLE_MAX_AGE)),       // Allowed timestamps
        );
    }

    oracle_clients[0].set_price(
        &if oracle_config_inputs.get(0).unwrap().is_stellar_data_based {
            xlm_asset_stellar.clone()
        } else {
            xlm_asset_other.clone()
        },
        &(200 * i128::pow(10, ORACLES_DECIMALS)),
        &(e.ledger().timestamp() - AGGREGATED_ORACLE_MAX_AGE - 1), // Expired timestamp
    );

    aggregated_oracle_client.add_asset(&xlm_ticker, &xlm_address);

    let lastprice = aggregated_oracle_client
        .lastprice(&xlm_asset_stellar)
        .unwrap();

    assert_eq!(
        lastprice.price,
        150 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS) // (100 + 200) / 2
    );
    assert_eq!(lastprice.timestamp, e.ledger().timestamp());
}

#[test]
fn test_median_price_with_all_expired_prices() {
    let TestFixture {
        e,
        oracle_clients,
        aggregated_oracle_client,
        oracle_config_inputs,
        ..
    } = TestFixture::new();

    let xlm_address = Address::generate(&e);
    let xlm_ticker = Symbol::new(&e, "XLM");

    let xlm_asset_other = Asset::Other(xlm_ticker.clone());
    let xlm_asset_stellar = Asset::Stellar(xlm_address.clone());

    // Set prices for all oracles to be expired
    for (idx, (oracle_client, oracle_config_input)) in
        oracle_clients.iter().zip(oracle_config_inputs).enumerate()
    {
        oracle_client.set_price(
            &if oracle_config_input.is_stellar_data_based {
                xlm_asset_stellar.clone()
            } else {
                xlm_asset_other.clone()
            },
            &(100 * (idx as i128 + 1) * i128::pow(10, ORACLES_DECIMALS)), // Oracles reports: 100, 200 and 300
            (&(e.ledger().timestamp() - AGGREGATED_ORACLE_MAX_AGE - 1)),   // Expired timestamps
        );
    }
    aggregated_oracle_client.add_asset(&xlm_ticker, &xlm_address);

    let lastprice = aggregated_oracle_client.lastprice(&xlm_asset_stellar);

    assert!(lastprice.is_none());
}

// TODO: Add more tests

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

        let oracle_config_inputs = svec![
            &e,
            oracle_1_config_input,
            oracle_2_config_input,
            oracle_3_config_input
        ];

        let (aggregated_oracle_address, aggregated_oracle_client) = deploy_aggregated_oracle(
            &e,
            &admin,
            &base_asset,
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
    base_asset: &Asset,
    decimals: u32,
    max_age: u64,
    oracles: SVec<OracleConfigInput>,
) -> (Address, AggregatedOracleContractClient<'a>) {
    let address = e.register(
        AggregatedOracleContract,
        (admin, base_asset.clone(), decimals, max_age, oracles),
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
    let address = e.register(
        MockOracleContract,
        (decimals, resolution, base_asset.clone()),
    );
    let client = MockOracleContractClient::new(e, &address);
    let oracle_config_input = OracleConfigInput {
        address: address.clone(),
        is_stellar_data_based,
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
