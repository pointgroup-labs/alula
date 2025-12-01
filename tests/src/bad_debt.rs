#![cfg(test)]

use market::{constants::*, error::MCError};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::testutils::Ledger;

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, compute_pool_collateral_value,
    compute_pool_debt_value, compute_user_obligation_collateral_value,
    compute_user_obligation_debt_value, get_obligation_d_tokens_as_tokens,
    get_pool_accumulated_host_fees, get_pool_accumulated_market_fees,
    get_pool_accumulated_reserve_fees, get_pool_available_reserve_fees, get_pool_fee_config,
    get_pool_total_available, get_pool_total_borrowed, get_pool_total_d_tokens,
    get_pool_total_j_tokens,
};

#[test]
fn test_accumulate_reserve_fees_are_empty_prior_accrual() {
    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();

    let borrower = &users[0];
    let loan_provider = &users[1];

    let accumulated_reserve_fees_before_borrow =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let accumulated_reserve_fees_after_borrow =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    assert_eq!(accumulated_reserve_fees_before_borrow, accumulated_reserve_fees_after_borrow);
    assert_eq!(accumulated_reserve_fees_before_borrow, 0);
}

#[test]
fn test_accumulate_reserve_fees() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_total_borrowed_before = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let accumulated_reserve_fees_before =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    // -- Accrue debt on the pool --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR / 12;
    });
    contract_client.refresh_pool(&usdc_pool_address);

    // -- Verify the reserve is populated accordingly --

    let accumulated_reserve_fees_after =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);
    let pool_total_borrowed_after = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    let accumulated_reserve_fees_diff =
        accumulated_reserve_fees_after.checked_sub(accumulated_reserve_fees_before).unwrap();
    let pool_total_borrowed_diff =
        pool_total_borrowed_after.checked_sub(pool_total_borrowed_before).unwrap();

    let take_rate = get_pool_fee_config(&contract_client, &usdc_pool_address).take_rate_bps;
    let expected_accumulated_reserve_fees_diff =
        pool_total_borrowed_diff.fixed_mul_ceil(take_rate as i128, BPS_FACTOR).unwrap();

    assert!(pool_total_borrowed_diff > 0);
    assert_eq!(accumulated_reserve_fees_diff, expected_accumulated_reserve_fees_diff);

    assert!(accumulated_reserve_fees_after > accumulated_reserve_fees_before);
}

#[test]
fn test_obligation_does_not_have_bad_debt_by_default() {
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    contract_client.update_market(&10, &100_000);
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.borrow(borrower, &usdc_pool_address, &i128::MAX);

    assert_eq!(
        contract_client.try_cover_obligation_bad_debt(borrower),
        Err(Ok(MCError::BadDebtCoverageCriterionIsNotMet))
    );
}

#[test]
fn test_partially_socialize_full_bad_debt_loss() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    contract_client.update_market(&10, &100_000);
    let borrower = &users[0];
    let loan_provider = &users[1];
    let liquidator = &users[2];

    contract_client.add_collateral(borrower, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    // Borrow max possible amount
    contract_client.borrow(borrower, &usdc_pool_address, &i128::MAX);

    // Verify obligation is still healthy
    assert_eq!(
        contract_client.try_liquidate(
            liquidator,
            borrower,
            &None,
            &usdc_pool_address,
            &gold_pool_address,
            &1,
            &0
        ),
        Err(Ok(MCError::ObligationIsHealthy))
    );

    // - Accrue bad debt on the pool -

    e.ledger().with_mut(|li| {
        li.timestamp += 5 * SECONDS_IN_YEAR;
    });
    contract_client.refresh_pool(&usdc_pool_address);

    // - Verify bad debt exists -

    let total_obligation_collateral_value =
        compute_user_obligation_collateral_value(&e, &contract_client, borrower);
    let total_obligation_debt_value =
        compute_user_obligation_debt_value(&e, &contract_client, borrower);

    assert!(total_obligation_debt_value > total_obligation_collateral_value);

    let gold_pool_collateral_value =
        compute_pool_collateral_value(&e, &contract_client, &gold_pool_address).unwrap();
    let usdc_pool_collateral_value =
        compute_pool_collateral_value(&e, &contract_client, &usdc_pool_address).unwrap();

    let gold_pool_debt_value =
        compute_pool_debt_value(&e, &contract_client, &gold_pool_address).unwrap();
    let usdc_pool_debt_value =
        compute_pool_debt_value(&e, &contract_client, &usdc_pool_address).unwrap();

    let market_debt_value_sum = gold_pool_debt_value.checked_add(usdc_pool_debt_value).unwrap();
    let market_collateral_value_sum =
        gold_pool_collateral_value.checked_add(usdc_pool_collateral_value).unwrap();

    let market_value_diff_before =
        market_collateral_value_sum.checked_sub(market_debt_value_sum).unwrap();

    // - Cover bad debt -

    let pool_available_before = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_j_tokens_before = get_pool_total_j_tokens(&contract_client, &usdc_pool_address);
    let pool_accumulated_market_fees_before =
        get_pool_accumulated_market_fees(&contract_client, &usdc_pool_address);
    let pool_accumulated_host_fees_before =
        get_pool_accumulated_host_fees(&contract_client, &usdc_pool_address);
    let pool_accumulated_reserve_fees_before =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    let available_reserve_fees_before =
        get_pool_available_reserve_fees(&contract_client, &usdc_pool_address);

    contract_client.cover_obligation_bad_debt(borrower);

    let available_reserve_fees_after =
        get_pool_available_reserve_fees(&contract_client, &usdc_pool_address);

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

    let pool_available_diff = pool_available_after.checked_sub(pool_available_before).unwrap();
    let pool_accumulated_market_fees_diff = pool_accumulated_market_fees_after
        .checked_sub(pool_accumulated_market_fees_before)
        .unwrap();
    let pool_accumulated_host_fees_diff =
        pool_accumulated_host_fees_after.checked_sub(pool_accumulated_host_fees_before).unwrap();
    let pool_accumulated_reserve_fees_diff = pool_accumulated_reserve_fees_before
        .checked_sub(pool_accumulated_reserve_fees_after)
        .unwrap();

    assert_eq!(pool_d_tokens_after, 0);
    assert_eq!(pool_borrowed_after, 0);
    assert_eq!(pool_accumulated_host_fees_diff, 0);
    assert_eq!(pool_accumulated_market_fees_diff, 0);
    assert_eq!(pool_j_tokens_after, pool_j_tokens_before);
    assert_eq!(pool_available_diff, available_reserve_fees_before); // reserve tokens become available
    assert_eq!(pool_accumulated_reserve_fees_diff, available_reserve_fees_before); // all reserve is spent
    assert_eq!(available_reserve_fees_after, 0); // same

    // - Verify obligation no longer exists -

    assert_eq!(
        contract_client.try_get_user_obligation(borrower),
        Err(Ok(MCError::ObligationDoesNotExist))
    );

    // - Verify that market became healthier -
    let gold_pool_collateral_value =
        compute_pool_collateral_value(&e, &contract_client, &gold_pool_address).unwrap();
    let usdc_pool_collateral_value =
        compute_pool_collateral_value(&e, &contract_client, &usdc_pool_address).unwrap();

    let gold_pool_debt_value =
        compute_pool_debt_value(&e, &contract_client, &gold_pool_address).unwrap();
    let usdc_pool_debt_value =
        compute_pool_debt_value(&e, &contract_client, &usdc_pool_address).unwrap();

    let market_debt_value_sum = gold_pool_debt_value.checked_add(usdc_pool_debt_value).unwrap();
    let market_collateral_value_sum =
        gold_pool_collateral_value.checked_add(usdc_pool_collateral_value).unwrap();

    let market_value_diff_after =
        market_collateral_value_sum.checked_sub(market_debt_value_sum).unwrap();

    dbg!(market_value_diff_before, market_value_diff_after); // MEGA_WARN. This issue still
    // persists
    assert!(market_value_diff_before > market_value_diff_after);
}

#[test]
fn test_completely_socialize_loss() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    contract_client.update_market(&10, &100_000);
    let borrower_1 = &users[0];
    let borrower_2 = &users[1];
    let loan_provider = &users[2];

    contract_client.add_collateral(borrower_1, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.add_collateral(borrower_2, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(loan_provider, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    // Borrow max possible amounts
    contract_client.borrow(borrower_1, &usdc_pool_address, &i128::MAX);
    contract_client.borrow(borrower_2, &usdc_pool_address, &i128::MAX);

    // - Accrue bad debt on the pool -

    e.ledger().with_mut(|li| {
        li.timestamp += 5 * SECONDS_IN_YEAR;
    });
    contract_client.refresh_obligation(borrower_1);

    // - Verify bad debt exists -

    let total_obligation_collateral_value =
        compute_user_obligation_collateral_value(&e, &contract_client, borrower_1);
    let total_obligation_debt_value =
        compute_user_obligation_debt_value(&e, &contract_client, borrower_1);

    assert!(total_obligation_debt_value > total_obligation_collateral_value);

    // - Partially cover bad debt to one of the borrowers -
    contract_client.cover_obligation_bad_debt(borrower_1);

    // - Verify no more reserves left -

    let available_reserve_fees =
        get_pool_available_reserve_fees(&contract_client, &usdc_pool_address);
    assert_eq!(available_reserve_fees, 0);

    // - Verify 2nd bad debt exists -

    let total_obligation_collateral_value =
        compute_user_obligation_collateral_value(&e, &contract_client, borrower_2);
    let total_obligation_debt_value =
        compute_user_obligation_debt_value(&e, &contract_client, borrower_2);

    assert!(total_obligation_debt_value > total_obligation_collateral_value);

    // - Verify complete socialization takes place -

    let pool_available_before = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_d_tokens_before = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);
    let pool_j_tokens_before = get_pool_total_j_tokens(&contract_client, &usdc_pool_address);
    let pool_accumulated_reserve_fees_before =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    let accumulated_reserve_fees_before =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);
    let available_reserve_fees_before =
        get_pool_available_reserve_fees(&contract_client, &usdc_pool_address);

    contract_client.cover_obligation_bad_debt(borrower_2);

    let accumulated_reserve_fees_after =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);
    let available_reserve_fees_after =
        get_pool_available_reserve_fees(&contract_client, &usdc_pool_address);

    let pool_borrowed_after = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_available_after = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_d_tokens_after = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);
    let pool_j_tokens_after = get_pool_total_j_tokens(&contract_client, &usdc_pool_address);
    let pool_accumulated_reserve_fees_after =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    let pool_available_diff = pool_available_after.checked_sub(pool_available_before).unwrap();
    let pool_accumulated_reserve_fees_diff = pool_accumulated_reserve_fees_before
        .checked_sub(pool_accumulated_reserve_fees_after)
        .unwrap();

    assert_eq!(pool_d_tokens_after, 0);
    assert!(pool_d_tokens_before > pool_d_tokens_after);
    assert_eq!(pool_borrowed_after, 0); // due to loss socialization
    assert_eq!(pool_j_tokens_after, pool_j_tokens_before);
    assert_eq!(pool_available_diff, 0); // no reserve to cover debt
    assert_eq!(pool_accumulated_reserve_fees_diff, 0); // no reserve to cover bad debt
    assert_eq!(available_reserve_fees_after, 0);
    assert_eq!(available_reserve_fees_before, 0);
    assert_eq!(accumulated_reserve_fees_after, 0);
    assert_eq!(accumulated_reserve_fees_before, 0);
}

#[test]
fn test_completely_cover_bad_debt() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    contract_client.update_market(&10, &100_000);
    let borrower_1 = &users[0];
    let borrower_2 = &users[1];
    let loan_provider = &users[2];

    contract_client.add_collateral(borrower_1, &gold_pool_address, &(10 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.add_collateral(borrower_2, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(loan_provider, &usdc_pool_address, &(20 * DEFAULT_DEPOSIT_AMOUNT));

    // Borrow max possible amounts
    contract_client.borrow(borrower_1, &usdc_pool_address, &i128::MAX); // will borrow x10 due to having x10 more collateral
    contract_client.borrow(borrower_2, &usdc_pool_address, &i128::MAX);

    // - Accrue bad debt on the pool -

    e.ledger().with_mut(|li| {
        li.timestamp += 15 * SECONDS_IN_YEAR;
    });
    contract_client.refresh_pool(&usdc_pool_address);

    // - Verify bad debt on 2nd borrower exists

    let total_obligation_collateral_value =
        compute_user_obligation_collateral_value(&e, &contract_client, borrower_2);
    let total_obligation_debt_value =
        compute_user_obligation_debt_value(&e, &contract_client, borrower_2);

    assert!(total_obligation_debt_value > total_obligation_collateral_value);

    // - Verify that reserve can cover it -

    let borrower_2_debt_before =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower_2, &usdc_pool_address)
            .unwrap();
    let pool_available_reserve_fees_before =
        get_pool_available_reserve_fees(&contract_client, &usdc_pool_address);
    let pool_accumulated_reserve_fees_before =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    assert!(pool_available_reserve_fees_before > borrower_2_debt_before);

    // - Verify complete bad debt covering takes place -

    let pool_borrowed_before = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_available_before = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_d_tokens_before = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);
    let pool_j_tokens_before = get_pool_total_j_tokens(&contract_client, &usdc_pool_address);

    contract_client.cover_obligation_bad_debt(borrower_2);

    // - Verify 2nd obligation no longer exists -

    assert_eq!(
        contract_client.try_get_user_obligation(borrower_2),
        Err(Ok(MCError::ObligationDoesNotExist))
    );

    let pool_borrowed_after = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_available_after = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_d_tokens_after = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);
    let pool_j_tokens_after = get_pool_total_j_tokens(&contract_client, &usdc_pool_address);
    let pool_accumulated_reserve_fees_after =
        get_pool_accumulated_reserve_fees(&contract_client, &usdc_pool_address);

    let pool_available_diff = pool_available_after.checked_sub(pool_available_before).unwrap();
    let pool_borrowed_diff = pool_borrowed_before.checked_sub(pool_borrowed_after).unwrap();
    let pool_accumulated_reserve_fees_diff = pool_accumulated_reserve_fees_before
        .checked_sub(pool_accumulated_reserve_fees_after)
        .unwrap();

    assert!(pool_d_tokens_after > 0); // another borrower still has bad debt
    assert!(pool_borrowed_after > 0);
    assert!(pool_d_tokens_before > pool_d_tokens_after);
    assert!(pool_borrowed_before > pool_borrowed_after);

    assert_eq!(pool_j_tokens_after, pool_j_tokens_before);
    assert_eq!(pool_available_diff, pool_accumulated_reserve_fees_diff); // complete coverage took place
    assert_eq!(pool_borrowed_diff, pool_accumulated_reserve_fees_diff);
    assert!(pool_available_after > 0); // reserve fees are left
}
