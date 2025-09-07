#![cfg(test)]

use market::{error::MarketContractError, pool::PoolConfig};
use soroban_sdk::{Address, testutils::Address as _};

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_USER_ASSET_MINT_AMOUNT, TestFixture,
    get_deposit_obligation, get_obligation_deposited, get_obligation_j_tokens, get_pool_available,
    get_pool_total_j_tokens,
};

#[test]
fn test_deposit() {
    let TestFixture {
        e,
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = &users[0];
    contract_client.deposit(user, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let total_shares_after = get_pool_total_j_tokens(&contract_client, &usdc_pool_address).unwrap();
    let tokens_from_shares_after =
        get_obligation_deposited(&e, &contract_client, user, &usdc_pool_address).unwrap();
    let shares_after = get_obligation_j_tokens(&contract_client, user, &usdc_pool_address).unwrap();
    let available_after = get_pool_available(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(total_shares_after, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(tokens_from_shares_after, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(shares_after, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(available_after, DEFAULT_DEPOSIT_AMOUNT);
}

#[test]
fn test_deposit_zero() {
    let TestFixture {
        e,
        contract_client,
        usdc_token_address,
        usdc_pool_address,
        ..
    } = TestFixture::new();

    let pool_before = contract_client.get_pool(&usdc_pool_address);

    let user = Address::generate(&e);
    contract_client.deposit(&user, &usdc_token_address, &0);

    let pool_after = contract_client.get_pool(&usdc_pool_address);

    assert_eq!(pool_after, pool_before);
}

#[test]
fn test_exceed_supply_limit() {
    #[allow(clippy::inconsistent_digit_grouping)]
    const SUPPLY_LIMIT: i128 = 1_000_000_0000000;
    let pool_config = PoolConfig {
        supply_limit: SUPPLY_LIMIT,
        ..Default::default()
    };

    let TestFixture {
        contract_client,
        usdc_token_address,
        users,
        ..
    } = TestFixture::new_with_pool_config(pool_config);

    let user = &users[0];
    contract_client.deposit(user, &usdc_token_address, &(SUPPLY_LIMIT));

    assert_eq!(
        contract_client.try_deposit(user, &usdc_token_address, &1),
        Err(Ok(MarketContractError::SupplyLimitExceeded)),
    );
}

#[test]
fn test_add_collateral() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = &users[0];
    contract_client.add_collateral(user, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    let obligation_collateral = get_deposit_obligation(&contract_client, user, &usdc_pool_address)
        .unwrap()
        .collateral;
    let pool_collateral = contract_client
        .get_pool(&usdc_pool_address)
        .total_collateral;

    assert_eq!(obligation_collateral, DEFAULT_COLLATERAL_AMOUNT);
    assert_eq!(pool_collateral, DEFAULT_COLLATERAL_AMOUNT);
}

#[test]
fn test_add_collateral_zero() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = &users[0];

    let pool_before = contract_client.get_pool(&usdc_pool_address);

    contract_client.add_collateral(user, &usdc_pool_address, &0);

    let pool_after = contract_client.get_pool(&usdc_pool_address);

    assert_eq!(pool_before, pool_after);
}

#[test]
fn test_add_collateral_negative() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = &users[0];

    assert_eq!(
        contract_client.try_add_collateral(user, &usdc_pool_address, &-1),
        Err(Ok(MarketContractError::NegativeCollateralAddition))
    );
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

    let user = &users[0];
    contract_client.deposit(user, &usdc_pool_address, &DEPOSIT_AMOUNT);
}

#[test]
#[should_panic(expected = "Error(Contract, #30)")]
fn test_deposit_negative() {
    let TestFixture {
        contract_client,
        usdc_token_address,
        users,
        ..
    } = TestFixture::new();

    let user = &users[0];
    contract_client.deposit(user, &usdc_token_address, &-1);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_deposit_pool_does_not_exist() {
    let TestFixture {
        e,
        contract_client,
        users,
        ..
    } = TestFixture::new();

    let missing_pool_address = Address::generate(&e);

    let user = &users[0];
    contract_client.deposit(user, &missing_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
}
