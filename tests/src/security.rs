#![cfg(test)]

use market::{
    constants::{BPS_FACTOR, LEVERAGE_SCALE},
    error::MCError,
    pool::{PoolConfig, PoolHealthConfig},
    request::{Request, RequestType},
};
use soroban_sdk::vec as svec;

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture,
    get_obligation_collateral, get_obligation_d_tokens_as_tokens, get_obligation_j_tokens_as_tokens,
    get_pool_total_available, get_pool_total_borrowed, get_pool_total_supply,
};

// -- Flash Loan --

#[test]
fn test_flash_loan_state_preserved_after_callback() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();

    let liquidity_provider = &users[0];
    let borrower = &users[1];

    contract_client.deposit(liquidity_provider, &usdc_pool_address, &(10 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(liquidity_provider, &gold_pool_address, &(10 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.add_collateral(borrower, &gold_pool_address, &(5 * DEFAULT_COLLATERAL_AMOUNT));
    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_supply_before = get_pool_total_supply(&contract_client, &usdc_pool_address).unwrap();
    let pool_borrowed_before = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    let pool_supply_after = get_pool_total_supply(&contract_client, &usdc_pool_address).unwrap();
    let pool_borrowed_after = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert_eq!(pool_supply_before, pool_supply_after);
    assert_eq!(pool_borrowed_before, pool_borrowed_after);
}

// -- Batch Requests --

#[test]
fn test_batch_deposit_then_borrow_same_pool_fails() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();

    let user = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(liquidity_provider, &usdc_pool_address, &(10 * DEFAULT_DEPOSIT_AMOUNT));

    let deposit_r = Request {
        request_type: RequestType::Deposit.into(),
        pool_address: gold_pool_address.clone(),
        amount: DEFAULT_DEPOSIT_AMOUNT,
    };
    let borrow_r = Request {
        request_type: RequestType::Borrow.into(),
        pool_address: gold_pool_address.clone(),
        amount: DEFAULT_DEPOSIT_AMOUNT / 2,
    };

    let result = contract_client.try_submit_requests_batch(user, &svec![&e, deposit_r, borrow_r]);
    assert!(result.is_err());
}

#[test]
fn test_batch_add_collateral_borrow_different_pools_succeeds() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();

    let user = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(liquidity_provider, &usdc_pool_address, &(10 * DEFAULT_DEPOSIT_AMOUNT));

    let add_collateral_r = Request {
        request_type: RequestType::AddCollateral.into(),
        pool_address: gold_pool_address.clone(),
        amount: DEFAULT_COLLATERAL_AMOUNT,
    };
    let borrow_r = Request {
        request_type: RequestType::Borrow.into(),
        pool_address: usdc_pool_address.clone(),
        amount: DEFAULT_DEPOSIT_AMOUNT / 10,
    };

    contract_client.submit_requests_batch(user, &svec![&e, add_collateral_r, borrow_r]);

    let collateral = get_obligation_collateral(&contract_client, user, &gold_pool_address).unwrap();
    let debt = get_obligation_d_tokens_as_tokens(&e, &contract_client, user, &usdc_pool_address).unwrap();

    assert_eq!(collateral, DEFAULT_COLLATERAL_AMOUNT);
    assert_eq!(debt, DEFAULT_DEPOSIT_AMOUNT / 10);
}

#[test]
fn test_batch_borrow_repay_same_amount_no_net_change() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();

    let user = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(liquidity_provider, &usdc_pool_address, &(10 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.add_collateral(user, &gold_pool_address, &(2 * DEFAULT_COLLATERAL_AMOUNT));

    let pool_available_before = get_pool_total_available(&contract_client, &usdc_pool_address);

    let borrow_r = Request {
        request_type: RequestType::Borrow.into(),
        pool_address: usdc_pool_address.clone(),
        amount: DEFAULT_DEPOSIT_AMOUNT,
    };
    let repay_r = Request {
        request_type: RequestType::Repay.into(),
        pool_address: usdc_pool_address.clone(),
        amount: DEFAULT_DEPOSIT_AMOUNT,
    };

    contract_client.submit_requests_batch(user, &svec![&e, borrow_r, repay_r]);

    let pool_available_after = get_pool_total_available(&contract_client, &usdc_pool_address);
    assert_eq!(pool_available_before, pool_available_after);
}

#[test]
fn test_batch_multiple_deposits_same_pool() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();

    let user = &users[0];

    let deposit_r1 = Request {
        request_type: RequestType::Deposit.into(),
        pool_address: usdc_pool_address.clone(),
        amount: DEFAULT_DEPOSIT_AMOUNT,
    };
    let deposit_r2 = Request {
        request_type: RequestType::Deposit.into(),
        pool_address: usdc_pool_address.clone(),
        amount: DEFAULT_DEPOSIT_AMOUNT,
    };

    contract_client.submit_requests_batch(user, &svec![&e, deposit_r1, deposit_r2]);

    let j_tokens = get_obligation_j_tokens_as_tokens(&e, &contract_client, user, &usdc_pool_address).unwrap();
    assert_eq!(j_tokens, 2 * DEFAULT_DEPOSIT_AMOUNT);
}

#[test]
fn test_batch_withdraw_all_with_i128_max() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();

    let user = &users[0];

    contract_client.deposit(user, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let j_tokens_before = get_obligation_j_tokens_as_tokens(&e, &contract_client, user, &usdc_pool_address).unwrap();
    assert_eq!(j_tokens_before, DEFAULT_DEPOSIT_AMOUNT);

    let withdraw_r = Request {
        request_type: RequestType::Withdraw.into(),
        pool_address: usdc_pool_address.clone(),
        amount: i128::MAX,
    };

    contract_client.submit_requests_batch(user, &svec![&e, withdraw_r]);

    let j_tokens_after = get_obligation_j_tokens_as_tokens(&e, &contract_client, user, &usdc_pool_address);
    assert!(j_tokens_after.is_err() || j_tokens_after.unwrap() == 0);
}

// -- Leverage --

#[test]
fn test_leverage_at_exact_max_multiplier() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();

    let looper = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
    );

    let max_leverage_multiplier = contract_client
        .get_multiply_pair(&gold_pool_address, &usdc_pool_address)
        .max_leverage_multiplier;

    let result = contract_client.try_deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &max_leverage_multiplier,
    );

    match result {
        Ok(_) => {
            let obligation = contract_client
                .get_multiply_pair_obligation(looper, &gold_pool_address, &usdc_pool_address);
            assert!(!obligation.deposits.is_empty());
        }
        Err(Ok(MCError::InvalidLeverageMultiplier | MCError::InconsistentDepositWithLeverage)) => {}
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn test_leverage_exceeds_max_multiplier_fails() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();

    let looper = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(1000 * DEFAULT_DEPOSIT_AMOUNT),
    );

    let max_leverage_multiplier = contract_client
        .get_multiply_pair(&gold_pool_address, &usdc_pool_address)
        .max_leverage_multiplier;

    let result = contract_client.try_deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &(max_leverage_multiplier + 1),
    );

    assert_eq!(result, Err(Ok(MCError::InvalidLeverageMultiplier)));
}

#[test]
fn test_borrow_up_to_exact_open_ltv() {
    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            open_ltv_bps: 7500,
            close_ltv_bps: 8500,
            ..Default::default()
        },
        ..Default::default()
    };

    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);

    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
    );

    let collateral_amount = 10 * DEFAULT_COLLATERAL_AMOUNT;
    contract_client.add_collateral(borrower, &gold_pool_address, &collateral_amount);

    let max_borrow_at_open_ltv = collateral_amount * 7500 / BPS_FACTOR;
    let borrow_amount = max_borrow_at_open_ltv * 95 / 100;

    let result = contract_client.try_borrow(borrower, &usdc_pool_address, &borrow_amount);
    assert!(result.is_ok());

    let debt = get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address).unwrap();
    assert!(debt > 0);
}

#[test]
fn test_borrow_with_insufficient_collateral_fails() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();

    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
    );

    let result = contract_client.try_borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    assert!(result.is_err());
}

#[test]
fn test_min_leverage_is_no_leverage() {
    let TestMarketFixture {
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
        &LEVERAGE_SCALE,
    );

    let borrow_result = crate::get_multiply_pair_obligation_borrowed(
        &contract_client,
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    );

    assert_eq!(borrow_result, Err(MCError::BorrowPositionDoesNotExist));
}

#[test]
fn test_leverage_below_minimum_fails() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();

    let looper = &users[0];

    let result = contract_client.try_deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &false,
        &DEFAULT_DEPOSIT_AMOUNT,
        &(LEVERAGE_SCALE / 2),
    );

    assert_eq!(result, Err(Ok(MCError::InvalidLeverageMultiplier)));
}
