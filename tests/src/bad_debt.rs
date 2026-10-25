#![cfg(test)]

use insurance_fund_interface::{CoverageStatus, IssueRequestResult};
use market::{
    constants::{
        DEFAULT_BAD_DEBT_LOCK_D, DEFAULT_MAX_POSITIONS, MAX_ORACLE_PRICE_AGE_SECONDS,
        SECONDS_IN_YEAR, SECONDS_PER_HOUR,
    },
    error::MCError,
    obligation::ObligationKey,
};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, Env, contract, contractimpl, map as smap, testutils::Ledger, token::TokenClient,
};

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture,
    compute_user_obligation_collateral_value, compute_user_obligation_debt_value,
    get_obligation_collateral, get_obligation_d_tokens_as_tokens, get_pool_total_available,
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

const DUST_GOLD_COLLATERAL: i128 = 40_000_000;
const DUST_USDC_DEBT: i128 = 10_000_000;

/// A borrower whose only collateral is `DUST_GOLD_COLLATERAL` plain GOLD ($4 at the fixture's price of 1),
/// below a `min_collateral_value_cents` of $5 set after it borrowed, so its debt qualifies for cover.
fn dust_collateral_borrower(f: &TestMarketFixture) -> ObligationKey {
    let (lender, borrower) = (&f.users[0], &f.users[1]);
    let borrower_key = ObligationKey::new(borrower.clone());
    f.contract_client.deposit(
        &ObligationKey::new(lender.clone()),
        &f.usdc_pool_address,
        &(100 * DUST_USDC_DEBT),
        &None,
    );
    f.contract_client.add_collateral(
        &borrower_key,
        &f.gold_pool_address,
        &DUST_GOLD_COLLATERAL,
        &None,
    );
    f.contract_client.borrow(&borrower_key, &f.usdc_pool_address, &DUST_USDC_DEBT, &None);

    let period = f.contract_client.get_global_state().update_in_queue_period;
    f.contract_client.queue_in_market_update(
        &DEFAULT_MAX_POSITIONS,
        &500,
        &DEFAULT_BAD_DEBT_LOCK_D,
    );
    f.e.ledger().with_mut(|li| li.timestamp += period);
    f.contract_client.apply_market_update();

    borrower_key
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
        f.usdc_sac.mint(&f.insurance_fund, &(10 * DUST_USDC_DEBT));
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
        f.usdc_sac.mint(&f.insurance_fund, &(10 * DUST_USDC_DEBT));
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
    let prices = soroban_sdk::vec![&f.e, 1_00000000000000, 1_00000000000000, 1_00000000000000];
    let stale_at = f.e.ledger().timestamp() - MAX_ORACLE_PRICE_AGE_SECONDS - 1;
    f.oracle_client.set_price(&prices, &stale_at);

    assert_eq!(
        f.contract_client.try_issue_cover_bad_debt(&borrower),
        Err(Ok(MCError::OracleStalePrice))
    );

    f.oracle_client.set_price_stable(&prices);
    assert_eq!(f.contract_client.try_issue_cover_bad_debt(&borrower), Ok(Ok(())));
}
