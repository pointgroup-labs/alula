#![cfg(test)]

use market::{
    constants::*,
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

    // 40% utilization -- below the 65% default target.
    // When util < target: utilization_diff < 0, so the modifier INCREASES each accrual.
    // Use short time steps so interest accrual doesn't compound the borrowed amount enough
    // to push utilization past the target (which would reverse the modifier direction).
    contract_client.borrow(
        &ObligationKey::new(debtor.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT * 4 / 10),
        &None,
    );

    // Many short refreshes: each one nudges the modifier upward without materially
    // shifting the utilization ratio via interest compounding.
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

    let decreased_modifier = e.as_contract(&contract_id, || {
        Pool::try_get(&e, &usdc_pool_address).unwrap().interest_rate_modifier_bps
    });

    assert!(decreased_modifier < initial_modifier);

    contract_client.repay(
        &ObligationKey::new(debtor.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT * 4 / 10),
        &None,
    );

    // -- Move time --

    e.ledger().with_mut(|li| li.timestamp += 100);
    contract_client.refresh_pool(&usdc_pool_address);

    let increased_modifier = e.as_contract(&contract_id, || {
        Pool::try_get(&e, &usdc_pool_address).unwrap().interest_rate_modifier_bps
    });

    assert!(increased_modifier > initial_modifier);
}
