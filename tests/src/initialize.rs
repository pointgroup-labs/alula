use {
    lending::contract::{LendingContract, LendingContractClient},
    soroban_sdk::{symbol_short, testutils::Address as _, Address, BytesN, Env},
};

#[test]
fn test_pool_initialize() {
    let e = Env::default();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        LendingContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = LendingContractClient::new(&e, &contract_id);

    let token_address = Address::generate(&e);
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
    let e = Env::default();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        LendingContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = LendingContractClient::new(&e, &contract_id);

    let token_address = Address::generate(&e);
    let token_ticker = symbol_short!("TCK1");

    let salt = BytesN::from_array(&e, &[0; 32]);
    let salt2 = BytesN::from_array(&e, &[1; 32]);

    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt), &None);
    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt2), &None);
}

#[test]
fn test_pool_initialize_non_conflicting() {
    let e = Env::default();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        LendingContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = LendingContractClient::new(&e, &contract_id);

    let token_address = Address::generate(&e);
    let token_ticker = symbol_short!("TCK1");

    let token_address_2 = Address::generate(&e);
    let token_ticker_2 = symbol_short!("TCK2");

    let salt = BytesN::from_array(&e, &[0; 32]);

    contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt.clone()), &None);

    contract_client.initialize_pool(&token_address_2, &token_ticker_2, &None, &None);
    contract_client.initialize_pool(&token_address_2, &token_ticker_2, &Some(salt), &None);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_pool_reinitialize_no_salt() {
    let e = Env::default();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        LendingContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = LendingContractClient::new(&e, &contract_id);

    let token_address = Address::generate(&e);
    let token_ticker = symbol_short!("TCK1");

    contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
    contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_pool_reinitialize_with_salt() {
    let e = Env::default();

    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        LendingContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = LendingContractClient::new(&e, &contract_id);

    let token_address = Address::generate(&e);
    let token_ticker = symbol_short!("TCK1");

    let salt = BytesN::from_array(&e, &[0; 32]);

    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt.clone()), &None);
    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt), &None);
}
