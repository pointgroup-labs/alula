#![cfg(test)]

use {
    crate::{
        get_borrow_obligation, get_obligation_borrowed, get_obligation_tokens_from_shares,
        tests::{get_amount_scaled_down, get_amount_scaled_up},
        TestFixture, DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT,
    },
    lending::{
        constants::{
            LCError, DEFAULT_FLASH_LOAN_FEE_BPS, DEFAULT_MAX_SLIPPAGE_BPS, MAX_LEVERAGE_MULTIPLIER,
            MIN_LEVERAGE_MULTIPLIER,
        },
        swap,
    },
};

// ---- Deposit with leverage ----

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
            &(MAX_LEVERAGE_MULTIPLIER + 1), // x(>100)
        )
    );
}

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
            &11,
        ),
        Err(Ok(LCError::NotEnoughPoolFunds))
    );
}

// TODO: This test will work when mock oracle will be able to change the price ratio
// #[test]
// fn test_deposit_with_unhealthy_leverage() {
//     let TestFixture {
//         contract_client,
//         usdc_pool_address,
//         gold_pool_address,
//         users,
//         ..
//     } = TestFixture::new();

//     let user = users.get(0).unwrap();
//     let user2 = users.get(1).unwrap();

//     // Deposit into a different pool to make flash loans possible
//     contract_client.deposit(&user2, &gold_pool_address, &(1000 * DEFAULT_DEPOSIT_AMOUNT));

//     assert_eq!(
//         contract_client.try_deposit_with_leverage(
//             &user,
//             &usdc_pool_address,
//             &gold_pool_address,
//             &DEFAULT_DEPOSIT_AMOUNT,
//             &900, // x80
//         ),
//         Err(Ok(LCError::NonPositiveRepay))
//     );
// }

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
    contract_client.deposit(&user2, &gold_pool_address, &(100 * DEFAULT_DEPOSIT_AMOUNT));

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
    let expected_deposited_amount = get_amount_scaled_down(amount_out, DEFAULT_MAX_SLIPPAGE_BPS);

    let amount_out = swap::get_amount_out(
        &e,
        &usdc_pool_address,
        &gold_pool_address,
        amount_in - DEFAULT_DEPOSIT_AMOUNT, // minus original borrow funds
    )
    .unwrap()
        / 2; // * 2 because order matters here TODO: Rewrite this
    let expected_borrowed_amount = get_amount_scaled_up(amount_out, DEFAULT_FLASH_LOAN_FEE_BPS);

    assert_eq!(obligation_tokens_from_shares, expected_deposited_amount);
    assert_eq!(obligation_borrowed, expected_borrowed_amount);

    // Check pools
    let deposit_pool = contract_client.get_pool(&usdc_pool_address);
    let borrow_pool = contract_client.get_pool(&gold_pool_address);

    let total_deposited = deposit_pool.total_liquidity().unwrap();
    let total_borrowed = borrow_pool.total_borrowed;

    assert_eq!(expected_deposited_amount, total_deposited);
    assert_eq!(total_borrowed, expected_borrowed_amount);
}

// ---- Deleverage and Withdraw ----

// #[test]
// fn test_withdraw_non_positive() {
//     let TestFixture {
//         contract_client,
//         usdc_pool_address,
//         gold_pool_address,
//         users,
//         ..
//     } = TestFixture::new();

//     let user = users.get(0).unwrap();
//     let user2 = users.get(1).unwrap();

//     // Deposit into a different pool to make flash loans possible
//     contract_client.deposit(&user2, &gold_pool_address, &(100 * DEFAULT_DEPOSIT_AMOUNT));

//     contract_client.deposit_with_leverage(
//         &user,
//         &usdc_pool_address,
//         &gold_pool_address,
//         &DEFAULT_DEPOSIT_AMOUNT,
//         &40, // x4 leverage
//     );

//     assert_eq!(
//         Err(Ok(LCError::NonPositiveWithdraw)),
//         contract_client.try_deleverage_and_withdraw(
//             &user,
//             &usdc_pool_address,
//             &gold_pool_address,
//             &0
//         )
//     );
// }

// #[test]
// fn test_withdraw() {
//     let TestFixture {
//         contract_client,
//         usdc_pool_address,
//         gold_pool_address,
//         users,
//         ..
//     } = TestFixture::new();

//     let user = users.get(0).unwrap();
//     let user2 = users.get(1).unwrap();

//     // Deposit into a different pool to make flash loans possible
//     contract_client.deposit(&user2, &gold_pool_address, &(100 * DEFAULT_DEPOSIT_AMOUNT));

//     contract_client.deposit_with_leverage(
//         &user,
//         &usdc_pool_address,
//         &gold_pool_address,
//         &DEFAULT_DEPOSIT_AMOUNT,
//         &40, // x4 leverage
//     );

//     let withdrawable_amount = get_amount_scaled_down(DEFAULT_DEPOSIT_AMOUNT, 2_00); // must be withing 2%

//     // We must be able to withdraw the initial amount
//     contract_client.deleverage_and_withdraw(
//         &user,
//         &usdc_pool_address,
//         &gold_pool_address,
//         &withdrawable_amount,
//     );

//     // Check obligation
//     let obligation_tokens_from_shares =
//         get_obligation_tokens_from_shares(&contract_client, &user, &usdc_pool_address).unwrap();

//     // less than 10% of originally deposited value must be left
//     let expected_deposit_left_upper_bound_amount =
//         get_amount_scaled_down(DEFAULT_COLLATERAL_AMOUNT, 90_00);
//     assert!(obligation_tokens_from_shares < expected_deposit_left_upper_bound_amount);

//     // more than 5 % of original deposited value must be left
//     let expected_left_lower_bound_amount = get_amount_scaled_down(DEFAULT_COLLATERAL_AMOUNT, 95_00);
//     assert!(obligation_tokens_from_shares > expected_left_lower_bound_amount);

//     let obligation_borrowed =
//         get_obligation_borrowed(&contract_client, &user, &gold_pool_address).unwrap();

//     // less than 10% of borrowed value must be left
//     let expected_borrow_left_upper_bound_amount =
//         get_amount_scaled_down(DEFAULT_COLLATERAL_AMOUNT, 95_00);
//     assert!(obligation_borrowed < expected_borrow_left_upper_bound_amount);

//     // more than 5 % of borrowed value must be left
//     let expected_borrow_left_lower_bound_amount =
//         get_amount_scaled_down(DEFAULT_COLLATERAL_AMOUNT, 97_00);
//     assert!(obligation_borrowed > expected_borrow_left_lower_bound_amount);

//     // Check pools
//     let deposit_pool = contract_client.get_pool(&usdc_pool_address);
//     let total_deposited = deposit_pool.total_liquidity().unwrap();

//     let borrow_pool = contract_client.get_pool(&gold_pool_address);
//     let total_borrowed = borrow_pool.total_borrowed;

//     assert_eq!(total_deposited, obligation_tokens_from_shares);
//     assert_eq!(total_borrowed, obligation_borrowed);
// }

// #[test]
// fn test_withdraw_over_balance() {
//     let TestFixture {
//         contract_client,
//         usdc_pool_address,
//         gold_pool_address,
//         users,
//         ..
//     } = TestFixture::new();

//     let user = users.get(0).unwrap();
//     let user2 = users.get(1).unwrap();

//     // Deposit into a different pool to make flash loans possible
//     contract_client.deposit(&user2, &gold_pool_address, &(100 * DEFAULT_DEPOSIT_AMOUNT));

//     contract_client.deposit_with_leverage(
//         &user,
//         &usdc_pool_address,
//         &gold_pool_address,
//         &DEFAULT_DEPOSIT_AMOUNT,
//         &40, // x4 leverage
//     );

//     let withdrawable_amount = get_amount_scaled_down(DEFAULT_DEPOSIT_AMOUNT, 2_00);
//     // must be withing 2%

//     // We must be able to withdraw the initial amount
//     assert_eq!(
//         Err(Ok(LCError::WithdrawOverBalance)),
//         contract_client.try_deleverage_and_withdraw(
//             &user,
//             &usdc_pool_address,
//             &gold_pool_address,
//             &((3 * withdrawable_amount) / 2),
//         )
//     );
// }
