//! The rate controller integrates over at most `MAX_IR_INTEGRATION_SECONDS`, so one accrual moves the
//! modifier by less than its band whatever the interval: unclamped, one idle day at zero utilization asks
//! for 56.7x the band, and lowering the reactivity constant cannot bound it (a year idle at the smallest
//! non-zero one asks ~207x).
#![cfg(test)]

use market::{
    constants::{
        MAX_IR_INTEGRATION_SECONDS, MAX_IR_MODIFIER, MAX_REACTIVITY_CONSTANT, MIN_IR_MODIFIER,
    },
    pool::{Pool, PoolConfig},
};
use soroban_sdk::testutils::Ledger;

use crate::TestMarketFixture;

/// A **new** pool with a live reactivity constant and no debt: maximally below target. It has to be new,
/// not reconfigured: ripening a queued set costs a day of ledger time, and the first accrual after that
/// would see the whole day and pin the modifier before the measurement starts.
fn reactive_and_idle<'a>() -> (TestMarketFixture<'a>, soroban_sdk::Address) {
    reactive_and_idle_at(MAX_REACTIVITY_CONSTANT)
}

fn reactive_and_idle_at<'a>(reactivity: u32) -> (TestMarketFixture<'a>, soroban_sdk::Address) {
    let f = TestMarketFixture::new();
    let token = crate::register_random_sac(&f.e);
    let reactive = PoolConfig { ir_reactivity_constant: reactivity, ..Default::default() };
    f.contract_client.queue_in_pool_set(&token, &reactive);
    let period = f.contract_client.get_global_state().update_in_queue_period;
    f.e.ledger().with_mut(|li| li.timestamp += period);
    f.contract_client.apply_pool_set(&token);
    (f, token)
}

/// From the ceiling, one idle day lowers the modifier but cannot take it to the floor: the window, not the
/// interval, bounds the step.
#[test]
fn test_one_idle_day_does_not_take_the_rate_modifier_across_its_band() {
    let (f, pool) = reactive_and_idle();
    f.e.as_contract(&f.contract_id, || {
        let mut stored = Pool::try_get(&f.e, &pool).unwrap();
        stored.interest_rate_modifier_bps = MAX_IR_MODIFIER;
        market::storage::set_pool(&f.e, &pool, &stored);
    });

    f.e.ledger().with_mut(|li| li.timestamp += 24 * 60 * 60);
    f.contract_client.refresh_pool(&pool);

    let after = f.contract_client.get_pool(&pool).interest_rate_modifier_bps;
    assert!(
        after < MAX_IR_MODIFIER,
        "the controller did not move, so the floor check proves nothing"
    );
    assert!(
        after > MIN_IR_MODIFIER,
        "one idle day took the modifier from the ceiling to the floor {MIN_IR_MODIFIER}"
    );
}

/// And the window does not change a live pool: accrual runs inside every money operation, so a
/// pool touched more often than the window never reaches the clamp. At a reactivity of 10 each 100 s
/// idle step is exactly `100 · 6 500 · 10 / 10 000 = 650` bps, far from the floor.
#[test]
fn test_a_frequently_accrued_pool_is_unaffected_by_the_window() {
    let (f, pool) = reactive_and_idle_at(10);
    let before = f.contract_client.get_pool(&pool).interest_rate_modifier_bps;

    for _ in 0..6 {
        f.e.ledger().with_mut(|li| li.timestamp += 100);
        f.contract_client.refresh_pool(&pool);
    }

    let after = f.contract_client.get_pool(&pool).interest_rate_modifier_bps;
    assert_eq!(after, before - 6 * 650);
}

/// **The window touches the controller, not the accrual.** Interest must still be charged over the
/// real interval. Run on the default pool, whose reactivity is 0, so only the accrual is left.
#[test]
fn test_the_window_does_not_shorten_interest_accrual() {
    use market::obligation::ObligationKey;

    use crate::{DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, get_pool_total_borrowed};

    fn borrowed_growth_over(seconds: u64) -> i128 {
        let f = TestMarketFixture::new();
        let (lender, borrower) =
            (ObligationKey::new(f.users[0].clone()), ObligationKey::new(f.users[1].clone()));
        f.contract_client.deposit(
            &lender,
            &f.usdc_pool_address,
            &(100 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );
        f.contract_client.add_collateral(
            &borrower,
            &f.gold_pool_address,
            &(400 * DEFAULT_COLLATERAL_AMOUNT),
            &None,
        );
        f.contract_client.borrow(
            &borrower,
            &f.usdc_pool_address,
            &(50 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );

        let before = get_pool_total_borrowed(&f.contract_client, &f.usdc_pool_address);
        f.e.ledger().with_mut(|li| li.timestamp += seconds);
        f.contract_client.refresh_pool(&f.usdc_pool_address);
        get_pool_total_borrowed(&f.contract_client, &f.usdc_pool_address) - before
    }

    let one_window = borrowed_growth_over(MAX_IR_INTEGRATION_SECONDS);
    let one_day = borrowed_growth_over(24 * 60 * 60);

    assert!(one_window > 0, "no interest accrued at all, so the comparison proves nothing");
    // If the window had reached the accrual, a day would have charged a window's worth. 96 windows fit in
    // a day; each rounds up on its own, so the single long accrual is slightly under 96x rather than over.
    assert!(
        one_day > one_window * 90,
        "a day accrued {one_day}, one window {one_window}: the window reached the accrual"
    );
}
