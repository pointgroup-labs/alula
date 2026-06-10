#![cfg(test)]

use ::market::constants::{
    DEFAULT_BAD_DEBT_LOCK_D, DEFAULT_INSOLVENCY_LTV_BPS,
    DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS, MAX_RESERVES,
};
use market_manager::contract::{MarketInitParams, MarketManagerClient, MarketManagerContract};
use soroban_sdk::{Address, BytesN, Env, String, testutils::Address as _};

use crate::get_default_env;

/// Per-market upgrade-in-queue period used by deploys in this module.
/// 24h matches the deploy-time minimum enforced by the manager.
const TEST_UPGRADE_IN_QUEUE_SECONDS: u64 = 24 * 60 * 60;

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
    market_wasm_hash: BytesN<32>,
}

impl<'a> ManagerSetup<'a> {
    fn new() -> Self {
        let e = get_default_env();
        e.cost_estimate().budget().reset_unlimited();
        let manager_admin = Address::generate(&e);
        let market_wasm_hash = e.deployer().upload_contract_wasm(market::WASM);

        let manager_address = e.register(MarketManagerContract, (&manager_admin,));
        let manager_client = MarketManagerClient::new(&e, &manager_address);

        Self { e, manager_client, manager_address, manager_admin, market_wasm_hash }
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
    let ManagerSetup { e, manager_client, .. } = ManagerSetup::new();

    // Replaces the old `get_markets().is_empty()` assertion. With the
    // `MarketsList` blob gone, "no markets are registered" is now an
    // assertion against the per-key `DataKey::DeployedMarket(addr)`
    // shape: any address we generate without going through `deploy`
    // must be reported as not deployed.
    let arbitrary = Address::generate(&e);

    assert!(
        !manager_client.is_deployed_by_manager(&arbitrary),
        "freshly registered manager must not report any address as deployed",
    );
}

#[test]
fn test_manager_deploy_markets() {
    let ManagerSetup { e, manager_client, market_wasm_hash, .. } = ManagerSetup::new();

    let market_admin = Address::generate(&e);
    let oracle = Address::generate(&e);
    let insurance_fund = Address::generate(&e);

    let salt_1 = BytesN::from_array(&e, &[0; 32]);
    let name_1 = String::from_str(&e, "market_1");
    let market_address_1 = manager_client.deploy(
        &salt_1,
        &market_wasm_hash,
        &market_admin,
        &name_1,
        &oracle,
        &insurance_fund,
        &default_params(false),
        &TEST_UPGRADE_IN_QUEUE_SECONDS,
    );

    // Replaces `assert!(market_list.contains_key(...))`. The per-key
    // membership probe is the post-`MarketsList`-removal way to ask
    // "did this manager deploy this market?".
    assert!(manager_client.is_deployed_by_manager(&market_address_1));

    let salt_2 = BytesN::from_array(&e, &[1; 32]);
    let name_2 = String::from_str(&e, "market_2");
    let market_address_2 = manager_client.deploy(
        &salt_2,
        &market_wasm_hash,
        &market_admin,
        &name_2,
        &oracle,
        &insurance_fund,
        &default_params(false),
        &TEST_UPGRADE_IN_QUEUE_SECONDS,
    );

    // Both deployments must report as deployed independently — the
    // per-key shape would still allow a buggy `register_market` that
    // only writes the *latest* deployment, so we have to re-check
    // market_1 after market_2 lands.
    assert!(manager_client.is_deployed_by_manager(&market_address_1));
    assert!(manager_client.is_deployed_by_manager(&market_address_2));

    // And an unrelated address still reports as not deployed — this is
    // the assertion that would have caught a buggy
    // `is_deployed_by_manager` that always returns true.
    let unrelated = Address::generate(&e);
    assert!(!manager_client.is_deployed_by_manager(&unrelated));
}

#[test]
fn test_manager_cannot_redeploy_market() {
    let ManagerSetup { e, manager_client, market_wasm_hash, .. } = ManagerSetup::new();

    let market_admin = Address::generate(&e);
    let oracle = Address::generate(&e);
    let insurance_fund = Address::generate(&e);

    let salt = BytesN::from_array(&e, &[0; 32]);
    let name_1 = String::from_str(&e, "market_1");
    manager_client.deploy(
        &salt,
        &market_wasm_hash,
        &market_admin,
        &name_1,
        &oracle,
        &insurance_fund,
        &default_params(false),
        &TEST_UPGRADE_IN_QUEUE_SECONDS,
    );

    let name_2 = String::from_str(&e, "market_2");

    assert!(
        manager_client
            .try_deploy(
                &salt,
                &market_wasm_hash,
                &market_admin,
                &name_2,
                &oracle,
                &insurance_fund,
                &default_params(false),
                &TEST_UPGRADE_IN_QUEUE_SECONDS,
            )
            .is_err()
    );
}

// ---- propose_admin / accept_admin ----

#[test]
fn test_propose_and_accept_admin() {
    let ManagerSetup { e, manager_client, .. } = ManagerSetup::new();

    let new_admin = Address::generate(&e);

    manager_client.propose_admin(&new_admin);
    manager_client.accept_admin();

    // `get_config().admin` no longer exists, so confirm the swap
    // behaviorally: an admin-gated entrypoint (`propose_admin`) called
    // again must now exercise `new_admin`'s auth, because
    // `require_admin` calls `storage::get_admin(...).require_auth()` on
    // whoever the current admin is. If the swap silently failed, the
    // auths list would still contain the original admin instead.
    let another_candidate = Address::generate(&e);
    manager_client.propose_admin(&another_candidate);
    let auths = e.auths();
    assert!(
        auths.iter().any(|(addr, _)| addr == &new_admin),
        "after accept_admin, the next admin-gated call must require new_admin's auth; recorded \
         auths: {:?}",
        auths,
    );
}

#[test]
fn test_accept_admin_without_proposal_fails() {
    let ManagerSetup { manager_client, .. } = ManagerSetup::new();

    let result = manager_client.try_accept_admin();

    assert_eq!(result, Err(Ok(market_manager::error::MMCError::NoPendingAdmin)));
}

#[test]
fn test_propose_admin_overwrites_previous_proposal() {
    let ManagerSetup { e, manager_client, .. } = ManagerSetup::new();

    let first_candidate = Address::generate(&e);
    let second_candidate = Address::generate(&e);

    manager_client.propose_admin(&first_candidate);
    manager_client.propose_admin(&second_candidate);

    manager_client.accept_admin();

    // Same trick as `test_propose_and_accept_admin`: with `get_config`
    // gone, observe the swap by recording whose auth the next
    // admin-gated call actually requires. If the overwrite didn't take
    // effect, `first_candidate`'s auth would show up here instead.
    let third_candidate = Address::generate(&e);
    manager_client.propose_admin(&third_candidate);
    let auths = e.auths();
    assert!(
        auths.iter().any(|(addr, _)| addr == &second_candidate),
        "overwrite + accept must leave second_candidate as the active admin; recorded auths: {:?}",
        auths,
    );
    assert!(
        !auths.iter().any(|(addr, _)| addr == &first_candidate),
        "first_candidate must not be the active admin after accept; recorded auths: {:?}",
        auths,
    );
}

#[test]
fn test_pending_admin_cleared_after_accept() {
    let ManagerSetup { e, manager_client, .. } = ManagerSetup::new();

    let new_admin = Address::generate(&e);
    manager_client.propose_admin(&new_admin);
    manager_client.accept_admin();

    let result = manager_client.try_accept_admin();
    assert_eq!(result, Err(Ok(market_manager::error::MMCError::NoPendingAdmin)));
}

#[test]
fn test_propose_admin_requires_admin_auth() {
    let ManagerSetup { e, manager_client, manager_admin, .. } = ManagerSetup::new();

    let new_admin = Address::generate(&e);
    manager_client.propose_admin(&new_admin);

    let auths = e.auths();
    assert!(
        auths.iter().any(|(addr, _)| addr == &manager_admin),
        "propose_admin must require the current admin's auth; recorded auths: {:?}",
        auths
    );
}

#[test]
fn test_manager_invalid_deploy() {
    let ManagerSetup { e, manager_client, market_wasm_hash, .. } = ManagerSetup::new();

    let market_admin = Address::generate(&e);
    let oracle = Address::generate(&e);
    let insurance_fund = Address::generate(&e);

    let salt = BytesN::from_array(&e, &[0; 32]);
    let name_1 = String::from_str(&e, "market_1");

    assert!(
        manager_client
            .try_deploy(
                &salt,
                &market_wasm_hash,
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
                &TEST_UPGRADE_IN_QUEUE_SECONDS,
            )
            .is_err(),
    );

    assert!(
        manager_client
            .try_deploy(
                &salt,
                &market_wasm_hash,
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
                &TEST_UPGRADE_IN_QUEUE_SECONDS,
            )
            .is_err(),
    );

    assert!(
        manager_client
            .try_deploy(
                &salt,
                &market_wasm_hash,
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
                &TEST_UPGRADE_IN_QUEUE_SECONDS,
            )
            .is_ok(),
    );
}
