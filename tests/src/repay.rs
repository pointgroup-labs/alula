#![cfg(test)]

use market::{constants::*, error::MCError};
use soroban_sdk::testutils::Ledger;

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, assert_approx_eq_abs,
    get_obligation_d_tokens_as_tokens, get_obligation_initially_borrowed,
    get_obligation_unpaid_interest, get_pool_total_available, get_pool_total_borrowed,
};

#[test]
fn test_repay() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    // Borrow 50% of the available
    contract_client.borrow(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    // Repay the half of the debt

    let borrower_balance_before = usdc_token_client.balance(borrower);
    contract_client.repay(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));
    let borrower_balance_after = usdc_token_client.balance(borrower);

    assert_eq!(
        borrower_balance_before.checked_sub(borrower_balance_after).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT / 4
    );

    let obligation_borrowed =
        get_obligation_initially_borrowed(&contract_client, borrower, &usdc_pool_address).unwrap();
    let obligation_d_tokens_as_tokens =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    assert_eq!(obligation_borrowed, DEFAULT_DEPOSIT_AMOUNT / 4);
    assert_eq!(obligation_d_tokens_as_tokens, DEFAULT_DEPOSIT_AMOUNT / 4);

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert_eq!(pool_total_borrowed, DEFAULT_DEPOSIT_AMOUNT / 4);
    assert_eq!(pool_total_available, (3 * DEFAULT_DEPOSIT_AMOUNT) / 4);

    // Repay the rest
    contract_client.repay(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    assert_eq!(
        get_obligation_initially_borrowed(&contract_client, borrower, &usdc_pool_address),
        Err(MCError::BorrowPositionDoesNotExist)
    );

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert_eq!(pool_total_borrowed, 0);
    assert_eq!(pool_total_available, DEFAULT_DEPOSIT_AMOUNT);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")] // NegativeInputAmount
fn test_repay_zero() {
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    contract_client.repay(borrower, &usdc_pool_address, &0);
}

#[test]
fn test_repay_with_interest_accrual() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    // -- Move time --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR / 12;
    });
    contract_client.refresh_pool(&usdc_pool_address);

    let unpaid_interest =
        get_obligation_unpaid_interest(&e, &contract_client, borrower, &usdc_pool_address).unwrap();

    contract_client.repay(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    let remaining_debt =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    assert_approx_eq_abs(remaining_debt, unpaid_interest, 10);
}

#[test]
fn test_repay_unpaid_interest_only() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    // -- Move time --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR / 12;
    });
    contract_client.refresh_pool(&usdc_pool_address);

    let obligation_unpaid_interest_before =
        get_obligation_unpaid_interest(&e, &contract_client, borrower, &usdc_pool_address).unwrap();

    let borrower_balance_before = usdc_token_client.balance(borrower);
    contract_client.repay(borrower, &usdc_pool_address, &obligation_unpaid_interest_before);
    let borrower_balance_after = usdc_token_client.balance(borrower);

    assert_eq!(
        borrower_balance_before.checked_sub(borrower_balance_after).unwrap(),
        obligation_unpaid_interest_before
    );

    let obligation_unpaid_interest_after =
        get_obligation_unpaid_interest(&e, &contract_client, borrower, &usdc_pool_address).unwrap();

    assert_approx_eq_abs(obligation_unpaid_interest_after, 0, 1);
}

#[test]
fn test_repay_all_with_bigger_than_debt_value() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(liquidity_provider, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    contract_client.borrow(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    let borrower_balance_before = usdc_token_client.balance(borrower);
    contract_client.repay(
        borrower,
        &usdc_pool_address,
        &(3 * DEFAULT_DEPOSIT_AMOUNT / 2), // x3 of borrowed amount
    );
    let borrower_balance_after = usdc_token_client.balance(borrower);

    assert_eq!(
        borrower_balance_before.checked_sub(borrower_balance_after).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT / 2
    );

    assert_eq!(
        get_obligation_initially_borrowed(&contract_client, borrower, &usdc_pool_address),
        Err(MCError::BorrowPositionDoesNotExist)
    );

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert_eq!(pool_total_borrowed, 0);
    assert_eq!(pool_total_available, (2 * DEFAULT_DEPOSIT_AMOUNT));
}

#[test]
fn test_consecutive_borrows_can_lead_to_unpaid_interest_become_negative() {
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let liquidity_provider = &users[0];
    let borrower_1 = &users[1];
    let borrower_2 = &users[2];
    let borrower_3 = &users[3];

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(100000000000 * DEFAULT_DEPOSIT_AMOUNT),
    );

    contract_client.add_collateral(borrower_1, &gold_pool_address, &7777777);
    contract_client.borrow(borrower_1, &usdc_pool_address, &i128::MAX);

    contract_client.add_collateral(borrower_2, &gold_pool_address, &177777);
    contract_client.borrow(borrower_2, &usdc_pool_address, &i128::MAX);

    contract_client.add_collateral(borrower_3, &gold_pool_address, &5325523);
    contract_client.borrow(borrower_3, &usdc_pool_address, &i128::MAX);

    // NB: Consecutive borrows can lead to 'unpaid_interest_becomes_negative' internal error when repaying the first borrow
    // right away. This is a consequence of generating an amount of dTokens with ceiling rounding to favour the protocol when borrowing
    assert_eq!(
        contract_client.try_repay(borrower_1, &usdc_pool_address, &1),
        Err(Ok(MCError::InternalError))
    );
}
