#![cfg(test)]

use insurance_fund_interface::{CoverageStatus, IssueRequestResult};
use market::{
    constants::{
        DEFAULT_BAD_DEBT_LOCK_D, MAX_ORACLE_PRICE_AGE_SECONDS, SECONDS_IN_YEAR, SECONDS_PER_DAY,
        SECONDS_PER_HOUR,
    },
    error::MCError,
    obligation::ObligationKey,
    pool::{PoolConfig, PoolHealthConfig},
};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, BytesN, Env, contract, contractimpl, map as smap,
    testutils::{Address as _, Ledger},
    token::TokenClient,
};

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, assert_approx_eq_abs,
    compute_user_obligation_collateral_value, compute_user_obligation_debt_value,
    get_obligation_collateral, get_obligation_d_tokens_as_tokens,
    get_obligation_j_tokens_as_tokens, get_pool_take_rate_fees_sum, get_pool_total_available,
    get_pool_total_borrowed, get_pool_total_d_tokens, get_pool_total_j_tokens,
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
    let (fixture, borrower, liquidity_provider) = setup_bad_debt_with_recorded_coverage();
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

    // Exactly at deadline: still locked because the pending request hasn't been
    // resolved yet (count > 0). This closes the audit's "Withdrawal Operations
    // Possible with Bad Debt" race window.
    e.ledger().with_mut(|li| li.timestamp = deadline);
    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(liquidity_provider.clone()),
            usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::PoolBadDebtLocked))
    );

    // Anyone can now finalize the expired pending request via the existing
    // permissionless claim entrypoint, which cancels with the Insurance Fund and
    // socializes the loss as zero recovery.
    contract_client.claim_cover_bad_debt_results(&ObligationKey::new(borrower.clone()));

    // After finalization the pool is unlocked and withdrawals succeed.
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

/// **A claim reports how many requests it settled.** Both outcomes return `Ok(())`, so the event
/// is the only way a keeper tells a closing claim from one that found every request pending.
#[test]
fn test_a_claim_reports_how_many_requests_it_settled() {
    use soroban_sdk::testutils::Events;

    fn settled_count(fixture: &TestMarketFixture) -> u32 {
        let events = fixture.e.events().all().filter_by_contract(&fixture.contract_id);
        let text = std::format!("{:?}", events.events());
        // The market's event structs are private, so the value is read out of the encoded form.
        let at = text.rfind("settled").expect("no settled field in the claim event");
        // Anchored past the type tag: the encoded form reads `val: U32(0)`, and skipping straight to the
        // first digit would read the 32 out of `U32`.
        let tail = &text[at..];
        let val_at = tail.find("U32(").expect("settled was not encoded as a u32");
        let digits: std::string::String =
            tail[val_at + 4..].chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse().expect("settled carried no number")
    }

    // Still pending: nothing can be closed.
    let (fixture, borrower, _lp) = setup_bad_debt_with_recorded_coverage();
    fixture.contract_client.claim_cover_bad_debt_results(&ObligationKey::new(borrower.clone()));
    assert_eq!(
        settled_count(&fixture),
        0,
        "a claim over a pending request reported settling something"
    );

    // Approved: exactly the one request closes.
    let balance = fixture.usdc_token_client.balance(&fixture.insurance_fund);
    fixture.controlled_insurance_fund_client.mark_ready(&0, &balance);
    fixture.contract_client.claim_cover_bad_debt_results(&ObligationKey::new(borrower.clone()));
    assert_eq!(settled_count(&fixture), 1, "a claim that closed a request reported settling none");
}

#[contract]
struct ImmediateZeroCoverFund;

#[contractimpl]
impl ImmediateZeroCoverFund {
    pub fn request_coverage(_e: Env, _token: Address, _amount: i128) -> IssueRequestResult {
        IssueRequestResult::Immediate(0)
    }
}

/// How the insurance fund answers the claim.
#[derive(Clone, Copy, Debug)]
enum Cover {
    Zero,
    Timeout,
    ImmediateZero,
}

/// The lender runs to the fee floor, a week of accrual lifts the fee claim above the cash, then GOLD gaps
/// to 0.001 and a liquidator takes all the collateral for 100 USDC. `users[3]` first borrows
/// `survivor_debt` against BTC, which the crash leaves healthy.
fn crash_after_a_run(f: &TestMarketFixture, survivor_debt: i128) {
    let (borrower, lender, liquidator, beneficiary) =
        (&f.users[0], &f.users[1], &f.users[2], &f.users[4]);
    let usdc = &f.usdc_pool_address;
    f.contract_client
        .set_take_rate_fees_beneficiaries(usdc, &smap![&f.e, (beneficiary.clone(), 10_000)]);
    f.contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &f.gold_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    f.contract_client.deposit(
        &ObligationKey::new(lender.clone()),
        usdc,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    if survivor_debt > 0 {
        let survivor = ObligationKey::new(f.users[3].clone());
        f.contract_client.add_collateral(
            &survivor,
            &f.btc_pool_address,
            &(2 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );
        f.contract_client.borrow(&survivor, usdc, &survivor_debt, &None);
    }
    f.contract_client.borrow(&ObligationKey::new(borrower.clone()), usdc, &i128::MAX, &None);
    f.contract_client.withdraw(&ObligationKey::new(lender.clone()), usdc, &i128::MAX, &None);

    f.e.ledger().with_mut(|li| li.timestamp += 7 * SECONDS_PER_DAY);
    f.contract_client.refresh_pool(usdc);

    // Feed order is [GOLD, BTC, USDC].
    f.oracle_client.set_price_stable(&soroban_sdk::vec![
        &f.e,
        1_00000000000,
        1_00000000000000,
        1_00000000000000
    ]);
    f.e.ledger().with_mut(|li| li.timestamp += 1);
    f.contract_client.liquidate(
        liquidator,
        &ObligationKey::new(borrower.clone()),
        usdc,
        &f.gold_pool_address,
        &100,
        &0,
    );
}

fn write_off(f: &TestMarketFixture, cover: Cover) {
    let borrower = ObligationKey::new(f.users[0].clone());
    if let Cover::ImmediateZero = cover {
        let fund = f.e.register(ImmediateZeroCoverFund, ());
        f.e.as_contract(&f.contract_id, || market::storage::set_insurance_fund(&f.e, &fund));
        f.contract_client.issue_cover_bad_debt(&borrower);
        return;
    }
    f.contract_client.issue_cover_bad_debt(&borrower);
    match cover {
        Cover::Zero => f.controlled_insurance_fund_client.mark_ready(&0, &0),
        _ => f.e.ledger().with_mut(|li| li.timestamp += DEFAULT_BAD_DEBT_LOCK_D),
    }
    f.contract_client.claim_cover_bad_debt_results(&borrower);
}

/// **D-4:** once a write-off leaves the fee claim above everything the pool holds, the next lender's
/// deposit must not fill the unbacked part and flow to the fee beneficiaries.
#[test]
fn test_a_deposit_after_a_wipe_out_is_not_paid_to_the_fee_beneficiaries() {
    let f = TestMarketFixture::new();
    let (lender, newcomer, beneficiary) = (
        ObligationKey::new(f.users[1].clone()),
        ObligationKey::new(f.users[3].clone()),
        &f.users[4],
    );
    let usdc = &f.usdc_pool_address;
    crash_after_a_run(&f, 0);
    write_off(&f, Cover::Zero);
    let cash = get_pool_total_available(&f.contract_client, usdc);
    // The old shares are worth nothing; burning them is what lets a deposit mint at all.
    f.contract_client.withdraw(&lender, usdc, &i128::MAX, &None);

    let deposited = 1_000;
    let beneficiary_before = f.usdc_token_client.balance(beneficiary);
    f.contract_client.deposit(&newcomer, usdc, &deposited, &None);
    f.contract_client.distribute_pool_fees(usdc);
    let newcomer_before = f.usdc_token_client.balance(&f.users[3]);
    f.contract_client.withdraw(&newcomer, usdc, &i128::MAX, &None);

    assert_approx_eq_abs(f.usdc_token_client.balance(&f.users[3]) - newcomer_before, deposited, 1);
    assert_eq!(f.usdc_token_client.balance(beneficiary) - beneficiary_before, cash);
}

/// Every write-off path caps the fee claim at the cash plus the remaining debt, including the fees the
/// doomed debt accrued during the lock.
#[test]
fn test_a_write_off_caps_the_fee_claim_at_what_the_pool_still_holds() {
    for cover in [Cover::Zero, Cover::Timeout, Cover::ImmediateZero] {
        let f = TestMarketFixture::new();
        crash_after_a_run(&f, 0);
        write_off(&f, cover);
        let pool = f.contract_client.get_pool(&f.usdc_pool_address);
        assert!(pool.total_available > 0, "{cover:?}: the scenario left no cash to back the claim");
        assert_eq!(
            pool.take_rate_fees_sum,
            pool.total_available + pool.total_borrowed,
            "{cover:?}: the fee claim outlived the write-off"
        );
    }
}

/// A loss the remaining debt still backs leaves the fee claim whole: fees rank ahead of lenders.
#[test]
fn test_a_partial_loss_leaves_the_fee_claim_senior() {
    let f = TestMarketFixture::new();
    let (survivor, beneficiary) = (ObligationKey::new(f.users[3].clone()), &f.users[4]);
    let usdc = &f.usdc_pool_address;
    crash_after_a_run(&f, 10_000);
    let claim = get_pool_take_rate_fees_sum(&f.contract_client, usdc);
    write_off(&f, Cover::Zero);
    let pool = f.contract_client.get_pool(usdc);
    assert!(pool.take_rate_fees_sum > pool.total_available, "the claim does not rest on the debt");
    assert_eq!(pool.take_rate_fees_sum, claim);

    let beneficiary_before = f.usdc_token_client.balance(beneficiary);
    f.contract_client.repay(&survivor, usdc, &DEFAULT_DEPOSIT_AMOUNT, &None);
    f.contract_client.distribute_pool_fees(usdc);
    assert_eq!(f.usdc_token_client.balance(beneficiary) - beneficiary_before, claim);
}

/// With no take rate there is no claim to cap: the lender keeps the cash and a deposit stays whole.
#[test]
fn test_a_wipe_out_without_a_take_rate_leaves_the_cash_to_the_lender() {
    let mut config = PoolConfig::default();
    config.fee_config.take_rate_bps = 0;
    let f = TestMarketFixture::new_with_pool_config(config);
    let (lender, newcomer) =
        (ObligationKey::new(f.users[1].clone()), ObligationKey::new(f.users[3].clone()));
    let usdc = &f.usdc_pool_address;
    crash_after_a_run(&f, 0);
    write_off(&f, Cover::Zero);
    let pool = f.contract_client.get_pool(usdc);
    assert_eq!(pool.take_rate_fees_sum, 0);

    let lender_before = f.usdc_token_client.balance(&f.users[1]);
    f.contract_client.withdraw(&lender, usdc, &i128::MAX, &None);
    assert_approx_eq_abs(
        f.usdc_token_client.balance(&f.users[1]) - lender_before,
        pool.total_available,
        1,
    );

    let deposited = 1_000;
    f.contract_client.deposit(&newcomer, usdc, &deposited, &None);
    assert_eq!(
        get_obligation_j_tokens_as_tokens(&f.e, &f.contract_client, &f.users[3], usdc),
        Ok(deposited)
    );
}

pub(crate) const ONE: i128 = 10_000_000;

/// The production $5 dust threshold.
pub(crate) fn set_5_usd_dust_threshold(f: &TestMarketFixture) {
    let period = f.contract_client.get_global_state().update_in_queue_period;
    f.contract_client.queue_in_market_update(&10, &500, &DEFAULT_BAD_DEBT_LOCK_D);
    f.e.ledger().with_mut(|li| li.timestamp += period);
    f.contract_client.apply_market_update();
}

/// A $5 dust threshold, a lender, and a borrower owing $1 of USDC against $10 of GOLD.
fn dust_position(f: &TestMarketFixture) -> (ObligationKey, ObligationKey) {
    set_5_usd_dust_threshold(f);
    let (lender, borrower) =
        (ObligationKey::new(f.users[0].clone()), ObligationKey::new(f.users[1].clone()));
    f.contract_client.deposit(&lender, &f.usdc_pool_address, &(100 * ONE), &None);
    f.contract_client.add_collateral(&borrower, &f.gold_pool_address, &(10 * ONE), &None);
    f.contract_client.borrow(&borrower, &f.usdc_pool_address, &ONE, &None);
    (lender, borrower)
}

/// GOLD, BTC and USDC prices in the oracle's feed order, in 14 decimals; BTC stays at $1.
fn set_dust_prices(f: &TestMarketFixture, gold: i128, usdc: i128) {
    f.oracle_client.set_price_stable(&soroban_sdk::vec![&f.e, gold, 1_00000000000000, usdc]);
    f.e.ledger().with_mut(|li| li.timestamp += 1);
}

pub(crate) fn gold_cents(cents: i128) -> i128 {
    cents * 1_000_000_000_000
}

pub(crate) fn lock_of(f: &TestMarketFixture, pool: &Address) -> (u32, u64) {
    let pool = f.contract_client.get_pool(pool);
    (pool.bad_debt_request_count, pool.bad_debt_lock_d)
}

/// A million BTC and a million USDC lent by two lenders.
pub(crate) fn lend_btc_and_usdc(f: &TestMarketFixture) -> (ObligationKey, ObligationKey) {
    let (btc_lender, usdc_lender) =
        (ObligationKey::new(f.users[3].clone()), ObligationKey::new(f.users[1].clone()));
    f.contract_client.deposit(&btc_lender, &f.btc_pool_address, &(1_000_000 * ONE), &None);
    f.contract_client.deposit(&usdc_lender, &f.usdc_pool_address, &(1_000_000 * ONE), &None);
    (btc_lender, usdc_lender)
}

/// $7.25 of GOLD, enough to back a borrow at a $5 threshold, against stroop-sized debts.
fn stroop_debts(f: &TestMarketFixture, key: &ObligationKey, btc: i128, usdc: i128) {
    f.contract_client.add_collateral(key, &f.gold_pool_address, &(725 * ONE / 100), &None);
    if btc > 0 {
        f.contract_client.borrow(key, &f.btc_pool_address, &btc, &None);
    }
    if usdc > 0 {
        f.contract_client.borrow(key, &f.usdc_pool_address, &usdc, &None);
    }
}

/// A debt worth no more than the dust threshold is written off at once instead of going to the fund, whose
/// request would lock the whole borrow pool until its deadline over a few dollars.
#[test]
fn test_a_dust_bad_debt_is_written_off_without_locking_the_pool() {
    let f = TestMarketFixture::new();
    let (lender, borrower) = dust_position(&f);
    set_dust_prices(&f, 1, 1_00000000000000);

    f.contract_client.issue_cover_bad_debt(&borrower);

    let pool = f.contract_client.get_pool(&f.usdc_pool_address);
    assert_eq!(pool.bad_debt_request_count, 0);
    assert_eq!(pool.total_borrowed, 0);
    f.contract_client.withdraw(&lender, &f.usdc_pool_address, &1, &None);
}

/// A solvent obligation is left to liquidation even once its collateral is dust, so its cover is refused and
/// locks nothing. Once the collateral is worth less than the debt, the dust debt is written off without a lock.
#[test]
fn test_a_solvent_dust_debt_is_written_off_only_once_insolvent() {
    let f = TestMarketFixture::new();
    let (lender, borrower) = dust_position(&f);
    // $0.45 a GOLD: $4.50 of collateral, under the $5 threshold but above the $1 debt.
    set_dust_prices(&f, gold_cents(45), 1_00000000000000);
    assert_eq!(
        f.contract_client.try_issue_cover_bad_debt(&borrower),
        Err(Ok(MCError::BadDebtCoverageCriterionIsNotMet))
    );
    assert_eq!(lock_of(&f, &f.usdc_pool_address), (0, 0));

    // $0.09 a GOLD: $0.90 of collateral, short of the debt.
    set_dust_prices(&f, gold_cents(9), 1_00000000000000);
    let gold_before = f.contract_client.get_pool(&f.gold_pool_address);

    f.contract_client.issue_cover_bad_debt(&borrower);

    assert_eq!(lock_of(&f, &f.usdc_pool_address), (0, 0));
    assert_eq!(f.contract_client.get_pool(&f.usdc_pool_address).total_borrowed, 0);
    let gold = f.contract_client.get_pool(&f.gold_pool_address);
    assert_eq!(gold.total_available, gold_before.total_available + 10 * ONE);
    assert_eq!(gold.total_collateral, gold_before.total_collateral - 10 * ONE);
    assert_eq!(
        f.contract_client.try_withdraw(&lender, &f.usdc_pool_address, &1, &None),
        Ok(Ok(()))
    );
}

/// A debt worth exactly the dust threshold is still dust: written off at once, with no claim and no lock.
#[test]
fn test_a_debt_worth_exactly_the_dust_threshold_is_written_off() {
    let f = TestMarketFixture::new();
    set_5_usd_dust_threshold(&f);
    let (lender, borrower) =
        (ObligationKey::new(f.users[0].clone()), ObligationKey::new(f.users[1].clone()));
    f.contract_client.deposit(&lender, &f.usdc_pool_address, &(100 * ONE), &None);
    f.contract_client.add_collateral(&borrower, &f.gold_pool_address, &(20 * ONE), &None);
    f.contract_client.borrow(&borrower, &f.usdc_pool_address, &(5 * ONE), &None);
    // $0.20 a GOLD: $4 of collateral. The price moves in the same ledger, so the $5 debt accrues nothing.
    f.oracle_client.set_price_stable(&soroban_sdk::vec![
        &f.e,
        gold_cents(20),
        1_00000000000000,
        1_00000000000000
    ]);
    assert_eq!(
        get_obligation_d_tokens_as_tokens(
            &f.e,
            &f.contract_client,
            &borrower.user,
            &f.usdc_pool_address
        ),
        Ok(5 * ONE)
    );

    f.contract_client.issue_cover_bad_debt(&borrower);

    assert_eq!(lock_of(&f, &f.usdc_pool_address), (0, 0));
    assert_eq!(f.contract_client.get_pool(&f.usdc_pool_address).total_borrowed, 0);
}

/// A debt above the dust threshold still goes to the fund and locks its pool until the fund answers.
#[test]
fn test_a_debt_above_the_dust_threshold_still_locks_its_pool() {
    let f = TestMarketFixture::new();
    set_5_usd_dust_threshold(&f);
    let (lender, borrower) =
        (ObligationKey::new(f.users[0].clone()), ObligationKey::new(f.users[1].clone()));
    f.contract_client.deposit(&lender, &f.usdc_pool_address, &(100 * ONE), &None);
    f.contract_client.add_collateral(&borrower, &f.gold_pool_address, &(20 * ONE), &None);
    f.contract_client.borrow(&borrower, &f.usdc_pool_address, &(6 * ONE), &None);
    // $0.20 a GOLD: $4 of collateral under the threshold, $6 of debt above it.
    set_dust_prices(&f, gold_cents(20), 1_00000000000000);

    f.contract_client.issue_cover_bad_debt(&borrower);

    assert_eq!(lock_of(&f, &f.usdc_pool_address).0, 1);
    assert_eq!(
        f.contract_client.try_withdraw(&lender, &f.usdc_pool_address, &1, &None),
        Err(Ok(MCError::PoolBadDebtLocked))
    );

    f.controlled_insurance_fund_client.mark_ready(&0, &0);
    f.contract_client.claim_cover_bad_debt_results(&borrower);
    assert_eq!(lock_of(&f, &f.usdc_pool_address), (0, 0));
    assert_eq!(
        f.contract_client.try_withdraw(&lender, &f.usdc_pool_address, &1, &None),
        Ok(Ok(()))
    );
}

/// A debt whose price cannot be read goes to the fund and locks the pool, instead of failing the cover and
/// letting lenders leave ahead of the loss.
#[test]
fn test_an_unpriceable_debt_still_locks_its_pool() {
    let f = TestMarketFixture::new();
    let (lender, borrower) = dust_position(&f);
    set_dust_prices(&f, 1, 0);

    f.contract_client.issue_cover_bad_debt(&borrower);

    assert_eq!(f.contract_client.get_pool(&f.usdc_pool_address).bad_debt_request_count, 1);
    assert_eq!(
        f.contract_client.try_withdraw(&lender, &f.usdc_pool_address, &1, &None),
        Err(Ok(MCError::PoolBadDebtLocked))
    );
}

/// Written-off dust leaves the books once: settling the obligation's other debt later does not touch it.
#[test]
fn test_a_written_off_dust_debt_is_socialized_once() {
    let f = TestMarketFixture::new();
    set_5_usd_dust_threshold(&f);
    let c = &f.contract_client;
    lend_btc_and_usdc(&f);
    let borrower = ObligationKey::new(f.users[0].clone());
    c.add_collateral(&borrower, &f.gold_pool_address, &(30 * ONE), &None);
    c.borrow(&borrower, &f.usdc_pool_address, &ONE, &None);
    c.borrow(&borrower, &f.btc_pool_address, &(6 * ONE), &None);
    // $0.10 a GOLD: $3 of collateral; the $1 USDC debt is dust, the $6 BTC debt is not.
    set_dust_prices(&f, gold_cents(10), 1_00000000000000);
    c.refresh_pool(&f.usdc_pool_address);
    let usdc_before = c.get_pool(&f.usdc_pool_address);
    let dust =
        get_obligation_d_tokens_as_tokens(&f.e, c, &f.users[0], &f.usdc_pool_address).unwrap();

    c.issue_cover_bad_debt(&borrower);

    let usdc_after = c.get_pool(&f.usdc_pool_address);
    assert_eq!(usdc_after.total_supply().unwrap(), usdc_before.total_supply().unwrap() - dust);
    assert_eq!(usdc_after.total_available, usdc_before.total_available);
    assert_eq!(usdc_after.total_d_tokens, 0);
    assert_eq!(lock_of(&f, &f.usdc_pool_address), (0, 0));
    assert_eq!(lock_of(&f, &f.btc_pool_address).0, 1);
    f.assert_invariants();

    f.controlled_insurance_fund_client.mark_ready(&0, &0);
    c.claim_cover_bad_debt_results(&borrower);

    assert_eq!(c.get_pool(&f.usdc_pool_address), usdc_after);
    assert_eq!(lock_of(&f, &f.btc_pool_address), (0, 0));
    assert_eq!(c.try_issue_cover_bad_debt(&borrower), Err(Ok(MCError::ObligationDoesNotExist)));
    f.assert_invariants();
}

/// One-stroop debts beside $4.93 of GOLD are solvent, so a third party's cover is refused and locks neither
/// borrow pool; its liquidation clears both debts instead.
#[test]
fn test_a_third_party_clears_stroop_debts_by_liquidation_and_locks_no_pool() {
    let f = TestMarketFixture::new();
    set_5_usd_dust_threshold(&f);
    let (btc_lender, usdc_lender) = lend_btc_and_usdc(&f);
    let c = &f.contract_client;
    let other_borrower = ObligationKey::new(f.users[2].clone());
    c.add_collateral(&other_borrower, &f.gold_pool_address, &(1_000 * ONE), &None);
    let borrower = ObligationKey::new(f.users[0].clone());
    stroop_debts(&f, &borrower, 1, 1);
    // $0.68 a GOLD: the first whole-cent price at which $7.25 of GOLD falls to the $5 threshold.
    set_dust_prices(&f, gold_cents(68), 1_00000000000000);

    assert_eq!(
        c.try_issue_cover_bad_debt(&borrower),
        Err(Ok(MCError::BadDebtCoverageCriterionIsNotMet))
    );
    assert_eq!(lock_of(&f, &f.btc_pool_address), (0, 0));
    assert_eq!(lock_of(&f, &f.usdc_pool_address), (0, 0));

    for pool in [&f.btc_pool_address, &f.usdc_pool_address] {
        c.liquidate(&f.users[4], &borrower, pool, &f.gold_pool_address, &ONE, &0);
    }
    assert_eq!(lock_of(&f, &f.btc_pool_address), (0, 0));
    assert_eq!(lock_of(&f, &f.usdc_pool_address), (0, 0));
    assert_eq!(c.get_pool(&f.btc_pool_address).total_borrowed, 0);
    assert_eq!(c.get_pool(&f.usdc_pool_address).total_borrowed, 0);
    assert_eq!(c.try_withdraw(&btc_lender, &f.btc_pool_address, &ONE, &None), Ok(Ok(())));
    assert_eq!(c.try_withdraw(&usdc_lender, &f.usdc_pool_address, &ONE, &None), Ok(Ok(())));
    assert_eq!(c.try_deposit(&btc_lender, &f.btc_pool_address, &ONE, &None), Ok(Ok(())));
    assert_eq!(c.try_borrow(&other_borrower, &f.btc_pool_address, &ONE, &None), Ok(Ok(())));
    assert_ne!(
        c.try_flash_loan(&Address::generate(&f.e), &f.users[2], &f.btc_pool_address, &ONE),
        Err(Ok(MCError::PoolBadDebtLocked))
    );
}

/// A borrower who liquidates their own dust collateral cannot leave debt with nothing behind it: a repay that
/// leaves dust is refused, full repays clear both debts, and a cover later, after prices recover, locks nothing.
#[test]
fn test_a_self_liquidation_of_dust_collateral_leaves_no_debt_behind() {
    let f = TestMarketFixture::new();
    set_5_usd_dust_threshold(&f);
    let (btc_lender, usdc_lender) = lend_btc_and_usdc(&f);
    let c = &f.contract_client;
    let borrower = ObligationKey::new(f.users[0].clone());
    stroop_debts(&f, &borrower, 1, 10);
    set_dust_prices(&f, gold_cents(68), 1_00000000000000);
    assert_eq!(
        c.try_liquidate(&f.users[4], &borrower, &f.usdc_pool_address, &f.gold_pool_address, &2, &0),
        Err(Ok(MCError::LiquidationLeavesDust))
    );
    for pool in [&f.usdc_pool_address, &f.btc_pool_address] {
        c.liquidate(&f.users[4], &borrower, pool, &f.gold_pool_address, &ONE, &0);
    }
    let obligation = c.get_user_obligation(&borrower);
    assert_eq!((obligation.deposits.len(), obligation.borrows.len()), (1, 0));

    set_dust_prices(&f, gold_cents(100), 1_00000000000000);
    f.pass_time(30 * 24 * SECONDS_PER_HOUR);
    assert_eq!(
        c.try_issue_cover_bad_debt(&borrower),
        Err(Ok(MCError::BadDebtCoverageCriterionIsNotMet))
    );

    assert_eq!(lock_of(&f, &f.btc_pool_address), (0, 0));
    assert_eq!(lock_of(&f, &f.usdc_pool_address), (0, 0));
    assert_eq!(c.try_withdraw(&btc_lender, &f.btc_pool_address, &ONE, &None), Ok(Ok(())));
    assert_eq!(c.try_withdraw(&usdc_lender, &f.usdc_pool_address, &ONE, &None), Ok(Ok(())));
}

/// Covers of several insolvent dust obligations hours apart never hold the pool shut, and need no user's auth.
#[test]
fn test_staggered_dust_covers_do_not_chain_a_lock() {
    let f = TestMarketFixture::new();
    set_5_usd_dust_threshold(&f);
    let (btc_lender, _) = lend_btc_and_usdc(&f);
    let c = &f.contract_client;
    let first = ObligationKey::new(f.users[0].clone());
    let second =
        ObligationKey::new_with_seed(f.users[0].clone(), BytesN::from_array(&f.e, &[7; 32]));
    stroop_debts(&f, &first, 1, 0);
    stroop_debts(&f, &second, 1, 0);
    // A GOLD price of one unit leaves the collateral worth less than a stroop of debt.
    set_dust_prices(&f, 1, 1_00000000000000);

    c.issue_cover_bad_debt(&first);
    assert!(f.e.auths().iter().all(|(address, _)| !f.users.contains(address)));
    assert_eq!(lock_of(&f, &f.btc_pool_address), (0, 0));
    f.pass_time(11 * SECONDS_PER_HOUR);
    c.issue_cover_bad_debt(&second);

    assert_eq!(lock_of(&f, &f.btc_pool_address), (0, 0));
    assert_eq!(c.try_withdraw(&btc_lender, &f.btc_pool_address, &ONE, &None), Ok(Ok(())));
}

/// A wipe-out whose remaining debt is dust takes the write-off without a claim; the fee claim is still
/// capped at what the pool holds, so a later deposit comes back whole.
#[test]
fn test_a_dust_write_off_caps_the_fee_claim_at_what_the_pool_still_holds() {
    let f = TestMarketFixture::new();
    set_5_usd_dust_threshold(&f);
    let (borrower, lender, newcomer) = (
        ObligationKey::new(f.users[0].clone()),
        ObligationKey::new(f.users[1].clone()),
        ObligationKey::new(f.users[3].clone()),
    );
    let usdc = &f.usdc_pool_address;
    let c = &f.contract_client;
    c.set_take_rate_fees_beneficiaries(usdc, &smap![&f.e, (f.users[4].clone(), 10_000)]);
    // $10 of GOLD less the $5 floor still leaves more borrowing room than the pool holds.
    c.add_collateral(&borrower, &f.gold_pool_address, &(10 * ONE), &None);
    c.deposit(&lender, usdc, &DEFAULT_DEPOSIT_AMOUNT, &None);
    c.borrow(&borrower, usdc, &i128::MAX, &None);
    c.withdraw(&lender, usdc, &i128::MAX, &None);
    f.e.ledger().with_mut(|li| li.timestamp += 7 * SECONDS_PER_DAY);
    c.refresh_pool(usdc);
    // $0.000001 a GOLD: the $0.00001 of collateral is worth less than the 100-stroop repay, which buys it whole.
    set_dust_prices(&f, 100_000_000, 1_00000000000000);
    c.liquidate(&f.users[2], &borrower, usdc, &f.gold_pool_address, &100, &0);
    assert_eq!(
        get_obligation_collateral(c, &borrower.user, &f.gold_pool_address),
        Err(MCError::DepositPositionDoesNotExist)
    );
    let before = c.get_pool(usdc);

    c.issue_cover_bad_debt(&borrower);

    let after = c.get_pool(usdc);
    assert_eq!(after.bad_debt_request_count, 0, "the write-off did not take the dust branch");
    assert!(before.take_rate_fees_sum > after.total_available, "the cap was never needed");
    assert_eq!(after.take_rate_fees_sum, after.total_available + after.total_borrowed);

    c.withdraw(&lender, usdc, &i128::MAX, &None);
    let deposited = 1_000;
    c.deposit(&newcomer, usdc, &deposited, &None);
    c.distribute_pool_fees(usdc);
    let newcomer_before = f.usdc_token_client.balance(&f.users[3]);
    c.withdraw(&newcomer, usdc, &i128::MAX, &None);
    assert_approx_eq_abs(f.usdc_token_client.balance(&f.users[3]) - newcomer_before, deposited, 1);
    f.assert_invariants();
}

/// A solvent obligation is left to liquidation: cover is refused while the collateral is worth at least the debt
/// and accepted once it is worth less, one price unit either side of equality.
#[test]
fn test_cover_is_refused_while_the_collateral_covers_the_debt() {
    for (gold_price_offset, covered) in [(0, false), (1, false), (-1, true)] {
        let f = TestMarketFixture::new();
        set_5_usd_dust_threshold(&f);
        lend_btc_and_usdc(&f);
        let borrower = ObligationKey::new(f.users[0].clone());
        f.contract_client.add_collateral(&borrower, &f.gold_pool_address, &(1_000 * ONE), &None);
        f.contract_client.borrow(&borrower, &f.usdc_pool_address, &(3 * ONE), &None);
        f.pass_time(60);
        f.contract_client.refresh_pool(&f.usdc_pool_address);
        let debt = get_obligation_d_tokens_as_tokens(
            &f.e,
            &f.contract_client,
            &borrower.user,
            &f.usdc_pool_address,
        )
        .unwrap();
        // 1,000 GOLD at `debt * 10^4` a GOLD are worth exactly the debt, about $3.
        let gold_price = debt * 10_000 + gold_price_offset;
        f.oracle_client.set_price_stable(&soroban_sdk::vec![
            &f.e,
            gold_price,
            1_00000000000000,
            1_00000000000000
        ]);
        assert_eq!(
            compute_user_obligation_collateral_value(&f.e, &f.contract_client, &borrower.user)
                - compute_user_obligation_debt_value(&f.e, &f.contract_client, &borrower.user),
            1_000 * gold_price_offset
        );

        let expected =
            if covered { Ok(Ok(())) } else { Err(Ok(MCError::BadDebtCoverageCriterionIsNotMet)) };
        assert_eq!(f.contract_client.try_issue_cover_bad_debt(&borrower), expected);
    }
}

/// A deposit in a pool with a zero close LTV cannot be seized, so it does not count toward that solvency: once
/// the GOLD is sold, the debt left beside $3 of such BTC is written off and the BTC goes to its pool.
#[test]
fn test_a_deposit_that_cannot_be_seized_does_not_hold_off_cover() {
    let f = TestMarketFixture::new();
    set_5_usd_dust_threshold(&f);
    lend_btc_and_usdc(&f);
    let c = &f.contract_client;
    let supply_only = PoolConfig {
        health_config: PoolHealthConfig { open_ltv_bps: 0, close_ltv_bps: 0, ..Default::default() },
        ..Default::default()
    };
    c.queue_in_pool_set(&f.btc_pool_address, &supply_only);
    f.pass_time(c.get_global_state().update_in_queue_period);
    c.apply_pool_set(&f.btc_pool_address);
    let borrower = ObligationKey::new(f.users[0].clone());
    c.add_collateral(&borrower, &f.gold_pool_address, &(10 * ONE), &None);
    c.deposit(&borrower, &f.btc_pool_address, &(3 * ONE), &None);
    c.borrow(&borrower, &f.usdc_pool_address, &(19 * ONE / 10), &None);
    // $0.10 a GOLD: $1 of GOLD and $3 of BTC against $1.90.
    set_dust_prices(&f, gold_cents(10), 1_00000000000000);

    // The smallest repay whose `repay * 1.1` buys all the GOLD.
    c.liquidate(&f.users[2], &borrower, &f.usdc_pool_address, &f.gold_pool_address, &9_090_910, &0);
    assert_eq!(
        c.try_liquidate(
            &f.users[2],
            &borrower,
            &f.usdc_pool_address,
            &f.btc_pool_address,
            &ONE,
            &0
        ),
        Err(Ok(MCError::AssetCannotBeUsedAsCollateral))
    );
    let btc_shares = get_pool_total_j_tokens(c, &f.btc_pool_address);

    c.issue_cover_bad_debt(&borrower);

    assert_eq!(lock_of(&f, &f.usdc_pool_address), (0, 0));
    assert_eq!(c.get_pool(&f.usdc_pool_address).total_borrowed, 0);
    assert_eq!(get_pool_total_j_tokens(c, &f.btc_pool_address), btc_shares - 3 * ONE);
    assert_eq!(c.try_get_user_obligation(&borrower), Err(Ok(MCError::ObligationDoesNotExist)));
}

/// A borrower who liquidates their own dust collateral from a second address and then covers the rest gains
/// nothing, and the lenders lose nothing: stroop repays are refused, full ones clear the debts, and with no debt
/// left there is nothing to cover.
#[test]
fn test_a_self_liquidation_and_cover_gain_nothing() {
    let f = TestMarketFixture::new();
    set_5_usd_dust_threshold(&f);
    lend_btc_and_usdc(&f);
    let c = &f.contract_client;
    let (owner, second) = (&f.users[0], &f.users[4]);
    let borrower = ObligationKey::new(owner.clone());
    c.add_collateral(&borrower, &f.gold_pool_address, &(10 * ONE), &None);
    c.borrow(&borrower, &f.usdc_pool_address, &(99 * ONE / 100), &None);
    c.borrow(&borrower, &f.btc_pool_address, &(99 * ONE / 100), &None);
    // $0.49 a GOLD: $4.90 of collateral against $1.98 of debt.
    let gold_price = gold_cents(49);
    set_dust_prices(&f, gold_price, 1_00000000000000);
    let debt_pools = [&f.usdc_pool_address, &f.btc_pool_address];

    let tokens = [&f.gold_token_client, &f.btc_token_client, &f.usdc_token_client];
    let prices = [gold_price, 1_00000000000000, 1_00000000000000];
    let wallets = || tokens.map(|token| token.balance(owner) + token.balance(second));
    let equity = || {
        compute_user_obligation_collateral_value(&f.e, c, owner)
            - compute_user_obligation_debt_value(&f.e, c, owner)
    };
    let supplies = || debt_pools.map(|pool| c.get_pool(pool).total_supply().unwrap());
    for pool in debt_pools {
        c.refresh_pool(pool);
    }
    let (wallets_before, equity_before, supplies_before) = (wallets(), equity(), supplies());

    for pool in debt_pools {
        assert_eq!(
            c.try_liquidate(second, &borrower, pool, &f.gold_pool_address, &10, &0),
            Err(Ok(MCError::LiquidationLeavesDust))
        );
    }
    for pool in debt_pools {
        c.liquidate(second, &borrower, pool, &f.gold_pool_address, &ONE, &0);
    }
    assert_eq!(
        c.try_issue_cover_bad_debt(&borrower),
        Err(Ok(MCError::BadDebtCoverageCriterionIsNotMet))
    );

    assert_eq!(supplies(), supplies_before);
    let wallets_after = wallets();
    let wallet_gain: i128 =
        (0..3).map(|i| (wallets_after[i] - wallets_before[i]) * prices[i] / ONE).sum();
    assert_eq!(wallet_gain + equity() - equity_before, 0);
}

/// An insurance fund that answers every request the same scripted way: immediately or as recorded request
/// `SCRIPTED_REQUEST_ID` that is ready at once. Its transfer falls `shortfall` units short of what it reports.
#[contract]
pub struct ScriptedInsuranceFund;

const SCRIPTED_REQUEST_ID: u64 = 7;

#[contractimpl]
impl ScriptedInsuranceFund {
    pub fn __constructor(e: Env, market: Address, immediate: bool, shortfall: i128) {
        e.storage().instance().set(&0_u32, &market);
        e.storage().instance().set(&1_u32, &immediate);
        e.storage().instance().set(&2_u32, &shortfall);
    }

    pub fn add_reserves(_e: Env, _token: Address, _amount: i128) {}

    pub fn request_coverage(e: Env, token: Address, amount: i128) -> IssueRequestResult {
        if e.storage().instance().get(&1_u32).unwrap() {
            Self::pay(&e, &token, amount);
            IssueRequestResult::Immediate(amount)
        } else {
            e.storage().instance().set(&3_u32, &(token, amount));
            IssueRequestResult::Recorded(SCRIPTED_REQUEST_ID)
        }
    }

    pub fn get_status(e: Env, _request_id: u64) -> Option<CoverageStatus> {
        let (_, amount): (Address, i128) = e.storage().instance().get(&3_u32)?;
        Some(CoverageStatus::Ready(amount))
    }

    pub fn claim_coverage(e: Env, _request_id: u64) -> i128 {
        let (token, amount): (Address, i128) = e.storage().instance().get(&3_u32).unwrap();
        e.storage().instance().remove(&3_u32);
        Self::pay(&e, &token, amount);
        amount
    }

    pub fn cancel(_e: Env, _request_id: u64) {}
}

impl ScriptedInsuranceFund {
    fn pay(e: &Env, token: &Address, reported: i128) {
        let market: Address = e.storage().instance().get(&0_u32).unwrap();
        let shortfall: i128 = e.storage().instance().get(&2_u32).unwrap();
        TokenClient::new(e, token).transfer(
            &e.current_contract_address(),
            &market,
            &(reported - shortfall),
        );
    }
}

const DUST_GOLD_COLLATERAL: i128 = 20 * ONE;
const COVERED_USDC_DEBT: i128 = 6 * ONE;

/// A borrower owing $6 of USDC whose only collateral is `DUST_GOLD_COLLATERAL` plain GOLD, priced down to $4:
/// under the $5 threshold, so its debt qualifies for cover, but the debt is above it and goes to the fund.
fn dust_collateral_borrower(f: &TestMarketFixture) -> ObligationKey {
    set_5_usd_dust_threshold(f);
    let (lender, borrower) =
        (ObligationKey::new(f.users[0].clone()), ObligationKey::new(f.users[1].clone()));
    f.contract_client.deposit(&lender, &f.usdc_pool_address, &(100 * ONE), &None);
    f.contract_client.add_collateral(&borrower, &f.gold_pool_address, &DUST_GOLD_COLLATERAL, &None);
    f.contract_client.borrow(&borrower, &f.usdc_pool_address, &COVERED_USDC_DEBT, &None);
    set_dust_prices(f, gold_cents(20), 1_00000000000000);
    borrower
}

/// Issuing cover releases a covered obligation's plain collateral to the pool's liquidity: what leaves
/// `total_collateral` arrives in `total_available`, and the market's token balance does not move.
#[test]
fn test_issuing_cover_moves_the_plain_collateral_into_the_pool_liquidity() {
    let f = TestMarketFixture::new();
    let borrower = dust_collateral_borrower(&f);
    let before = f.contract_client.get_pool(&f.gold_pool_address);
    let balance_before = f.gold_token_client.balance(&f.contract_id);

    f.contract_client.issue_cover_bad_debt(&borrower);

    let after = f.contract_client.get_pool(&f.gold_pool_address);
    assert_eq!(before.total_collateral - after.total_collateral, DUST_GOLD_COLLATERAL);
    assert_eq!(after.total_available - before.total_available, DUST_GOLD_COLLATERAL);
    assert_eq!(f.gold_token_client.balance(&f.contract_id), balance_before);
    assert_eq!(
        get_obligation_collateral(&f.contract_client, &borrower.user, &f.gold_pool_address),
        Err(MCError::DepositPositionDoesNotExist)
    );
}

/// An immediate cover is accepted only when the fund's transfer matches the amount it reports; a matching one
/// writes the debt off against the received tokens.
#[test]
fn test_an_immediate_cover_must_deliver_what_the_fund_reports() {
    for shortfall in [0_i128, 1, 1_000] {
        let f = TestMarketFixture::new();
        f.e.register_at(
            &f.insurance_fund,
            ScriptedInsuranceFund,
            (f.contract_id.clone(), true, shortfall),
        );
        f.usdc_sac.mint(&f.insurance_fund, &(10 * COVERED_USDC_DEBT));
        let borrower = dust_collateral_borrower(&f);
        f.contract_client.refresh_pool(&f.usdc_pool_address);
        let debt = get_obligation_d_tokens_as_tokens(
            &f.e,
            &f.contract_client,
            &borrower.user,
            &f.usdc_pool_address,
        )
        .unwrap();
        let available_before = get_pool_total_available(&f.contract_client, &f.usdc_pool_address);
        let fund_before = f.usdc_token_client.balance(&f.insurance_fund);

        let outcome = f.contract_client.try_issue_cover_bad_debt(&borrower);

        if shortfall == 0 {
            assert_eq!(outcome, Ok(Ok(())));
            assert_eq!(fund_before - f.usdc_token_client.balance(&f.insurance_fund), debt);
            assert_eq!(
                get_pool_total_available(&f.contract_client, &f.usdc_pool_address)
                    - available_before,
                debt
            );
            assert_eq!(f.contract_client.get_pool(&f.usdc_pool_address).total_d_tokens, 0);
        } else {
            assert_eq!(outcome, Err(Ok(MCError::DependencyContractError)));
        }
    }
}

/// Claiming a ready request whose transfer falls short of the approved amount settles nothing: the request,
/// the pool's lock and the debt stay in place. A matching transfer settles it.
#[test]
fn test_a_cover_claim_that_delivers_less_than_approved_is_not_settled() {
    for shortfall in [0_i128, 1, 1_000] {
        let f = TestMarketFixture::new();
        f.e.register_at(
            &f.insurance_fund,
            ScriptedInsuranceFund,
            (f.contract_id.clone(), false, shortfall),
        );
        f.usdc_sac.mint(&f.insurance_fund, &(10 * COVERED_USDC_DEBT));
        let borrower = dust_collateral_borrower(&f);
        f.contract_client.issue_cover_bad_debt(&borrower);
        let d_tokens = get_pool_total_d_tokens(&f.contract_client, &f.usdc_pool_address);

        f.contract_client.claim_cover_bad_debt_results(&borrower);

        let pool = f.contract_client.get_pool(&f.usdc_pool_address);
        let obligation = f.contract_client.try_get_user_obligation(&borrower);
        if shortfall == 0 {
            assert_eq!(pool.bad_debt_request_count, 0);
            assert_eq!(pool.total_d_tokens, 0);
            assert_eq!(obligation, Err(Ok(MCError::ObligationDoesNotExist)));
        } else {
            assert_eq!(pool.bad_debt_request_count, 1);
            assert_eq!(pool.total_d_tokens, d_tokens);
            let obligation = obligation.unwrap().unwrap();
            assert!(obligation.borrows.contains_key(f.usdc_pool_address.clone()));
            assert_eq!(obligation.insurance_fund_requests_ids.len(), 1);
        }
    }
}

/// The cover criterion values collateral at a validated price: with a stale feed, issuing cover fails
/// with `OracleStalePrice` instead of valuing the collateral at a fallback.
#[test]
fn test_issuing_cover_refuses_a_stale_collateral_price() {
    let f = TestMarketFixture::new();
    let borrower = dust_collateral_borrower(&f);
    let prices = soroban_sdk::vec![&f.e, gold_cents(20), 1_00000000000000, 1_00000000000000];
    let stale_at = f.e.ledger().timestamp() - MAX_ORACLE_PRICE_AGE_SECONDS - 1;
    f.oracle_client.set_price(&prices, &stale_at);

    assert_eq!(
        f.contract_client.try_issue_cover_bad_debt(&borrower),
        Err(Ok(MCError::OracleStalePrice))
    );

    f.oracle_client.set_price_stable(&prices);
    assert_eq!(f.contract_client.try_issue_cover_bad_debt(&borrower), Ok(Ok(())));
}
