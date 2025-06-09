#![cfg(test)]

use {
    crate::{
        get_deposit_obligation, TestFixture, DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT,
        DEFAULT_USER_ASSET_MINT_AMOUNT,
    },
    soroban_sdk::{testutils::Address as _, Address},
};

#[test]
fn test_deposit() {
    let TestFixture {
        contract_client,
        usdc_token_address,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    contract_client.deposit(&user, &usdc_token_address, &DEFAULT_DEPOSIT_AMOUNT);

    let deposit_obligation = get_deposit_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .deposited;
    let pool_deposited = contract_client
        .get_pool(&usdc_pool_address)
        .unwrap()
        .total_supply;

    // TODO: We should introduce operation fees which will make the deposited amount smaller
    assert_eq!(deposit_obligation, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_deposited, DEFAULT_DEPOSIT_AMOUNT);
}

#[test]
fn test_deposit_collateral() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    contract_client.deposit_collateral(&user, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    let obligation_collateral = get_deposit_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .collateral;
    let pool_collateral = contract_client
        .get_pool(&usdc_pool_address)
        .unwrap()
        .total_collateral;

    assert_eq!(obligation_collateral, DEFAULT_COLLATERAL_AMOUNT);
    assert_eq!(pool_collateral, DEFAULT_COLLATERAL_AMOUNT);
}

#[test]
#[should_panic]
fn test_deposit_non_existing_tokens() {
    const DEPOSIT_AMOUNT: i128 = DEFAULT_USER_ASSET_MINT_AMOUNT + 1;

    let TestFixture {
        contract_client,
        users,
        usdc_pool_address,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    contract_client.deposit(&user, &usdc_pool_address, &DEPOSIT_AMOUNT);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_deposit_non_positive() {
    let TestFixture {
        contract_client,
        usdc_token_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    contract_client.deposit(&user, &usdc_token_address, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_deposit_pool_does_not_exist() {
    let TestFixture {
        e,
        contract_client,
        users,
        ..
    } = TestFixture::new();

    let missing_pool_address = Address::generate(&e);

    let user = users.get(0).unwrap();
    contract_client.deposit(&user, &missing_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
}
