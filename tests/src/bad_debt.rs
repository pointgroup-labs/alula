#![cfg(test)]

use std::i128;

use market::{constants::*, error::MCError};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::testutils::Ledger;

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, compute_user_obligation_collateral_value,
    compute_user_obligation_debt_value, get_obligation_borrowed, get_pool_accumulated_host_fees,
    get_pool_accumulated_market_fees, get_pool_accumulated_reserve_fees, get_pool_fee_config,
    get_pool_total_available, get_pool_total_borrowed, get_pool_total_d_tokens,
    get_pool_total_j_tokens,
};

#[test]
fn test_accumulate_reserve_fees_are_empty_prior_accrual() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    let accumulated_reserve_fees_before_borrow =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let accumulated_reserve_fees_after_borrow =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    assert_eq!(
        accumulated_reserve_fees_before_borrow,
        accumulated_reserve_fees_after_borrow
    );
    assert_eq!(accumulated_reserve_fees_before_borrow, 0);
}

#[test]
fn test_accumulate_reserve_fees() {
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

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_total_borrowed_before = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let accumulated_reserve_fees_before =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    // -- Accrue debt on the pool --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR / 12;
    });

    // -- Verify the reserve is populated accordingly --

    let accumulated_reserve_fees_after =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);
    let pool_total_borrowed_after = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    let accumulated_reserve_fees_diff = accumulated_reserve_fees_after
        .checked_sub(accumulated_reserve_fees_before)
        .unwrap();
    let pool_total_borrowed_diff = pool_total_borrowed_after
        .checked_sub(pool_total_borrowed_before)
        .unwrap();

    let take_rate = get_pool_fee_config(&contract_client, &usdc_pool_address).take_rate_bps;
    let expected_accumulated_reserve_fees_diff = pool_total_borrowed_diff
        .fixed_mul_ceil(take_rate as i128, BPS_FACTOR)
        .unwrap();

    assert!(pool_total_borrowed_diff > 0);
    assert_eq!(
        accumulated_reserve_fees_diff,
        expected_accumulated_reserve_fees_diff
    );

    assert!(accumulated_reserve_fees_after > accumulated_reserve_fees_before);
}

#[test]
fn test_obligation_does_not_have_bad_debt_by_default() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.borrow(borrower, &usdc_pool_address, &i128::MAX);

    assert_eq!(
        contract_client.try_cover_obligation_bad_debt(borrower),
        Err(Ok(MCError::PositionDoesNotHaveBadDebt))
    );
}

#[test]
fn test_socialize_full_bad_debt_loss() {
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
    let liquidator = &users[2];

    contract_client.add_collateral(borrower, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    // - Borrow max possible amount -
    contract_client.borrow(borrower, &usdc_pool_address, &i128::MAX);

    // - Verify obligation is still healthy -
    assert_eq!(
        contract_client.try_liquidate(
            liquidator,
            borrower,
            &usdc_pool_address,
            &gold_pool_address,
            &1,
        ),
        Err(Ok(MCError::LiquidatedPositionIsHealthy))
    );

    // - Accrue bad debt on the pool -

    e.ledger().with_mut(|li| {
        li.timestamp += 5 * SECONDS_IN_YEAR;
    });

    let total_obligation_collateral_value =
        compute_user_obligation_collateral_value(&e, &contract_client, borrower);
    let total_obligation_debt_value =
        compute_user_obligation_debt_value(&e, &contract_client, borrower);

    // - Verify bad debt exists -

    assert!(total_obligation_debt_value > total_obligation_collateral_value);

    // - Cover bad debt(socialize all loss) -

    let pool_borrowed_before = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_available_before = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_d_tokens_before = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);
    let pool_j_tokens_before = get_pool_total_j_tokens(&contract_client, &usdc_pool_address);
    let pool_accumulated_market_fees_before =
        get_pool_accumulated_market_fees(&contract_client, &usdc_pool_address);
    let pool_accumulated_host_fees_before =
        get_pool_accumulated_host_fees(&contract_client, &usdc_pool_address);
    let pool_accumulated_reserve_fees_before =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    let available_reserve_fees =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    contract_client.cover_obligation_bad_debt(borrower);

    let pool_borrowed_after = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_available_after = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_d_tokens_after = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);
    let pool_j_tokens_after = get_pool_total_j_tokens(&contract_client, &usdc_pool_address);
    let pool_accumulated_market_fees_after =
        get_pool_accumulated_market_fees(&contract_client, &usdc_pool_address);
    let pool_accumulated_host_fees_after =
        get_pool_accumulated_host_fees(&contract_client, &usdc_pool_address);
    let pool_accumulated_reserve_fees_after =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    let pool_borrowed_diff = pool_borrowed_before
        .checked_sub(pool_borrowed_after)
        .unwrap();
    let pool_available_diff = pool_available_after
        .checked_sub(pool_available_before)
        .unwrap();
    let pool_d_tokens_diff = pool_d_tokens_before
        .checked_sub(pool_d_tokens_after)
        .unwrap();
    let pool_j_tokens_diff = pool_j_tokens_after
        .checked_sub(pool_j_tokens_before)
        .unwrap();
    let pool_accumulated_market_fees_diff = pool_accumulated_market_fees_after
        .checked_sub(pool_accumulated_market_fees_before)
        .unwrap();
    let pool_accumulated_host_fees_diff = pool_accumulated_host_fees_after
        .checked_sub(pool_accumulated_host_fees_before)
        .unwrap();
    let pool_accumulated_reserve_fees_diff = pool_accumulated_reserve_fees_before
        .checked_sub(pool_accumulated_reserve_fees_after)
        .unwrap();

    assert_eq!(pool_borrowed_diff, available_reserve_fees);
}

#[test]
fn test_partially_cover_bad_debt_socialize_loss_1() {}

#[test]
fn test_partially_cover_bad_debt_socialize_loss_2() {}
