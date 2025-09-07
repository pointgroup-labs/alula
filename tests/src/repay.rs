#![cfg(test)]

use market::error::MarketContractError;
use soroban_sdk::testutils::Ledger;

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestFixture, get_borrow_obligation, get_obligation_total_debt,
    get_obligation_unpaid_interest,
};

#[test]
fn test_repay_zero() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = &users[0];
    let user2 = &users[1];
    // Deposit gold as a collateral to satisfy the health factor threshold
    contract_client.add_collateral(user, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(user2, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // Borrow 50% of the deposited value
    contract_client.borrow(user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    let usdc_pool_before = contract_client.get_pool(&usdc_pool_address);
    let gold_pool_before = contract_client.get_pool(&usdc_pool_address);
    let obligation_before = contract_client.get_user_obligation(user);

    contract_client.repay(user, &usdc_pool_address, &0);

    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    let gold_pool_after = contract_client.get_pool(&usdc_pool_address);
    let obligation_after = contract_client.get_user_obligation(user);

    assert_eq!(obligation_before, obligation_after);
    assert_eq!(usdc_pool_before, usdc_pool_after);
    assert_eq!(gold_pool_before, gold_pool_after);
}

#[test]
fn test_repay() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = &users[0];
    let user2 = &users[1];
    // Deposit gold as a collateral to satisfy the health factor threshold
    contract_client.add_collateral(user, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(user2, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // Borrow 50% of the deposited value
    contract_client.borrow(user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    let obligation_borrowed = get_borrow_obligation(&contract_client, user, &usdc_pool_address)
        .unwrap()
        .borrowed;
    let pool_borrowed = contract_client.get_pool(&usdc_pool_address).total_borrowed;

    assert_eq!(obligation_borrowed, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(pool_borrowed, DEFAULT_DEPOSIT_AMOUNT / 2);

    // Repay the half
    contract_client.repay(user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    let obligation_borrowed = get_borrow_obligation(&contract_client, user, &usdc_pool_address)
        .unwrap()
        .borrowed;
    let pool_borrowed = contract_client.get_pool(&usdc_pool_address).total_borrowed;

    assert_eq!(obligation_borrowed, DEFAULT_DEPOSIT_AMOUNT / 4);
    assert_eq!(pool_borrowed, DEFAULT_DEPOSIT_AMOUNT / 4);

    // Repay the rest
    contract_client.repay(user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    let obligation = contract_client.get_user_obligation(user);
    assert!(obligation.borrows.get(usdc_pool_address.clone()).is_none());

    let pool_borrowed = contract_client.get_pool(&usdc_pool_address).total_borrowed;

    assert_eq!(pool_borrowed, 0);
}

#[test]
fn test_repay_with_interest_accrual() {
    let TestFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = &users[0];
    let user2 = &users[2];
    // Deposit gold as a collateral to satisfy the health factor threshold
    contract_client.add_collateral(user, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(user2, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // Borrow 50% of the deposited value
    contract_client.borrow(user, &usdc_pool_address, &(5 * DEFAULT_DEPOSIT_AMOUNT / 10));

    let obligation_total_debt =
        get_obligation_total_debt(&e, &contract_client, user, &usdc_pool_address).unwrap();
    let pool_borrowed = contract_client.get_pool(&usdc_pool_address).total_borrowed;

    assert_eq!(obligation_total_debt, 5 * DEFAULT_DEPOSIT_AMOUNT / 10);
    assert_eq!(pool_borrowed, 5 * DEFAULT_DEPOSIT_AMOUNT / 10);

    // Wait for 5 hours to pass by
    e.ledger().with_mut(|li| li.timestamp += 60 * 60 * 5);

    let borrow_obligation =
        get_borrow_obligation(&contract_client, user, &usdc_pool_address).unwrap();

    let unpaid_interest =
        get_obligation_unpaid_interest(&e, &contract_client, user, &usdc_pool_address).unwrap();

    assert_eq!(borrow_obligation.borrowed, 5 * DEFAULT_DEPOSIT_AMOUNT / 10);
    assert!(unpaid_interest > 0);

    let left =
        get_obligation_unpaid_interest(&e, &contract_client, user, &usdc_pool_address).unwrap();

    contract_client.repay(user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

    let obligation_borrowed_new_debt =
        get_obligation_total_debt(&e, &contract_client, user, &usdc_pool_address).unwrap();

    // Notice interest rate accrual
    assert_eq!(
        obligation_borrowed_new_debt,
        left - (DEFAULT_DEPOSIT_AMOUNT / 10)
    );

    // Wait another 15 hours to pass by
    e.ledger().with_mut(|li| li.timestamp += 60 * 60 * 15);

    let obligation_borrowed_new_debt =
        get_obligation_total_debt(&e, &contract_client, user, &usdc_pool_address).unwrap();

    // Notice interest rate accrual
    assert!(obligation_borrowed_new_debt > left - (DEFAULT_DEPOSIT_AMOUNT / 10));

    // Repay everything
    contract_client.repay(user, &usdc_pool_address, &obligation_borrowed_new_debt);

    let obligation = contract_client.get_user_obligation(user);
    assert!(obligation.borrows.is_empty());
}

#[test]
fn test_repay_unpaid_interest_only() {
    let TestFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = &users[0];
    let user2 = &users[2];

    // Deposit gold as a collateral to satisfy the health factor threshold
    contract_client.add_collateral(user, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(user2, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    // Borrow 50% of the deposited value
    contract_client.borrow(user, &usdc_pool_address, &(5 * DEFAULT_DEPOSIT_AMOUNT / 10));

    let unpaid_interest =
        get_obligation_unpaid_interest(&e, &contract_client, user, &usdc_pool_address).unwrap();
    assert_eq!(unpaid_interest, 0);

    e.ledger().with_mut(|li| li.timestamp += 5 * 60 * 60); // 5 hours

    let unpaid_interest =
        get_obligation_unpaid_interest(&e, &contract_client, user, &usdc_pool_address).unwrap();

    assert!(unpaid_interest > 0);

    contract_client.repay(user, &usdc_pool_address, &unpaid_interest);

    assert_eq!(
        get_obligation_unpaid_interest(&e, &contract_client, user, &usdc_pool_address).unwrap(),
        0
    );
}

#[test]
fn test_repay_more_than_borrowed() {
    const BORROW_AMOUNT: i128 = 5 * DEFAULT_DEPOSIT_AMOUNT / 10;

    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = &users[0];
    let user2 = &users[2];

    // Deposit gold as a collateral to satisfy the health factor threshold
    contract_client.add_collateral(user, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(user2, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(user, &usdc_pool_address, &BORROW_AMOUNT);

    let pool_borrowed_before = contract_client.get_pool(&usdc_pool_address).total_borrowed;

    // Repay more
    contract_client.repay(user, &usdc_pool_address, &(BORROW_AMOUNT + 1));

    let pool_borrowed_after = contract_client.get_pool(&usdc_pool_address).total_borrowed;

    assert_eq!(pool_borrowed_after + BORROW_AMOUNT, pool_borrowed_before);
    assert_eq!(
        get_borrow_obligation(&contract_client, user, &usdc_pool_address),
        Err(MarketContractError::BorrowDoesNotExist)
    );
}

#[test]
fn test_repay_all_with_i128_max() {
    const BORROW_AMOUNT: i128 = 5 * DEFAULT_DEPOSIT_AMOUNT / 10;

    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = &users[0];
    let user2 = &users[2];

    // Deposit gold as a collateral to satisfy the health factor threshold
    contract_client.add_collateral(user, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(user2, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(user, &usdc_pool_address, &BORROW_AMOUNT);

    let pool_borrowed_before = contract_client.get_pool(&usdc_pool_address).total_borrowed;

    // Repay all debt
    contract_client.repay(user, &usdc_pool_address, &i128::MAX);

    let pool_borrowed_after = contract_client.get_pool(&usdc_pool_address).total_borrowed;

    assert_eq!(pool_borrowed_after + BORROW_AMOUNT, pool_borrowed_before);
    assert_eq!(
        get_borrow_obligation(&contract_client, user, &usdc_pool_address),
        Err(MarketContractError::BorrowDoesNotExist)
    );
}
