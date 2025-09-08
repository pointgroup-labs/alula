#![cfg(test)]

use market::{error::MCError, pool::PoolConfig};
use soroban_sdk::{Address, testutils::Address as _};

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_USER_ASSET_MINT_AMOUNT,
    TestMarketFixture, get_obligation_collateral, get_obligation_d_tokens,
    get_obligation_deposited, get_obligation_j_tokens, get_obligation_j_tokens_as_tokens,
    get_pool_total_available, get_pool_total_collateral, get_pool_total_d_tokens,
    get_pool_total_j_tokens, get_pool_total_supply,
};

#[test]
fn test_deposit() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    contract_client.deposit(user, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_total_j_tokens =
        get_pool_total_j_tokens(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_available =
        get_pool_total_available(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_supply = get_pool_total_supply(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_d_tokens =
        get_pool_total_d_tokens(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(pool_total_j_tokens, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_total_available, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_total_supply, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_total_d_tokens, 0);

    let obligation_j_tokens =
        get_obligation_j_tokens(&contract_client, user, &usdc_pool_address).unwrap();
    let obligation_j_tokens_as_tokens =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, user, &usdc_pool_address).unwrap();
    let obligation_deposited =
        get_obligation_deposited(&contract_client, user, &usdc_pool_address).unwrap();

    assert_eq!(obligation_j_tokens, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(obligation_j_tokens_as_tokens, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(obligation_deposited, DEFAULT_DEPOSIT_AMOUNT);

    assert_eq!(
        get_obligation_d_tokens(&contract_client, user, &usdc_pool_address),
        Err(MCError::BorrowDoesNotExist)
    );
}

#[test]
fn test_deposit_zero() {
    let TestMarketFixture {
        contract_client,
        usdc_token_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    let pool_before = contract_client.get_pool(&usdc_pool_address);
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

    let TestMarketFixture {
        contract_client,
        usdc_token_address,
        users,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let user = &users[0];

    contract_client.deposit(user, &usdc_token_address, &SUPPLY_LIMIT);

    assert_eq!(
        contract_client.try_deposit(user, &usdc_token_address, &1),
        Err(Ok(MCError::PoolSupplyLimitExceeded)),
    );
}

#[test]
fn test_add_collateral() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    contract_client.add_collateral(user, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    let obligation_collateral =
        get_obligation_collateral(&contract_client, user, &usdc_pool_address).unwrap();
    assert_eq!(obligation_collateral, DEFAULT_COLLATERAL_AMOUNT);

    let pool_collateral = get_pool_total_collateral(&contract_client, &usdc_pool_address).unwrap();
    assert_eq!(pool_collateral, DEFAULT_COLLATERAL_AMOUNT);
}

#[test]
fn test_add_collateral_zero() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    let pool_before = contract_client.get_pool(&usdc_pool_address);
    contract_client.add_collateral(user, &usdc_pool_address, &0);
    let pool_after = contract_client.get_pool(&usdc_pool_address);

    assert_eq!(pool_before, pool_after);
}

#[test]
fn test_add_collateral_negative() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    assert_eq!(
        contract_client.try_add_collateral(user, &usdc_pool_address, &-1),
        Err(Ok(MCError::NegativeAmount))
    );
}

#[test]
fn test_deposit_non_existing_tokens() {
    const DEPOSIT_AMOUNT: i128 = DEFAULT_USER_ASSET_MINT_AMOUNT + 1;

    let TestMarketFixture {
        contract_client,
        users,
        usdc_pool_address,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    assert!(
        contract_client
            .try_deposit(user, &usdc_pool_address, &DEPOSIT_AMOUNT)
            .is_err()
    );
}

#[test]
fn test_deposit_negative() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    assert_eq!(
        contract_client.try_deposit(user, &usdc_pool_address, &-1),
        Err(Ok(MCError::NegativeAmount))
    );
}

#[test]
fn test_deposit_pool_does_not_exist() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    let missing_pool_address = Address::generate(&e);

    assert_eq!(
        contract_client.try_deposit(user, &missing_pool_address, &DEFAULT_DEPOSIT_AMOUNT),
        Err(Ok(MCError::PoolDoesNotExist))
    );
}

#[test]
fn test_deposit_multiple_shareholders() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_token_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let user_1 = &users[0];
    let user_2 = &users[1];

    contract_client.deposit(&user_1, &usdc_token_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(&user_2, &usdc_token_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    // Assert that jTokens shares are preserved(without interest accrual)
    let pool_j_tokens = get_pool_total_j_tokens(&contract_client, &usdc_pool_address).unwrap();
    let pool_available = get_pool_total_available(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(pool_j_tokens, (3 * DEFAULT_DEPOSIT_AMOUNT) / 2);
    assert_eq!(pool_available, (3 * DEFAULT_DEPOSIT_AMOUNT) / 2);

    let obligation_1_j_tokens =
        get_obligation_j_tokens(&contract_client, user_1, &usdc_pool_address).unwrap();
    let obligation_1_j_tokens_as_tokens =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, user_1, &usdc_pool_address)
            .unwrap();

    assert_eq!(obligation_1_j_tokens, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(obligation_1_j_tokens_as_tokens, DEFAULT_DEPOSIT_AMOUNT);

    let obligation_2_j_tokens =
        get_obligation_j_tokens(&contract_client, user_2, &usdc_pool_address).unwrap();
    let obligation_2_j_tokens_as_tokens =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, user_2, &usdc_pool_address)
            .unwrap();

    assert_eq!(obligation_2_j_tokens, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(obligation_2_j_tokens_as_tokens, DEFAULT_DEPOSIT_AMOUNT / 2);

    // TODO: Add time passing
}
