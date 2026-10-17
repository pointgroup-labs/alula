#![cfg(test)]

use market::{
    constants::*,
    error::MCError,
    interest_rate_model::kinked::KinkedIRConfig,
    obligation::ObligationKey,
    pool::{Pool, PoolConfig, PoolFeeConfig, PoolHealthConfig},
};
use soroban_sdk::testutils::Ledger;

use crate::{DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture};

#[test]
#[allow(clippy::mistyped_literal_suffixes)]
#[allow(clippy::zero_prefixed_literal)]
#[allow(clippy::inconsistent_digit_grouping)]
fn test_interest_rates() {
    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: BPS_FACTOR,
            ..Default::default()
        },
        ..Default::default()
    };
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let debtor = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(
        &ObligationKey::new(debtor.clone()),
        &gold_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    // -- Move time --
    e.ledger().with_mut(|li| li.timestamp += 1);
    contract_client.refresh_pool(&usdc_pool_address);

    // 0% UR
    let borrow_bps = contract_client.get_pool(&usdc_pool_address).borrow_apr_bps;
    let supply_bps = contract_client.get_pool(&usdc_pool_address).supply_apr_bps;
    assert_eq!(borrow_bps, 00_01); // NB: calculations for APY yield 0% due to a precision loss
    assert_eq!(supply_bps, 00_00);

    // Borrow 50% of the deposited value
    contract_client.borrow(
        &ObligationKey::new(debtor.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
    );

    let rates = contract_client.get_pool_data(&usdc_pool_address).apy;
    assert_eq!(rates.borrow_bps, 23_89);
    assert_eq!(rates.supply_bps, 10_10);

    // Borrow 75% of the deposited value
    contract_client.borrow(
        &ObligationKey::new(debtor.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 4),
        &None,
    );

    let rates = contract_client.get_pool_data(&usdc_pool_address).apy;
    assert_eq!(rates.borrow_bps, 56_83);
    assert_eq!(rates.supply_bps, 35_48);

    // Borrow 80% of the deposited value
    contract_client.borrow(
        &ObligationKey::new(debtor.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 20),
        &None,
    );

    let rates = contract_client.get_pool_data(&usdc_pool_address).apy;
    assert_eq!(rates.borrow_bps, 82_21);
    assert_eq!(rates.supply_bps, 54_03);

    // Borrow 90% of the deposited value
    contract_client.borrow(
        &ObligationKey::new(debtor.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 10),
        &None,
    );

    let rates = contract_client.get_pool_data(&usdc_pool_address).apy;
    assert_eq!(rates.borrow_bps, 897_41);
    assert_eq!(rates.supply_bps, 544_30);

    // Borrow 100% of the deposited value
    contract_client.borrow(
        &ObligationKey::new(debtor.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 10),
        &None,
    );

    let rates = contract_client.get_pool_data(&usdc_pool_address).apy;
    assert_eq!(rates.supply_bps, 355_982);
    assert_eq!(rates.borrow_bps, 535_981);
}

#[test]
#[allow(clippy::inconsistent_digit_grouping)]
fn test_interest_rates_no_take_rate() {
    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: BPS_FACTOR,
            ..Default::default()
        },
        fee_config: PoolFeeConfig { take_rate_bps: 0, ..Default::default() },
        ..Default::default()
    };
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new_with_pool_config(pool_config);
    let debtor = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(
        &ObligationKey::new(debtor.clone()),
        &gold_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    // Borrow 100% of the deposited value
    contract_client.borrow(
        &ObligationKey::new(debtor.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    let rates = contract_client.get_pool_data(&usdc_pool_address).apy;
    assert_eq!(rates.borrow_bps, 535_981);
    assert_eq!(rates.supply_bps, 535_981);
}

// -- APY >= APR invariant tests --
//
// By definition, APY (compounded) must always be >= APR (non-compounded) for any positive rate.
// The tests below verify this invariant holds for both the borrow and supply sides.
//
// `test_apy_gte_apr_zero_reactivity` is the baseline: with ir_reactivity_constant=0 the modifier
// never moves from BPS_FACTOR, so get_apy() and accrue_interest() use the same underlying APR.
// The invariant holds trivially here.
//
// `test_apy_gte_apr_active_reactivity` is the test that exposes the same behaviour for non-zero reactivity constant.

#[test]
fn test_apy_gte_apr_zero_reactivity() {
    // ir_reactivity_constant = 0 means the modifier never moves: the control mechanism is disabled.
    // get_apy() and accrue_interest() therefore both operate on the raw kinked-model APR, so
    // APY >= APR must hold at every utilization level.
    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: BPS_FACTOR,
            ..Default::default()
        },
        ir_reactivity_constant: 0, // disabled
        ..Default::default()
    };

    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new_with_pool_config(pool_config);

    let debtor = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(
        &ObligationKey::new(debtor.clone()),
        &gold_pool_address,
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    // Borrow above the target utilization ratio (default target is 65%)
    // so we're in the regime where the modifier *would* be pushed up if reactivity were non-zero.
    contract_client.borrow(
        &ObligationKey::new(debtor.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT * 8 / 10), // 80% utilization
        &None,
    );

    // Let enough time pass that the modifier would have drifted substantially if it could.
    e.ledger().with_mut(|li| li.timestamp += SECONDS_IN_YEAR);
    contract_client.refresh_pool(&usdc_pool_address);

    let pool = contract_client.get_pool(&usdc_pool_address);
    let apy = contract_client.get_pool_data(&usdc_pool_address).apy;

    // With zero reactivity the modifier stays at BPS_FACTOR, so the stored APR equals the raw
    // kinked APR, which is exactly what get_apy() compounds. APY >= APR must hold.
    assert!(
        apy.borrow_bps >= pool.borrow_apr_bps as u32,
        "borrow APY ({}) must be >= borrow APR ({}) -- zero reactivity baseline",
        apy.borrow_bps,
        pool.borrow_apr_bps,
    );
    assert!(
        apy.supply_bps >= pool.supply_apr_bps as u32,
        "supply APY ({}) must be >= supply APR ({}) -- zero reactivity baseline",
        apy.supply_bps,
        pool.supply_apr_bps,
    );
}

#[test]
fn test_apy_gte_apr_active_reactivity() {
    // With a non-zero reactivity constant the modifier drifts when utilization diverges from
    // the target. Before the fix, get_apy() ignored the modifier entirely, so once the modifier
    // grew above BPS_FACTOR the stored borrow_apr_bps (modified) exceeded apy.borrow_bps
    // (un-modified), violating APY >= APR. After the fix get_apy() applies the same modifier
    // as accrue_interest(), so the invariant holds again.
    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: BPS_FACTOR,
            ..Default::default()
        },
        ir_reactivity_constant: MAX_REACTIVITY_CONSTANT, // maximum drift speed
        ..Default::default()
    };

    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new_with_pool_config(pool_config);

    let debtor = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(
        &ObligationKey::new(debtor.clone()),
        &gold_pool_address,
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    // 80% utilization -- above the 65% default target, so the modifier rises each accrual, and the
    // interest only pushes utilization further above it.
    contract_client.borrow(
        &ObligationKey::new(debtor.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT * 8 / 10),
        &None,
    );

    // Fifty days of stress: the modifier climbs to its ceiling and the year's APY saturates rather than
    // failing the view.
    for _ in 0..50 {
        e.ledger().with_mut(|li| li.timestamp += SECONDS_PER_DAY);
        contract_client.refresh_pool(&usdc_pool_address);
    }

    let pool = contract_client.get_pool(&usdc_pool_address);
    let apy = contract_client.get_pool_data(&usdc_pool_address).apy;

    // Confirm the modifier actually drifted above BPS_FACTOR so the test is meaningful.
    assert!(
        pool.interest_rate_modifier_bps > BPS_FACTOR,
        "modifier ({}) must exceed BPS_FACTOR ({}) for this test to be meaningful",
        pool.interest_rate_modifier_bps,
        BPS_FACTOR,
    );

    // After the fix: get_apy() applies the modifier, so apy.borrow_bps compounds the same
    // effective APR that accrue_interest() uses. APY >= APR must hold.
    assert!(
        apy.borrow_bps >= pool.borrow_apr_bps as u32,
        "borrow APY ({}) must be >= borrow APR ({}) (modifier={})",
        apy.borrow_bps,
        pool.borrow_apr_bps,
        pool.interest_rate_modifier_bps,
    );
    assert!(
        apy.supply_bps >= pool.supply_apr_bps as u32,
        "supply APY ({}) must be >= supply APR ({}) (modifier={})",
        apy.supply_bps,
        pool.supply_apr_bps,
        pool.interest_rate_modifier_bps,
    );
}

#[test]
fn test_interest_rate_reactivity() {
    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: BPS_FACTOR,
            ..Default::default()
        },
        ir_reactivity_constant: MAX_REACTIVITY_CONSTANT,
        ..Default::default()
    };

    let TestMarketFixture {
        e,
        contract_client,
        contract_id,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);

    let debtor = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(
        &ObligationKey::new(debtor.clone()),
        &gold_pool_address,
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    // 80% utilization
    contract_client.borrow(
        &ObligationKey::new(debtor.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT * 8 / 10),
        &None,
    );

    let initial_modifier = e.as_contract(&contract_id, || {
        Pool::try_get(&e, &usdc_pool_address).unwrap().interest_rate_modifier_bps
    });

    assert_eq!(initial_modifier, BPS_FACTOR);

    // -- Move time --

    e.ledger().with_mut(|li| li.timestamp += 100);
    contract_client.refresh_pool(&usdc_pool_address);

    // Above the target the modifier rises, so borrowing gets dearer while the pool is short of cash.
    let raised_modifier = e.as_contract(&contract_id, || {
        Pool::try_get(&e, &usdc_pool_address).unwrap().interest_rate_modifier_bps
    });

    assert!(raised_modifier > initial_modifier);

    contract_client.repay(
        &ObligationKey::new(debtor.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT * 4 / 10),
        &None,
    );

    // -- Move time --

    e.ledger().with_mut(|li| li.timestamp += 100);
    contract_client.refresh_pool(&usdc_pool_address);

    let lowered_modifier = e.as_contract(&contract_id, || {
        Pool::try_get(&e, &usdc_pool_address).unwrap().interest_rate_modifier_bps
    });

    assert!(lowered_modifier < initial_modifier);
}

/// A pool set carrying an invalid interest-rate model must be refused when it is queued. Accrual runs
/// inside deposit, withdraw, repay, borrow and liquidate, so a model that cannot compute an APR takes
/// the pool out of service entirely.
#[test]
fn test_invalid_interest_rate_model_is_refused() {
    let TestMarketFixture { contract_client, btc_pool_address, .. } = TestMarketFixture::new();

    let with_model = |model: KinkedIRConfig| market::pool::PoolConfig {
        interest_rate_model: market::interest_rate_model::InterestRateModel::Kinked(model),
        ..Default::default()
    };
    let default_model = KinkedIRConfig::default();

    // Accrual rejects a negative APR, so once such a set is applied every money operation on the
    // pool fails with `InvalidInputAmount`.
    let negative_apr =
        with_model(KinkedIRConfig { base_apr_bps: -1000, kink1_apr_bps: -500, ..default_model });
    // Refused too, although both are harmless at runtime: equal kinks make the middle branch
    // unreachable, and a descending slope only lowers the rate.
    let equal_kinks =
        with_model(KinkedIRConfig { kink1_ur_bps: 5000, kink2_ur_bps: 5000, ..default_model });
    let descending =
        with_model(KinkedIRConfig { kink1_apr_bps: 9000, kink2_apr_bps: 100, ..default_model });

    for (label, config) in [
        ("negative APRs", negative_apr),
        ("equal kinks", equal_kinks),
        ("descending APRs", descending),
    ] {
        let queued = contract_client.try_queue_in_pool_set(&btc_pool_address, &config);
        assert_eq!(
            queued,
            Err(Ok(MCError::InvalidLoanPoolConfig)),
            "{label}: an unusable interest-rate model was accepted"
        );
    }
}

/// The APY view saturates where `u32` bps runs out, and nowhere earlier: 129 703 bps of APR still
/// compounds to a year that fits, 129 704 bps no longer does.
#[test]
fn test_the_apy_saturates_exactly_where_u32_bps_runs_out() {
    use market::accrual::{Accrual, AccrualModel};

    let e = soroban_sdk::Env::default();
    let increase_bps = |apr: i128| {
        AccrualModel::Compounded.compute_multiplier(&e, apr, SECONDS_IN_YEAR).unwrap()
            / (SCALED_FIXED_POINT_DENOMINATOR / BPS_FACTOR)
            - BPS_FACTOR
    };
    assert!(increase_bps(129_703) <= u32::MAX as i128);
    assert!(increase_bps(129_704) > u32::MAX as i128);
    const { assert!(market::interest_rate::APY_SATURATING_APR_BPS > 129_703) };
}

/// Three pools fully lent with their modifier at the ceiling: at 129 800 bps of APR, just past where a
/// year stops fitting `u32` bps and below the early return; on the default curve, 4 000 %; and at
/// `MAX_APR_BPS`, 200 000 %, whose year cannot even be compounded. The market's data view still answers,
/// every APY saturated, every pool listed.
#[test]
fn test_market_data_survives_a_pool_past_the_apy_ceiling() {
    let f = TestMarketFixture::new();
    let pools = [
        (&f.btc_pool_address, Some(12_980)),
        (&f.usdc_pool_address, None),
        (&f.gold_pool_address, Some(MAX_APR_BPS)),
    ];
    for (address, max_apr_bps) in pools {
        f.e.as_contract(&f.contract_id, || {
            let mut pool = Pool::try_get(&f.e, address).unwrap();
            pool.total_available = 0;
            pool.take_rate_fees_sum = 0;
            pool.total_borrowed = 1_000_000_000;
            pool.total_d_tokens = 1_000_000_000;
            pool.interest_rate_modifier_bps = MAX_IR_MODIFIER;
            if let Some(max_apr_bps) = max_apr_bps {
                let market::interest_rate_model::InterestRateModel::Kinked(curve) =
                    &mut pool.config.interest_rate_model;
                curve.max_apr_bps = max_apr_bps;
            }
            market::storage::set_pool(&f.e, address, &pool);
        });
    }

    let data = f.contract_client.get_market_data();
    assert_eq!(data.pools_data.len(), f.contract_client.get_all_pools().len());
    for (address, _) in pools {
        let stressed = data.pools_data.iter().find(|p| &p.pool.pool_address == address);
        assert_eq!(stressed.expect("a stressed pool was dropped").apy.borrow_bps, u32::MAX);
    }
}

/// **B18:** once take-rate fees outgrow a pool's cash, lenders still own cash plus debt less the fees, so a
/// deposit buys shares at that backing instead of overpaying for the earlier lenders and the fee bucket.
#[test]
fn test_a_deposit_into_a_pool_whose_fees_exceed_its_cash_is_priced_at_its_backing() {
    let f = TestMarketFixture::new();
    let (lender, borrower, newcomer) = (
        ObligationKey::new(f.users[0].clone()),
        ObligationKey::new(f.users[1].clone()),
        ObligationKey::new(f.users[2].clone()),
    );
    let usdc = &f.usdc_pool_address;
    f.contract_client.deposit(&lender, usdc, &DEFAULT_DEPOSIT_AMOUNT, &None);
    f.contract_client.add_collateral(
        &borrower,
        &f.gold_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    f.contract_client.borrow(&borrower, usdc, &i128::MAX, &None);
    f.e.ledger().with_mut(|li| li.timestamp += 365 * 24 * 60 * 60);
    f.contract_client.refresh_pool(usdc);
    let pool = f.contract_client.get_pool(usdc);
    assert!(
        pool.take_rate_fees_sum > pool.total_available,
        "the fees-over-cash state was not reached"
    );

    let backing = |p: &Pool| p.total_available + p.total_borrowed - p.take_rate_fees_sum;
    let before = f.contract_client.get_pool(usdc);
    f.contract_client.deposit(&newcomer, usdc, &DEFAULT_DEPOSIT_AMOUNT, &None);
    let after = f.contract_client.get_pool(usdc);

    // The newcomer's shares, valued at what actually backs them, are worth the deposit within one share.
    let shares = after.total_j_tokens - before.total_j_tokens;
    let worth = shares * backing(&after) / after.total_j_tokens;
    let one_share = backing(&after) / after.total_j_tokens + 1;
    assert!(
        worth >= DEFAULT_DEPOSIT_AMOUNT - one_share,
        "a {DEFAULT_DEPOSIT_AMOUNT} deposit bought shares backed by {worth}"
    );
}

/// A pool whose fees outgrew its whole backing after a write-off still has debt: it is fully utilized, not
/// idle, so its remaining borrowers keep paying the top of the curve.
#[test]
fn test_a_pool_with_debt_but_no_net_backing_is_fully_utilized() {
    let f = TestMarketFixture::new();
    let usdc = &f.usdc_pool_address;
    f.e.as_contract(&f.contract_id, || {
        let mut pool = Pool::try_get(&f.e, usdc).unwrap();
        (pool.total_available, pool.total_borrowed, pool.take_rate_fees_sum) = (0, 20, 50);
        market::storage::set_pool(&f.e, usdc, &pool);
    });

    assert_eq!(f.contract_client.get_pool(usdc).compute_utilization_ratio_bps(), Ok(BPS_FACTOR));
}

/// **B20:** the curve in force until a queued set applies prices that interval; the incoming curve only
/// prices what follows. Accruing just before `apply_pool_set` must change nothing.
#[test]
fn test_apply_pool_set_prices_the_past_at_the_outgoing_curve() {
    let run = |accrue_first: bool| -> i128 {
        let f = TestMarketFixture::new();
        let (lender, borrower) =
            (ObligationKey::new(f.users[0].clone()), ObligationKey::new(f.users[1].clone()));
        let usdc = &f.usdc_pool_address;
        f.contract_client.deposit(&lender, usdc, &(10 * DEFAULT_DEPOSIT_AMOUNT), &None);
        f.contract_client.add_collateral(
            &borrower,
            &f.gold_pool_address,
            &(100 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );
        f.contract_client.borrow(&borrower, usdc, &(5 * DEFAULT_DEPOSIT_AMOUNT), &None);

        let mut steeper = f.contract_client.get_pool(usdc).config;
        let market::interest_rate_model::InterestRateModel::Kinked(curve) =
            &mut steeper.interest_rate_model;
        (curve.base_apr_bps, curve.kink1_apr_bps, curve.kink2_apr_bps, curve.max_apr_bps) =
            (5_000, 20_000, 40_000, 80_000);
        f.contract_client.queue_in_pool_set(usdc, &steeper);
        let period = f.contract_client.get_global_state().update_in_queue_period;
        f.e.ledger().with_mut(|li| li.timestamp += period);

        if accrue_first {
            f.contract_client.refresh_pool(usdc);
        }
        f.contract_client.apply_pool_set(usdc);
        f.contract_client.refresh_pool(usdc);
        f.contract_client.get_pool(usdc).total_borrowed
    };

    assert_eq!(run(false), run(true));
}
