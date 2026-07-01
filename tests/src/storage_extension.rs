#![cfg(test)]

use market::{constants::*, error::MCError, obligation::ObligationKey, storage::DataKey};
use soroban_sdk::testutils::{
    Ledger,
    storage::{Instance, Persistent},
};

use crate::{TestMarketFixture, make_oracle_prices_uniform, make_oracle_prices_zero};

const ONE_USD: i128 = 1_00000000000000;
const TWO_USD: i128 = 2_00000000000000;

#[test]
fn test_initial_shared_ttl() {
    let TestMarketFixture { e, contract_id, usdc_pool_address, .. } = TestMarketFixture::new();

    e.as_contract(&contract_id, || {
        assert_eq!(e.storage().instance().get_ttl(), INSTANCE_BUMP);
        assert_eq!(
            e.storage().persistent().get_ttl(&DataKey::Pool(usdc_pool_address)),
            SHARED_BUMP
        );
    });
}

#[test]
fn test_individual_bump_on_deposit_ttl() {
    let TestMarketFixture { e, contract_client, contract_id, usdc_pool_address, users, .. } =
        TestMarketFixture::new();

    let obligation_key = ObligationKey::new(users[0].clone());

    contract_client.deposit(&obligation_key, &usdc_pool_address, &1, &None);

    e.as_contract(&contract_id, || {
        assert_eq!(
            e.storage().persistent().get_ttl(&DataKey::Obligation(obligation_key)),
            INDIVIDUAL_BUMP
        );
    });
}

#[test]
fn test_ttl_decay_over_time() {
    let TestMarketFixture { e, contract_client, contract_id, usdc_pool_address, users, .. } =
        TestMarketFixture::new();

    let obligation_key = ObligationKey::new(users[0].clone());
    contract_client.deposit(&obligation_key, &usdc_pool_address, &1, &None);

    e.ledger().with_mut(|li| {
        li.sequence_number += 2 * LEDGERS_PER_DAY;
    });

    e.as_contract(&contract_id, || {
        assert_eq!(e.storage().instance().get_ttl(), INSTANCE_BUMP - 2 * LEDGERS_PER_DAY);
        assert_eq!(
            e.storage().persistent().get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
            SHARED_BUMP - 2 * LEDGERS_PER_DAY
        );
        assert_eq!(
            e.storage().persistent().get_ttl(&DataKey::Obligation(obligation_key)),
            INDIVIDUAL_BUMP - 2 * LEDGERS_PER_DAY
        );
    });
}

#[test]
fn test_isolated_instance_ttl_bump() {
    let TestMarketFixture { e, contract_client, contract_id, usdc_pool_address, users, .. } =
        TestMarketFixture::new();

    let obligation_key = ObligationKey::new(users[0].clone());
    contract_client.deposit(&obligation_key, &usdc_pool_address, &1, &None);

    e.ledger().with_mut(|li| li.sequence_number += 2 * LEDGERS_PER_DAY);

    // Affects only affect instance storage
    contract_client.update_market_status(&0);

    e.as_contract(&contract_id, || {
        // Instance is bumped
        assert_eq!(e.storage().instance().get_ttl(), INSTANCE_BUMP);

        // Pool and Obligation TTLs are NOT bumped
        assert_eq!(
            e.storage().persistent().get_ttl(&DataKey::Pool(usdc_pool_address)),
            SHARED_BUMP - 2 * LEDGERS_PER_DAY
        );
    });
}

#[test]
fn test_persistent_ttl_bump() {
    let TestMarketFixture { e, contract_client, contract_id, usdc_pool_address, users, .. } =
        TestMarketFixture::new();

    let obligation_key = ObligationKey::new(users[0].clone());

    contract_client.deposit(&obligation_key, &usdc_pool_address, &1, &None);
    e.ledger().with_mut(|li| li.sequence_number += 2 * LEDGERS_PER_DAY);

    // Second deposit: Should bump Shared (Pool) but leave Individual (Obligation) decaying.
    contract_client.deposit(&obligation_key, &usdc_pool_address, &1, &None);

    e.as_contract(&contract_id, || {
        assert_eq!(
            e.storage().persistent().get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
            SHARED_BUMP
        );
        assert_eq!(
            e.storage().persistent().get_ttl(&DataKey::Obligation(obligation_key.clone())),
            INDIVIDUAL_BUMP - 2 * LEDGERS_PER_DAY
        );
    });

    // Advance bump past threshold
    e.ledger().with_mut(|li| li.sequence_number += 20 * LEDGERS_PER_DAY);

    // Final deposit to trigger the individual bump
    contract_client.deposit(&obligation_key, &usdc_pool_address, &1, &None);

    e.as_contract(&contract_id, || {
        assert_eq!(
            e.storage().persistent().get_ttl(&DataKey::Obligation(obligation_key)),
            INDIVIDUAL_BUMP
        );
    });
}

// ---- Same-ledger oracle price cache ----

#[test]
fn test_price_cache_pins_value_within_ledger() {
    let TestMarketFixture { e, contract_client, usdc_pool_address, oracle_client, .. } =
        TestMarketFixture::new();

    let pinned = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);
    assert_eq!(pinned, ONE_USD);

    make_oracle_prices_uniform(&e, &oracle_client, TWO_USD);

    // The pinned value is returned; the same-ledger update is ignored by design.
    let second = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);
    assert_eq!(second, pinned, "same-ledger read must return the pinned price, not the update");
}

#[test]
fn test_price_cache_refreshes_on_next_ledger() {
    let TestMarketFixture { e, contract_client, usdc_pool_address, oracle_client, .. } =
        TestMarketFixture::new();

    let first = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);
    assert_eq!(first, ONE_USD);

    // New ledger: bump the timestamp and publish the new price at that time.
    e.ledger().with_mut(|li| li.timestamp += 1);
    make_oracle_prices_uniform(&e, &oracle_client, TWO_USD);

    let second = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);
    assert_eq!(second, TWO_USD, "a new ledger must bypass the stale cache and observe the update");
}

#[test]
fn test_price_cache_lives_in_temporary_storage_keyed_by_ledger_timestamp() {
    let TestMarketFixture { e, contract_client, contract_id, usdc_pool_address, .. } =
        TestMarketFixture::new();

    let now = e.ledger().timestamp();
    let price = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);

    e.as_contract(&contract_id, || {
        let key = DataKey::CachedPrice(usdc_pool_address.clone());

        // Stored in temporary storage as (price, current_ledger_timestamp).
        let cached: Option<(i128, u64)> = e.storage().temporary().get(&key);
        assert_eq!(cached, Some((price, now)));

        assert!(!e.storage().instance().has(&key));
        assert!(!e.storage().persistent().has(&key));
    });
}

#[test]
fn test_price_cache_masks_invalid_price_until_next_ledger() {
    let TestMarketFixture { e, contract_client, usdc_pool_address, oracle_client, .. } =
        TestMarketFixture::new();

    // Cache a valid price in this ledger
    let cached = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);
    assert_eq!(cached, ONE_USD);

    // Oracle turns invalid within the same ledger (prints a non-positive price)
    make_oracle_prices_zero(&e, &oracle_client);

    // Same ledger: the cached valid price masks the invalid oracle state.
    let still_cached = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);
    assert_eq!(still_cached, cached, "same-ledger invalid price must be masked by the cache");

    // Next ledger: the cache is bypassed and the invalid price is rejected
    e.ledger().with_mut(|li| li.timestamp += 1);
    assert_eq!(
        contract_client.try_get_pool_asset_oracle_price(&usdc_pool_address),
        Err(Ok(MCError::NonPositiveOraclePrice)),
        "the next ledger must re-fetch and reject the invalid price",
    );
}

// ---- White-box cache validity, observed by reading storage directly ----

fn read_cached_price(
    e: &soroban_sdk::Env,
    contract_id: &soroban_sdk::Address,
    asset: &soroban_sdk::Address,
) -> Option<(i128, u64)> {
    e.as_contract(contract_id, || e.storage().temporary().get(&DataKey::CachedPrice(asset.clone())))
}

#[test]
fn test_price_cache_is_populated_by_getter_with_current_timestamp() {
    let TestMarketFixture { e, contract_client, contract_id, usdc_pool_address, .. } =
        TestMarketFixture::new();

    let now = e.ledger().timestamp();

    // Cache starts empty: nothing in the fixture setup reads a price.
    assert_eq!(
        read_cached_price(&e, &contract_id, &usdc_pool_address),
        None,
        "cache must be empty before the first read",
    );

    // The getter is the only writer; it persists the fetched price at `now`.
    let price = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);
    assert_eq!(price, ONE_USD);
    assert_eq!(
        read_cached_price(&e, &contract_id, &usdc_pool_address),
        Some((ONE_USD, now)),
        "getter must persist (price, current_timestamp) via set_cached_price",
    );
}

#[test]
fn test_price_cache_hit_serves_stored_value_without_rewriting() {
    let TestMarketFixture {
        e, contract_client, contract_id, usdc_pool_address, oracle_client, ..
    } = TestMarketFixture::new();

    let now = e.ledger().timestamp();

    // Populate the cache through the getter (the only writer).
    assert_eq!(contract_client.get_pool_asset_oracle_price(&usdc_pool_address), ONE_USD);
    assert_eq!(read_cached_price(&e, &contract_id, &usdc_pool_address), Some((ONE_USD, now)));

    // Move the oracle within the same ledger. A hit must ignore this entirely.
    make_oracle_prices_uniform(&e, &oracle_client, TWO_USD);

    let price = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);
    assert_eq!(price, ONE_USD, "entry keyed to the current timestamp must be a cache hit");

    // The hit returned early, so storage still holds the original tuple.
    assert_eq!(
        read_cached_price(&e, &contract_id, &usdc_pool_address),
        Some((ONE_USD, now)),
        "a cache hit must not rewrite the stored entry",
    );
}

#[test]
fn test_price_cache_invalidated_when_stored_timestamp_is_stale() {
    let TestMarketFixture {
        e, contract_client, contract_id, usdc_pool_address, oracle_client, ..
    } = TestMarketFixture::new();

    let first_ledger = e.ledger().timestamp();

    assert_eq!(contract_client.get_pool_asset_oracle_price(&usdc_pool_address), ONE_USD);
    assert_eq!(
        read_cached_price(&e, &contract_id, &usdc_pool_address),
        Some((ONE_USD, first_ledger)),
    );

    // Next ledger: the stored timestamp is now stale (stored < current), and the
    // oracle has published a new price.
    e.ledger().with_mut(|li| li.timestamp += 1);
    let second_ledger = e.ledger().timestamp();
    make_oracle_prices_uniform(&e, &oracle_client, TWO_USD);

    // The stale entry is bypassed; the fresh oracle price is returned...
    let price = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);
    assert_eq!(price, TWO_USD, "a stored timestamp != now must invalidate the cache");

    // ...and the getter overwrote the entry with the new (price, timestamp).
    assert_eq!(
        read_cached_price(&e, &contract_id, &usdc_pool_address),
        Some((TWO_USD, second_ledger)),
        "a miss must refresh the entry at the current timestamp",
    );
}
