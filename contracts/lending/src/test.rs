#![cfg(test)]

use {
    crate::{contract::*, storage::Obligation},
    soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, BytesN, Env},
};

#[test]
fn test_pool_initialization() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(LendingContract, (Address::generate(&e),));
    let client = LendingContractClient::new(&e, &contract_id);

    let token_address = Address::generate(&e);
    client.initialize_pool(&token_address, &None, &120);
    client.initialize_pool(
        &token_address,
        &Some(BytesN::from_array(&e, &[0; 32])),
        &120,
    );
}

#[test]
fn test_pool_initialization_with_different_name() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(LendingContract, (Address::generate(&e),));
    let client = LendingContractClient::new(&e, &contract_id);

    let token_address = Address::generate(&e);
    let salt1 = BytesN::from_array(&e, &[0; 32]);

    assert!(client
        .try_initialize_pool(&token_address, &Some(salt1), &0)
        .is_ok());

    let salt2 = BytesN::from_array(&e, &[1; 32]);
    assert!(client
        .try_initialize_pool(&token_address, &Some(salt2), &0)
        .is_ok());
}

#[test]
fn test_pool_not_conflicting_initializations() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(LendingContract, (Address::generate(&e),));
    let client = LendingContractClient::new(&e, &contract_id);

    let token_address1 = Address::generate(&e);
    let token_address2 = Address::generate(&e);

    assert!(client
        .try_initialize_pool(&token_address1, &None, &0)
        .is_ok());

    assert!(client
        .try_initialize_pool(&token_address2, &None, &0)
        .is_ok());
    let salt = BytesN::from_array(&e, &[0; 32]);

    assert!(client
        .try_initialize_pool(&token_address1, &Some(salt.clone()), &0)
        .is_ok());

    assert!(client
        .try_initialize_pool(&token_address2, &Some(salt), &0)
        .is_ok());
}

#[test]
#[should_panic]
fn test_pool_reinitialization_no_salt() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(LendingContract, (Address::generate(&e),));
    let client = LendingContractClient::new(&e, &contract_id);

    let token_address = Address::generate(&e);

    assert!(client
        .try_initialize_pool(&token_address, &None, &0)
        .is_ok());

    assert!(client
        .try_initialize_pool(&token_address, &None, &0)
        .is_ok());
}

#[test]
#[should_panic]
fn test_pool_reinitialization_with_salt() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(LendingContract, (Address::generate(&e),));
    let client = LendingContractClient::new(&e, &contract_id);

    let token_address = Address::generate(&e);
    let salt = BytesN::from_array(&e, &[0; 32]);

    assert!(client
        .try_initialize_pool(&token_address, &Some(salt.clone()), &0)
        .is_ok());

    assert!(client
        .try_initialize_pool(&token_address, &Some(salt.clone()), &0)
        .is_ok());
}

#[test]
fn test_pool_deposit() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(LendingContract, (Address::generate(&e),));
    let client = LendingContractClient::new(&e, &contract_id);

    let user = Address::generate(&e);
    let token_admin = Address::generate(&e);
    let token_address = e.register_stellar_asset_contract_v2(token_admin).address();
    let token_asset_client = StellarAssetClient::new(&e, &token_address);
    const DEPOSITED_AMOUNT: i128 = 100;
    token_asset_client.mint(&user, &DEPOSITED_AMOUNT);
    let pool_address = client.initialize_pool(&token_address, &None, &0);
    // Deposit token
    client.deposit(&user, &pool_address, &DEPOSITED_AMOUNT);
    // Check obligation
    let Obligation { deposits, .. } = client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address).unwrap();

    assert_eq!(deposited_amount, DEPOSITED_AMOUNT)
}
