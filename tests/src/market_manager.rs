#![cfg(test)]

use ::market::constants::{
    DEFAULT_BAD_DEBT_LOCK_D, DEFAULT_INSOLVENCY_LTV_BPS,
    DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS, MAX_RESERVES,
};
use market_manager::contract::{MarketInitParams, MarketManagerClient, MarketManagerContract};
use soroban_sdk::{Address, BytesN, Env, String, testutils::Address as _};

use crate::get_default_env;

mod market {
    #![allow(clippy::too_many_arguments)]
    use soroban_sdk::contractimport;

    contractimport!(file = "../wasms/market.wasm");
}

#[allow(unused)]
struct ManagerSetup<'a> {
    e: Env,
    manager_client: MarketManagerClient<'a>,
    manager_address: Address,
    manager_admin: Address,
}

impl<'a> ManagerSetup<'a> {
    fn new() -> Self {
        let e = get_default_env();
        e.cost_estimate().budget().reset_unlimited();
        let manager_admin = Address::generate(&e);
        let market_contract_wasm_hash = e.deployer().upload_contract_wasm(market::WASM);

        let manager_address =
            e.register(MarketManagerContract, (&manager_admin, market_contract_wasm_hash));
        let manager_client = MarketManagerClient::new(&e, &manager_address);

        Self { e, manager_client, manager_address, manager_admin }
    }
}

fn default_params(is_owned: bool) -> MarketInitParams {
    MarketInitParams {
        max_positions: 2,
        min_collateral_value_cents: 1,
        insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
        update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
        is_owned,
        bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
    }
}

#[test]
fn test_manager_has_no_markets_initially() {
    let ManagerSetup { manager_client, .. } = ManagerSetup::new();

    let market_addresses = manager_client.get_markets();

    assert!(market_addresses.is_empty());
}

#[test]
fn test_manager_deploy_markets() {
    let ManagerSetup { e, manager_client, .. } = ManagerSetup::new();

    let market_admin = Address::generate(&e);
    let oracle = Address::generate(&e);
    let insurance_fund = Address::generate(&e);

    let salt_1 = BytesN::from_array(&e, &[0; 32]);
    let name_1 = String::from_str(&e, "market_1");
    let market_address_1 = manager_client.deploy(
        &salt_1,
        &market_admin,
        &name_1,
        &oracle,
        &insurance_fund,
        &default_params(false),
    );

    let market_list = manager_client.get_markets();
    assert_eq!(market_list.len(), 1);
    assert!(market_list.contains_key(market_address_1));

    let salt_2 = BytesN::from_array(&e, &[1; 32]);
    let name_2 = String::from_str(&e, "market_2");
    let market_address_2 = manager_client.deploy(
        &salt_2,
        &market_admin,
        &name_2,
        &oracle,
        &insurance_fund,
        &default_params(false),
    );

    let market_list = manager_client.get_markets();

    assert_eq!(market_list.len(), 2);
    assert!(market_list.contains_key(market_address_2));
}

#[test]
fn test_manager_cannot_redeploy_market() {
    let ManagerSetup { e, manager_client, .. } = ManagerSetup::new();

    let market_admin = Address::generate(&e);
    let oracle = Address::generate(&e);
    let insurance_fund = Address::generate(&e);

    let salt = BytesN::from_array(&e, &[0; 32]);
    let name_1 = String::from_str(&e, "market_1");
    manager_client.deploy(
        &salt,
        &market_admin,
        &name_1,
        &oracle,
        &insurance_fund,
        &default_params(false),
    );

    let name_2 = String::from_str(&e, "market_2");

    assert!(
        manager_client
            .try_deploy(
                &salt,
                &market_admin,
                &name_2,
                &oracle,
                &insurance_fund,
                &default_params(false),
            )
            .is_err()
    );
}

#[test]
fn test_manager_invalid_deploy() {
    let ManagerSetup { e, manager_client, .. } = ManagerSetup::new();

    let market_admin = Address::generate(&e);
    let oracle = Address::generate(&e);
    let insurance_fund = Address::generate(&e);

    let salt = BytesN::from_array(&e, &[0; 32]);
    let name_1 = String::from_str(&e, "market_1");

    assert!(
        manager_client
            .try_deploy(
                &salt,
                &market_admin,
                &name_1,
                &oracle,
                &insurance_fund,
                &MarketInitParams {
                    max_positions: 2,
                    min_collateral_value_cents: -1,
                    insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                    update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                    is_owned: false,
                    bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
                },
            )
            .is_err(),
    );

    assert!(
        manager_client
            .try_deploy(
                &salt,
                &market_admin,
                &name_1,
                &oracle,
                &insurance_fund,
                &MarketInitParams {
                    max_positions: MAX_RESERVES + 1,
                    min_collateral_value_cents: 0,
                    insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                    update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                    is_owned: false,
                    bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
                },
            )
            .is_err(),
    );

    assert!(
        manager_client
            .try_deploy(
                &salt,
                &market_admin,
                &name_1,
                &oracle,
                &insurance_fund,
                &MarketInitParams {
                    max_positions: MAX_RESERVES - 1,
                    min_collateral_value_cents: 0,
                    insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                    update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                    is_owned: false,
                    bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
                },
            )
            .is_ok(),
    );
}
