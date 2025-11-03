#![cfg(test)]

use market::{
    constants::*,
    error::MCError,
    pool::{PoolConfig, PoolHealthConfig},
};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::testutils::Ledger;

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, get_obligation_borrowed, get_obligation_d_tokens,
    get_obligation_d_tokens_as_tokens, get_pool_fee_config, get_pool_total_available,
    get_pool_total_borrowed, get_pool_total_d_tokens,
};

#[test]
fn test_borrow() {
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
    let loan_provider = &users[1];

    // NB: GOLD is used as the main collateral in integration tests
    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // NB: USDC is used as the main borrowed token in integration tests
    contract_client.deposit(loan_provider, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    let borrower_balance_before = usdc_token_client.balance(borrower);
    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    let borrower_balance_after = usdc_token_client.balance(borrower);

    let borrow_fee_bps = get_pool_fee_config(&contract_client, &usdc_pool_address).borrow_fee_bps;

    assert_eq!(
        borrower_balance_after.checked_sub(borrower_balance_before).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT
            .fixed_mul_ceil(BPS_FACTOR - borrow_fee_bps as i128, BPS_FACTOR)
            .unwrap()
    );

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

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_total_d_tokens = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);

    assert_eq!(pool_total_d_tokens, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_total_borrowed, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_total_available, DEFAULT_DEPOSIT_AMOUNT);
}

#[test]
fn test_borrow_multiple_shareholders() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    let borrower_1 = &users[0];
    let borrower_2 = &users[1];
    let loan_provider = &users[2];

    contract_client.deposit(loan_provider, &usdc_pool_address, &(3 * DEFAULT_DEPOSIT_AMOUNT));

    contract_client.deposit(borrower_1, &gold_pool_address, &(3 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(borrower_2, &gold_pool_address, &(3 * DEFAULT_DEPOSIT_AMOUNT));

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

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_total_d_tokens = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);

    const BORROWED: i128 = DEFAULT_DEPOSIT_AMOUNT + BORROWER_2_BORROW_AMOUNT;

    assert_eq!(pool_total_d_tokens, BORROWED);
    assert_eq!(pool_total_borrowed, BORROWED);
    assert_eq!(pool_total_borrowed, BORROWED);
    assert_eq!(pool_total_available, (3 * DEFAULT_DEPOSIT_AMOUNT) - BORROWED);

    // -- Accrue debt on the pool --

    // - Wait 1 month -
    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR / 12;
    });
    contract_client.poke_pool(&usdc_pool_address);

    // - Assert that the total debt has increased -

    let obligation_borrowed_1 =
        get_obligation_borrowed(&contract_client, borrower_1, &usdc_pool_address).unwrap();
    let obligation_d_tokens_1 =
        get_obligation_d_tokens(&contract_client, borrower_1, &usdc_pool_address).unwrap();
    let obligation_d_tokens_as_tokens_1 =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower_1, &usdc_pool_address)
            .unwrap();

    assert_eq!(obligation_borrowed_1, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(obligation_d_tokens_1, DEFAULT_DEPOSIT_AMOUNT);
    assert!(obligation_d_tokens_as_tokens_1 > DEFAULT_DEPOSIT_AMOUNT);

    let obligation_borrowed_2 =
        get_obligation_borrowed(&contract_client, borrower_2, &usdc_pool_address).unwrap();
    let obligation_d_tokens_2 =
        get_obligation_d_tokens(&contract_client, borrower_2, &usdc_pool_address).unwrap();
    let obligation_d_tokens_as_tokens_2 =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower_2, &usdc_pool_address)
            .unwrap();

    assert_eq!(obligation_borrowed_2, BORROWER_2_BORROW_AMOUNT);
    assert_eq!(obligation_d_tokens_2, BORROWER_2_BORROW_AMOUNT);
    assert!(obligation_d_tokens_as_tokens_2 > BORROWER_2_BORROW_AMOUNT);

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_total_d_tokens = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);

    assert_eq!(pool_total_d_tokens, BORROWED);
    assert!(pool_total_borrowed > BORROWED);
    assert_eq!(pool_total_available, (3 * DEFAULT_DEPOSIT_AMOUNT) - BORROWED);
}
#[test]
fn test_borrow_exceeds_utilization_cap() {
    const UTILIZATION_RATIO_LIMIT_BPS: i128 = 9000; // 90%
    const BORROW_AMOUNT: i128 = (DEFAULT_DEPOSIT_AMOUNT * UTILIZATION_RATIO_LIMIT_BPS) / BPS_FACTOR;

    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: UTILIZATION_RATIO_LIMIT_BPS,
            ..Default::default()
        },
        ..Default::default()
    };

    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(2 * &DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let borrower_balance_before = usdc_token_client.balance(borrower);
    contract_client.borrow(borrower, &usdc_pool_address, &BORROW_AMOUNT);
    let borrower_balance_after = usdc_token_client.balance(borrower);

    let borrow_fee_bps = get_pool_fee_config(&contract_client, &usdc_pool_address).borrow_fee_bps;

    assert_eq!(
        borrower_balance_after.checked_sub(borrower_balance_before).unwrap(),
        BORROW_AMOUNT.fixed_mul_ceil(BPS_FACTOR - borrow_fee_bps as i128, BPS_FACTOR).unwrap()
    );

    assert_eq!(
        contract_client.try_borrow(borrower, &usdc_pool_address, &1),
        Err(Ok(MCError::PoolUtilizationRatioCapExceeded))
    );
}

#[test]
fn test_borrow_zero() {
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new();
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
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(borrower, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    assert_eq!(
        contract_client.try_borrow(borrower, &usdc_pool_address, &-1),
        Err(Ok(MCError::NegativeInputAmount))
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
        usdc_token_client,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.add_collateral(borrower, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let borrower_balance_before = usdc_token_client.balance(borrower);
    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    let borrower_balance_after = usdc_token_client.balance(borrower);

    let borrow_fee_bps = get_pool_fee_config(&contract_client, &usdc_pool_address).borrow_fee_bps;

    assert!(
        borrower_balance_after.checked_sub(borrower_balance_before).unwrap()
            < DEFAULT_DEPOSIT_AMOUNT
                .fixed_mul_ceil(BPS_FACTOR - borrow_fee_bps as i128, BPS_FACTOR)
                .unwrap()
    );

    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligation_borrowed =
        get_obligation_borrowed(&contract_client, borrower, &usdc_pool_address).unwrap();
    let obligation_d_tokens =
        get_obligation_d_tokens(&contract_client, borrower, &usdc_pool_address).unwrap();
    let obligation_d_tokens_as_tokens =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    const MAX_HEALTHY_BORROW_AMOUNT: i128 =
        (DEFAULT_OPEN_LTV_BPS * DEFAULT_DEPOSIT_AMOUNT) / BPS_FACTOR;

    assert_eq!(obligation_borrowed, MAX_HEALTHY_BORROW_AMOUNT);
    assert_eq!(obligation_d_tokens, MAX_HEALTHY_BORROW_AMOUNT);
    assert_eq!(obligation_d_tokens_as_tokens, MAX_HEALTHY_BORROW_AMOUNT);

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_total_d_tokens = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);

    assert_eq!(pool_total_d_tokens, MAX_HEALTHY_BORROW_AMOUNT);
    assert_eq!(pool_total_borrowed, MAX_HEALTHY_BORROW_AMOUNT);
    assert_eq!(pool_total_borrowed, MAX_HEALTHY_BORROW_AMOUNT);
    assert_eq!(pool_total_available, DEFAULT_DEPOSIT_AMOUNT - MAX_HEALTHY_BORROW_AMOUNT);
}
