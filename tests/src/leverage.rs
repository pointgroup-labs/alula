#![cfg(test)]

use {
    crate::{
        get_borrow_obligation, get_obligation_borrowed, get_obligation_tokens_from_shares,
        tests::{get_amount_scaled_down, get_amount_scaled_up},
        LCError, TestFixture, DEFAULT_DEPOSIT_AMOUNT,
    },
    lending::{
        constants::{
            DEFAULT_FLASH_LOAN_FEE_BPS, DEFAULT_MAX_SLIPPAGE_BPS, MAX_LEVERAGE_MULTIPLIER,
            MIN_LEVERAGE_MULTIPLIER,
        },
        swap,
    },
};

// ---- Deposit with leverage ----

#[test]
fn test_deposit_zero() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit into a different pool to make flash loans possible
    contract_client.deposit(&user2, &gold_pool_address, &(1000 * DEFAULT_DEPOSIT_AMOUNT));

    let usdc_pool_before = contract_client.get_pool(&usdc_pool_address);
    let gold_pool_before = contract_client.get_pool(&usdc_pool_address);

    contract_client.deposit_with_leverage(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &0,
        &40, // x4
    );

    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    let gold_pool_after = contract_client.get_pool(&usdc_pool_address);

    assert_eq!(usdc_pool_before, usdc_pool_after);
    assert_eq!(gold_pool_before, gold_pool_after);
}

#[test]
fn test_deposit_with_invalid_leverage_multiplier() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();

    assert_eq!(
        Err(Ok(LCError::InvalidLeverageMultiplier)),
        contract_client.try_deposit_with_leverage(
            &user,
            &usdc_pool_address,
            &gold_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &(MIN_LEVERAGE_MULTIPLIER - 1), // x(<1)
        )
    );

    assert_eq!(
        Err(Ok(LCError::InvalidLeverageMultiplier)),
        contract_client.try_deposit_with_leverage(
            &user,
            &usdc_pool_address,
            &gold_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &(MAX_LEVERAGE_MULTIPLIER + 1),
        )
    );
}

// TODO: Add tests which check for supply and borrow limit constraints. This affects flash loans, right?
#[test]
fn test_deposit_with_no_leverage() {
    let TestFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();

    contract_client.deposit_with_leverage(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &MIN_LEVERAGE_MULTIPLIER, // x1
    );

    // Check if this is equivalent to a plain deposit
    let obligation_tokens_from_shares =
        get_obligation_tokens_from_shares(&contract_client, &user, &usdc_pool_address).unwrap();

    let amount_out = swap::get_amount_out(
        &e,
        &usdc_pool_address,
        &gold_pool_address,
        DEFAULT_DEPOSIT_AMOUNT,
    )
    .unwrap();
    let expected_deposited_amount = get_amount_scaled_down(amount_out, DEFAULT_MAX_SLIPPAGE_BPS);

    assert_eq!(expected_deposited_amount, obligation_tokens_from_shares);
    // No borrowing position must be created
    assert!(get_borrow_obligation(&contract_client, &user, &gold_pool_address).is_err());
}

#[test]
fn test_deposit_with_unavailable_flash_loan_capacity() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();

    assert_eq!(
        contract_client.try_deposit_with_leverage(
            &user,
            &usdc_pool_address,
            &gold_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &11, // x1.1
        ),
        Err(Ok(LCError::NotEnoughPoolFunds))
    );
}

#[test]
fn test_deposit_with_unhealthy_leverage() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit into a different pool to make flash loans possible
    contract_client.deposit(&user2, &gold_pool_address, &(1000 * DEFAULT_DEPOSIT_AMOUNT));

    assert_eq!(
        contract_client.try_deposit_with_leverage(
            &user,
            &usdc_pool_address,
            &gold_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &400, // x40
        ),
        Err(Ok(LCError::HealthFactorIsLowerThanRequiredThreshold))
    );
}

#[test]
fn test_deposit_with_leverage() {
    let TestFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit into a different pool to make flash loans possible
    contract_client.deposit(&user2, &gold_pool_address, &(1000 * DEFAULT_DEPOSIT_AMOUNT));

    contract_client.deposit_with_leverage(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &40, // x4 leverage
    );

    // Check obligation
    let obligation_tokens_from_shares =
        get_obligation_tokens_from_shares(&contract_client, &user, &usdc_pool_address).unwrap();

    let obligation_borrowed =
        get_obligation_borrowed(&contract_client, &user, &gold_pool_address).unwrap();

    let amount_in = 4 * DEFAULT_DEPOSIT_AMOUNT;
    let amount_out =
        swap::get_amount_out(&e, &usdc_pool_address, &gold_pool_address, amount_in).unwrap();
    let expected_supply_amount = get_amount_scaled_down(amount_out, DEFAULT_MAX_SLIPPAGE_BPS);

    let amount_out = swap::get_amount_out(
        &e,
        &usdc_pool_address,
        &gold_pool_address,
        amount_in - DEFAULT_DEPOSIT_AMOUNT, // minus original borrow funds
    )
    .unwrap();
    let expected_borrowed_amount = get_amount_scaled_up(amount_out, DEFAULT_FLASH_LOAN_FEE_BPS);

    assert_eq!(obligation_tokens_from_shares, expected_supply_amount);
    assert_eq!(obligation_borrowed, expected_borrowed_amount);

    // Check pools
    let deposit_pool = contract_client.get_pool(&usdc_pool_address);
    let borrow_pool = contract_client.get_pool(&gold_pool_address);

    let total_supply = deposit_pool.total_supply().unwrap();
    let total_borrowed = borrow_pool.total_borrowed;

    assert_eq!(expected_supply_amount, total_supply);
    assert_eq!(total_borrowed, expected_borrowed_amount);
}

// ---- Deleverage and Withdraw ----

#[test]
fn test_withdraw_zero() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit into a different pool to make flash loans possible
    contract_client.deposit(&user2, &gold_pool_address, &(1000 * DEFAULT_DEPOSIT_AMOUNT));

    contract_client.deposit_with_leverage(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &40, // x4 leverage
    );

    let usdc_pool_before = contract_client.get_pool(&usdc_pool_address);
    let gold_pool_before = contract_client.get_pool(&usdc_pool_address);
    let obligation_before = contract_client.get_user_obligation(&user);

    contract_client.deleverage_and_withdraw(&user, &usdc_pool_address, &gold_pool_address, &0);

    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    let gold_pool_after = contract_client.get_pool(&usdc_pool_address);
    let obligation_after = contract_client.get_user_obligation(&user);

    assert_eq!(usdc_pool_before, usdc_pool_after);
    assert_eq!(gold_pool_before, gold_pool_after);
    assert_eq!(obligation_before, obligation_after);
}

#[test]
fn test_withdraw_negative() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit into a different pool to make flash loans possible
    contract_client.deposit(&user2, &gold_pool_address, &(1000 * DEFAULT_DEPOSIT_AMOUNT));

    contract_client.deposit_with_leverage(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &40, // x4 leverage
    );

    assert_eq!(
        Err(Ok(LCError::NegativeWithdraw)),
        contract_client.try_deleverage_and_withdraw(
            &user,
            &usdc_pool_address,
            &gold_pool_address,
            &-1
        )
    );
}

#[test]
fn test_withdraw_for_position_with_no_leverage() {
    let TestFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();

    contract_client.deposit_with_leverage(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &MIN_LEVERAGE_MULTIPLIER, // x1
    );

    // No borrow position must exist
    assert!(get_borrow_obligation(&contract_client, &user, &gold_pool_address).is_err());

    // Check obligation
    let amount_out = swap::get_amount_out(
        &e,
        &gold_pool_address,
        &usdc_pool_address,
        DEFAULT_DEPOSIT_AMOUNT,
    )
    .unwrap();

    let expected_amount = get_amount_scaled_down(amount_out, DEFAULT_MAX_SLIPPAGE_BPS);
    let obligation_tokens_from_shares =
        get_obligation_tokens_from_shares(&contract_client, &user, &usdc_pool_address).unwrap();

    // No borrow position must exist still
    assert!(get_borrow_obligation(&contract_client, &user, &gold_pool_address).is_err());
    assert_eq!(expected_amount, obligation_tokens_from_shares);
}

#[test]
fn test_withdraw() {
    const LEVERAGE_MULTIPLIER: u32 = 4; // x4 leverage

    let TestFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit into a different pool to make flash loans possible
    contract_client.deposit(&user2, &gold_pool_address, &(1000 * DEFAULT_DEPOSIT_AMOUNT));

    contract_client.deposit_with_leverage(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &(10 * LEVERAGE_MULTIPLIER),
    );

    let amount_out = swap::get_amount_out(
        &e,
        &gold_pool_address,
        &usdc_pool_address,
        DEFAULT_DEPOSIT_AMOUNT,
    )
    .unwrap();
    let withdrawable_amount = get_amount_scaled_down(amount_out, 10_00);

    // We must be able to withdraw the initial amount
    contract_client.deleverage_and_withdraw(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &withdrawable_amount,
    );

    // Check obligation
    let approximate_borrowed_amount = ((LEVERAGE_MULTIPLIER - 1) as i128) * DEFAULT_DEPOSIT_AMOUNT;
    let approximate_deposited_amount = swap::get_amount_out(
        &e,
        &gold_pool_address,
        &usdc_pool_address,
        (LEVERAGE_MULTIPLIER as i128) * DEFAULT_DEPOSIT_AMOUNT,
    )
    .unwrap();

    let obligation_borrowed =
        get_obligation_borrowed(&contract_client, &user, &gold_pool_address).unwrap();

    let obligation_tokens_from_shares =
        get_obligation_tokens_from_shares(&contract_client, &user, &usdc_pool_address).unwrap();

    // less than 10% of originally deposited value must be left
    let expected_deposit_left_upper_bound_amount =
        get_amount_scaled_down(approximate_deposited_amount, 90_00);
    assert!(obligation_tokens_from_shares < expected_deposit_left_upper_bound_amount);

    // more than 5 % of original deposited value must be left
    let expected_left_lower_bound_amount =
        get_amount_scaled_down(approximate_deposited_amount, 95_00);
    assert!(obligation_tokens_from_shares > expected_left_lower_bound_amount);

    // less than 10% of borrowed value must be left
    let expected_borrow_left_upper_bound_amount =
        get_amount_scaled_down(approximate_borrowed_amount, 90_00);
    assert!(obligation_borrowed < expected_borrow_left_upper_bound_amount);

    // more than 5 % of borrowed value must be left
    let expected_borrow_left_lower_bound_amount =
        get_amount_scaled_down(approximate_borrowed_amount, 95_00);
    assert!(obligation_borrowed > expected_borrow_left_lower_bound_amount);

    // Check pools
    let deposit_pool = contract_client.get_pool(&usdc_pool_address);
    let total_supply = deposit_pool.total_supply().unwrap();

    let borrow_pool = contract_client.get_pool(&gold_pool_address);
    let total_borrowed = borrow_pool.total_borrowed;

    assert_eq!(total_supply, obligation_tokens_from_shares);
    assert_eq!(total_borrowed, obligation_borrowed);
}

#[test]
fn test_withdraw_over_balance() {
    let TestFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit into a different pool to make flash loans possible
    contract_client.deposit(&user2, &gold_pool_address, &(1000 * DEFAULT_DEPOSIT_AMOUNT));

    contract_client.deposit_with_leverage(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT),
        &40, // x4 leverage
    );

    let amount_out = swap::get_amount_out(
        &e,
        &gold_pool_address,
        &usdc_pool_address,
        DEFAULT_DEPOSIT_AMOUNT,
    )
    .unwrap();
    let withdrawable_amount = get_amount_scaled_down(amount_out, 2_00);

    // We must be able to withdraw not more than the initial amount
    assert_eq!(
        Err(Ok(LCError::WithdrawOverBalance)),
        contract_client.try_deleverage_and_withdraw(
            &user,
            &usdc_pool_address,
            &gold_pool_address,
            &(10 * withdrawable_amount / 9),
        )
    );

    // On the contrary, withdrawable amount must be able to be withdrawn
    contract_client.deleverage_and_withdraw(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &withdrawable_amount,
    );
}

#[test]
fn test_withdraw_all_available_with_i128_max() {
    let TestFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit into a different pool to make flash loans possible
    contract_client.deposit(&user2, &gold_pool_address, &(1000 * DEFAULT_DEPOSIT_AMOUNT));

    contract_client.deposit_with_leverage(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT),
        &40, // x4 leverage
    );

    let amount_out = swap::get_amount_out(
        &e,
        &gold_pool_address,
        &usdc_pool_address,
        DEFAULT_DEPOSIT_AMOUNT,
    )
    .unwrap();

    let deposited_before = contract_client
        .get_pool(&usdc_pool_address)
        .total_supply()
        .unwrap();
    let borrowed_before = contract_client.get_pool(&gold_pool_address).total_borrowed;

    contract_client.deleverage_and_withdraw(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &i128::MAX,
    );

    let deposited_after = contract_client
        .get_pool(&usdc_pool_address)
        .total_supply()
        .unwrap();
    let borrowed_after = contract_client.get_pool(&gold_pool_address).total_borrowed;

    assert_eq!(deposited_before, deposited_after);
    assert_eq!(borrowed_after, borrowed_before);
}
