#![cfg(test)]

use lending::{
    constants::{BPS_FACTOR, DEFAULT_LIQUIDATION_THRESHOLD},
    pool::PoolConfig,
    LCError,
};
use soroban_sdk::Address;

use crate::{
    get_borrow_obligation, get_deposit_obligation, get_obligation_borrowed, TestFixture,
    DEFAULT_DEPOSIT_AMOUNT,
};

#[test]
fn test_borrow() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user: Address = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();
    // Deposit gold to satisfy the health factor threshold
    contract_client.deposit(&user, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));
    contract_client.borrow(&user, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligation_borrowed = get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .borrowed;
    let pool_borrowed = contract_client.get_pool(&usdc_pool_address).total_borrowed;

    assert_eq!(obligation_borrowed, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_borrowed, DEFAULT_DEPOSIT_AMOUNT);
}

#[test]
#[ignore]
fn test_exceed_borrow_limit() {
    const UTILIZATION_RATION_LIMIT_BPS: i128 = 9000; // 90%

    let pool_config = PoolConfig {
        utilization_ratio_limit_bps: UTILIZATION_RATION_LIMIT_BPS,
        ..Default::default()
    };

    let TestFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new_with_pool_config(pool_config);

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit gold to satisfy the health factor threshold
    contract_client.deposit(&user, &gold_pool_address, &(2 * &DEFAULT_DEPOSIT_AMOUNT));

    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    contract_client.borrow(
        &user,
        &usdc_pool_address,
        &((DEFAULT_DEPOSIT_AMOUNT * UTILIZATION_RATION_LIMIT_BPS) / BPS_FACTOR),
    );

    assert_eq!(
        contract_client.try_borrow(&user, &usdc_pool_address, &1),
        Err(Ok(LCError::BorrowLimitExceeded))
    );
}

#[test]
fn test_borrow_zero() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    // Deposit gold to satisfy the health factor threshold
    contract_client.deposit(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    let pool_before = contract_client.get_pool(&usdc_pool_address);

    // TODO: This borrow will create a `BorrowObligation` with `borrowed` == 0. Should we care about
    // that?
    contract_client.borrow(&user, &usdc_pool_address, &0);

    let pool_after = contract_client.get_pool(&usdc_pool_address);

    assert_eq!(pool_before, pool_after);
}

#[test]
fn test_borrow_negative() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    // Deposit gold to satisfy the health factor threshold
    contract_client.deposit(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    assert_eq!(
        contract_client.try_borrow(&user, &usdc_pool_address, &-1),
        Err(Ok(LCError::NegativeBorrow))
    );
}

#[test]
#[ignore]
fn test_borrow_health_factor_add_collateral() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user: Address = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();
    // Deposit gold to satisfy the health factor threshold
    contract_client.add_collateral(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    let deposit_obligation =
        get_deposit_obligation(&contract_client, &user, &gold_pool_address).unwrap();
    assert_eq!(deposit_obligation.collateral, DEFAULT_DEPOSIT_AMOUNT);

    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    let deposit_obligation2 =
        get_deposit_obligation(&contract_client, &user2, &usdc_pool_address).unwrap();
    assert_eq!(deposit_obligation2.shares, 2 * DEFAULT_DEPOSIT_AMOUNT);

    // Borrow 50% of the deposited value
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    // Borrow 75%
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    // Borrow 80% - equals test's fixture liquidation threshold
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 20));

    // Borrow which leads to the health factor threshold constraint violation
    assert_eq!(
        contract_client.try_borrow(&user, &usdc_pool_address, &(1)),
        Err(Ok(LCError::HealthFactorIsLowerThanRequiredThreshold))
    );

    // Improve health factor
    contract_client.add_collateral(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    // Borrow without health factor violation
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));
}

#[test]
#[ignore]
fn test_borrow_health_factor_deposit() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user: Address = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();
    // Deposit gold to satisfy the health factor threshold
    contract_client.deposit(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    // Borrow 50% of the deposited value
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    // Borrow 75%
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    // Borrow 80% - equals test's fixture liquidation threshold
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 20));

    // Borrow which leads to the health factor threshold constraint violation
    assert_eq!(
        contract_client.try_borrow(&user, &usdc_pool_address, &(1)),
        Err(Ok(LCError::HealthFactorIsLowerThanRequiredThreshold))
    );

    // Improve health factor
    contract_client.deposit(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    // Borrow without health factor violation
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));
}

#[test]
fn borrow_more_than_open_ltv_allows() {
    const MAX_BORROWING_AMOUNT: i128 =
        (DEFAULT_DEPOSIT_AMOUNT * DEFAULT_LIQUIDATION_THRESHOLD) / 100;

    let TestFixture {
        e,
        contract_client,
        contract_id,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Fill up the borrowing pool with liquidity
    contract_client.deposit(&user2, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    // Deposit gold as deposit that backs future borrows
    contract_client.deposit(&user, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligation = contract_client.get_user_obligation(&user);
    e.as_contract(&contract_id, || {
        let max_borrowing_amount = obligation
            .compute_max_healthy_borrow_added_amount(&e, &usdc_pool_address)
            .unwrap();

        assert_eq!(max_borrowing_amount, MAX_BORROWING_AMOUNT);
    });

    // Borrow twice as possible
    contract_client.borrow(&user, &usdc_pool_address, &(MAX_BORROWING_AMOUNT * 2));

    let pool_borrowed = contract_client.get_pool(&usdc_pool_address).total_borrowed;
    let obligation_borrowed =
        get_obligation_borrowed(&contract_client, &user, &usdc_pool_address).unwrap();

    assert_eq!(obligation_borrowed, MAX_BORROWING_AMOUNT);
    assert_eq!(pool_borrowed, MAX_BORROWING_AMOUNT);

    let obligation = contract_client.get_user_obligation(&user);
    e.as_contract(&contract_id, || {
        let max_borrowing_amount = obligation
            .compute_max_healthy_borrow_added_amount(&e, &gold_pool_address)
            .unwrap();

        assert_eq!(max_borrowing_amount, 0);
    });
}
