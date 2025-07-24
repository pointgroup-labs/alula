#![cfg(test)]

use {
    crate::{
        get_borrow_obligation, get_deposit_obligation, get_obligation_borrowed,
        get_obligation_tokens_from_shares,
        tests::{get_amount_scaled_down, get_amount_scaled_up},
        LCError, TestFixture, DEFAULT_DEPOSIT_AMOUNT,
    },
    lending::{
        constants::{
            DEFAULT_FLASH_LOAN_FEE_BPS, DEFAULT_MAX_SLIPPAGE_BPS, LEVERAGE_SCALE,
            MAX_LEVERAGE_MULTIPLIER, MIN_LEVERAGE_MULTIPLIER,
        },
        swap,
    },
    soroban_sdk::{testutils::Ledger, Env},
};

// ---- Deposit with leverage ----

#[test]
fn test_deposit_zero() {
    const LEVERAGE: u32 = 4; // x4 leverage
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

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
        &LEVERAGE_MULTIPLIER,
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
        get_obligation_tokens_from_shares(&e, &contract_client, &user, &usdc_pool_address).unwrap();

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
    const LEVERAGE: u32 = 11; // x11 leverage
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

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
            &LEVERAGE_MULTIPLIER,
        ),
        Err(Ok(LCError::NotEnoughPoolFunds))
    );
}

#[test]
#[ignore]
fn test_deposit_with_unhealthy_leverage() {
    const LEVERAGE: u32 = 40;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

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
            &LEVERAGE_MULTIPLIER,
        ),
        Err(Ok(LCError::HealthFactorIsLowerThanRequiredThreshold))
    );
}

#[test]
fn test_deposit_with_leverage() {
    const LEVERAGE: u32 = 4;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

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
        &LEVERAGE_MULTIPLIER,
    );

    // Check obligation
    let obligation_tokens_from_shares =
        get_obligation_tokens_from_shares(&e, &contract_client, &user, &usdc_pool_address).unwrap();

    let obligation_borrowed =
        get_obligation_borrowed(&contract_client, &user, &gold_pool_address).unwrap();

    let amount_in = LEVERAGE as i128 * DEFAULT_DEPOSIT_AMOUNT;
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
    const LEVERAGE: u32 = 4;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

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
        &LEVERAGE_MULTIPLIER,
    );

    let usdc_pool_before = contract_client.get_pool(&usdc_pool_address);
    let gold_pool_before = contract_client.get_pool(&usdc_pool_address);
    let obligation_before = contract_client.get_user_obligation(&user);

    contract_client.withdraw_from_leveraged(&user, &usdc_pool_address, &gold_pool_address, &0);

    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    let gold_pool_after = contract_client.get_pool(&usdc_pool_address);
    let obligation_after = contract_client.get_user_obligation(&user);

    assert_eq!(usdc_pool_before, usdc_pool_after);
    assert_eq!(gold_pool_before, gold_pool_after);
    assert_eq!(obligation_before, obligation_after);
}

#[test]
fn test_withdraw_negative() {
    const LEVERAGE: u32 = 4;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

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
        &LEVERAGE_MULTIPLIER,
    );

    assert_eq!(
        Err(Ok(LCError::NegativeWithdraw)),
        contract_client.try_withdraw_from_leveraged(
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
        get_obligation_tokens_from_shares(&e, &contract_client, &user, &usdc_pool_address).unwrap();

    // No borrow position must exist still
    assert!(get_borrow_obligation(&contract_client, &user, &gold_pool_address).is_err());
    assert_eq!(expected_amount, obligation_tokens_from_shares);
}

#[test]
fn test_withdraw() {
    const LEVERAGE: u32 = 4;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

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
        &LEVERAGE_MULTIPLIER,
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
    contract_client.withdraw_from_leveraged(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &withdrawable_amount,
    );

    // Check obligation
    let approximate_borrowed_amount = ((LEVERAGE - 1) as i128) * DEFAULT_DEPOSIT_AMOUNT;
    let approximate_deposited_amount = swap::get_amount_out(
        &e,
        &gold_pool_address,
        &usdc_pool_address,
        (LEVERAGE as i128) * DEFAULT_DEPOSIT_AMOUNT,
    )
    .unwrap();

    let obligation_borrowed =
        get_obligation_borrowed(&contract_client, &user, &gold_pool_address).unwrap();

    let obligation_tokens_from_shares =
        get_obligation_tokens_from_shares(&e, &contract_client, &user, &usdc_pool_address).unwrap();

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
    const LEVERAGE: u32 = 4;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

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

    let borrowed_token_supply_before = contract_client
        .get_pool(&gold_pool_address)
        .total_supply()
        .unwrap();

    contract_client.deposit_with_leverage(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT),
        &LEVERAGE_MULTIPLIER,
    );

    let amount_out = swap::get_amount_out(
        &e,
        &gold_pool_address,
        &usdc_pool_address,
        DEFAULT_DEPOSIT_AMOUNT,
    )
    .unwrap();
    let withdrawable_amount = get_amount_scaled_down(amount_out, 2_00);

    // Withdrawing more than max available amount must succeed because of the inner cap
    contract_client.withdraw_from_leveraged(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &(10 * withdrawable_amount / 9),
    );

    let deposited_token_supply_after = contract_client
        .get_pool(&usdc_pool_address)
        .total_supply()
        .unwrap();
    let borrowed_token_supply_after = contract_client
        .get_pool(&gold_pool_address)
        .total_supply()
        .unwrap();

    assert_eq!(deposited_token_supply_after, 0); // Everything has been withdrawn
    assert!(borrowed_token_supply_after > borrowed_token_supply_before); // flash loan fees (TODO: Add a more rigorous check)
}

#[test]
fn test_withdraw_all_available_with_i128_max() {
    const LEVERAGE: u32 = 4;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

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
        &(DEFAULT_DEPOSIT_AMOUNT),
        &LEVERAGE_MULTIPLIER,
    );

    let borrowed_token_supply_before = contract_client
        .get_pool(&gold_pool_address)
        .total_supply()
        .unwrap();

    contract_client.withdraw_from_leveraged(
        &user,
        &usdc_pool_address,
        &gold_pool_address,
        &i128::MAX,
    );

    let deposited_token_supply_after = contract_client
        .get_pool(&usdc_pool_address)
        .total_supply()
        .unwrap();
    let borrowed_token_supply_after = contract_client
        .get_pool(&gold_pool_address)
        .total_supply()
        .unwrap();

    // Full withdraw took place
    assert_eq!(deposited_token_supply_after, 0); // Everything has been withdrawn
    assert!(borrowed_token_supply_after > borrowed_token_supply_before); // flash loan fees(TODO: Add a more rigorous check)
}

// #[test]
// fn custom_test1() {
//     let TestFixture {
//         e,
//         contract_client,
//         usdc_pool_address,
//         gold_pool_address,
//         users,
//         ..
//     } = TestFixture::new();

//     let me = users.get(0).unwrap();
//     let kyryl = users.get(1).unwrap();
//     let kyryl2 = users.get(2).unwrap();

//     // let deposit_amount: i128 = 50000000;

//     contract_client.deposit(&me, &gold_pool_address, &50000000);
//     contract_client.deposit(&me, &usdc_pool_address, &50000000);

//     contract_client.deposit(&kyryl, &usdc_pool_address, &12000000000);

//     contract_client.borrow(&kyryl, &gold_pool_address, &49900000);

//     wait_for(&e, 15);

//     contract_client.repay(&kyryl, &gold_pool_address, &49900000);

//     let kyryl_borrowed = get_borrow_obligation(&contract_client, &kyryl, &gold_pool_address)
//         .unwrap()
//         .borrowed;

//     wait_for(&e, 15);

//     contract_client.repay(&kyryl, &gold_pool_address, &kyryl_borrowed);

//     assert!(get_borrow_obligation(&contract_client, &kyryl, &gold_pool_address).is_err());

//     wait_for(&e, 11);

//     contract_client.withdraw(&kyryl, &usdc_pool_address, &12000000000);

//     assert!(get_deposit_obligation(&contract_client, &kyryl, &gold_pool_address).is_err());

//     wait_for(&e, 20);

//     contract_client.deposit(&kyryl, &usdc_pool_address, &11000000000);

//     wait_for(&e, 60 * 4); // 4 minutes

//     contract_client.deposit(&kyryl2, &gold_pool_address, &30000000000);

//     wait_for(&e, 30);

//     contract_client.borrow(&kyryl2, &usdc_pool_address, &1000000000);

//     let kyryl2_borrowed = get_borrow_obligation(&contract_client, &kyryl2, &usdc_pool_address)
//         .unwrap()
//         .borrowed;

//     assert_eq!(kyryl2_borrowed, 1000000000);

//     wait_for(&e, 30);

//     contract_client.borrow(&kyryl2, &usdc_pool_address, &3000000000);

//     let kyryl2_borrowed = get_borrow_obligation(&contract_client, &kyryl2, &usdc_pool_address)
//         .unwrap()
//         .borrowed;

//     assert_eq!(kyryl2_borrowed, 1000000000 + 3000000000);

//     let kyryl2_total_debt = get_borrow_obligation(&contract_client, &kyryl2, &usdc_pool_address)
//         .unwrap()
//         .total_debt()
//         .unwrap();

//     assert!(kyryl2_total_debt > 1000000000 + 3000000000);

//     // Okay, what do we have at this point....
//     // Everything seems to be working just fine, to be honest...
//     // Now, we better check everything else...

//     wait_for(&e, 80 * 60); // 80 minutes

//     let kyryl2_gold_deposit =
//         get_deposit_obligation(&contract_client, &kyryl2, &gold_pool_address).unwrap();

//     let kyryl2_gold_deposit_tokens =
//         get_obligation_tokens_from_shares(&e, &contract_client, &kyryl2, &gold_pool_address)
//             .unwrap();
//     // let kyryl2_gold_borrow =
//     //     get_borrow_obligation(&contract_client, &kyryl2, &gold_pool_address).unwrap();

//     // let kyryl2_usdc_deposit =
//     //     get_deposit_obligation(&contract_client, &kyryl2, &usdc_pool_address).unwrap();
//     let kyryl2_usdc_borrow =
//         get_borrow_obligation(&contract_client, &kyryl2, &usdc_pool_address).unwrap();

//     let gold_pool = contract_client.get_pool(&gold_pool_address);
//     let usdc_pool = contract_client.get_pool(&usdc_pool_address);

//     std::dbg!(
//         kyryl2_gold_deposit,
//         kyryl2_gold_deposit_tokens,
//         // kyryl2_gold_borrow,
//         // kyryl2_usdc_deposit,
//         kyryl2_usdc_borrow,
//         gold_pool,
//         usdc_pool,
//     );

//     contract_client.deposit_with_leverage(
//         &kyryl2,
//         &usdc_pool_address,
//         &gold_pool_address,
//         &500000000,
//         &590,
//     );

//     // So, what should happen here?

//     // kyryl2 must have an increase in borrow for ~ 2500000000
//     // and increase in deposited tokens for ~ 3000000000

//     let kyryl2_gold_borrowed_new =
//         get_borrow_obligation(&contract_client, &kyryl2, &gold_pool_address)
//             .unwrap()
//             .total_debt()
//             .unwrap();

//     let kyryl2_usdc_deposited_new =
//         get_obligation_tokens_from_shares(&e, &contract_client, &kyryl2, &usdc_pool_address)
//             .unwrap();

//     assert!(somewhat_equals(
//         kyryl2_gold_borrowed_new,
//         (500000000 * 49) / 10,
//         1
//     ));

//     assert!(somewhat_equals(
//         kyryl2_usdc_deposited_new,
//         (500000000 * 59) / 10,
//         1
//     ));

//     wait_for(&e, 45);

//     // тепер робимо собі repay gold'и...

//     contract_client.repay(&kyryl2, &gold_pool_address, &1450145000);

//     wait_for(&e, 36);

//     contract_client.repay(&kyryl2, &usdc_pool_address, &3000000000);

//     wait_for(&e, 40);

//     let b_gold_before = get_borrow_obligation(&contract_client, &kyryl2, &gold_pool_address)
//         .unwrap()
//         .total_debt()
//         .unwrap();
//     let d_usdc_before =
//         get_obligation_tokens_from_shares(&e, &contract_client, &kyryl2, &usdc_pool_address)
//             .unwrap();

//     std::dbg!(b_gold_before, d_usdc_before);

//     contract_client.deposit_with_leverage(
//         &kyryl2,
//         &usdc_pool_address,
//         &gold_pool_address,
//         &1000000000,
//         &450,
//     );

//     let b_gold_after = get_borrow_obligation(&contract_client, &kyryl2, &gold_pool_address)
//         .unwrap()
//         .total_debt()
//         .unwrap();
//     let d_usdc_after =
//         get_obligation_tokens_from_shares(&e, &contract_client, &kyryl2, &usdc_pool_address)
//             .unwrap();

//     std::dbg!(b_gold_after, d_usdc_after);
// }

#[allow(unused)]
fn wait_for(e: &Env, seconds: u64) {
    e.ledger().with_mut(|li| li.timestamp += seconds);
}

#[allow(unused)]
fn somewhat_equals(x: i128, y: i128, tolerance_percent: i128) -> bool {
    let bigger = if x > y { x } else { y };
    let tolerance = (bigger * tolerance_percent) / 100;

    (x - y).abs() <= tolerance
}
