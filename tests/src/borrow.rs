#![cfg(test)]

use {
    crate::{get_borrow_obligation, get_deposit_obligation, TestFixture, DEFAULT_DEPOSIT_AMOUNT},
    lending::constants::LCError,
    soroban_sdk::Address,
};

#[test]
fn test_borrow() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user: Address = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();
    // Deposit gold to satisfy the health factor threshold
    contract_client.deposit(&user, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));
    contract_client.borrow(&user, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligation_borrowed = get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .borrowed;
    let pool_borrowed = contract_client.get_pool(&usdc_pool_address).total_borrowed;

    assert_eq!(obligation_borrowed, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_borrowed, DEFAULT_DEPOSIT_AMOUNT);
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")]
fn test_borrow_non_positive() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user: Address = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));
    contract_client.borrow(&user, &usdc_pool_address, &0);
}

#[test]
fn test_borrow_health_factor_add_collateral() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user: Address = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();
    // Deposit gold to satisfy the health factor threshold
    contract_client.add_collateral(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    let deposit_obligation =
        get_deposit_obligation(&contract_client, &user, &gold_pool_address).unwrap();
    assert_eq!(deposit_obligation.collateral, DEFAULT_DEPOSIT_AMOUNT);

    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    let deposit_obligation2 =
        get_deposit_obligation(&contract_client, &user2, &usdc_pool_address).unwrap();
    assert_eq!(deposit_obligation2.shares, 2 * DEFAULT_DEPOSIT_AMOUNT);

    // Borrow 50% of the deposited value
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    // Borrow 75%
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    // Borrow 80% - equals test's fixture liquidation threshold
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 20));

    // Borrow which leads to the health factor threshold constraint violation
    assert_eq!(
        contract_client.try_borrow(&user, &usdc_pool_address, &(1)),
        Err(Ok(LCError::HealthFactorIsLowerThanRequiredThreshold))
    );

    // Improve health factor
    contract_client.add_collateral(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    // Borrow without health factor violation
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));
}

#[test]
fn test_borrow_health_factor_deposit() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user: Address = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();
    // Deposit gold to satisfy the health factor threshold
    contract_client.deposit(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    // Borrow 50% of the deposited value
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    // Borrow 75%
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    // Borrow 80% - equals test's fixture liquidation threshold
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 20));

    // Borrow which leads to the health factor threshold constraint violation
    assert_eq!(
        contract_client.try_borrow(&user, &usdc_pool_address, &(1)),
        Err(Ok(LCError::HealthFactorIsLowerThanRequiredThreshold))
    );

    // Improve health factor
    contract_client.deposit(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    // Borrow without health factor violation
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));
}
