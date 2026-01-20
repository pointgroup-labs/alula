#![cfg(test)]

use market::{
    constants::{BPS_FACTOR, DEFAULT_UTILIZATION_RATIO_LIMIT_BPS, SECONDS_IN_YEAR},
    error::MCError,
    obligation::{OperationFees, compute_operation_fees},
    pool::{PoolConfig, PoolFeeConfig},
};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, map as smap,
    testutils::{Address as _, Ledger},
};

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, assert_approx_eq_abs,
    get_obligation_collateral, get_obligation_d_tokens_as_tokens,
    get_obligation_j_tokens_as_tokens, get_pool_fee_config, get_pool_operation_fees_sum,
    get_pool_take_rate_fees_sum, get_pool_total_borrowed,
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
    let liquidity_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT), &None);
    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let beneficiaries_sum_before =
        get_pool_operation_fees_sum(&contract_client, &usdc_pool_address);
    assert_eq!(beneficiaries_sum_before, 0);

    let pool_balance_before = usdc_token_client.balance(&contract_id);
    let borrower_balance_before = usdc_token_client.balance(&borrower.address);

    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let pool_balance_after = usdc_token_client.balance(&contract_id);
    let borrower_balance_after = usdc_token_client.balance(&borrower.address);

    let pool_balance_diff = pool_balance_before.checked_sub(pool_balance_after).unwrap();
    let borrower_balance_diff =
        borrower_balance_after.checked_sub(borrower_balance_before).unwrap();

    let beneficiaries_sum_after = get_pool_operation_fees_sum(&contract_client, &usdc_pool_address);
    let borrower_debt_after =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();
    let fee_sum_diff = beneficiaries_sum_after.checked_sub(beneficiaries_sum_before).unwrap();

    let PoolFeeConfig { borrow_fee_bps, .. } =
        get_pool_fee_config(&contract_client, &usdc_pool_address);

    let usdc_pool_fee_config = get_pool_fee_config(&contract_client, &usdc_pool_address);
    let OperationFees { fee_sum, .. } = compute_operation_fees(
        DEFAULT_DEPOSIT_AMOUNT,
        borrow_fee_bps,
        &None,
        &usdc_pool_fee_config,
    )
    .unwrap();

    let expected_borrower_balance_diff = DEFAULT_DEPOSIT_AMOUNT.checked_sub(fee_sum).unwrap();
    let expected_pool_balance_diff = expected_borrower_balance_diff;
    let expected_fee_sum_diff = fee_sum;

    assert_eq!(borrower_balance_diff, expected_borrower_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);
    assert_eq!(fee_sum_diff, expected_fee_sum_diff);

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
    let creditor_balance_before = gold_token_client.balance(&creditor.address);
    let operation_fees_before = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let pool_balance_after = gold_token_client.balance(&contract_id);
    let creditor_balance_after = gold_token_client.balance(&creditor.address);
    let operation_fees_after = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    let pool_balance_diff = pool_balance_after.checked_sub(pool_balance_before).unwrap();
    let creditor_balance_diff =
        creditor_balance_before.checked_sub(creditor_balance_after).unwrap();
    let fees_diff = operation_fees_after.checked_sub(operation_fees_before).unwrap();

    let pool_market_fees_after = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    let creditor_deposit_after =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address)
            .unwrap();

    let pool_fee_config = get_pool_fee_config(&contract_client, &gold_pool_address);
    let OperationFees { fee_sum, .. } = compute_operation_fees(
        DEFAULT_DEPOSIT_AMOUNT,
        pool_fee_config.deposit_fee_bps,
        &None,
        &pool_fee_config,
    )
    .unwrap();

    let expected_creditor_balance_diff = DEFAULT_DEPOSIT_AMOUNT;
    let expected_pool_balance_diff = expected_creditor_balance_diff;
    let expected_fee_sum_diff = fee_sum;
    let expected_creditor_deposit_after = DEFAULT_DEPOSIT_AMOUNT.checked_sub(fee_sum).unwrap();

    assert_eq!(creditor_balance_diff, expected_creditor_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);
    assert_eq!(fees_diff, expected_fee_sum_diff);
    assert_eq!(pool_market_fees_after, expected_fee_sum_diff);

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
    let collateral_adder_balance_before = gold_token_client.balance(&collateral_adder.address);
    let operation_fees_before = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    contract_client.add_collateral(
        collateral_adder,
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    );

    let pool_balance_after = gold_token_client.balance(&contract_id);
    let collateral_adder_balance_after = gold_token_client.balance(&collateral_adder.address);
    let operation_fees_after = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    let pool_balance_diff = pool_balance_after.checked_sub(pool_balance_before).unwrap();
    let collateral_adder_balance_diff =
        collateral_adder_balance_before.checked_sub(collateral_adder_balance_after).unwrap();
    let fees_diff = operation_fees_after.checked_sub(operation_fees_before).unwrap();

    let collateral_adder_collateral_after =
        get_obligation_collateral(&contract_client, collateral_adder, &gold_pool_address).unwrap();

    let pool_fee_config = get_pool_fee_config(&contract_client, &gold_pool_address);
    let OperationFees { fee_sum, .. } = compute_operation_fees(
        DEFAULT_COLLATERAL_AMOUNT,
        pool_fee_config.add_collateral_fee_bps,
        &None,
        &pool_fee_config,
    )
    .unwrap();

    let expected_collateral_adder_balance_diff = DEFAULT_COLLATERAL_AMOUNT;
    let expected_pool_balance_diff = expected_collateral_adder_balance_diff;
    let expected_fee_diff = fee_sum;
    let expected_collateral_adder_collateral_after =
        DEFAULT_COLLATERAL_AMOUNT.checked_sub(fee_sum).unwrap();

    assert_eq!(collateral_adder_balance_diff, expected_collateral_adder_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);
    assert_eq!(fees_diff, expected_fee_diff);

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
        &None,
    );

    let pool_balance_before = gold_token_client.balance(&contract_id);
    let collateral_adder_balance_before = gold_token_client.balance(&collateral_adder.address);
    let collateral_adder_collateral_before =
        get_obligation_collateral(&contract_client, collateral_adder, &gold_pool_address).unwrap();

    let fees_sum_before = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    contract_client.remove_collateral(
        collateral_adder,
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    );

    let pool_balance_after = gold_token_client.balance(&contract_id);
    let collateral_adder_balance_after = gold_token_client.balance(&collateral_adder.address);
    let collateral_adder_collateral_after =
        get_obligation_collateral(&contract_client, collateral_adder, &gold_pool_address).unwrap();
    let fees_sum_after = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    let pool_balance_diff = pool_balance_before.checked_sub(pool_balance_after).unwrap();
    let collateral_adder_balance_diff =
        collateral_adder_balance_after.checked_sub(collateral_adder_balance_before).unwrap();
    let collateral_adder_collateral_diff =
        collateral_adder_collateral_before.checked_sub(collateral_adder_collateral_after).unwrap();

    let fees_diff = fees_sum_after.checked_sub(fees_sum_before).unwrap();

    let pool_fee_config = get_pool_fee_config(&contract_client, &gold_pool_address);
    let OperationFees { fee_sum, .. } = compute_operation_fees(
        DEFAULT_COLLATERAL_AMOUNT,
        pool_fee_config.remove_collateral_fee_bps,
        &None,
        &pool_fee_config,
    )
    .unwrap();

    let expected_collateral_adder_balance_diff =
        DEFAULT_COLLATERAL_AMOUNT.checked_sub(fee_sum).unwrap();
    let expected_pool_balance_diff = expected_collateral_adder_balance_diff;
    let expected_fee_sum_diff = fee_sum;
    let expected_collateral_adder_collateral_diff = DEFAULT_COLLATERAL_AMOUNT;

    assert_eq!(collateral_adder_balance_diff, expected_collateral_adder_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);
    assert_eq!(fees_diff, expected_fee_sum_diff);
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

    contract_client.deposit(creditor, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT), &None);

    let pool_balance_before = gold_token_client.balance(&contract_id);
    let creditor_balance_before = gold_token_client.balance(&creditor.address);
    let creditor_deposit_before =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address)
            .unwrap();
    let fees_before = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    contract_client.withdraw(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let pool_balance_after = gold_token_client.balance(&contract_id);
    let creditor_balance_after = gold_token_client.balance(&creditor.address);
    let creditor_deposit_after =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address)
            .unwrap();
    let fees_after = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    let pool_balance_diff = pool_balance_before.checked_sub(pool_balance_after).unwrap();
    let creditor_balance_diff =
        creditor_balance_after.checked_sub(creditor_balance_before).unwrap();
    let creditor_deposit_diff =
        creditor_deposit_before.checked_sub(creditor_deposit_after).unwrap();
    let fees_diff = fees_after.checked_sub(fees_before).unwrap();

    let pool_fee_config = get_pool_fee_config(&contract_client, &gold_pool_address);
    let OperationFees { fee_sum, .. } = compute_operation_fees(
        DEFAULT_DEPOSIT_AMOUNT,
        pool_fee_config.withdraw_fee_bps,
        &None,
        &pool_fee_config,
    )
    .unwrap();

    let expected_creditor_balance_diff = DEFAULT_DEPOSIT_AMOUNT.checked_sub(fee_sum).unwrap();
    let expected_pool_balance_diff = expected_creditor_balance_diff;
    let expected_fee_sum_diff = fee_sum;
    let expected_creditor_deposit_diff = DEFAULT_DEPOSIT_AMOUNT;

    assert_eq!(creditor_balance_diff, expected_creditor_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);
    assert_eq!(fees_diff, expected_fee_sum_diff);

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

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(
        borrower,
        &usdc_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    // - Borrow up to utilization ratio cap -

    let borrow_amount: i128 = DEFAULT_DEPOSIT_AMOUNT
        .fixed_mul_ceil(DEFAULT_UTILIZATION_RATIO_LIMIT_BPS, BPS_FACTOR)
        .unwrap();
    let withdraw_amount = DEFAULT_DEPOSIT_AMOUNT.checked_sub(borrow_amount).unwrap();

    contract_client.borrow(borrower, &gold_pool_address, &borrow_amount, &None);

    // - Withdraw rest and check the fees -

    let pool_balance_before = gold_token_client.balance(&contract_id);
    let creditor_balance_before = gold_token_client.balance(&creditor.address);
    let creditor_deposit_before =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address)
            .unwrap();
    let fees_before = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    let simulated_withdraw_result =
        contract_client.simulate_withdraw(creditor, &gold_pool_address, &withdraw_amount, &None);

    contract_client.withdraw(creditor, &gold_pool_address, &withdraw_amount, &None);

    let pool_balance_after = gold_token_client.balance(&contract_id);
    let creditor_balance_after = gold_token_client.balance(&creditor.address);
    let creditor_deposit_after =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address)
            .unwrap();
    let fees_after = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    let pool_balance_diff = pool_balance_before.checked_sub(pool_balance_after).unwrap();
    let creditor_balance_diff =
        creditor_balance_after.checked_sub(creditor_balance_before).unwrap();
    let creditor_deposit_diff =
        creditor_deposit_before.checked_sub(creditor_deposit_after).unwrap();

    let fees_diff = fees_after.checked_sub(fees_before).unwrap();

    let pool_fee_config = get_pool_fee_config(&contract_client, &gold_pool_address);

    let withdraw_scarcity_fee_bps = {
        let bps_diff = BPS_FACTOR.checked_sub(DEFAULT_UTILIZATION_RATIO_LIMIT_BPS).unwrap();

        bps_diff
            .fixed_mul_ceil(pool_fee_config.withdraw_scarcity_fee_sc_bps as i128, BPS_FACTOR)
            .unwrap()
    } as u32;

    let OperationFees { fee_sum, .. } = compute_operation_fees(
        withdraw_amount,
        pool_fee_config.withdraw_fee_bps + withdraw_scarcity_fee_bps,
        &None,
        &pool_fee_config,
    )
    .unwrap();

    let expected_creditor_balance_diff = withdraw_amount.checked_sub(fee_sum).unwrap();
    let expected_pool_balance_diff = expected_creditor_balance_diff;
    let expected_fee_diff = fee_sum;
    let expected_creditor_deposit_diff = withdraw_amount;

    assert_eq!(creditor_balance_diff, expected_creditor_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);
    assert_eq!(fees_diff, expected_fee_diff);

    assert_eq!(creditor_deposit_diff, expected_creditor_deposit_diff);
    assert_eq!(creditor_balance_diff, simulated_withdraw_result.withdrawer_to_receive);
}

#[test]
fn test_withdraw_scarcity_fee_no_borrow() {
    let TestMarketFixture { contract_client, gold_pool_address, users, gold_token_client, .. } =
        TestMarketFixture::new();
    let creditor = &users[0];

    let creditor_balance_before = gold_token_client.balance(&creditor.address);
    let fees_before = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.withdraw(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let creditor_balance_after = gold_token_client.balance(&creditor.address);
    let fees_after = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    assert_eq!(creditor_balance_before, creditor_balance_after);
    assert_eq!(fees_after, fees_before);
    assert_eq!(fees_after, 0);
}

#[test]
fn test_simulate_withdraw_scarcity_fee() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        gold_token_client,
        ..
    } = TestMarketFixture::new();
    let creditor = &users[0];
    let borrower = &users[1];

    let utilization_ratio_limit_bps = contract_client
        .get_pool(&gold_pool_address)
        .config
        .health_config
        .utilization_ratio_limit_bps;
    let remaining_utilization_bps = BPS_FACTOR.checked_sub(utilization_ratio_limit_bps).unwrap();

    let (borrow_amount, withdraw_amount) = (
        DEFAULT_DEPOSIT_AMOUNT
            .fixed_mul_floor((utilization_ratio_limit_bps).min(BPS_FACTOR), BPS_FACTOR)
            .unwrap(),
        DEFAULT_DEPOSIT_AMOUNT.fixed_mul_floor(remaining_utilization_bps, BPS_FACTOR).unwrap(),
    );

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(borrower, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.borrow(borrower, &gold_pool_address, &borrow_amount, &None);

    let creditor_balance_before = gold_token_client.balance(&creditor.address);
    let simulated_withdraw_result =
        contract_client.simulate_withdraw(creditor, &gold_pool_address, &withdraw_amount, &None);
    let fees_sum_before = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    contract_client.withdraw(creditor, &gold_pool_address, &withdraw_amount, &None);

    let creditor_balance_after = gold_token_client.balance(&creditor.address);
    let creditor_balance_diff =
        creditor_balance_after.checked_sub(creditor_balance_before).unwrap();
    let fees_sum_after = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    let market_fees_diff = fees_sum_after.checked_sub(fees_sum_before).unwrap();

    assert_eq!(creditor_balance_diff, simulated_withdraw_result.withdrawer_to_receive);
    assert_eq!(market_fees_diff, simulated_withdraw_result.operation_fees.fee_sum);
}

// TODO: WIll be rewritten with new earn obligations

// #[test]
// fn test_simulate_withdraw_earn_scarcity_fee() {
//     let TestMarketFixture {
//         contract_client,
//         gold_pool_address,
//         usdc_pool_address,
//         users,
//         gold_token_client,
//         ..
//     } = TestMarketFixture::new();
//     let creditor = &users[0];
//     let borrower = &users[1];

//     let utilization_ratio_limit_bps = contract_client
//         .get_pool(&gold_pool_address)
//         .config
//         .health_config
//         .utilization_ratio_limit_bps;
//     let remaining_utilization_bps = BPS_FACTOR.checked_sub(utilization_ratio_limit_bps).unwrap();

//     let (borrow_amount, withdraw_amount) = (
//         DEFAULT_DEPOSIT_AMOUNT
//             .fixed_mul_floor((utilization_ratio_limit_bps).min(BPS_FACTOR), BPS_FACTOR)
//             .unwrap(),
//         DEFAULT_DEPOSIT_AMOUNT.fixed_mul_floor(remaining_utilization_bps, BPS_FACTOR).unwrap(),
//     );

//     contract_client.deposit_earn(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
//     contract_client.add_collateral(borrower, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
//     contract_client.borrow(borrower, &gold_pool_address, &borrow_amount, &None);

//     let creditor_balance_before = gold_token_client.balance(&creditor.address);
//     let simulated_withdraw_result = contract_client.simulate_earn_withdraw(
//         creditor,
//         &gold_pool_address,
//         &withdraw_amount,
//         &None,
//     );
//     let fees_before = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

//     contract_client.withdraw_earn(creditor, &gold_pool_address, &withdraw_amount, &None);

//     let creditor_balance_after = gold_token_client.balance(&creditor.address);
//     let creditor_balance_diff =
//         creditor_balance_after.checked_sub(creditor_balance_before).unwrap();
//     let fees_after = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

//     let fees_diff = fees_after.checked_sub(fees_before).unwrap();

//     assert_eq!(creditor_balance_diff, simulated_withdraw_result.withdrawer_to_receive);
//     assert_eq!(fees_diff, simulated_withdraw_result.operation_fees.fee_sum);
// }

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
    let liquidity_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(4 * DEFAULT_DEPOSIT_AMOUNT), &None);
    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(3 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    contract_client.borrow(borrower, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT), &None);

    let pool_balance_before = usdc_token_client.balance(&contract_id);
    let borrower_balance_before = usdc_token_client.balance(&borrower.address);
    let borrower_debt_before =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();
    let fees_before = get_pool_operation_fees_sum(&contract_client, &usdc_pool_address);

    contract_client.repay(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let pool_balance_after = usdc_token_client.balance(&contract_id);
    let borrower_balance_after = usdc_token_client.balance(&borrower.address);
    let borrower_debt_after =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();
    let fees_after = get_pool_operation_fees_sum(&contract_client, &usdc_pool_address);

    let pool_balance_diff = pool_balance_after.checked_sub(pool_balance_before).unwrap();
    let borrower_balance_diff =
        borrower_balance_before.checked_sub(borrower_balance_after).unwrap();
    let borrower_debt_diff = borrower_debt_before.checked_sub(borrower_debt_after).unwrap();

    let fees_diff = fees_after.checked_sub(fees_before).unwrap();

    let pool_fee_config = get_pool_fee_config(&contract_client, &usdc_pool_address);
    let OperationFees { fee_sum, .. } = compute_operation_fees(
        DEFAULT_DEPOSIT_AMOUNT,
        pool_fee_config.repay_fee_bps,
        &None,
        &pool_fee_config,
    )
    .unwrap();

    let expected_borrower_balance_diff = DEFAULT_DEPOSIT_AMOUNT;
    let expected_pool_balance_diff = expected_borrower_balance_diff;
    let expected_fee_sum_diff = fee_sum;
    let expected_borrower_debt_diff = DEFAULT_DEPOSIT_AMOUNT.checked_sub(fee_sum).unwrap();

    assert_eq!(borrower_balance_diff, expected_borrower_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);

    assert_eq!(fees_diff, expected_fee_sum_diff);
    assert_eq!(borrower_debt_diff, expected_borrower_debt_diff);
}

#[test]
fn test_distribute_all_pools_fees() {
    const DEPOSIT_FEE_BPS: u32 = 1_000; // 10%

    let pool_config = PoolConfig {
        fee_config: PoolFeeConfig { deposit_fee_bps: DEPOSIT_FEE_BPS, ..Default::default() },
        ..Default::default()
    };
    let TestMarketFixture {
        e,
        contract_id,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        gold_token_client,
        usdc_token_client,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let creditor = &users[0];
    let debtor = &users[1];
    let liquidity_provider = &users[2];
    let beneficiary_1 = &Address::generate(&e);
    let beneficiary_2 = &Address::generate(&e);

    let gold_pool_operation_fees =
        get_pool_operation_fees_sum(&contract_client, &gold_pool_address);
    let usdc_pool_operation_fees =
        get_pool_operation_fees_sum(&contract_client, &usdc_pool_address);

    assert_eq!(gold_pool_operation_fees, 0);
    assert_eq!(usdc_pool_operation_fees, 0);

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.borrow(debtor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let gold_pool_operation_fees =
        get_pool_operation_fees_sum(&contract_client, &gold_pool_address);
    let usdc_pool_operation_fees =
        get_pool_operation_fees_sum(&contract_client, &usdc_pool_address);

    assert!(gold_pool_operation_fees > 0);
    assert!(usdc_pool_operation_fees > 0);

    let gold_market_balance_before = gold_token_client.balance(&contract_id);
    let usdc_market_balance_before = usdc_token_client.balance(&contract_id);

    contract_client.distribute_all_pools_fees();

    let gold_market_balance_after = gold_token_client.balance(&contract_id);
    let usdc_market_balance_after = usdc_token_client.balance(&contract_id);

    assert_eq!(gold_market_balance_after, gold_market_balance_before);
    assert_eq!(usdc_market_balance_after, usdc_market_balance_before);

    let gold_pool_operation_fees =
        get_pool_operation_fees_sum(&contract_client, &gold_pool_address);
    let usdc_pool_operation_fees =
        get_pool_operation_fees_sum(&contract_client, &usdc_pool_address);

    assert_eq!(gold_pool_operation_fees, 0);
    assert_eq!(usdc_pool_operation_fees, 0);

    assert_eq!(
        contract_client.try_set_operation_fees_beneficiaries(
            &gold_pool_address,
            &smap![&e, (beneficiary_1.clone(), 3_000), (beneficiary_2.clone(), 8000)],
        ),
        Err(Ok(MCError::InvalidLoanPoolConfig))
    );

    // -- Set up beneficiaries --

    contract_client.set_operation_fees_beneficiaries(
        &gold_pool_address,
        &smap![&e, (beneficiary_1.clone(), 3_000), (beneficiary_2.clone(), 7000)],
    );
    contract_client.set_operation_fees_beneficiaries(
        &usdc_pool_address,
        &smap![&e, (beneficiary_1.clone(), 3_000), (beneficiary_2.clone(), 7000)],
    );

    // -- Repeat all operations --

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.borrow(debtor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    // -- Verify that fees are distributed --

    let gold_pool_operation_fees =
        get_pool_operation_fees_sum(&contract_client, &gold_pool_address);
    let usdc_pool_operation_fees =
        get_pool_operation_fees_sum(&contract_client, &usdc_pool_address);

    assert!(gold_pool_operation_fees > 0);
    assert!(usdc_pool_operation_fees > 0);

    let gold_market_balance_before = gold_token_client.balance(&contract_id);
    let usdc_market_balance_before = usdc_token_client.balance(&contract_id);

    contract_client.distribute_all_pools_fees();

    let gold_market_balance_after = gold_token_client.balance(&contract_id);
    let usdc_market_balance_after = usdc_token_client.balance(&contract_id);

    assert_eq!(
        gold_market_balance_before.checked_sub(gold_market_balance_after).unwrap(),
        gold_pool_operation_fees
    );
    assert_eq!(
        usdc_market_balance_before.checked_sub(usdc_market_balance_after).unwrap(),
        usdc_pool_operation_fees
    );

    let beneficiary_1_gold_balance = gold_token_client.balance(beneficiary_1);
    let beneficiary_2_gold_balance = gold_token_client.balance(beneficiary_2);

    assert_eq!(
        beneficiary_1_gold_balance,
        gold_pool_operation_fees.fixed_mul_floor(3000, BPS_FACTOR).unwrap()
    );
    assert_eq!(
        beneficiary_2_gold_balance,
        gold_pool_operation_fees.fixed_mul_floor(7000, BPS_FACTOR).unwrap()
    );

    let beneficiary_1_usdc_balance = usdc_token_client.balance(beneficiary_1);
    let beneficiary_2_usdc_balance = usdc_token_client.balance(beneficiary_2);

    assert_eq!(
        beneficiary_1_usdc_balance,
        usdc_pool_operation_fees.fixed_mul_floor(3000, BPS_FACTOR).unwrap()
    );
    assert_eq!(
        beneficiary_2_usdc_balance,
        usdc_pool_operation_fees.fixed_mul_floor(7000, BPS_FACTOR).unwrap()
    );

    let gold_pool_operation_fees =
        get_pool_operation_fees_sum(&contract_client, &gold_pool_address);
    let usdc_pool_operation_fees =
        get_pool_operation_fees_sum(&contract_client, &usdc_pool_address);

    assert_eq!(gold_pool_operation_fees, 0);
    assert_eq!(usdc_pool_operation_fees, 0);
}

#[test]
fn test_take_rate_fees_are_empty_prior_accrual() {
    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();

    let borrower = &users[0];
    let liquidity_provider = &users[1];

    let take_rate_fee_before = get_pool_take_rate_fees_sum(&contract_client, &usdc_pool_address);

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT), &None);
    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let take_rate_fees_after = get_pool_operation_fees_sum(&contract_client, &usdc_pool_address);

    assert_eq!(take_rate_fee_before, take_rate_fees_after);
    assert_eq!(take_rate_fee_before, 0);
}

#[test]
fn test_accumulate_take_rate_fees() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT), &None);
    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let pool_total_borrowed_before = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let fees_before = get_pool_take_rate_fees_sum(&contract_client, &usdc_pool_address);

    // -- Accrue debt on the pool --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR / 12;
    });
    contract_client.refresh_pool(&usdc_pool_address);

    let fees_after = get_pool_take_rate_fees_sum(&contract_client, &usdc_pool_address);
    let pool_total_borrowed_after = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    let fees_diff = fees_after.checked_sub(fees_before).unwrap();
    let pool_total_borrowed_diff =
        pool_total_borrowed_after.checked_sub(pool_total_borrowed_before).unwrap();

    let take_rate = get_pool_fee_config(&contract_client, &usdc_pool_address).take_rate_bps;
    let expected_accumulated_reserve_fees_diff =
        pool_total_borrowed_diff.fixed_mul_ceil(take_rate as i128, BPS_FACTOR).unwrap();

    assert!(pool_total_borrowed_diff > 0);
    assert_eq!(fees_diff, expected_accumulated_reserve_fees_diff);

    assert!(fees_after > fees_before);
}

#[test]
fn test_distribute_take_rate_fees() {
    let TestMarketFixture {
        e,
        contract_id,
        contract_client,
        usdc_token_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];
    let beneficiary_1 = Address::generate(&e);
    let beneficiary_2 = Address::generate(&e);

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT), &None);
    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    // -- Accrue debt on the pool --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR / 12;
    });
    contract_client.refresh_pool(&usdc_pool_address);

    let fees_before = get_pool_take_rate_fees_sum(&contract_client, &usdc_pool_address);
    let balance_before = usdc_token_client.balance(&contract_id);

    contract_client.distribute_pool_fees(&usdc_pool_address);

    let fees_after = get_pool_take_rate_fees_sum(&contract_client, &usdc_pool_address);
    let balance_after = usdc_token_client.balance(&contract_id);

    assert_eq!(balance_before, balance_after);
    assert_ne!(fees_before, fees_after);
    assert_eq!(fees_after, 0);

    assert_eq!(
        contract_client.try_set_take_rate_fees_beneficiaries(
            &usdc_pool_address,
            &smap![&e, (beneficiary_1.clone(), 3_000), (beneficiary_2.clone(), 8000)],
        ),
        Err(Ok(MCError::InvalidLoanPoolConfig))
    );

    contract_client.set_take_rate_fees_beneficiaries(
        &usdc_pool_address,
        &smap![&e, (beneficiary_1.clone(), 3_000), (beneficiary_2.clone(), 7_000)],
    );

    // -- Accrue debt on the pool --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR / 12;
    });
    contract_client.refresh_pool(&usdc_pool_address);

    let fees_before = get_pool_take_rate_fees_sum(&contract_client, &usdc_pool_address);
    let balance_before = usdc_token_client.balance(&contract_id);
    let beneficiary_1_balance_before = usdc_token_client.balance(&beneficiary_1);
    let beneficiary_2_balance_before = usdc_token_client.balance(&beneficiary_2);

    contract_client.distribute_pool_fees(&usdc_pool_address);

    let fees_after = get_pool_operation_fees_sum(&contract_client, &usdc_pool_address);
    let balance_after = usdc_token_client.balance(&contract_id);
    let beneficiary_1_balance_after = usdc_token_client.balance(&beneficiary_1);
    let beneficiary_2_balance_after = usdc_token_client.balance(&beneficiary_2);

    assert_approx_eq_abs(balance_before.checked_sub(balance_after).unwrap(), fees_before, 1);
    assert_approx_eq_abs(
        beneficiary_1_balance_after.checked_sub(beneficiary_1_balance_before).unwrap(),
        fees_before.fixed_mul_floor(3000, BPS_FACTOR).unwrap(),
        1,
    );
    assert_approx_eq_abs(
        beneficiary_2_balance_after.checked_sub(beneficiary_2_balance_before).unwrap(),
        fees_before.fixed_mul_floor(7000, BPS_FACTOR).unwrap(),
        1,
    );
    assert_eq!(fees_after, 0);
}

// TODO: Add test for `distribute_pool_fees`
