#![cfg(test)]

use market::{
    constants::{BPS_FACTOR, DEFAULT_UTILIZATION_RATIO_LIMIT_BPS},
    obligation::{ComputedFees, compute_fees},
    pool::{PoolConfig, PoolFeeConfig},
};
use soroban_fixed_point_math::FixedPoint;

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture,
    get_obligation_collateral, get_obligation_d_tokens_as_tokens,
    get_obligation_j_tokens_as_tokens, get_pool_accumulated_host_fees,
    get_pool_accumulated_market_fees, get_pool_fee_config,
};

// -- Default Fees(only for borrow and flash loan(the latter tested in 'flash_loan_taker_mock')) --

#[test]
fn test_borrow_fee() {
    let TestMarketFixture {
        e,
        contract_id,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    let market_fees_before = get_pool_accumulated_market_fees(&contract_client, &usdc_pool_address);
    let host_fees_before = get_pool_accumulated_host_fees(&contract_client, &usdc_pool_address);
    assert_eq!(market_fees_before, 0);
    assert_eq!(host_fees_before, 0);

    let pool_balance_before = usdc_token_client.balance(&contract_id);
    let borrower_balance_before = usdc_token_client.balance(borrower);

    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_balance_after = usdc_token_client.balance(&contract_id);
    let borrower_balance_after = usdc_token_client.balance(borrower);

    let pool_balance_diff = pool_balance_before.checked_sub(pool_balance_after).unwrap();
    let borrower_balance_diff =
        borrower_balance_after.checked_sub(borrower_balance_before).unwrap();

    let pool_market_fees_after =
        get_pool_accumulated_market_fees(&contract_client, &usdc_pool_address);
    let pool_host_fees_after = get_pool_accumulated_host_fees(&contract_client, &usdc_pool_address);

    let borrower_debt_after =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    let PoolFeeConfig { borrow_fee_bps, host_fee_bps, .. } =
        get_pool_fee_config(&contract_client, &usdc_pool_address);
    let ComputedFees { fee_sum, market_fee, host_fee } =
        compute_fees(DEFAULT_DEPOSIT_AMOUNT, borrow_fee_bps, host_fee_bps).unwrap();

    let expected_borrower_balance_diff = DEFAULT_DEPOSIT_AMOUNT.checked_sub(fee_sum).unwrap();
    let expected_pool_balance_diff = expected_borrower_balance_diff;
    let expected_market_fees_diff = market_fee;
    let expected_host_fees_diff = host_fee;

    assert_eq!(borrower_balance_diff, expected_borrower_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);

    assert_eq!(pool_market_fees_after, expected_market_fees_diff);
    assert_eq!(pool_host_fees_after, expected_host_fees_diff);

    assert_eq!(borrower_debt_after, DEFAULT_DEPOSIT_AMOUNT);
}

// -- Non-default Fees --

#[test]
fn test_deposit_fee() {
    const DEPOSIT_FEE_BPS: u32 = 1_000; // 10%

    let pool_config = PoolConfig {
        fee_config: PoolFeeConfig { deposit_fee_bps: DEPOSIT_FEE_BPS, ..Default::default() },
        ..Default::default()
    };
    let TestMarketFixture {
        e,
        contract_id,
        contract_client,
        gold_pool_address,
        users,
        gold_token_client,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let creditor = &users[0];

    let pool_balance_before = gold_token_client.balance(&contract_id);
    let creditor_balance_before = gold_token_client.balance(creditor);

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_balance_after = gold_token_client.balance(&contract_id);
    let creditor_balance_after = gold_token_client.balance(creditor);

    let pool_balance_diff = pool_balance_after.checked_sub(pool_balance_before).unwrap();
    let creditor_balance_diff =
        creditor_balance_before.checked_sub(creditor_balance_after).unwrap();

    let pool_market_fees_after =
        get_pool_accumulated_market_fees(&contract_client, &gold_pool_address);
    let pool_host_fees_after = get_pool_accumulated_host_fees(&contract_client, &gold_pool_address);

    let creditor_deposit_after =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address)
            .unwrap();

    let PoolFeeConfig { deposit_fee_bps, host_fee_bps, .. } =
        get_pool_fee_config(&contract_client, &gold_pool_address);
    let ComputedFees { fee_sum, market_fee, host_fee } =
        compute_fees(DEFAULT_DEPOSIT_AMOUNT, deposit_fee_bps, host_fee_bps).unwrap();

    let expected_creditor_balance_diff = DEFAULT_DEPOSIT_AMOUNT;
    let expected_pool_balance_diff = expected_creditor_balance_diff;
    let expected_market_fees_diff = market_fee;
    let expected_host_fees_diff = host_fee;
    let expected_creditor_deposit_after = DEFAULT_DEPOSIT_AMOUNT.checked_sub(fee_sum).unwrap();

    assert_eq!(creditor_balance_diff, expected_creditor_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);

    assert_eq!(pool_market_fees_after, expected_market_fees_diff);
    assert_eq!(pool_host_fees_after, expected_host_fees_diff);

    assert_eq!(creditor_deposit_after, expected_creditor_deposit_after);
}

#[test]
fn test_add_collateral_fee() {
    const ADD_COLLATERAL_FEE_BPS: u32 = 1_500; // 15%

    let pool_config = PoolConfig {
        fee_config: PoolFeeConfig {
            add_collateral_fee_bps: ADD_COLLATERAL_FEE_BPS,
            ..Default::default()
        },
        ..Default::default()
    };
    let TestMarketFixture {
        contract_id,
        contract_client,
        gold_pool_address,
        users,
        gold_token_client,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let collateral_adder = &users[0];

    let pool_balance_before = gold_token_client.balance(&contract_id);
    let collateral_adder_balance_before = gold_token_client.balance(collateral_adder);

    contract_client.add_collateral(
        collateral_adder,
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
    );

    let pool_balance_after = gold_token_client.balance(&contract_id);
    let collateral_adder_balance_after = gold_token_client.balance(collateral_adder);

    let pool_balance_diff = pool_balance_after.checked_sub(pool_balance_before).unwrap();
    let collateral_adder_balance_diff =
        collateral_adder_balance_before.checked_sub(collateral_adder_balance_after).unwrap();

    let pool_market_fees_after =
        get_pool_accumulated_market_fees(&contract_client, &gold_pool_address);
    let pool_host_fees_after = get_pool_accumulated_host_fees(&contract_client, &gold_pool_address);

    let collateral_adder_collateral_after =
        get_obligation_collateral(&contract_client, collateral_adder, &gold_pool_address).unwrap();

    let PoolFeeConfig { add_collateral_fee_bps, host_fee_bps, .. } =
        get_pool_fee_config(&contract_client, &gold_pool_address);
    let ComputedFees { fee_sum, market_fee, host_fee } =
        compute_fees(DEFAULT_COLLATERAL_AMOUNT, add_collateral_fee_bps, host_fee_bps).unwrap();

    let expected_collateral_adder_balance_diff = DEFAULT_COLLATERAL_AMOUNT;
    let expected_pool_balance_diff = expected_collateral_adder_balance_diff;
    let expected_market_fees_diff = market_fee;
    let expected_host_fees_diff = host_fee;
    let expected_collateral_adder_collateral_after =
        DEFAULT_COLLATERAL_AMOUNT.checked_sub(fee_sum).unwrap();

    assert_eq!(collateral_adder_balance_diff, expected_collateral_adder_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);

    assert_eq!(pool_market_fees_after, expected_market_fees_diff);
    assert_eq!(pool_host_fees_after, expected_host_fees_diff);

    assert_eq!(collateral_adder_collateral_after, expected_collateral_adder_collateral_after);
}

#[test]
fn test_remove_collateral_fee() {
    const REMOVE_COLLATERAL_FEE_BPS: u32 = 500; // 5%

    let pool_config = PoolConfig {
        fee_config: PoolFeeConfig {
            remove_collateral_fee_bps: REMOVE_COLLATERAL_FEE_BPS,
            ..Default::default()
        },
        ..Default::default()
    };
    let TestMarketFixture {
        contract_id,
        contract_client,
        gold_pool_address,
        users,
        gold_token_client,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let collateral_adder = &users[0];

    contract_client.add_collateral(
        collateral_adder,
        &gold_pool_address,
        &(2 * DEFAULT_COLLATERAL_AMOUNT),
    );

    let pool_balance_before = gold_token_client.balance(&contract_id);
    let collateral_adder_balance_before = gold_token_client.balance(collateral_adder);
    let collateral_adder_collateral_before =
        get_obligation_collateral(&contract_client, collateral_adder, &gold_pool_address).unwrap();

    let pool_market_fees_before =
        get_pool_accumulated_market_fees(&contract_client, &gold_pool_address);
    let pool_host_fees_before =
        get_pool_accumulated_host_fees(&contract_client, &gold_pool_address);

    contract_client.remove_collateral(
        collateral_adder,
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
    );

    let pool_balance_after = gold_token_client.balance(&contract_id);
    let collateral_adder_balance_after = gold_token_client.balance(collateral_adder);
    let collateral_adder_collateral_after =
        get_obligation_collateral(&contract_client, collateral_adder, &gold_pool_address).unwrap();

    let pool_balance_diff = pool_balance_before.checked_sub(pool_balance_after).unwrap();
    let collateral_adder_balance_diff =
        collateral_adder_balance_after.checked_sub(collateral_adder_balance_before).unwrap();
    let collateral_adder_collateral_diff =
        collateral_adder_collateral_before.checked_sub(collateral_adder_collateral_after).unwrap();

    let pool_market_fees_after =
        get_pool_accumulated_market_fees(&contract_client, &gold_pool_address);
    let pool_host_fees_after = get_pool_accumulated_host_fees(&contract_client, &gold_pool_address);

    let pool_market_fees_diff =
        pool_market_fees_after.checked_sub(pool_market_fees_before).unwrap();
    let pool_host_fees_diff = pool_host_fees_after.checked_sub(pool_host_fees_before).unwrap();

    let PoolFeeConfig { remove_collateral_fee_bps, host_fee_bps, .. } =
        get_pool_fee_config(&contract_client, &gold_pool_address);
    let ComputedFees { fee_sum, market_fee, host_fee } =
        compute_fees(DEFAULT_COLLATERAL_AMOUNT, remove_collateral_fee_bps, host_fee_bps).unwrap();

    let expected_collateral_adder_balance_diff =
        DEFAULT_COLLATERAL_AMOUNT.checked_sub(fee_sum).unwrap();
    let expected_pool_balance_diff = expected_collateral_adder_balance_diff;
    let expected_market_fees_diff = market_fee;
    let expected_host_fees_diff = host_fee;
    let expected_collateral_adder_collateral_diff = DEFAULT_COLLATERAL_AMOUNT;

    assert_eq!(collateral_adder_balance_diff, expected_collateral_adder_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);

    assert_eq!(pool_market_fees_diff, expected_market_fees_diff);
    assert_eq!(pool_host_fees_diff, expected_host_fees_diff);

    assert_eq!(collateral_adder_collateral_diff, expected_collateral_adder_collateral_diff);
}

// TODO: Add cap checks
// TODO: Add a test that verifies the constant `market` and `host` fees availability regardless of
// `total_available` on pool(contrary to `reserve` fees)

#[test]
fn test_withdraw_fee() {
    const WITHDRAW_FEE_BPS: u32 = 500; // 5%

    let pool_config = PoolConfig {
        fee_config: PoolFeeConfig { withdraw_fee_bps: WITHDRAW_FEE_BPS, ..Default::default() },
        ..Default::default()
    };
    let TestMarketFixture {
        e,
        contract_id,
        contract_client,
        gold_pool_address,
        users,
        gold_token_client,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let creditor = &users[0];

    contract_client.deposit(creditor, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    let pool_balance_before = gold_token_client.balance(&contract_id);
    let creditor_balance_before = gold_token_client.balance(creditor);
    let creditor_deposit_before =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address)
            .unwrap();

    let pool_market_fees_before =
        get_pool_accumulated_market_fees(&contract_client, &gold_pool_address);
    let pool_host_fees_before =
        get_pool_accumulated_host_fees(&contract_client, &gold_pool_address);

    contract_client.withdraw(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_balance_after = gold_token_client.balance(&contract_id);
    let creditor_balance_after = gold_token_client.balance(creditor);
    let creditor_deposit_after =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address)
            .unwrap();

    let pool_balance_diff = pool_balance_before.checked_sub(pool_balance_after).unwrap();
    let creditor_balance_diff =
        creditor_balance_after.checked_sub(creditor_balance_before).unwrap();
    let creditor_deposit_diff =
        creditor_deposit_before.checked_sub(creditor_deposit_after).unwrap();

    let pool_market_fees_after =
        get_pool_accumulated_market_fees(&contract_client, &gold_pool_address);
    let pool_host_fees_after = get_pool_accumulated_host_fees(&contract_client, &gold_pool_address);

    let pool_market_fees_diff =
        pool_market_fees_after.checked_sub(pool_market_fees_before).unwrap();
    let pool_host_fees_diff = pool_host_fees_after.checked_sub(pool_host_fees_before).unwrap();

    let PoolFeeConfig { withdraw_fee_bps, host_fee_bps, .. } =
        get_pool_fee_config(&contract_client, &gold_pool_address);
    let ComputedFees { fee_sum, market_fee, host_fee } =
        compute_fees(DEFAULT_DEPOSIT_AMOUNT, withdraw_fee_bps, host_fee_bps).unwrap();

    let expected_creditor_balance_diff = DEFAULT_DEPOSIT_AMOUNT.checked_sub(fee_sum).unwrap();
    let expected_pool_balance_diff = expected_creditor_balance_diff;
    let expected_market_fees_diff = market_fee;
    let expected_host_fees_diff = host_fee;
    let expected_creditor_deposit_diff = DEFAULT_DEPOSIT_AMOUNT;

    assert_eq!(creditor_balance_diff, expected_creditor_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);

    assert_eq!(pool_market_fees_diff, expected_market_fees_diff);
    assert_eq!(pool_host_fees_diff, expected_host_fees_diff);

    assert_eq!(creditor_deposit_diff, expected_creditor_deposit_diff);
}

#[test]
fn test_withdraw_scarcity_fee() {
    const WITHDRAW_FEE_BPS: u32 = 500; // 5%

    let pool_config = PoolConfig {
        fee_config: PoolFeeConfig {
            withdraw_fee_bps: WITHDRAW_FEE_BPS,
            borrow_fee_bps: 0,
            ..Default::default()
        },
        ..Default::default()
    };
    let TestMarketFixture {
        e,
        contract_id,
        contract_client,
        gold_pool_address,
        users,
        gold_token_client,
        usdc_pool_address,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let creditor = &users[0];
    let borrower = &users[1];

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.add_collateral(borrower, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    // - Borrow up to utilization ratio cap -

    let borrow_amount: i128 = DEFAULT_DEPOSIT_AMOUNT
        .fixed_mul_ceil(DEFAULT_UTILIZATION_RATIO_LIMIT_BPS, BPS_FACTOR)
        .unwrap();
    let withdraw_amount = DEFAULT_DEPOSIT_AMOUNT.checked_sub(borrow_amount).unwrap();

    contract_client.borrow(borrower, &gold_pool_address, &borrow_amount);

    // - Withdraw rest and check the fees -

    let pool_balance_before = gold_token_client.balance(&contract_id);
    let creditor_balance_before = gold_token_client.balance(creditor);
    let creditor_deposit_before =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address)
            .unwrap();

    let pool_market_fees_before =
        get_pool_accumulated_market_fees(&contract_client, &gold_pool_address);
    let pool_host_fees_before =
        get_pool_accumulated_host_fees(&contract_client, &gold_pool_address);

    contract_client.withdraw(creditor, &gold_pool_address, &withdraw_amount);

    let pool_balance_after = gold_token_client.balance(&contract_id);
    let creditor_balance_after = gold_token_client.balance(creditor);
    let creditor_deposit_after =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address)
            .unwrap();

    let pool_balance_diff = pool_balance_before.checked_sub(pool_balance_after).unwrap();
    let creditor_balance_diff =
        creditor_balance_after.checked_sub(creditor_balance_before).unwrap();
    let creditor_deposit_diff =
        creditor_deposit_before.checked_sub(creditor_deposit_after).unwrap();

    let pool_market_fees_after =
        get_pool_accumulated_market_fees(&contract_client, &gold_pool_address);
    let pool_host_fees_after = get_pool_accumulated_host_fees(&contract_client, &gold_pool_address);

    let pool_market_fees_diff =
        pool_market_fees_after.checked_sub(pool_market_fees_before).unwrap();
    let pool_host_fees_diff = pool_host_fees_after.checked_sub(pool_host_fees_before).unwrap();

    let PoolFeeConfig { withdraw_fee_bps, withdraw_scarcity_fee_scalar_p, host_fee_bps, .. } =
        get_pool_fee_config(&contract_client, &gold_pool_address);

    let withdraw_scarcity_fee_bps = {
        let bps_diff = BPS_FACTOR.checked_sub(DEFAULT_UTILIZATION_RATIO_LIMIT_BPS).unwrap();

        bps_diff.fixed_mul_ceil(withdraw_scarcity_fee_scalar_p as i128, 100).unwrap()
    } as u32;
    let withdraw_fee_bps = withdraw_fee_bps.checked_add(withdraw_scarcity_fee_bps).unwrap();

    let ComputedFees { fee_sum, market_fee, host_fee } =
        compute_fees(withdraw_amount, withdraw_fee_bps, host_fee_bps).unwrap();

    let expected_creditor_balance_diff = withdraw_amount.checked_sub(fee_sum).unwrap();
    let expected_pool_balance_diff = expected_creditor_balance_diff;
    let expected_market_fees_diff = market_fee;
    let expected_host_fees_diff = host_fee;
    let expected_creditor_deposit_diff = withdraw_amount;

    assert_eq!(creditor_balance_diff, expected_creditor_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);

    assert_eq!(pool_market_fees_diff, expected_market_fees_diff);
    assert_eq!(pool_host_fees_diff, expected_host_fees_diff);

    assert_eq!(creditor_deposit_diff, expected_creditor_deposit_diff);
}

#[test]
fn test_repay_fee() {
    const REPAY_FEE_BPS: u32 = 300; // 3%

    let pool_config = PoolConfig {
        fee_config: PoolFeeConfig { repay_fee_bps: REPAY_FEE_BPS, ..Default::default() },
        ..Default::default()
    };
    let TestMarketFixture {
        e,
        contract_id,
        contract_client,
        gold_pool_address,
        users,
        usdc_token_client,
        usdc_pool_address,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(4 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &(3 * DEFAULT_DEPOSIT_AMOUNT));

    contract_client.borrow(borrower, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    let pool_market_fees_before =
        get_pool_accumulated_market_fees(&contract_client, &usdc_pool_address);
    let pool_host_fees_before =
        get_pool_accumulated_host_fees(&contract_client, &usdc_pool_address);

    let pool_balance_before = usdc_token_client.balance(&contract_id);
    let borrower_balance_before = usdc_token_client.balance(borrower);
    let borrower_debt_before =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    contract_client.repay(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_balance_after = usdc_token_client.balance(&contract_id);
    let borrower_balance_after = usdc_token_client.balance(borrower);
    let borrower_debt_after =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    let pool_balance_diff = pool_balance_after.checked_sub(pool_balance_before).unwrap();
    let borrower_balance_diff =
        borrower_balance_before.checked_sub(borrower_balance_after).unwrap();
    let borrower_debt_diff = borrower_debt_before.checked_sub(borrower_debt_after).unwrap();

    let pool_market_fees_after =
        get_pool_accumulated_market_fees(&contract_client, &usdc_pool_address);
    let pool_host_fees_after = get_pool_accumulated_host_fees(&contract_client, &usdc_pool_address);

    let pool_market_fees_diff =
        pool_market_fees_after.checked_sub(pool_market_fees_before).unwrap();
    let pool_host_fees_diff = pool_host_fees_after.checked_sub(pool_host_fees_before).unwrap();

    let PoolFeeConfig { repay_fee_bps, host_fee_bps, .. } =
        get_pool_fee_config(&contract_client, &usdc_pool_address);
    let ComputedFees { fee_sum, market_fee, host_fee } =
        compute_fees(DEFAULT_DEPOSIT_AMOUNT, repay_fee_bps, host_fee_bps).unwrap();

    let expected_borrower_balance_diff = DEFAULT_DEPOSIT_AMOUNT;
    let expected_pool_balance_diff = expected_borrower_balance_diff;
    let expected_market_fees_diff = market_fee;
    let expected_host_fees_diff = host_fee;
    let expected_borrower_debt_diff = DEFAULT_DEPOSIT_AMOUNT.checked_sub(fee_sum).unwrap();

    assert_eq!(borrower_balance_diff, expected_borrower_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);

    assert_eq!(pool_market_fees_diff, expected_market_fees_diff);
    assert_eq!(pool_host_fees_diff, expected_host_fees_diff);

    assert_eq!(borrower_debt_diff, expected_borrower_debt_diff);
}

#[test]
fn redeem_market_fees() {
    let TestMarketFixture {
        contract_id,
        contract_client,
        contract_admin,
        usdc_pool_address,
        gold_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_market_fees = get_pool_accumulated_market_fees(&contract_client, &usdc_pool_address);
    dbg!(pool_market_fees);

    let contract_admin_balance_before = usdc_token_client.balance(&contract_admin);
    let pool_balance_before = usdc_token_client.balance(&contract_id);

    contract_client.redeem_accumulated_market_fees(
        &contract_admin,
        &usdc_pool_address,
        &pool_market_fees,
    );

    let contract_admin_balance_after = usdc_token_client.balance(&contract_admin);
    let contract_admin_balance_diff =
        contract_admin_balance_after.checked_sub(contract_admin_balance_before).unwrap();

    let pool_balance_after = usdc_token_client.balance(&contract_id);
    let pool_balance_diff = pool_balance_before.checked_sub(pool_balance_after).unwrap();

    let pool_market_fees_new =
        get_pool_accumulated_market_fees(&contract_client, &usdc_pool_address);

    assert_eq!(contract_admin_balance_diff, pool_market_fees);
    assert_eq!(pool_balance_diff, pool_market_fees);

    assert_eq!(pool_market_fees_new, 0);
}

#[test]
fn redeem_host_fees() {
    let TestMarketFixture {
        contract_id,
        contract_client,
        contract_admin,
        usdc_pool_address,
        gold_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_host_fees = get_pool_accumulated_host_fees(&contract_client, &usdc_pool_address);
    dbg!(pool_host_fees);

    let contract_admin_balance_before = usdc_token_client.balance(&contract_admin);
    let pool_balance_before = usdc_token_client.balance(&contract_id);

    contract_client.redeem_accumulated_host_fees(
        &contract_admin,
        &usdc_pool_address,
        &pool_host_fees,
    );

    let contract_admin_balance_after = usdc_token_client.balance(&contract_admin);
    let contract_admin_balance_diff =
        contract_admin_balance_after.checked_sub(contract_admin_balance_before).unwrap();

    let pool_balance_after = usdc_token_client.balance(&contract_id);
    let pool_balance_diff = pool_balance_before.checked_sub(pool_balance_after).unwrap();

    let pool_host_fees_new = get_pool_accumulated_host_fees(&contract_client, &usdc_pool_address);

    assert_eq!(contract_admin_balance_diff, pool_host_fees);
    assert_eq!(pool_balance_diff, pool_host_fees);

    assert_eq!(pool_host_fees_new, 0);
}
