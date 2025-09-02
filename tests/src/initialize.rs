#![cfg(test)]

use market::{
    LCError,
    contract::{MarketContract, MarketContractClient},
    storage,
};
use soroban_sdk::{Address, BytesN, symbol_short, testutils::Address as _};

use crate::get_default_env;

#[test]
fn test_pool_initialize() {
    let e = get_default_env();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        MarketContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = MarketContractClient::new(&e, &contract_id);

    let token_admin = Address::generate(&e);
    let token_address = e.register_stellar_asset_contract_v2(token_admin).address();
    let token_ticker = symbol_short!("TCK1");

    contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);

    // Initialize another pool for the same token with a provided salt
    contract_client.initialize_pool(
        &token_address,
        &token_ticker,
        &Some(BytesN::from_array(&e, &[0; 32])),
        &None,
    );
}

#[test]
fn test_pool_initialize_with_different_salt() {
    let e = get_default_env();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        MarketContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = MarketContractClient::new(&e, &contract_id);

    let token_admin = Address::generate(&e);
    let token_address = e.register_stellar_asset_contract_v2(token_admin).address();
    let token_ticker = symbol_short!("TCK1");

    let salt = BytesN::from_array(&e, &[0; 32]);
    let salt2 = BytesN::from_array(&e, &[1; 32]);

    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt), &None);
    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt2), &None);
}

#[test]
fn test_pool_initialize_non_conflicting() {
    let e = get_default_env();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        MarketContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = MarketContractClient::new(&e, &contract_id);

    let token_admin = Address::generate(&e);
    let token_address = e.register_stellar_asset_contract_v2(token_admin).address();
    let token_ticker = symbol_short!("TCK1");

    let token_admin2 = Address::generate(&e);
    let token_address2 = e.register_stellar_asset_contract_v2(token_admin2).address();
    let token_ticker2 = symbol_short!("TCK2");

    let salt = BytesN::from_array(&e, &[0; 32]);

    contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt.clone()), &None);

    contract_client.initialize_pool(&token_address2, &token_ticker2, &None, &None);
    contract_client.initialize_pool(&token_address2, &token_ticker2, &Some(salt), &None);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_pool_reinitialize_no_salt() {
    let e = get_default_env();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        MarketContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = MarketContractClient::new(&e, &contract_id);

    let token_admin = Address::generate(&e);
    let token_address = e.register_stellar_asset_contract_v2(token_admin).address();
    let token_ticker = symbol_short!("TCK1");

    contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
    contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_pool_reinitialize_with_salt() {
    let e = get_default_env();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        MarketContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = MarketContractClient::new(&e, &contract_id);

    let token_admin = Address::generate(&e);
    let token_address = e.register_stellar_asset_contract_v2(token_admin).address();
    let token_ticker = symbol_short!("TCK1");

    let salt = BytesN::from_array(&e, &[0; 32]);

    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt.clone()), &None);
    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt), &None);
}

#[test]
fn test_multiply_pair_initialize() {
    let e = get_default_env();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        MarketContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = MarketContractClient::new(&e, &contract_id);

    // Initialize pools first
    let deposit_token_admin = Address::generate(&e);
    let deposit_token_address = e
        .register_stellar_asset_contract_v2(deposit_token_admin)
        .address();
    let deposit_token_ticker = symbol_short!("TCK1");

    let borrow_token_admin = Address::generate(&e);
    let borrow_token_address = e
        .register_stellar_asset_contract_v2(borrow_token_admin)
        .address();
    let borrow_token_ticker = symbol_short!("TCK2");

    contract_client.initialize_pool(&deposit_token_address, &deposit_token_ticker, &None, &None);
    contract_client.initialize_pool(&borrow_token_address, &borrow_token_ticker, &None, &None);

    // Initialize a multiply pair
    contract_client.initialize_multiply_pair(&deposit_token_address, &borrow_token_address);

    // Check that pair is initialized
    e.as_contract(&contract_id, || {
        assert!(storage::multiply_pair_exists(
            &e,
            &deposit_token_address,
            &borrow_token_address,
        ));
    })
}

#[test]
fn test_multiply_pair_already_initialized() {
    let e = get_default_env();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        MarketContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = MarketContractClient::new(&e, &contract_id);

    // Initialize pools first
    let deposit_token_admin = Address::generate(&e);
    let deposit_token_address = e
        .register_stellar_asset_contract_v2(deposit_token_admin)
        .address();
    let deposit_token_ticker = symbol_short!("TCK1");

    let borrow_token_admin = Address::generate(&e);
    let borrow_token_address = e
        .register_stellar_asset_contract_v2(borrow_token_admin)
        .address();
    let borrow_token_ticker = symbol_short!("TCK2");

    contract_client.initialize_pool(&deposit_token_address, &deposit_token_ticker, &None, &None);
    contract_client.initialize_pool(&borrow_token_address, &borrow_token_ticker, &None, &None);

    // Initialize a multiply pair
    contract_client.initialize_multiply_pair(&deposit_token_address, &borrow_token_address);

    assert_eq!(
        contract_client.try_initialize_multiply_pair(&deposit_token_address, &borrow_token_address),
        Err(Ok(LCError::MultiplyPairAlreadyExists))
    );
}

#[test]
fn test_multiply_pair_with_inexistent_pool() {
    let e = get_default_env();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        MarketContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = MarketContractClient::new(&e, &contract_id);

    let borrow_pool_address = Address::generate(&e);
    let deposit_pool_address = Address::generate(&e);

    // Try to initialize a multiply pair
    assert_eq!(
        contract_client.try_initialize_multiply_pair(&deposit_pool_address, &borrow_pool_address),
        Err(Ok(LCError::DepositPoolDoesNotExist))
    );
}
