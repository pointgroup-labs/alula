#![cfg(test)]

use market::{
    constants::{INDIVIDUAL_BUMP, INSTANCE_BUMP, LEDGERS_PER_DAY, SHARED_BUMP},
    obligation::ObligationKey,
    storage::DataKey,
};
use soroban_sdk::testutils::{
    Ledger,
    storage::{Instance, Persistent},
};

use crate::TestMarketFixture;

#[test]
fn test_storage_ttl_extension() {
    let TestMarketFixture {
        e,
        contract_client,
        contract_id,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();

    let user = &users[0];
    let obligation_key = ObligationKey::new(user.clone());

    e.as_contract(&contract_id, || {
        assert_eq!(e.storage().instance().get_ttl(), INSTANCE_BUMP);
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
            SHARED_BUMP
        );
    });

    // Extend individual user's storage
    contract_client.deposit(user, &usdc_pool_address, &1);

    e.as_contract(&contract_id, || {
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Obligation(obligation_key.clone())),
            INDIVIDUAL_BUMP
        );
    });

    e.ledger().with_mut(|li| {
        li.sequence_number += 2 * LEDGERS_PER_DAY;
    });

    e.as_contract(&contract_id, || {
        assert_eq!(
            e.storage().instance().get_ttl(),
            INSTANCE_BUMP - 2 * LEDGERS_PER_DAY
        );
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
            SHARED_BUMP - 2 * LEDGERS_PER_DAY
        );

        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Obligation(obligation_key.clone())),
            INDIVIDUAL_BUMP - 2 * LEDGERS_PER_DAY
        );
    });

    // Extend instance storage
    contract_client.get_global_state();

    e.as_contract(&contract_id, || {
        // Instance's TTL is bumped
        assert_eq!(e.storage().instance().get_ttl(), INSTANCE_BUMP);

        // Others aren't
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
            SHARED_BUMP - 2 * LEDGERS_PER_DAY
        );

        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Obligation(obligation_key.clone())),
            INDIVIDUAL_BUMP - 2 * LEDGERS_PER_DAY
        );
    });

    // Deposit once more to bump shared persistent token storage
    contract_client.deposit(user, &usdc_pool_address, &1);

    e.as_contract(&contract_id, || {
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
            SHARED_BUMP
        );

        // Individual persistent storage TTL is still the same, since it has
        // more ledgers between threshold and bump
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Obligation(obligation_key.clone())),
            INDIVIDUAL_BUMP - 2 * LEDGERS_PER_DAY
        );
    });

    e.ledger().with_mut(|li| {
        li.sequence_number += 20 * LEDGERS_PER_DAY;
    });

    e.as_contract(&contract_id, || {
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Obligation(obligation_key.clone())),
            INDIVIDUAL_BUMP - 22 * LEDGERS_PER_DAY
        );
    });

    // Read user's obligation to bump individual user's storage TTL
    contract_client.deposit(user, &usdc_pool_address, &1);

    e.as_contract(&contract_id, || {
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Obligation(obligation_key.clone())),
            INDIVIDUAL_BUMP
        );
    });
}
