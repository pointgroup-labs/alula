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

// ---- No price outlives the call that read it ----

/// A later call in the same ledger reads the oracle again, so an update or an invalid price published
/// between two transactions of one ledger is seen at once instead of masked by the earlier read.
#[test]
fn test_a_price_change_within_the_ledger_is_seen_by_the_next_call() {
    let TestMarketFixture { e, contract_client, usdc_pool_address, oracle_client, .. } =
        TestMarketFixture::new();

    assert_eq!(contract_client.get_pool_asset_oracle_price(&usdc_pool_address), ONE_USD);

    make_oracle_prices_uniform(&e, &oracle_client, TWO_USD);
    assert_eq!(contract_client.get_pool_asset_oracle_price(&usdc_pool_address), TWO_USD);

    make_oracle_prices_zero(&e, &oracle_client);
    assert_eq!(
        contract_client.try_get_pool_asset_oracle_price(&usdc_pool_address),
        Err(Ok(MCError::NonPositiveOraclePrice))
    );
}

/// A borrow later in the same ledger as a collateral price drop is priced at the drop.
#[test]
fn test_a_borrow_after_a_same_ledger_price_drop_sees_the_drop() {
    let f = TestMarketFixture::new();
    let (lender, borrower) =
        (ObligationKey::new(f.users[0].clone()), ObligationKey::new(f.users[1].clone()));
    f.contract_client.deposit(&lender, &f.usdc_pool_address, &1_000_000_000, &None);
    f.contract_client.add_collateral(&borrower, &f.gold_pool_address, &100_000_000, &None);
    f.contract_client.borrow(&borrower, &f.usdc_pool_address, &1_000_000, &None);

    // GOLD loses 99 % in the same ledger: the second borrow must be priced at it.
    f.oracle_client.set_price_stable(&soroban_sdk::vec![&f.e, ONE_USD / 100, ONE_USD, ONE_USD]);
    assert_eq!(
        f.contract_client.try_borrow(&borrower, &f.usdc_pool_address, &1_000_000, &None),
        Err(Ok(MCError::UnhealthyOperation))
    );
}
