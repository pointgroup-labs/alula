#![cfg(test)]

use market::{
    constants::{BPS_FACTOR, DEFAULT_OPEN_LTV},
    error::MCError,
    pool::PoolConfig,
};

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, get_obligation_borrowed, get_obligation_d_tokens,
    get_obligation_d_tokens_as_tokens, get_pool_total_available, get_pool_total_borrowed,
    get_pool_total_d_tokens,
};

#[test]
fn test_borrow() {
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

    // NB: GOLD is used as the main collateral in integration tests
    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // NB: USDC is used as the main borrowed token in integration tests
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligation_borrowed =
        get_obligation_borrowed(&contract_client, borrower, &usdc_pool_address).unwrap();
    let obligation_d_tokens =
        get_obligation_d_tokens(&contract_client, borrower, &usdc_pool_address).unwrap();
    let obligation_d_tokens_as_tokens =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    assert_eq!(obligation_borrowed, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(obligation_d_tokens, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(obligation_d_tokens_as_tokens, DEFAULT_DEPOSIT_AMOUNT);

    let pool_total_available =
        get_pool_total_available(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_borrowed =
        get_pool_total_borrowed(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_d_tokens =
        get_pool_total_d_tokens(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(pool_total_d_tokens, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_total_borrowed, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_total_available, 0);
}

#[test]
fn test_borrow_multiple_shareholders() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let borrower_1 = &users[0];
    let borrower_2 = &users[1];
    let loan_provider = &users[2];

    contract_client.deposit(
        loan_provider,
        &usdc_pool_address,
        &(3 * DEFAULT_DEPOSIT_AMOUNT),
    );

    contract_client.deposit(
        borrower_1,
        &gold_pool_address,
        &(3 * DEFAULT_DEPOSIT_AMOUNT),
    );
    contract_client.deposit(
        borrower_2,
        &gold_pool_address,
        &(3 * DEFAULT_DEPOSIT_AMOUNT),
    );

    contract_client.borrow(borrower_1, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    const BORROWER_2_BORROW_AMOUNT: i128 = (3 * DEFAULT_DEPOSIT_AMOUNT) / 2;
    contract_client.borrow(borrower_2, &usdc_pool_address, &BORROWER_2_BORROW_AMOUNT);

    let obligation_borrowed_1 =
        get_obligation_borrowed(&contract_client, borrower_1, &usdc_pool_address).unwrap();
    let obligation_d_tokens_1 =
        get_obligation_d_tokens(&contract_client, borrower_1, &usdc_pool_address).unwrap();
    let obligation_d_tokens_as_tokens_1 =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower_1, &usdc_pool_address)
            .unwrap();

    assert_eq!(obligation_borrowed_1, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(obligation_d_tokens_1, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(obligation_d_tokens_as_tokens_1, DEFAULT_DEPOSIT_AMOUNT);

    let obligation_borrowed_2 =
        get_obligation_borrowed(&contract_client, borrower_2, &usdc_pool_address).unwrap();
    let obligation_d_tokens_2 =
        get_obligation_d_tokens(&contract_client, borrower_2, &usdc_pool_address).unwrap();
    let obligation_d_tokens_as_tokens_2 =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower_2, &usdc_pool_address)
            .unwrap();

    assert_eq!(obligation_borrowed_2, BORROWER_2_BORROW_AMOUNT);
    assert_eq!(obligation_d_tokens_2, BORROWER_2_BORROW_AMOUNT);
    assert_eq!(obligation_d_tokens_as_tokens_2, BORROWER_2_BORROW_AMOUNT);

    let pool_total_available =
        get_pool_total_available(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_borrowed =
        get_pool_total_borrowed(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_d_tokens =
        get_pool_total_d_tokens(&contract_client, &usdc_pool_address).unwrap();

    const BORROWED: i128 = DEFAULT_DEPOSIT_AMOUNT + BORROWER_2_BORROW_AMOUNT;

    assert_eq!(pool_total_d_tokens, BORROWED);
    assert_eq!(pool_total_borrowed, BORROWED);
    assert_eq!(pool_total_borrowed, BORROWED);
    assert_eq!(
        pool_total_available,
        (3 * DEFAULT_DEPOSIT_AMOUNT) - BORROWED
    );

    // TODO: Add time passing
}
#[test]
fn test_borrow_exceeds_utilization_cap() {
    const UTILIZATION_RATIO_LIMIT_BPS: i128 = 9000; // 90%

    let pool_config = PoolConfig {
        utilization_ratio_limit_bps: UTILIZATION_RATIO_LIMIT_BPS,
        ..Default::default()
    };

    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(2 * &DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(
        borrower,
        &usdc_pool_address,
        &((DEFAULT_DEPOSIT_AMOUNT * UTILIZATION_RATIO_LIMIT_BPS) / BPS_FACTOR),
    );

    assert_eq!(
        contract_client.try_borrow(borrower, &usdc_pool_address, &1),
        Err(Ok(MCError::PoolUtilizationRatioCapExceeded))
    );
}

#[test]
fn test_borrow_zero() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(borrower, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_before = contract_client.get_pool(&usdc_pool_address);
    contract_client.borrow(borrower, &usdc_pool_address, &0);
    let pool_after = contract_client.get_pool(&usdc_pool_address);

    assert_eq!(pool_before, pool_after);
}

#[test]
fn test_borrow_negative() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(borrower, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    assert_eq!(
        contract_client.try_borrow(borrower, &usdc_pool_address, &-1),
        Err(Ok(MCError::NegativeAmount))
    );
}

#[test]
fn test_borrow_amount_is_reduced_to_satisfy_obligation_health() {
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

    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.add_collateral(borrower, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligation_borrowed =
        get_obligation_borrowed(&contract_client, borrower, &usdc_pool_address).unwrap();
    let obligation_d_tokens =
        get_obligation_d_tokens(&contract_client, borrower, &usdc_pool_address).unwrap();
    let obligation_d_tokens_as_tokens =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    const MAX_HEALTHY_BORROW_AMOUNT: i128 = (DEFAULT_OPEN_LTV * DEFAULT_DEPOSIT_AMOUNT) / 100;

    assert_eq!(obligation_borrowed, MAX_HEALTHY_BORROW_AMOUNT);
    assert_eq!(obligation_d_tokens, MAX_HEALTHY_BORROW_AMOUNT);
    assert_eq!(obligation_d_tokens_as_tokens, MAX_HEALTHY_BORROW_AMOUNT);

    let pool_total_available =
        get_pool_total_available(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_borrowed =
        get_pool_total_borrowed(&contract_client, &usdc_pool_address).unwrap();
    let pool_total_d_tokens =
        get_pool_total_d_tokens(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(pool_total_d_tokens, MAX_HEALTHY_BORROW_AMOUNT);
    assert_eq!(pool_total_borrowed, MAX_HEALTHY_BORROW_AMOUNT);
    assert_eq!(pool_total_borrowed, MAX_HEALTHY_BORROW_AMOUNT);
    assert_eq!(
        pool_total_available,
        DEFAULT_DEPOSIT_AMOUNT - MAX_HEALTHY_BORROW_AMOUNT
    );
}

// TODO: Add more time passing tests
