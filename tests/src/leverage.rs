#![cfg(test)]

use market::{
    constants::{BPS_FACTOR, LEVERAGE_SCALE, MIN_LEVERAGE_MULTIPLIER},
    pool::{PoolConfig, PoolHealthConfig},
    swap,
};
use soroban_fixed_point_math::FixedPoint;

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, MCError, TestMarketFixture, assert_approx_eq_abs,
    get_amount_scaled_down, get_amount_scaled_up, get_borrow_position, get_deposit_position,
    get_multiply_pair_obligation_borrowed, get_multiply_pair_obligation_j_tokens_as_tokens,
    get_pool_total_borrowed, get_pool_total_supply,
};

// ---- Deposit with leverage ----

#[test]
fn test_deposit_zero_is_prohibited() {
    const LEVERAGE: u32 = 3; // x3 leverage
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let looper = &users[0];
    let liquidity_provider = &users[1];

    // Deposit into a different pool to make flash loans possible
    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    assert!(
        contract_client
            .try_deposit_with_leverage(
                looper,
                &gold_pool_address,
                &usdc_pool_address,
                &false,
                &0,
                &LEVERAGE_MULTIPLIER,
                &None,
            )
            .is_err()
    );
}

#[test]
fn test_deposit_with_invalid_leverage_multiplier() {
    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let looper = &users[0];

    assert_eq!(
        Err(Ok(MCError::InvalidLeverageInputs)),
        contract_client.try_deposit_with_leverage(
            looper,
            &gold_pool_address,
            &usdc_pool_address,
            &false,
            &DEFAULT_DEPOSIT_AMOUNT,
            &(MIN_LEVERAGE_MULTIPLIER - 1), // x(<1)
            &None
        )
    );

    let max_leverage_multiplier = contract_client
        .get_multiply_pair(&gold_pool_address, &usdc_pool_address)
        .max_leverage_multiplier;

    assert_eq!(
        Err(Ok(MCError::InvalidLeverageInputs)),
        contract_client.try_deposit_with_leverage(
            looper,
            &gold_pool_address,
            &usdc_pool_address,
            &false,
            &DEFAULT_DEPOSIT_AMOUNT,
            &(max_leverage_multiplier + 1),
            &None
        )
    );
}

// TODO: Add tests which check for supply and borrow limit constraints. This affects flash loans,
// right?
#[test]
fn test_deposit_with_no_leverage() {
    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let looper = &users[0];

    assert_eq!(
        contract_client.try_deposit_with_leverage(
            looper,
            &gold_pool_address,
            &usdc_pool_address,
            &false,
            &DEFAULT_DEPOSIT_AMOUNT,
            &100, // x1.0
            &None,
        ),
        Err(Ok(MCError::InvalidLeverageInputs))
    );
}

#[test]
fn test_deposit_with_unavailable_flash_loan_capacity() {
    const LEVERAGE: u32 = 3; // x3 leverage
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let looper = &users[0];

    assert_eq!(
        contract_client.try_deposit_with_leverage(
            looper,
            &gold_pool_address,
            &usdc_pool_address,
            &false,
            &1,
            &LEVERAGE_MULTIPLIER,
            &None
        ),
        Err(Ok(MCError::NotEnoughPoolFunds))
    );
}

#[test]
fn test_deposit_with_unhealthy_leverage() {
    const LEVERAGE: u32 = 4; // x4 leverage
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let looper = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    assert_eq!(
        contract_client.try_deposit_with_leverage(
            looper,
            &gold_pool_address,
            &usdc_pool_address,
            &false,
            &DEFAULT_DEPOSIT_AMOUNT,
            &LEVERAGE_MULTIPLIER,
            &None
        ),
        Err(Ok(MCError::InvalidLeverageInputs))
    );
}

#[test]
fn test_deposit_borrow_as_margin() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        e,
        contract_id,
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];
    let liquidity_provider = &users[1];
    let usdc_pool = contract_client.get_pool(&usdc_pool_address);

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
        &None,
    );

    // 'borrow' position is expected to have 'initial_amount * (leverage - 1)'
    let divisor = BPS_FACTOR + usdc_pool.config.fee_config.flash_loan_fee_bps as i128;
    let expected_borrowed_amount = DEFAULT_DEPOSIT_AMOUNT * (LEVERAGE as i128 - 1);
    // 'flash_borrowed_amount' is expected to equal 'expected_borrowed_amount' when repaid with flash loan fees.
    // So, we divide accordingly
    let flash_borrowed_amount =
        expected_borrowed_amount.fixed_div_floor(divisor, BPS_FACTOR).unwrap();
    // 'supply' position is expected to have 'amount_out(initial_amount * leverage)'
    let amount_in = flash_borrowed_amount.checked_add(DEFAULT_DEPOSIT_AMOUNT).unwrap();
    let amount_out = e.as_contract(&contract_id, || {
        swap::get_amount_out(&e, &usdc_pool_address, &gold_pool_address, amount_in).unwrap()
    });

    let expected_deposited_amount = amount_out;

    let obligation_j_tokens_as_tokens = get_multiply_pair_obligation_j_tokens_as_tokens(
        &e,
        &contract_client,
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    )
    .unwrap();

    assert_eq!(expected_deposited_amount, obligation_j_tokens_as_tokens);

    let gold_pool_total_supply =
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let usdc_pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert_eq!(expected_deposited_amount, gold_pool_total_supply);
    assert_eq!(expected_borrowed_amount, usdc_pool_total_borrowed);
}

#[test]
fn test_deposit_deposit_as_margin() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        e,
        contract_id,
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let looper = &users[0];
    let liquidity_provider = &users[1];
    let usdc_pool = contract_client.get_pool(&usdc_pool_address);

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &true,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
        &None,
    );

    // 'supply' position is expected to have 'initial_amount * leverage'
    let expected_deposited_amount = DEFAULT_DEPOSIT_AMOUNT * (LEVERAGE as i128);
    // 'borrow' position is expected to have 'amount_in(initial_amount * (leverage - 1))' +
    // flash_borrow_fees
    let amount_out = DEFAULT_DEPOSIT_AMOUNT * ((LEVERAGE - 1) as i128);
    let amount_in = e.as_contract(&contract_id, || {
        swap::get_amount_in(&e, &usdc_pool_address, &gold_pool_address, amount_out).unwrap()
    });
    let expected_borrowed_amount =
        get_amount_scaled_up(amount_in, usdc_pool.config.fee_config.flash_loan_fee_bps as i128);
    let obligation_j_tokens_as_tokens = get_multiply_pair_obligation_j_tokens_as_tokens(
        &e,
        &contract_client,
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    )
    .unwrap();

    assert_eq!(expected_deposited_amount, obligation_j_tokens_as_tokens);

    let gold_pool_total_supply =
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let usdc_pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert_eq!(expected_deposited_amount, gold_pool_total_supply);
    assert_approx_eq_abs(expected_borrowed_amount, usdc_pool_total_borrowed, 1);
}

#[test]
fn test_multiplied_deposits_are_isolated() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let looper = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
        &None,
    );

    assert_eq!(
        get_deposit_position(&contract_client, looper, &gold_pool_address),
        Err(MCError::ObligationDoesNotExist),
    );
    assert_eq!(
        get_borrow_position(&contract_client, looper, &usdc_pool_address),
        Err(MCError::ObligationDoesNotExist),
    );
}

// ---- Deleverage and Withdraw ----

#[test]
fn test_withdraw() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: BPS_FACTOR,
            ..Default::default()
        },
        ..Default::default()
    };
    let TestMarketFixture {
        e,
        contract_id,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let looper = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
        &None,
    );

    let amount_out = e.as_contract(&contract_id, || {
        swap::get_amount_out(&e, &usdc_pool_address, &gold_pool_address, DEFAULT_DEPOSIT_AMOUNT)
            .unwrap()
    });
    let withdrawable_amount = get_amount_scaled_down(amount_out, 10_00); // 90%

    // We must be able to withdraw the initial amount
    contract_client.withdraw_from_leveraged(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &withdrawable_amount,
        &None,
    );

    // Check obligation
    let approximate_borrowed_amount = ((LEVERAGE - 1) as i128) * DEFAULT_DEPOSIT_AMOUNT;
    let approximate_deposited_amount = e.as_contract(&contract_id, || {
        swap::get_amount_out(
            &e,
            &usdc_pool_address,
            &gold_pool_address,
            (LEVERAGE as i128) * DEFAULT_DEPOSIT_AMOUNT,
        )
        .unwrap()
    });

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
    let usdc_pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert_eq!(gold_pool_total_supply, obligation_j_tokens_as_tokens);
    assert_eq!(usdc_pool_total_borrowed, obligation_borrowed);
}

#[test]
fn test_withdraw_over_balance() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: BPS_FACTOR,
            ..Default::default()
        },
        ..Default::default()
    };
    let TestMarketFixture {
        e,
        contract_id,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let looper = &users[0];
    let liquidity_provider = &users[1];

    // Deposit into a different pool to make flash loans possible
    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
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
        &None,
    );

    let amount_out = e.as_contract(&contract_id, || {
        swap::get_amount_out(&e, &usdc_pool_address, &gold_pool_address, DEFAULT_DEPOSIT_AMOUNT)
            .unwrap()
    });
    let withdrawable_amount = get_amount_scaled_down(amount_out, 2_00);

    // Withdrawing more than max available amount must succeed because of the inner cap
    // TODO: With 90% utilization cap this behaves weird
    contract_client.withdraw_from_leveraged(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &(10 * withdrawable_amount / 9),
        &None,
    );

    assert_eq!(
        contract_client.try_get_multiply_pair_obligation(
            looper,
            &gold_pool_address,
            &usdc_pool_address
        ),
        Err(Ok(MCError::ObligationDoesNotExist))
    );

    let deposited_token_supply_after =
        contract_client.get_pool(&gold_pool_address).total_supply().unwrap();
    let borrowed_token_supply_after =
        get_pool_total_supply(&contract_client, &usdc_pool_address).unwrap();

    assert_eq!(deposited_token_supply_after, 0); // Everything has been withdrawn
    assert_eq!(borrowed_token_supply_after, borrowed_token_supply_before);
}

#[test]
fn test_withdraw_all_available_with_i128_max() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: BPS_FACTOR,
            ..Default::default()
        },
        ..Default::default()
    };
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new_with_pool_config(pool_config);
    let looper = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
        &None,
    );

    let borrowed_token_supply_before =
        get_pool_total_supply(&contract_client, &usdc_pool_address).unwrap();

    // TODO: With 90 % utilization cap this behaves weird. Check
    contract_client.withdraw_from_leveraged(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &i128::MAX,
        &None,
    );

    let deposited_token_supply_after =
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let borrowed_token_supply_after =
        get_pool_total_supply(&contract_client, &usdc_pool_address).unwrap();

    // Full withdraw took place
    assert_eq!(deposited_token_supply_after, 0); // Everything has been withdrawn
    assert!(borrowed_token_supply_after == borrowed_token_supply_before);
    // TODO: Add a more rigorous check
}

#[test]
fn test_withdraw_zero() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let looper = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
        &None,
    );

    assert!(
        contract_client
            .try_withdraw_from_leveraged(looper, &gold_pool_address, &usdc_pool_address, &0, &None,)
            .is_err()
    );
}

#[test]
fn test_withdraw_negative() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let looper = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
        &None,
    );

    assert_eq!(
        Err(Ok(MCError::NegativeInputAmount)),
        contract_client.try_withdraw_from_leveraged(
            looper,
            &gold_pool_address,
            &usdc_pool_address,
            &-1,
            &None
        )
    );
}
