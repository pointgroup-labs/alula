#![cfg(test)]

use market::{
    constants::{DEFAULT_BAD_DEBT_LOCK_D, SECONDS_IN_YEAR, SECONDS_PER_HOUR},
    error::MCError,
    obligation::ObligationKey,
};
use insurance_fund_interface::CoverageStatus;
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{map as smap, testutils::Ledger};

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture,
    compute_user_obligation_collateral_value, compute_user_obligation_debt_value,
    get_obligation_d_tokens_as_tokens, get_pool_total_available, get_pool_total_borrowed,
    get_pool_total_d_tokens, get_pool_total_j_tokens,
};

#[test]
fn test_obligation_does_not_have_bad_debt_by_default() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    let update_in_queue_period = contract_client.get_global_state().update_in_queue_period;
    contract_client.queue_in_market_update(&10, &1, &DEFAULT_BAD_DEBT_LOCK_D);
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

// ---- Bad Debt Tests ----

/// Helper: creates a market with bad debt on USDC pool (Recorded coverage path),
/// returns fixture with the lock already active on the USDC pool.
fn setup_bad_debt_with_recorded_coverage() -> (
    TestMarketFixture<'static>,
    soroban_sdk::Address, // borrower
    soroban_sdk::Address, // liquidity_provider
) {
    let fixture = TestMarketFixture::new();
    let TestMarketFixture {
        ref e,
        ref contract_client,
        ref insurance_fund,
        ref usdc_pool_address,
        ref gold_pool_address,
        ref users,
        ..
    } = fixture;
    let borrower = users[0].clone();
    let liquidity_provider = users[1].clone();
    let liquidator = &users[2];

    contract_client.set_take_rate_fees_beneficiaries(
        usdc_pool_address,
        &smap![e, (insurance_fund.clone(), 10_000)],
    );

    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        usdc_pool_address,
        &i128::MAX,
        &None,
    );

    e.ledger().with_mut(|li| {
        li.timestamp += 5 * SECONDS_IN_YEAR;
    });
    contract_client.refresh_pool(usdc_pool_address);
    contract_client.distribute_all_pools_fees();

    let debt_amount =
        get_obligation_d_tokens_as_tokens(e, contract_client, &borrower, usdc_pool_address)
            .unwrap();

    contract_client.liquidate(
        liquidator,
        &ObligationKey::new(borrower.clone()),
        usdc_pool_address,
        gold_pool_address,
        &debt_amount.fixed_mul_ceil(90, 100).unwrap(),
        &DEFAULT_DEPOSIT_AMOUNT,
    );

    // issue_cover_bad_debt triggers the Recorded path → activates the lock
    contract_client.issue_cover_bad_debt(&ObligationKey::new(borrower.clone()));

    (fixture, borrower, liquidity_provider)
}

#[test]
fn test_bad_debt_lock_blocks_withdraw() {
    let (fixture, _borrower, liquidity_provider) = setup_bad_debt_with_recorded_coverage();
    let TestMarketFixture { ref contract_client, ref usdc_pool_address, .. } = fixture;

    let pool = contract_client.get_pool(usdc_pool_address);
    assert!(pool.bad_debt_lock_d > 0);
    assert_eq!(pool.bad_debt_request_count, 1);

    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(liquidity_provider.clone()),
            usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::PoolBadDebtLocked))
    );
}

#[test]
fn test_bad_debt_lock_blocks_simulate_withdraw() {
    let (fixture, _borrower, liquidity_provider) = setup_bad_debt_with_recorded_coverage();
    let TestMarketFixture { ref contract_client, ref usdc_pool_address, .. } = fixture;

    assert!(matches!(
        contract_client.try_simulate_withdraw(
            &ObligationKey::new(liquidity_provider.clone()),
            usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::PoolBadDebtLocked))
    ));
}

#[test]
fn test_bad_debt_lock_blocks_deposit() {
    let (fixture, _borrower, _liquidity_provider) = setup_bad_debt_with_recorded_coverage();
    let TestMarketFixture { ref contract_client, ref usdc_pool_address, ref users, .. } = fixture;
    let another_user = &users[3];

    assert_eq!(
        contract_client.try_deposit(
            &ObligationKey::new(another_user.clone()),
            usdc_pool_address,
            &100,
            &None
        ),
        Err(Ok(MCError::PoolBadDebtLocked))
    );
}

#[test]
fn test_bad_debt_lock_does_not_block_collateral_ops() {
    let (fixture, _borrower, _liquidity_provider) = setup_bad_debt_with_recorded_coverage();
    let TestMarketFixture { ref contract_client, ref gold_pool_address, ref users, .. } = fixture;
    let another_user = &users[3];

    assert!(
        contract_client
            .try_add_collateral(
                &ObligationKey::new(another_user.clone()),
                gold_pool_address,
                &DEFAULT_COLLATERAL_AMOUNT,
                &None
            )
            .is_ok()
    );
}

#[test]
fn test_bad_debt_lock_does_not_affect_other_pools() {
    let (fixture, _borrower, _liquidity_provider) = setup_bad_debt_with_recorded_coverage();
    let TestMarketFixture {
        ref contract_client,
        ref gold_pool_address,
        ref usdc_pool_address,
        ref users,
        ..
    } = fixture;
    let depositor = &users[3];

    // USDC pool is locked
    let usdc_pool = contract_client.get_pool(usdc_pool_address);
    assert!(usdc_pool.bad_debt_lock_d > 0);

    // GOLD pool is not locked
    let gold_pool = contract_client.get_pool(gold_pool_address);
    assert_eq!(gold_pool.bad_debt_lock_d, 0);

    contract_client.deposit(
        &ObligationKey::new(depositor.clone()),
        gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    assert!(
        contract_client
            .try_withdraw(&ObligationKey::new(depositor.clone()), gold_pool_address, &1, &None)
            .is_ok()
    );
}

#[test]
fn test_bad_debt_lock_expires_after_deadline() {
    let (fixture, _borrower, liquidity_provider) = setup_bad_debt_with_recorded_coverage();
    let TestMarketFixture { ref e, ref contract_client, ref usdc_pool_address, .. } = fixture;

    let pool = contract_client.get_pool(usdc_pool_address);
    let deadline = pool.bad_debt_lock_d;

    // Still locked 1 second before deadline
    e.ledger().with_mut(|li| li.timestamp = deadline - 1);
    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(liquidity_provider.clone()),
            usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::PoolBadDebtLocked))
    );

    // Exactly at deadline: lock expires
    e.ledger().with_mut(|li| li.timestamp = deadline);
    assert!(
        contract_client
            .try_withdraw(
                &ObligationKey::new(liquidity_provider.clone()),
                usdc_pool_address,
                &1,
                &None
            )
            .is_ok()
    );
}

#[test]
fn test_withdraw_unblocked_at_deadline_while_request_still_pending() {
    let (fixture, _borrower, liquidity_provider) = setup_bad_debt_with_recorded_coverage();
    let TestMarketFixture {
        ref e,
        ref contract_client,
        ref insurance_fund_client,
        ref usdc_token_client,
        ref contract_id,
        ref usdc_pool_address,
        ..
    } = fixture;

    let mut ts_before = 0u64;
    e.ledger().with_mut(|li| ts_before = li.timestamp);
    let pool_before = contract_client.get_pool(usdc_pool_address);
    let status_before = insurance_fund_client.get_status(&0);
    let status_before_label = match &status_before {
        Some(CoverageStatus::Pending) => "Pending",
        Some(CoverageStatus::Ready(_)) => "Ready",
        None => "None",
    };

    println!(
        "[poc-before] ts={} lock_deadline={} request_count={} request_status={}",
        ts_before,
        pool_before.bad_debt_lock_d,
        pool_before.bad_debt_request_count,
        status_before_label
    );
    let user_balance_before = usdc_token_client.balance(&liquidity_provider);
    let market_balance_before = usdc_token_client.balance(contract_id);
    println!(
        "[poc-before-balances] user_usdc={} market_usdc={}",
        user_balance_before, market_balance_before
    );

    assert!(pool_before.bad_debt_lock_d > 0);
    assert_eq!(pool_before.bad_debt_request_count, 1);
    assert!(matches!(status_before, Some(CoverageStatus::Pending)));

    // Lock expires exactly at deadline even though request is unresolved.
    e.ledger().with_mut(|li| li.timestamp = pool_before.bad_debt_lock_d);
    let requested_withdraw = i128::MAX;
    let withdraw_result = contract_client.try_withdraw(
        &ObligationKey::new(liquidity_provider.clone()),
        usdc_pool_address,
        &requested_withdraw,
        &None,
    );
    println!(
        "[poc-action] moved_to_deadline_ts={} requested_withdraw={} withdraw_result_is_ok={}",
        pool_before.bad_debt_lock_d,
        requested_withdraw,
        withdraw_result.is_ok()
    );
    assert!(withdraw_result.is_ok());

    let mut ts_after = 0u64;
    e.ledger().with_mut(|li| ts_after = li.timestamp);
    let pool_after = contract_client.get_pool(usdc_pool_address);
    let status_after = insurance_fund_client.get_status(&0);
    let status_after_label = match &status_after {
        Some(CoverageStatus::Pending) => "Pending",
        Some(CoverageStatus::Ready(_)) => "Ready",
        None => "None",
    };

    println!(
        "[poc-after] ts={} lock_deadline={} request_count={} request_status={}",
        ts_after,
        pool_after.bad_debt_lock_d,
        pool_after.bad_debt_request_count,
        status_after_label
    );
    let user_balance_after = usdc_token_client.balance(&liquidity_provider);
    let market_balance_after = usdc_token_client.balance(contract_id);
    println!(
        "[poc-after-balances] user_usdc={} market_usdc={} user_delta={} market_delta={}",
        user_balance_after,
        market_balance_after,
        user_balance_after - user_balance_before,
        market_balance_after - market_balance_before
    );

    assert_eq!(pool_after.bad_debt_request_count, 1);
    assert!(matches!(status_after, Some(CoverageStatus::Pending)));
    assert!(user_balance_after > user_balance_before);
    assert!(user_balance_after - user_balance_before > 1);
}

#[test]
fn test_bad_debt_lock_cleared_on_claim() {
    let (fixture, borrower, liquidity_provider) = setup_bad_debt_with_recorded_coverage();
    let TestMarketFixture {
        ref contract_client,
        ref controlled_insurance_fund_client,
        ref usdc_token_client,
        ref usdc_pool_address,
        ref insurance_fund,
        ..
    } = fixture;

    // Pool is locked
    let pool_before = contract_client.get_pool(usdc_pool_address);
    assert!(pool_before.bad_debt_lock_d > 0);
    assert_eq!(pool_before.bad_debt_request_count, 1);

    // Resolve the bad debt via the insurance fund
    let insurance_fund_balance = usdc_token_client.balance(insurance_fund);
    controlled_insurance_fund_client.mark_ready(&0, &insurance_fund_balance);
    contract_client.claim_cover_bad_debt_results(&ObligationKey::new(borrower.clone()));

    // Lock is cleared
    let pool_after = contract_client.get_pool(usdc_pool_address);
    assert_eq!(pool_after.bad_debt_lock_d, 0);
    assert_eq!(pool_after.bad_debt_request_count, 0);

    // Withdrawal now works
    assert!(
        contract_client
            .try_withdraw(
                &ObligationKey::new(liquidity_provider.clone()),
                usdc_pool_address,
                &1,
                &None
            )
            .is_ok()
    );
}

#[test]
fn test_consecutive_bad_debt_requests_extend_deadline_and_increment_counter() {
    let fixture = TestMarketFixture::new();
    let TestMarketFixture {
        ref e,
        ref contract_client,
        ref insurance_fund,
        ref controlled_insurance_fund_client,
        ref usdc_token_client,
        ref usdc_pool_address,
        ref gold_pool_address,
        ref users,
        ..
    } = fixture;
    let borrower_1 = &users[0];
    let borrower_2 = &users[1];
    let liquidity_provider = &users[2];
    let liquidator = &users[3];

    contract_client.set_take_rate_fees_beneficiaries(
        usdc_pool_address,
        &smap![e, (insurance_fund.clone(), 10_000)],
    );

    contract_client.add_collateral(
        &ObligationKey::new(borrower_1.clone()),
        gold_pool_address,
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.add_collateral(
        &ObligationKey::new(borrower_2.clone()),
        gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        usdc_pool_address,
        &(20 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    contract_client.borrow(
        &ObligationKey::new(borrower_1.clone()),
        usdc_pool_address,
        &i128::MAX,
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(borrower_2.clone()),
        usdc_pool_address,
        &i128::MAX,
        &None,
    );

    e.ledger().with_mut(|li| {
        li.timestamp += 15 * SECONDS_IN_YEAR;
    });
    contract_client.refresh_pool(usdc_pool_address);
    contract_client.distribute_pool_fees(usdc_pool_address);

    // Liquidate borrower_2 down to bad debt
    let debt_2 =
        get_obligation_d_tokens_as_tokens(e, contract_client, borrower_2, usdc_pool_address)
            .unwrap();
    contract_client.liquidate(
        liquidator,
        &ObligationKey::new(borrower_2.clone()),
        usdc_pool_address,
        gold_pool_address,
        &debt_2.fixed_mul_ceil(98, 100).unwrap(),
        &DEFAULT_DEPOSIT_AMOUNT,
    );

    // Liquidate borrower_1 down to bad debt
    let debt_1 =
        get_obligation_d_tokens_as_tokens(e, contract_client, borrower_1, usdc_pool_address)
            .unwrap();
    contract_client.liquidate(
        liquidator,
        &ObligationKey::new(borrower_1.clone()),
        usdc_pool_address,
        gold_pool_address,
        &debt_1.fixed_mul_ceil(98, 100).unwrap(),
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
    );

    // -- First issue_cover_bad_debt --
    contract_client.issue_cover_bad_debt(&ObligationKey::new(borrower_2.clone()));

    let pool_after_first = contract_client.get_pool(usdc_pool_address);
    let first_deadline = pool_after_first.bad_debt_lock_d;
    assert!(first_deadline > 0);
    assert_eq!(pool_after_first.bad_debt_request_count, 1);

    // Advance 8 hours
    e.ledger().with_mut(|li| li.timestamp += 8 * SECONDS_PER_HOUR);

    // -- Second issue_cover_bad_debt --
    contract_client.issue_cover_bad_debt(&ObligationKey::new(borrower_1.clone()));

    let pool_after_second = contract_client.get_pool(usdc_pool_address);
    let second_deadline = pool_after_second.bad_debt_lock_d;
    assert_eq!(pool_after_second.bad_debt_request_count, 2);
    assert!(second_deadline > first_deadline, "Deadline should be extended");

    // -- Resolve first request, counter decrements but lock stays --
    let insurance_balance = usdc_token_client.balance(insurance_fund);
    controlled_insurance_fund_client.mark_ready(&0, &insurance_balance);
    contract_client.claim_cover_bad_debt_results(&ObligationKey::new(borrower_2.clone()));

    let pool_after_first_claim = contract_client.get_pool(usdc_pool_address);
    assert_eq!(pool_after_first_claim.bad_debt_request_count, 1);
    assert!(
        pool_after_first_claim.bad_debt_lock_d > 0,
        "Lock must remain active with 1 outstanding request"
    );

    // Withdrawals still blocked
    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(liquidity_provider.clone()),
            usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::PoolBadDebtLocked))
    );

    // -- Resolve second request, counter reaches 0 → lock cleared --
    let insurance_balance = usdc_token_client.balance(insurance_fund);
    controlled_insurance_fund_client.mark_ready(&1, &insurance_balance);
    contract_client.claim_cover_bad_debt_results(&ObligationKey::new(borrower_1.clone()));

    let pool_final = contract_client.get_pool(usdc_pool_address);
    assert_eq!(pool_final.bad_debt_request_count, 0);
    assert_eq!(pool_final.bad_debt_lock_d, 0);

    // Withdrawals unblocked
    assert!(
        contract_client
            .try_withdraw(
                &ObligationKey::new(liquidity_provider.clone()),
                usdc_pool_address,
                &1,
                &None
            )
            .is_ok()
    );
}
