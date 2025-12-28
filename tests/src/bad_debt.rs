#![cfg(test)]

use market::{constants::SECONDS_IN_YEAR, error::MCError};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{map as smap, testutils::Ledger};

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, compute_pool_collateral_value,
    compute_pool_debt_value, compute_user_obligation_collateral_value,
    compute_user_obligation_debt_value, get_obligation_d_tokens_as_tokens,
    get_pool_total_available, get_pool_total_borrowed, get_pool_total_d_tokens,
    get_pool_total_j_tokens,
};

#[test]
fn test_obligation_does_not_have_bad_debt_by_default() {
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    contract_client.update_market(&10, &1);
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(10 * DEFAULT_DEPOSIT_AMOUNT), &None);
    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.borrow(borrower, &usdc_pool_address, &i128::MAX, &None);

    assert_eq!(
        contract_client.try_issue_cover_bad_debt(borrower),
        Err(Ok(MCError::BadDebtCoverageCriterionIsNotMet))
    );
}

#[test]
fn test_partially_socialize_full_bad_debt_loss() {
    let TestMarketFixture {
        e,
        contract_client,
        insurance_fund,

        controlled_insurance_fund_client,
        usdc_token_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    // contract_client.update_market(&10, &10);
    let borrower = &users[0];
    let liquidity_provider = &users[1];
    let liquidator = &users[2];

    contract_client.set_take_rate_fees_beneficiaries(
        &usdc_pool_address,
        &smap![&e, (insurance_fund.clone(), 10_000)],
    );

    contract_client.add_collateral(borrower, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.deposit(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    // Borrow max possible amount
    contract_client.borrow(borrower, &usdc_pool_address, &i128::MAX, &None);

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

    let insurance_fund_balance_before = usdc_token_client.balance(&insurance_fund);
    assert_eq!(insurance_fund_balance_before, 0);

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

    let debt_amount =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    contract_client.liquidate(
        liquidator,
        borrower,
        &None,
        &usdc_pool_address,
        &gold_pool_address,
        &debt_amount.fixed_mul_ceil(98, 100).unwrap(),
        &DEFAULT_DEPOSIT_AMOUNT,
    );

    contract_client.distribute_all_pools_fees();

    let insurance_fund_balance_after = usdc_token_client.balance(&insurance_fund);
    assert!(insurance_fund_balance_after > 0);

    let pool_j_tokens_before = get_pool_total_j_tokens(&contract_client, &usdc_pool_address);

    contract_client.issue_cover_bad_debt(borrower);
    controlled_insurance_fund_client.mark_ready(&0, &insurance_fund_balance_after);
    contract_client.claim_cover_bad_debt_results(borrower);

    let pool_borrowed_after = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_d_tokens_after = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);
    let pool_j_tokens_after = get_pool_total_j_tokens(&contract_client, &usdc_pool_address);

    assert_eq!(pool_d_tokens_after, 0);
    assert_eq!(pool_borrowed_after, 0);
    assert_eq!(pool_j_tokens_after, pool_j_tokens_before);

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

// TODO: Add missing test for complete socialization

#[test]
fn test_completely_cover_bad_debt() {
    let TestMarketFixture {
        e,
        contract_client,
        insurance_fund,
        controlled_insurance_fund_client,
        usdc_token_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let borrower_1 = &users[0];
    let borrower_2 = &users[1];
    let liquidity_provider = &users[2];
    let liquidator = &users[3];

    contract_client.set_take_rate_fees_beneficiaries(
        &usdc_pool_address,
        &smap![&e, (insurance_fund.clone(), 10_000)],
    );

    contract_client.add_collateral(
        borrower_1,
        &gold_pool_address,
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.add_collateral(borrower_2, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(20 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    // Borrow max possible amounts
    contract_client.borrow(borrower_1, &usdc_pool_address, &i128::MAX, &None); // will borrow x10 due to having x10 more collateral
    contract_client.borrow(borrower_2, &usdc_pool_address, &i128::MAX, &None);

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
    contract_client.distribute_pool_fees(&usdc_pool_address);

    let borrower_2_debt_before =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower_2, &usdc_pool_address)
            .unwrap();
    let insurance_fund_balance = usdc_token_client.balance(&insurance_fund);
    assert!(borrower_2_debt_before < insurance_fund_balance);

    contract_client.liquidate(
        liquidator,
        borrower_2,
        &None,
        &usdc_pool_address,
        &gold_pool_address,
        &borrower_2_debt_before.fixed_mul_ceil(98, 100).unwrap(),
        &DEFAULT_DEPOSIT_AMOUNT,
    );

    let borrower_2_new_debt =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower_2, &usdc_pool_address)
            .unwrap();

    // - Verify complete bad debt covering takes place -

    let pool_borrowed_before = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_available_before = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_d_tokens_before = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);
    let pool_j_tokens_before = get_pool_total_j_tokens(&contract_client, &usdc_pool_address);

    contract_client.issue_cover_bad_debt(borrower_2);
    controlled_insurance_fund_client.mark_ready(&0, &insurance_fund_balance);
    contract_client.claim_cover_bad_debt_results(borrower_2);

    // - Verify 2nd obligation no longer exists -

    assert_eq!(
        contract_client.try_get_user_obligation(borrower_2),
        Err(Ok(MCError::ObligationDoesNotExist))
    );

    let pool_borrowed_after = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_available_after = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_d_tokens_after = get_pool_total_d_tokens(&contract_client, &usdc_pool_address);
    let pool_j_tokens_after = get_pool_total_j_tokens(&contract_client, &usdc_pool_address);

    let pool_available_diff = pool_available_after.checked_sub(pool_available_before).unwrap();
    let pool_borrowed_diff = pool_borrowed_before.checked_sub(pool_borrowed_after).unwrap();

    assert!(pool_d_tokens_after > 0); // another borrower still has bad debt
    assert!(pool_borrowed_after > 0);
    assert!(pool_d_tokens_before > pool_d_tokens_after);
    assert!(pool_borrowed_before > pool_borrowed_after);

    assert_eq!(pool_j_tokens_after, pool_j_tokens_before);
    assert_eq!(pool_available_diff, borrower_2_new_debt); // complete coverage took place
    assert_eq!(pool_borrowed_diff, borrower_2_new_debt);
}

#[test]
fn test_donate() {
    let TestMarketFixture { contract_client, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let donor = &users[0];

    let reserve_before = contract_client.get_pool(&usdc_pool_address).total_available;
    assert_eq!(reserve_before, 0);

    let donation_amount = 1_000_000_000;
    contract_client.donate(donor, &usdc_pool_address, &donation_amount);

    let reserve_after = contract_client.get_pool(&usdc_pool_address).total_available;
    assert_eq!(reserve_after, donation_amount);
}
