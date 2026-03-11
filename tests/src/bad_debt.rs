#![cfg(test)]

use market::{constants::SECONDS_IN_YEAR, error::MCError, obligation::ObligationKey};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{map as smap, testutils::Ledger};

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, compute_user_obligation_collateral_value,
    compute_user_obligation_debt_value, get_obligation_d_tokens_as_tokens,
    get_pool_total_available, get_pool_total_borrowed, get_pool_total_d_tokens,
    get_pool_total_j_tokens,
};

#[test]
fn test_obligation_does_not_have_bad_debt_by_default() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    let update_in_queue_period = contract_client.get_global_state().update_in_queue_period;
    contract_client.queue_in_market_update(&10, &1);
    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);
    contract_client.apply_market_update();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &i128::MAX,
        &None,
    );

    assert_eq!(
        contract_client.try_issue_cover_bad_debt(&ObligationKey::new(borrower.clone())),
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
    let borrower = &users[0];
    let liquidity_provider = &users[1];
    let liquidator = &users[2];

    contract_client.set_take_rate_fees_beneficiaries(
        &usdc_pool_address,
        &smap![&e, (insurance_fund.clone(), 10_000)],
    );

    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    // Borrow max possible amount
    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &i128::MAX,
        &None,
    );

    // Verify obligation is still healthy
    assert_eq!(
        contract_client.try_liquidate(
            liquidator,
            &ObligationKey::new(borrower.clone()),
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

    // - Cover bad debt -

    let debt_amount =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    contract_client.liquidate(
        liquidator,
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &gold_pool_address,
        &debt_amount.fixed_mul_ceil(90, 100).unwrap(),
        &DEFAULT_DEPOSIT_AMOUNT,
    );

    let remaining_debt =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();
    assert_eq!(remaining_debt, debt_amount.fixed_mul_ceil(10, 100).unwrap());

    let total_obligation_collateral_value =
        compute_user_obligation_collateral_value(&e, &contract_client, borrower);
    let total_obligation_debt_value =
        compute_user_obligation_debt_value(&e, &contract_client, borrower);

    assert!(total_obligation_debt_value > total_obligation_collateral_value);
    assert_eq!(total_obligation_collateral_value, 0); // i.e. bad debt

    contract_client.distribute_all_pools_fees();

    let insurance_fund_balance_after = usdc_token_client.balance(&insurance_fund);
    assert!(insurance_fund_balance_after > 0);
    assert!(insurance_fund_balance_after < remaining_debt); // partial coverage

    let pool_data_before = contract_client.get_pool_data(&usdc_pool_address);

    let j_token_rate_before = pool_data_before.j_token_rate_floor_bps;
    let pool_total_available_before = pool_data_before.pool.total_available;
    assert_eq!(pool_data_before.pool.take_rate_fees_sum, 0);

    // - Partially cover bad debt -

    contract_client.issue_cover_bad_debt(&ObligationKey::new(borrower.clone()));
    controlled_insurance_fund_client.mark_ready(&0, &insurance_fund_balance_after);
    contract_client.claim_cover_bad_debt_results(&ObligationKey::new(borrower.clone()));

    let pool_data_after = contract_client.get_pool_data(&usdc_pool_address);

    let j_token_rate_after = pool_data_after.j_token_rate_floor_bps;
    let pool_debt_after = pool_data_after.pool.total_borrowed;
    let pool_total_available_after = pool_data_after.pool.total_available;

    // - Verify obligation no longer exists -

    assert_eq!(
        contract_client.try_get_user_obligation(&ObligationKey::new(borrower.clone())),
        Err(Ok(MCError::ObligationDoesNotExist))
    );
    // - Verify that partial bad debt coverage took place -

    assert_eq!(pool_debt_after, 0);
    assert_eq!(
        pool_total_available_after,
        pool_total_available_before.checked_add(insurance_fund_balance_after).unwrap()
    );

    assert!(j_token_rate_before > j_token_rate_after);
}

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
        &ObligationKey::new(borrower_1.clone()),
        &gold_pool_address,
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.add_collateral(
        &ObligationKey::new(borrower_2.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(20 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    // Borrow max possible amounts
    contract_client.borrow(
        &ObligationKey::new(borrower_1.clone()),
        &usdc_pool_address,
        &i128::MAX,
        &None,
    ); // will borrow x10 due to having x10 more collateral
    contract_client.borrow(
        &ObligationKey::new(borrower_2.clone()),
        &usdc_pool_address,
        &i128::MAX,
        &None,
    );

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
        &ObligationKey::new(borrower_2.clone()),
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

    contract_client.issue_cover_bad_debt(&ObligationKey::new(borrower_2.clone()));
    controlled_insurance_fund_client.mark_ready(&0, &insurance_fund_balance);
    contract_client.claim_cover_bad_debt_results(&ObligationKey::new(borrower_2.clone()));

    // - Verify 2nd obligation no longer exists -

    assert_eq!(
        contract_client.try_get_user_obligation(&ObligationKey::new(borrower_2.clone())),
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
