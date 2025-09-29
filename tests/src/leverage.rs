#![cfg(test)]

use market::{
    constants::{LEVERAGE_SCALE, MIN_LEVERAGE_MULTIPLIER},
    swap,
};

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, MCError, TestMarketFixture, assert_approx_eq_abs,
    get_amount_scaled_down, get_amount_scaled_up, get_borrow_obligation, get_deposit_obligation,
    get_multiply_pair_obligation_borrowed, get_multiply_pair_obligation_d_tokens,
    get_multiply_pair_obligation_j_tokens_as_tokens, get_pool_total_borrowed,
    get_pool_total_supply,
};

// ---- Deposit with leverage ----

#[test]
fn test_deposit_zero() {
    const LEVERAGE: u32 = 3; // x3 leverage
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];
    let loan_provider = &users[1];

    // Deposit into a different pool to make flash loans possible
    contract_client.deposit(
        loan_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
    );

    let gold_pool_before = contract_client.get_pool(&gold_pool_address);
    let usdc_pool_before = contract_client.get_pool(&gold_pool_address);

    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &0,
        &LEVERAGE_MULTIPLIER,
    );

    let gold_pool_after = contract_client.get_pool(&gold_pool_address);
    let usdc_pool_after = contract_client.get_pool(&gold_pool_address);

    assert_eq!(gold_pool_before, gold_pool_after);
    assert_eq!(usdc_pool_before, usdc_pool_after);
}

#[test]
fn test_deposit_with_invalid_leverage_multiplier() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];

    assert_eq!(
        Err(Ok(MCError::InvalidLeverageMultiplier)),
        contract_client.try_deposit_with_leverage(
            looper,
            &gold_pool_address,
            &usdc_pool_address,
            &false,
            &DEFAULT_DEPOSIT_AMOUNT,
            &(MIN_LEVERAGE_MULTIPLIER - 1), // x(<1)
        )
    );

    let max_leverage_multiplier = contract_client
        .get_multiply_pair(&gold_pool_address, &usdc_pool_address)
        .max_leverage_multiplier;

    assert_eq!(
        Err(Ok(MCError::InvalidLeverageMultiplier)),
        contract_client.try_deposit_with_leverage(
            looper,
            &gold_pool_address,
            &usdc_pool_address,
            &false,
            &DEFAULT_DEPOSIT_AMOUNT,
            &(max_leverage_multiplier + 1),
        )
    );
}

// TODO: Add tests which check for supply and borrow limit constraints. This affects flash loans,
// right?
#[test]
fn test_deposit_with_no_leverage() {
    let TestMarketFixture {
        e,
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];

    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &MIN_LEVERAGE_MULTIPLIER, // x1.0
    );

    // Check if this is equivalent to a plain deposit
    let obligation_j_tokens_as_tokens = get_multiply_pair_obligation_j_tokens_as_tokens(
        &e,
        &contract_client,
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    )
    .unwrap();

    let amount_out = swap::get_amount_out(
        &e,
        &gold_pool_address,
        &usdc_pool_address,
        DEFAULT_DEPOSIT_AMOUNT,
    )
    .unwrap();

    // TODO: Doesn't this account for fees?
    assert_eq!(amount_out, obligation_j_tokens_as_tokens);

    let obligation_borrowed = get_multiply_pair_obligation_borrowed(
        &contract_client,
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    )
    .unwrap();
    let obligation_d_tokens = get_multiply_pair_obligation_d_tokens(
        &contract_client,
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    )
    .unwrap();

    assert_eq!(obligation_borrowed, 0);
    assert_eq!(obligation_d_tokens, 0);
}

#[test]
fn test_deposit_with_unavailable_flash_loan_capacity() {
    const LEVERAGE: u32 = 3; // x3 leverage
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];

    assert_eq!(
        contract_client.try_deposit_with_leverage(
            looper,
            &gold_pool_address,
            &usdc_pool_address,
            &false,
            &1,
            &LEVERAGE_MULTIPLIER,
        ),
        Err(Ok(MCError::NotEnoughPoolFunds))
    );
}

#[test]
fn test_deposit_with_unhealthy_leverage() {
    const LEVERAGE: u32 = 4; // x4 leverage
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(
        loan_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
    );

    assert_eq!(
        contract_client.try_deposit_with_leverage(
            looper,
            &gold_pool_address,
            &usdc_pool_address,
            &false,
            &DEFAULT_DEPOSIT_AMOUNT,
            &LEVERAGE_MULTIPLIER,
        ),
        Err(Ok(MCError::InvalidLeverageMultiplier))
    );
}

#[test]
fn test_deposit_borrow_as_margin() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        e,
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];
    let loan_provider = &users[1];
    let usdc_pool = contract_client.get_pool(&usdc_pool_address);

    contract_client.deposit(
        loan_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
    );

    // 'borrow' position is expected to have 'initial_amount * (leverage - 1) + flash_borrow_fees'
    let flash_borrowed_amount = DEFAULT_DEPOSIT_AMOUNT * (LEVERAGE as i128 - 1);
    let expected_borrowed_amount = get_amount_scaled_up(
        flash_borrowed_amount,
        usdc_pool.fee_config.flash_loan_fee_bps as i128,
    ); // TODO: This better be checked once more
    // 'supply' position is expected to have 'amount_out(initial_amount * leverage)'
    let amount_in = DEFAULT_DEPOSIT_AMOUNT * (LEVERAGE as i128);
    let amount_out =
        swap::get_amount_out(&e, &usdc_pool_address, &gold_pool_address, amount_in).unwrap();
    let expected_deposited_amount = amount_out;

    let obligation_j_tokens_as_tokens = get_multiply_pair_obligation_j_tokens_as_tokens(
        &e,
        &contract_client,
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    )
    .unwrap();
    let obligation_borrowed = get_multiply_pair_obligation_borrowed(
        &contract_client,
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    )
    .unwrap();

    assert_eq!(expected_borrowed_amount, obligation_borrowed);
    assert_eq!(expected_deposited_amount, obligation_j_tokens_as_tokens);

    let gold_pool_total_supply =
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let usdc_pool_total_borrowed =
        get_pool_total_borrowed(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(expected_deposited_amount, gold_pool_total_supply);
    assert_eq!(expected_borrowed_amount, usdc_pool_total_borrowed);
}

#[test]
fn test_deposit_deposit_as_margin() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        e,
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];
    let loan_provider = &users[1];
    let usdc_pool = contract_client.get_pool(&usdc_pool_address);

    contract_client.deposit(
        loan_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &true,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
    );

    // 'supply' position is expected to have 'initial_amount * leverage'
    let expected_deposited_amount = DEFAULT_DEPOSIT_AMOUNT * (LEVERAGE as i128);
    // 'borrow' position is expected to have 'amount_in(initial_amount * (leverage - 1))' +
    // flash_borrow_fees
    let amount_out = DEFAULT_DEPOSIT_AMOUNT * ((LEVERAGE - 1) as i128);
    let amount_in =
        swap::get_amount_in(&e, &usdc_pool_address, &gold_pool_address, amount_out).unwrap();
    let expected_borrowed_amount =
        get_amount_scaled_up(amount_in, usdc_pool.fee_config.flash_loan_fee_bps as i128);
    let obligation_j_tokens_as_tokens = get_multiply_pair_obligation_j_tokens_as_tokens(
        &e,
        &contract_client,
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    )
    .unwrap();
    let obligation_borrowed = get_multiply_pair_obligation_borrowed(
        &contract_client,
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    )
    .unwrap();

    assert_approx_eq_abs(expected_borrowed_amount, obligation_borrowed, 1);
    assert_eq!(expected_deposited_amount, obligation_j_tokens_as_tokens);

    let gold_pool_total_supply =
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let usdc_pool_total_borrowed =
        get_pool_total_borrowed(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(expected_deposited_amount, gold_pool_total_supply);
    assert_approx_eq_abs(expected_borrowed_amount, usdc_pool_total_borrowed, 1);
}

#[test]
fn test_multiplied_deposits_are_isolated() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(
        loan_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
    );

    assert_eq!(
        get_deposit_obligation(&contract_client, looper, &gold_pool_address),
        Err(MCError::ObligationDoesNotExist),
    );
    assert_eq!(
        get_borrow_obligation(&contract_client, looper, &usdc_pool_address),
        Err(MCError::ObligationDoesNotExist),
    );
}

// ---- Deleverage and Withdraw ----

#[test]
fn test_withdraw() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        e,
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(
        loan_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
    );

    let amount_out = swap::get_amount_out(
        &e,
        &usdc_pool_address,
        &gold_pool_address,
        DEFAULT_DEPOSIT_AMOUNT,
    )
    .unwrap();
    let withdrawable_amount = get_amount_scaled_down(amount_out, 10_00); // 90%

    // We must be able to withdraw the initial amount
    contract_client.withdraw_from_leveraged(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &withdrawable_amount,
    );

    // Check obligation
    let approximate_borrowed_amount = ((LEVERAGE - 1) as i128) * DEFAULT_DEPOSIT_AMOUNT;
    let approximate_deposited_amount = swap::get_amount_out(
        &e,
        &usdc_pool_address,
        &gold_pool_address,
        (LEVERAGE as i128) * DEFAULT_DEPOSIT_AMOUNT,
    )
    .unwrap();

    let obligation_borrowed = get_multiply_pair_obligation_borrowed(
        &contract_client,
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    )
    .unwrap();
    let obligation_j_tokens_as_tokens = get_multiply_pair_obligation_j_tokens_as_tokens(
        &e,
        &contract_client,
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    )
    .unwrap();

    // Less than 10% of originally deposited value must be left
    let expected_deposit_left_upper_bound_amount =
        get_amount_scaled_down(approximate_deposited_amount, 90_00);
    assert!(obligation_j_tokens_as_tokens < expected_deposit_left_upper_bound_amount);

    // More than 5 % of original deposited value must be left
    let expected_left_lower_bound_amount =
        get_amount_scaled_down(approximate_deposited_amount, 95_00);
    assert!(obligation_j_tokens_as_tokens > expected_left_lower_bound_amount);

    // Less than 10% of borrowed value must be left
    let expected_borrow_left_upper_bound_amount =
        get_amount_scaled_down(approximate_borrowed_amount, 90_00);
    assert!(obligation_borrowed < expected_borrow_left_upper_bound_amount);

    // More than 5 % of borrowed value must be left
    let expected_borrow_left_lower_bound_amount =
        get_amount_scaled_down(approximate_borrowed_amount, 95_00);
    assert!(obligation_borrowed > expected_borrow_left_lower_bound_amount);

    // Check pools
    let gold_pool_total_supply =
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let usdc_pool_total_borrowed =
        get_pool_total_borrowed(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(gold_pool_total_supply, obligation_j_tokens_as_tokens);
    assert_eq!(usdc_pool_total_borrowed, obligation_borrowed);
}

#[test]
fn test_withdraw_over_balance() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        e,
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];
    let loan_provider = &users[1];

    // Deposit into a different pool to make flash loans possible
    contract_client.deposit(
        loan_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
    );

    let borrowed_token_supply_before =
        get_pool_total_supply(&contract_client, &usdc_pool_address).unwrap();

    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
    );

    let amount_out = swap::get_amount_out(
        &e,
        &usdc_pool_address,
        &gold_pool_address,
        DEFAULT_DEPOSIT_AMOUNT,
    )
    .unwrap();
    let withdrawable_amount = get_amount_scaled_down(amount_out, 2_00);

    // Withdrawing more than max available amount must succeed because of the inner cap
    contract_client.withdraw_from_leveraged(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &(10 * withdrawable_amount / 9),
    );

    let deposited_token_supply_after = contract_client
        .get_pool(&gold_pool_address)
        .total_supply()
        .unwrap();
    let borrowed_token_supply_after =
        get_pool_total_supply(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(deposited_token_supply_after, 0); // Everything has been withdrawn
    assert!(borrowed_token_supply_after > borrowed_token_supply_before); // flash loan fees (TODO:
    // Add a more rigorous
    // check)
}

#[test]
fn test_withdraw_all_available_with_i128_max() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(
        loan_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
    );

    let borrowed_token_supply_before =
        get_pool_total_supply(&contract_client, &usdc_pool_address).unwrap();

    contract_client.withdraw_from_leveraged(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &i128::MAX,
    );

    let deposited_token_supply_after =
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let borrowed_token_supply_after =
        get_pool_total_supply(&contract_client, &usdc_pool_address).unwrap();

    // Full withdraw took place
    assert_eq!(deposited_token_supply_after, 0); // Everything has been withdrawn
    assert!(borrowed_token_supply_after > borrowed_token_supply_before); // flash loan fees(TODO:
    // Add a more rigorous
    // check)
}

#[test]
fn test_withdraw_zero() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(
        loan_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
    );

    let gold_pool_before = contract_client.get_pool(&gold_pool_address);
    let usdc_pool_before = contract_client.get_pool(&gold_pool_address);
    let obligation_before = contract_client.get_multiply_pair_obligation(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    );

    contract_client.withdraw_from_leveraged(looper, &gold_pool_address, &usdc_pool_address, &0);

    let gold_pool_after = contract_client.get_pool(&gold_pool_address);
    let usdc_pool_after = contract_client.get_pool(&gold_pool_address);
    let obligation_after = contract_client.get_multiply_pair_obligation(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    );

    assert_eq!(gold_pool_before, gold_pool_after);
    assert_eq!(usdc_pool_before, usdc_pool_after);
    assert_eq!(obligation_before, obligation_after);
}

#[test]
fn test_withdraw_negative() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(
        loan_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
    );

    assert_eq!(
        Err(Ok(MCError::NegativeAmount)),
        contract_client.try_withdraw_from_leveraged(
            looper,
            &gold_pool_address,
            &usdc_pool_address,
            &-1
        )
    );
}
