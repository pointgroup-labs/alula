// Tests for `MarketManager::is_deployed_by_manager`.
//
// These tests cover the per-key `DataKey::DeployedMarket(Address)`
// storage shape that backs the manager's O(1) provenance check. The
// shape was introduced together with the removal of the old
// `MarketsList` instance-storage map, which used to be loaded in full
// on every `get_markets` / `register_market` call and could not scale
// past low thousands of markets under mainnet's per-tx budget. With
// the map gone, `is_deployed_by_manager` is the only entrypoint that
// answers "did this manager deploy this market?" and it must work in
// O(1) regardless of how many markets have ever been registered.
//
// What's intentionally NOT here:
//
//   * A `#[should_panic = "Budget, ExceededLimit"]` scaling demo. The
//     `MarketsList` blob it asserted against no longer exists, and
//     `is_deployed_by_manager`'s per-key shape has no equivalent
//     N-dependent failure mode to demonstrate.
//   * A "no `MarketsList` blob is written" storage probe. The
//     `DataKey::MarketsList` variant has been removed from the enum,
//     so the assertion is enforced by the type system: code that tries
//     to write that key fails to compile.

#![cfg(test)]

use market_manager::contract::{MarketInitParams, MarketManagerClient, MarketManagerContract};
use soroban_sdk::{Address, BytesN, Env, String, testutils::Address as _};

// The real market WASM, imported so `deploy` can actually instantiate
// a market in-test. Without this the host raises
// `Storage, MissingValue` on the deploy site, because `deploy`
// internally calls `Deployer::deploy_v2(market_wasm_hash, ...)` which
// looks up the previously-uploaded WASM by hash.
mod market {
    use soroban_sdk::contractimport;

    contractimport!(file = "../wasms/market.wasm");
}

// Per-market upgrade timelock used by these tests. Must be `>=` the
// manager's `MIN_UPGRADE_IN_QUEUE_SECONDS` (24h) or `deploy` will
// reject the call.
const TEST_UPGRADE_IN_QUEUE_SECONDS: u64 = 24 * 60 * 60;

fn dummy_market_init_params() -> MarketInitParams {
    MarketInitParams {
        max_positions: 5,
        min_collateral_value_cents: 500,
        insolvency_ltv_bps: 9_850,
        update_in_queue_period: 60,
        is_owned: false,
        bad_debt_lock_d: 0,
    }
}

fn setup() -> (Env, Address, MarketManagerClient<'static>, BytesN<32>) {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let market_wasm_hash = e.deployer().upload_contract_wasm(market::WASM);

    let manager_addr = e.register(MarketManagerContract, (admin.clone(),));
    let manager = MarketManagerClient::new(&e, &manager_addr);

    (e, admin, manager, market_wasm_hash)
}

#[test]
fn deployed_addresses_report_as_deployed() {
    let (e, _admin, manager, market_wasm_hash) = setup();

    let oracle = Address::generate(&e);
    let insurance_fund = Address::generate(&e);
    let market_admin = Address::generate(&e);

    let deployed: Vec<Address> = (0..3u8)
        .map(|i| {
            let mut salt_bytes = [0u8; 32];
            salt_bytes[0] = i;
            let salt = BytesN::from_array(&e, &salt_bytes);
            manager.deploy(
                &salt,
                &market_wasm_hash,
                &market_admin,
                &String::from_str(&e, "test-market"),
                &oracle,
                &insurance_fund,
                &dummy_market_init_params(),
                &TEST_UPGRADE_IN_QUEUE_SECONDS,
            )
        })
        .collect();

    for addr in &deployed {
        assert!(
            manager.is_deployed_by_manager(addr),
            "expected manager to report {:?} as deployed",
            addr,
        );
    }
}

#[test]
fn non_deployed_addresses_report_as_not_deployed() {
    let (e, _admin, manager, _wasm_hash) = setup();

    // Never went through `deploy`.
    let bogus = Address::generate(&e);

    assert!(
        !manager.is_deployed_by_manager(&bogus),
        "expected manager to report unknown address {:?} as not deployed",
        bogus,
    );
}
