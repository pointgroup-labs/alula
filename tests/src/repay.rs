#![cfg(test)]

use std::i128;

use market::error::MCError;

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, get_borrow_obligation, get_obligation_borrowed,
    get_obligation_d_tokens, get_obligation_d_tokens_as_tokens, get_pool_total_available,
    get_pool_total_borrowed,
};

#[test]
fn test_repay() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.add_collateral(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    // Borrow 50% of the available
    contract_client.borrow(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    // Repay the half of the debt
    contract_client.repay(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    let obligation_borrowed =
        get_obligation_borrowed(&contract_client, borrower, &usdc_pool_address).unwrap();
    let obligation_d_tokens =
        get_obligation_d_tokens(&contract_client, borrower, &usdc_pool_address).unwrap();
    let obligation_d_tokens_as_tokens =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    assert_eq!(obligation_borrowed, DEFAULT_DEPOSIT_AMOUNT / 4);
    assert_eq!(obligation_d_tokens, DEFAULT_DEPOSIT_AMOUNT / 4);
    assert_eq!(obligation_d_tokens_as_tokens, DEFAULT_DEPOSIT_AMOUNT / 4);

    let pool_total_available =
        get_pool_total_available(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_borrowed =
        get_pool_total_borrowed(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_d_tokens =
        get_pool_total_borrowed(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(pool_total_d_tokens, DEFAULT_DEPOSIT_AMOUNT / 4);
    assert_eq!(pool_total_borrowed, DEFAULT_DEPOSIT_AMOUNT / 4);
    assert_eq!(pool_total_available, (3 * DEFAULT_DEPOSIT_AMOUNT) / 4);

    // Repay the rest
    contract_client.repay(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    assert_eq!(
        get_obligation_borrowed(&contract_client, &borrower, &usdc_pool_address),
        Err(MCError::BorrowDoesNotExist)
    );

    let pool_total_available =
        get_pool_total_available(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_borrowed =
        get_pool_total_borrowed(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_d_tokens =
        get_pool_total_borrowed(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(pool_total_d_tokens, 0);
    assert_eq!(pool_total_borrowed, 0);
    assert_eq!(pool_total_available, DEFAULT_DEPOSIT_AMOUNT);
}

#[test]
fn test_repay_zero() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.add_collateral(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    let obligation_before =
        get_borrow_obligation(&contract_client, &borrower, &usdc_pool_address).unwrap();
    let usdc_pool_before = contract_client.get_pool(&usdc_pool_address);
    let gold_pool_before = contract_client.get_pool(&gold_pool_address);

    contract_client.repay(borrower, &usdc_pool_address, &0);

    let obligation_after =
        get_borrow_obligation(&contract_client, &borrower, &usdc_pool_address).unwrap();
    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    let gold_pool_after = contract_client.get_pool(&gold_pool_address);

    assert_eq!(obligation_before, obligation_after);
    assert_eq!(usdc_pool_before, usdc_pool_after);
    assert_eq!(gold_pool_before, gold_pool_after);
}

#[test]
fn test_repay_with_interest_accrual() {
    // TODO
}

#[test]
fn test_repay_unpaid_interest_only() {
    // TODO
}

#[test]
fn test_repay_all_with_i128_max() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.add_collateral(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));
    contract_client.repay(borrower, &usdc_pool_address, &i128::MAX);

    assert_eq!(
        get_obligation_borrowed(&contract_client, borrower, &usdc_pool_address),
        Err(MCError::BorrowDoesNotExist)
    );

    let pool_total_available =
        get_pool_total_available(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_borrowed =
        get_pool_total_borrowed(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_d_tokens =
        get_pool_total_borrowed(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(pool_total_d_tokens, 0);
    assert_eq!(pool_total_borrowed, 0);
    assert_eq!(pool_total_available, DEFAULT_DEPOSIT_AMOUNT);
}
