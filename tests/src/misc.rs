#![cfg(test)]

use market::LCError;
use soroban_sdk::{Address, testutils::Address as _};

use crate::TestFixture;

#[test]
fn test_obligation_does_not_exist_prior_anything() {
    let TestFixture {
        users,
        contract_client,
        ..
    } = TestFixture::new();

    let user = &users[0];

    let obligation = contract_client.try_get_user_obligation(user);
    assert_eq!(obligation, Err(Ok(LCError::ObligationDoesNotExist)));
}

#[test]
fn test_pool_with_random_address_does_not_exist() {
    let TestFixture {
        e, contract_client, ..
    } = TestFixture::new();

    let rand_addr = Address::generate(&e);
    let res = contract_client.try_get_pool(&rand_addr);

    assert_eq!(res, Err(Ok(LCError::PoolDoesNotExist)));
}

#[test]
fn test_pool_is_empty_prior_anything() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        ..
    } = TestFixture::new();

    let pool = contract_client.get_pool(&usdc_pool_address);

    assert_eq!(pool.total_borrowed, 0);
    assert_eq!(pool.total_collateral, 0);
    assert_eq!(pool.total_j_tokens_amount, 0);
    assert_eq!(pool.available, 0);
}

#[test]
fn test_remove_obligation() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = &users[0];

    assert!(contract_client.try_get_user_obligation(user).is_err());

    contract_client.deposit(user, &usdc_pool_address, &1000);
    assert!(contract_client.try_get_user_obligation(user).is_ok());

    contract_client.reset_storage();
    assert!(contract_client.try_get_user_obligation(user).is_err());
}

#[test]
fn test_remove_many_obligations() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

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
fn test_remove_pool() {
    let TestFixture {
        contract_client, ..
    } = TestFixture::new();

    assert_eq!(contract_client.get_all_pools().len(), 3);

    contract_client.reset_storage();

    assert!(contract_client.get_all_pools().is_empty());
}

#[test]
fn test_remove_multiply_pairs() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        btc_pool_address,
        ..
    } = TestFixture::new();

    assert_eq!(contract_client.get_all_multiply_pairs().len(), 1); // 1 pair is set initially

    contract_client.initialize_multiply_pair(&usdc_pool_address, &btc_pool_address);

    assert_eq!(contract_client.get_all_multiply_pairs().len(), 2);

    contract_client.reset_storage();

    assert!(contract_client.get_all_multiply_pairs().is_empty());
}
