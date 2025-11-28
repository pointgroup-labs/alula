#![cfg(test)]

use market::{constants::SECONDS_IN_YEAR, error::MCError, obligation::ObligationKey};
use soroban_sdk::{
    Address,
    testutils::{Address as _, Ledger},
};

use crate::{DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, get_obligation_received_interest};

#[test]
fn test_obligation_does_not_exist_prior_anything() {
    let TestMarketFixture { users, contract_client, .. } = TestMarketFixture::new();

    let user = &users[0];
    let obligation = contract_client.try_get_user_obligation(user);

    assert_eq!(obligation, Err(Ok(MCError::ObligationDoesNotExist)));
}

#[test]
fn test_pool_with_random_address_does_not_exist() {
    let TestMarketFixture { e, contract_client, .. } = TestMarketFixture::new();

    let rand_addr = Address::generate(&e);
    let res = contract_client.try_get_pool(&rand_addr);

    assert_eq!(res, Err(Ok(MCError::PoolDoesNotExist)));
}

#[test]
fn test_pool_is_empty_prior_anything() {
    let TestMarketFixture { contract_client, usdc_pool_address, .. } = TestMarketFixture::new();

    let pool = contract_client.get_pool(&usdc_pool_address);

    assert_eq!(pool.total_available, 0);
    assert_eq!(pool.total_borrowed, 0);
    assert_eq!(pool.total_j_tokens, 0);
    assert_eq!(pool.total_d_tokens, 0);
    assert_eq!(pool.total_collateral, 0);
}

#[test]
fn test_reset_storage_removes_obligations() {
    let TestMarketFixture { contract_client, usdc_pool_address, users, .. } =
        TestMarketFixture::new();

    let user1 = &users[0];
    let user2 = &users[2];

    assert!(contract_client.get_all_obligations().is_empty());

    contract_client.deposit(user1, &usdc_pool_address, &1000);
    contract_client.deposit(user2, &usdc_pool_address, &1000);

    assert!(contract_client.try_get_user_obligation(user1).is_ok());
    assert!(contract_client.try_get_user_obligation(user2).is_ok());
    assert_eq!(contract_client.get_all_obligations().len(), 2);

    contract_client.reset_storage();

    assert!(contract_client.try_get_user_obligation(user1).is_err());
    assert!(contract_client.try_get_user_obligation(user2).is_err());
    assert!(contract_client.get_all_obligations().is_empty());
}

#[test]
fn test_reset_storage_removes_pool() {
    let TestMarketFixture { contract_client, .. } = TestMarketFixture::new();

    assert_eq!(contract_client.get_all_pools().len(), 3); // NB: 3 pools are set initially

    contract_client.reset_storage();

    assert!(contract_client.get_all_pools().is_empty());
}

#[test]
fn test_reset_storage_removes_multiply_pairs() {
    let TestMarketFixture { contract_client, usdc_pool_address, btc_pool_address, .. } =
        TestMarketFixture::new();

    assert_eq!(contract_client.get_all_multiply_pairs().len(), 1); // NB: 1 pair is set initially

    contract_client.initialize_multiply_pair(&usdc_pool_address, &btc_pool_address);

    assert_eq!(contract_client.get_all_multiply_pairs().len(), 2);

    contract_client.reset_storage();

    assert!(contract_client.get_all_multiply_pairs().is_empty());
}

#[test]
fn test_obligations_list_contains_unique_obligations() {
    let TestMarketFixture { contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let liquidity_provider = &users[0];
    let creditor = &users[1];

    contract_client.deposit(liquidity_provider, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligations = contract_client.get_all_obligations();
    assert_eq!(obligations.len(), 2);
    assert!(obligations.contains(ObligationKey::new(creditor.clone())));

    contract_client.withdraw(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligations = contract_client.get_all_obligations();
    assert_eq!(obligations.len(), 1);
    assert!(!obligations.contains(ObligationKey::new(creditor.clone())));
}

#[test]
fn test_bootstrap_pool() {
    let TestMarketFixture {
        e,
        contract_client,
        gold_pool_address,
        gold_token_client,
        contract_id,
        users,
        ..
    } = TestMarketFixture::new();
    let liquidity_provider = &users[0];
    let creditor_1 = &users[1];
    let creditor_2 = &users[2];

    contract_client.deposit(creditor_1, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(creditor_2, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    // -- Move time --

    e.ledger().with_mut(|li| li.timestamp += SECONDS_IN_YEAR);
    contract_client.refresh_pool(&gold_pool_address);

    // -- Assert no received interest has accrued due to 0% utilization --

    let received_interest_1 =
        get_obligation_received_interest(&e, &contract_client, &creditor_1, &gold_pool_address)
            .unwrap();
    let received_interest_2 =
        get_obligation_received_interest(&e, &contract_client, &creditor_2, &gold_pool_address)
            .unwrap();

    assert_eq!(received_interest_1, 0);
    assert_eq!(received_interest_2, 0);

    // -- Bootstrap pool --

    gold_token_client.approve(
        &liquidity_provider,
        &contract_id,
        &DEFAULT_DEPOSIT_AMOUNT,
        &(e.ledger().sequence()),
    );

    contract_client.bootstrap_pool(
        &gold_pool_address,
        &liquidity_provider,
        &DEFAULT_DEPOSIT_AMOUNT,
        &e.ledger().timestamp(),
        &(e.ledger().timestamp() + SECONDS_IN_YEAR),
    );

    // -- Assert half of bootstrapped value has accrued --

    e.ledger().with_mut(|li| li.timestamp += SECONDS_IN_YEAR / 4);
    contract_client.refresh_pool(&gold_pool_address);

    let received_interest_1 =
        get_obligation_received_interest(&e, &contract_client, &creditor_1, &gold_pool_address)
            .unwrap();
    let received_interest_2 =
        get_obligation_received_interest(&e, &contract_client, &creditor_2, &gold_pool_address)
            .unwrap();

    assert_eq!(received_interest_1, received_interest_2);
    assert_eq!(
        received_interest_1.checked_add(received_interest_2).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT / 2
    );
}
