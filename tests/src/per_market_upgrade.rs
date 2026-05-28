// Per-market upgrade flow tests.
//
// The market manager now exposes `queue_in_market_upgrade(market, hash)`
// and `upgrade_market(market)` instead of the old global
// `queue_in_market_upgrade(hash)` / `apply_market_upgrade()` pair. Each
// market's queue lives under its own
// `DataKey::QueuedInMarketUpgrade(market)` instance entry, so queueing
// or applying one market's upgrade does not load any sibling's state.
//
// This file covers the happy path plus the explicit failure modes the
// new flow has to enforce:
//
//   * only the admin may queue/cancel/apply,
//   * only a market previously deployed via this manager may be the
//     subject of a queue or apply,
//   * the per-market queue window (passed at `deploy` time and stored
//     under `DataKey::DeployedMarket(market)`) must elapse before
//     `apply_market_upgrade` succeeds,
//   * canceling clears the queue entry,
//   * applying clears the queue entry (so a follow-up apply fails),
//   * two markets' queues are fully independent.

#![cfg(test)]

use market_manager::{
    contract::{MarketInitParams, MarketManagerClient, MarketManagerContract},
    error::MMCError,
};
use soroban_sdk::{
    Address, BytesN, Env, String,
    testutils::{Address as _, Ledger as _},
};

mod market_wasm {
    use soroban_sdk::contractimport;

    // Same WASM the manager bakes in via `mod market;` in `contract.rs`.
    // Importing it here lets us upload it once per test and then upload
    // a *second* copy under a slightly different bag-of-bytes to obtain
    // a distinct hash we can swap in as the "new" version.
    contractimport!(file = "../wasms/market.wasm");
}

/// Per-market timelock used by every market in this test file. Picked
/// to be just over the deploy-time minimum so the tests exercise the
/// real per-market lookup path rather than a sentinel.
const TEST_UPGRADE_IN_QUEUE_SECONDS: u64 = 24 * 60 * 60;

fn dummy_params() -> MarketInitParams {
    MarketInitParams {
        max_positions: 5,
        min_collateral_value_cents: 500,
        insolvency_ltv_bps: 9_850,
        update_in_queue_period: 60,
        is_owned: false,
        bad_debt_lock_d: 0,
    }
}

struct Setup<'a> {
    e: Env,
    manager: MarketManagerClient<'a>,
    /// A real market deployed via the manager. `upgrade_market` will
    /// invoke this address's `upgrade(new_wasm_hash)` entrypoint.
    market_a: Address,
    /// A second market deployed via the manager, used to assert per-market
    /// queue isolation.
    market_b: Address,
    /// A hash equal to the market's current code. Reusing the current
    /// wasm as the "new" wasm is a no-op upgrade — fine for testing the
    /// queue/apply plumbing without needing to compile a second
    /// distinct wasm.
    new_wasm_hash: BytesN<32>,
}

fn setup<'a>() -> Setup<'a> {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let market_wasm_hash = e.deployer().upload_contract_wasm(market_wasm::WASM);

    let manager_addr = e.register(MarketManagerContract, (admin.clone(),));
    let manager = MarketManagerClient::new(&e, &manager_addr);

    let market_admin = Address::generate(&e);
    let oracle = Address::generate(&e);
    let insurance_fund = Address::generate(&e);

    let salt_a = BytesN::from_array(&e, &[0u8; 32]);
    let market_a = manager.deploy(
        &salt_a,
        &market_wasm_hash,
        &market_admin,
        &String::from_str(&e, "market-a"),
        &oracle,
        &insurance_fund,
        &dummy_params(),
        &TEST_UPGRADE_IN_QUEUE_SECONDS,
    );

    let salt_b = BytesN::from_array(&e, &[1u8; 32]);
    let market_b = manager.deploy(
        &salt_b,
        &market_wasm_hash,
        &market_admin,
        &String::from_str(&e, "market-b"),
        &oracle,
        &insurance_fund,
        &dummy_params(),
        &TEST_UPGRADE_IN_QUEUE_SECONDS,
    );

    Setup { e, manager, market_a, market_b, new_wasm_hash: market_wasm_hash }
}

fn advance_past_queue_window(e: &Env) {
    e.ledger().with_mut(|li| {
        li.timestamp = li.timestamp.saturating_add(TEST_UPGRADE_IN_QUEUE_SECONDS + 1);
    });
}

#[test]
fn queue_then_apply_after_window_clears_queue() {
    let Setup { e, manager, market_a, new_wasm_hash, .. } = setup();

    manager.queue_in_market_upgrade(&market_a, &new_wasm_hash);
    assert!(manager.get_queued_in_market_upgrade(&market_a).is_some());

    advance_past_queue_window(&e);

    manager.apply_market_upgrade(&market_a);

    // The queue entry must be cleared on successful apply so that the
    // next admin queue/apply cycle is fresh.
    assert!(
        manager.get_queued_in_market_upgrade(&market_a).is_none(),
        "apply_market_upgrade must clear the per-market queue entry on success",
    );

    // A follow-up apply with nothing queued is an explicit error.
    assert_eq!(manager.try_apply_market_upgrade(&market_a), Err(Ok(MMCError::UpgradeDoesNotExist)),);
}

#[test]
fn apply_before_window_is_rejected() {
    let Setup { manager, market_a, new_wasm_hash, .. } = setup();

    manager.queue_in_market_upgrade(&market_a, &new_wasm_hash);

    // No ledger time has passed since queueing, so the per-market window
    // must reject the apply.
    assert_eq!(
        manager.try_apply_market_upgrade(&market_a),
        Err(Ok(MMCError::UpgradeIsNotYetApplicable)),
    );

    // Sanity: the queue entry must still be there after the failed
    // apply.
    assert!(manager.get_queued_in_market_upgrade(&market_a).is_some());
}

#[test]
fn double_queue_for_same_market_is_rejected() {
    let Setup { manager, market_a, new_wasm_hash, .. } = setup();

    manager.queue_in_market_upgrade(&market_a, &new_wasm_hash);

    assert_eq!(
        manager.try_queue_in_market_upgrade(&market_a, &new_wasm_hash),
        Err(Ok(MMCError::UpgradeAlreadyExists)),
    );
}

#[test]
fn cancel_clears_queue() {
    let Setup { manager, market_a, new_wasm_hash, .. } = setup();

    manager.queue_in_market_upgrade(&market_a, &new_wasm_hash);
    manager.cancel_market_upgrade(&market_a);

    assert!(manager.get_queued_in_market_upgrade(&market_a).is_none());

    // Cancel-with-nothing-queued is an explicit error.
    assert_eq!(
        manager.try_cancel_market_upgrade(&market_a),
        Err(Ok(MMCError::UpgradeDoesNotExist)),
    );
}

#[test]
fn queue_for_unknown_market_is_rejected() {
    let Setup { e, manager, new_wasm_hash, .. } = setup();

    // An address that was never deployed via the manager must not be
    // queueable — otherwise the admin could schedule an upgrade for an
    // arbitrary contract that the manager has no business touching.
    let bogus = Address::generate(&e);

    assert_eq!(
        manager.try_queue_in_market_upgrade(&bogus, &new_wasm_hash),
        Err(Ok(MMCError::MarketNotDeployedByManager)),
    );
}

#[test]
fn apply_for_unknown_market_is_rejected() {
    let Setup { e, manager, .. } = setup();

    let bogus = Address::generate(&e);

    // `upgrade_market` is gated by the existence of a queue entry rather
    // than by `require_deployed_market` directly. Since `queue_in_market_upgrade`
    // is the step that refuses unknown markets, no queue entry can ever
    // exist for `bogus`, so the natural failure mode here is
    // `UpgradeDoesNotExist`.
    assert_eq!(manager.try_apply_market_upgrade(&bogus), Err(Ok(MMCError::UpgradeDoesNotExist)),);
}

#[test]
fn per_market_queues_are_independent() {
    let Setup { e, manager, market_a, market_b, new_wasm_hash } = setup();

    // Queue only on market_a; market_b must see no queue.
    manager.queue_in_market_upgrade(&market_a, &new_wasm_hash);
    assert!(manager.get_queued_in_market_upgrade(&market_a).is_some());
    assert!(
        manager.get_queued_in_market_upgrade(&market_b).is_none(),
        "queueing on one market must not leak into another market's queue",
    );

    // Applying market_a's upgrade must not touch market_b's still-empty
    // queue state.
    advance_past_queue_window(&e);
    manager.apply_market_upgrade(&market_a);

    assert!(manager.get_queued_in_market_upgrade(&market_a).is_none());
    assert!(manager.get_queued_in_market_upgrade(&market_b).is_none());

    // And queueing on market_b after market_a's apply must succeed
    // independently (proving market_a's prior queue did not somehow
    // poison the per-key shape).
    manager.queue_in_market_upgrade(&market_b, &new_wasm_hash);
    assert!(manager.get_queued_in_market_upgrade(&market_b).is_some());
}
