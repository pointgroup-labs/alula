#![cfg(test)]

use {
    crate::contract::*,
    soroban_sdk::{symbol_short, testutils::Address as _, Address, Env},
};

#[test]
fn test_pool_initialization() {
    // @TODO: Move initialization out
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(LendingContract, (Address::generate(&e),));
    let client = LendingContractClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    let token_address = Address::generate(&e);
    let pool_name = symbol_short!("name");

    assert!(client
        .try_initialize_pool(&pool_name, &admin, &token_address, &0)
        .is_ok());
}

#[test]
fn test_pool_initialization_with_different_name() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(LendingContract, (Address::generate(&e),));
    let client = LendingContractClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    let token_address = Address::generate(&e);
    let pool_name = symbol_short!("name1");

    assert!(client
        .try_initialize_pool(&pool_name, &admin, &token_address, &0)
        .is_ok());

    let pool_name2 = symbol_short!("name2");
    assert!(client
        .try_initialize_pool(&pool_name2, &admin, &token_address, &0)
        .is_ok());
}

#[test]
fn test_pool_initialization_with_different_token_address() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(LendingContract, (Address::generate(&e),));
    let client = LendingContractClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    let token_address1 = Address::generate(&e);
    let pool_name = symbol_short!("name");

    assert!(client
        .try_initialize_pool(&pool_name, &admin, &token_address1, &0)
        .is_ok());

    let token_address2 = Address::generate(&e);
    assert!(client
        .try_initialize_pool(&pool_name, &admin, &token_address2, &0)
        .is_ok());
}

#[test]
#[should_panic]
fn test_pool_is_already_initialized() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(LendingContract, (Address::generate(&e),));
    let client = LendingContractClient::new(&e, &contract_id);
    let admin1 = Address::generate(&e);
    let token_address = Address::generate(&e);
    let pool_name = symbol_short!("name");

    assert!(client
        .try_initialize_pool(&pool_name, &admin1, &token_address, &0)
        .is_ok());
    let admin2 = Address::generate(&e);

    assert!(client
        .try_initialize_pool(&pool_name, &admin2, &token_address, &0)
        .is_ok())
}
