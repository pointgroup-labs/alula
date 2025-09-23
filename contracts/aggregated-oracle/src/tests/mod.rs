use sep_40_oracle::{Asset, PriceData};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    vec as svec, Address, Env, Symbol, Vec as SVec,
};

use crate::{
    contract::{
        AggregatedOracleContract, AggregatedOracleContractClient, AggregatedPriceFeedTrait,
    },
    storage::OracleConfigInput,
    tests::mock_oracle::{MockOracleContract, MockOracleContractClient},
};

extern crate std;
use std::{vec, vec::Vec};

#[test]
fn test_median_price_with_multiple_oracles() {
    let TestFixture {
        e,
        oracle_clients,
        oracle_config_inputs,
        aggregated_oracle_client,
        ..
    } = TestFixture::new();

    // Set XLM prices on the mock oracles: [100, 300, 200]
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
            &(100 * (idx as i128 + 1) * i128::pow(10, ORACLES_DECIMALS)),
            &1_000_000_600,
        );
    }

    aggregated_oracle_client.add_asset(&xlm_ticker, &xlm_address);

    let xlm_asset = Asset::Stellar(xlm_address.clone());

    let lastprice = aggregated_oracle_client.lastprice(&xlm_asset).unwrap();

    assert_eq!(
        lastprice.price,
        200 * i128::pow(10, AGGREGATED_ORACLE_DECIMALS) as i128
    );
    assert_eq!(lastprice.timestamp, e.ledger().timestamp());
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

// impl<'a> TestFixture<'a> {
//     fn new(oracles_amount: usize) -> TestFixture<'a> {
//         let e = get_default_env();
//         let base_asset = Asset::Other(Symbol::new(&e, "USD"));

//         // ---- Deploy mock oracles ----

//         let mut oracles = std::vec![];

//         for idx in 0..oracles_amount {
//             let address = Address::generate(&e);
//             let is_stellar_data_based = idx % 2 == 0;

//             let oracle_config_input = OracleConfigInput {
//                 address,
//                 is_stellar_data_based,
//             };

//             oracles.push(oracle_config_input);
//         }

//         let mut oracle_addresses = std::vec![];
//         let mut oracle_contract_clients = std::vec![];

//         for (idx, oracle_config_input) in oracles.clone().iter().enumerate() {
//             let idx = idx as u32;

//             let decimals = 10;
//             let resolution = 360;

//             let (oracle_address, oracle_contract_client) =
//                 deploy_mock_oracle(&e, decimals, resolution, &base_asset);

//             oracle_addresses.push(oracle_address);
//             oracle_contract_clients.push(oracle_contract_client);
//         }

//         // Deploy the aggregated oracle contract
//         let admin = Address::generate(&e);
//         let max_age = 360;
//         let decimals = 14;

//         let (aggregated_oracle_address, aggregated_oracle_client) =
//             deploy_aggregated_oracle(&e, &admin, &base_asset, decimals, max_age, oracles);

//         TestFixture {
//             e,
//             oracle_addresses,
//             oracle_clients: todo!(),
//             aggregated_oracle_address,
//             aggregated_oracle_client,
//         }
//     }

//     fn add_asset(&self, symbol: &Symbol, token_address: &Address) {
//         let asset = Asset::Stellar(token_address.clone());

//         for oracle_client in self.oracle_clients {
//             oracle_client.set_price(&asset, price, timestamp);
//         }

//         self.aggregated_oracle_client
//             .add_asset(symbol, token_address);
//     }

//     fn add_asset_per_oracle(&self, asset: &Asset, oracle_idx: usize) {}

//     fn change_oracle_price(&self, asset: &Asset) {}
// }

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
