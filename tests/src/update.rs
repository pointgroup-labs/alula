#![cfg(test)]

use market::{
    constants::MAX_RESERVES,
    error::MCError,
    pool::{PoolConfig, PoolFeeConfig, PoolHealthConfig, PoolStatus},
};
use soroban_sdk::testutils::Ledger;

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, get_default_env,
    get_pool_fee_config, register_random_sac, setup_market_client,
};

#[test]
fn test_queue_in_pool_config_update() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    let pool_address = contract_client.initialize_pool(
        &token_address,
        &None,
        &None, // default pool config
    );

    assert_eq!(
        contract_client.try_cancel_pool_config_update(&pool_address),
        Err(Ok(MCError::PoolDoesNotHaveQueuedInConfigUpdate))
    );

    let before_borrow_fee_bps = get_pool_fee_config(&contract_client, &pool_address).borrow_fee_bps;

    const NEW_BORROW_FEE_BPS: u32 = 1000;
    let new_pool_config = PoolConfig {
        fee_config: PoolFeeConfig { borrow_fee_bps: NEW_BORROW_FEE_BPS, ..Default::default() },
        ..Default::default()
    };

    contract_client.queue_in_pool_config_update(&pool_address, &new_pool_config);

    let pool_config_update_queue_in_period =
        contract_client.get_global_state().update_in_queue_period.unwrap();

    // - Move time -

    e.ledger().with_mut(|li| li.timestamp += pool_config_update_queue_in_period - 1);

    assert_eq!(
        contract_client.try_apply_pool_config_update(&pool_address),
        Err(Ok(MCError::PoolConfigUpdateIsNotYetApplicable))
    );

    e.ledger().with_mut(|li| li.timestamp += 1);

    // - Apply config update -

    contract_client.apply_pool_config_update(&pool_address);

    let after_borrow_fee_bps = get_pool_fee_config(&contract_client, &pool_address).borrow_fee_bps;

    assert_ne!(before_borrow_fee_bps, NEW_BORROW_FEE_BPS);
    assert_eq!(after_borrow_fee_bps, NEW_BORROW_FEE_BPS);
}

#[test]
fn test_queue_in_invalid_pool_config_update() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    let pool_address = contract_client.initialize_pool(
        &token_address,
        &None,
        &None, // default pool config
    );

    const NEW_SUPPLY_LIMIT: i128 = -1;

    let new_pool_config = PoolConfig {
        health_config: PoolHealthConfig { supply_limit: NEW_SUPPLY_LIMIT, ..Default::default() },
        ..Default::default()
    };

    assert_eq!(
        contract_client.try_queue_in_pool_config_update(&pool_address, &new_pool_config),
        Err(Ok(MCError::InvalidLoanPoolConfig))
    );
}

#[test]
fn test_queue_in_disable_borrowing_pool_config_update() {
    let TestMarketFixture {
        e, contract_client, gold_pool_address, users, usdc_pool_address, ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[0];
    let creditor = &users[1];

    contract_client.add_collateral(borrower, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);
    contract_client.deposit_earn(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    assert!(contract_client.try_borrow(borrower, &usdc_pool_address, &1).is_ok());
    assert!(contract_client.try_deposit(creditor, &usdc_pool_address, &1).is_ok());

    let pool_config_update_queue_in_period =
        contract_client.get_global_state().update_in_queue_period.unwrap();

    let new_pool_config = PoolConfig {
        status: PoolStatus { borrow_enabled: false, deposit_enabled: true },
        ..Default::default()
    };

    contract_client.queue_in_pool_config_update(&usdc_pool_address, &new_pool_config);

    // - Move time -

    e.ledger().with_mut(|li| li.timestamp += pool_config_update_queue_in_period);

    contract_client.apply_pool_config_update(&usdc_pool_address);

    assert_eq!(
        contract_client.try_borrow(borrower, &usdc_pool_address, &1),
        Err(Ok(MCError::BorrowForbiddenOnPool))
    );
    assert!(contract_client.try_deposit(creditor, &usdc_pool_address, &1).is_ok());

    let new_pool_config = PoolConfig {
        status: PoolStatus { borrow_enabled: false, deposit_enabled: false },
        ..Default::default()
    };

    contract_client.queue_in_pool_config_update(&usdc_pool_address, &new_pool_config);

    // - Move time -

    e.ledger().with_mut(|li| li.timestamp += pool_config_update_queue_in_period);

    contract_client.apply_pool_config_update(&usdc_pool_address);

    assert_eq!(
        contract_client.try_borrow(borrower, &usdc_pool_address, &1),
        Err(Ok(MCError::BorrowForbiddenOnPool))
    );
    assert_eq!(
        contract_client.try_deposit(creditor, &usdc_pool_address, &1),
        Err(Ok(MCError::DepositForbiddenOnPool))
    );
}

#[test]
fn test_cancel_pool_config_update() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    let pool_address = contract_client.initialize_pool(
        &token_address,
        &None,
        &None, // default pool config
    );

    const NEW_SUPPLY_LIMIT: i128 = 100;

    let new_pool_config = PoolConfig {
        health_config: PoolHealthConfig { supply_limit: NEW_SUPPLY_LIMIT, ..Default::default() },
        ..Default::default()
    };

    assert_eq!(
        contract_client.try_cancel_pool_config_update(&pool_address),
        Err(Ok(MCError::PoolDoesNotHaveQueuedInConfigUpdate))
    );

    contract_client.queue_in_pool_config_update(&pool_address, &new_pool_config);

    assert_eq!(
        contract_client
            .get_pool_config_queued_in_update(&pool_address)
            .new_config
            .health_config
            .supply_limit,
        NEW_SUPPLY_LIMIT
    );

    contract_client.cancel_pool_config_update(&pool_address);

    assert_eq!(
        contract_client.try_get_pool_config_queued_in_update(&pool_address),
        Err(Ok(MCError::PoolDoesNotHaveQueuedInConfigUpdate))
    );
}

#[test]
fn test_update_market_fails_for_permissionless_market() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, false);

    let token_address = register_random_sac(&e);

    let pool_address = contract_client.initialize_pool(
        &token_address,
        &None,
        &None, // default pool config
    );

    const NEW_SUPPLY_LIMIT: i128 = 100;

    let new_pool_config = PoolConfig {
        health_config: PoolHealthConfig { supply_limit: NEW_SUPPLY_LIMIT, ..Default::default() },
        ..Default::default()
    };

    assert_eq!(
        contract_client.try_queue_in_pool_config_update(&pool_address, &new_pool_config),
        Err(Ok(MCError::MarketIsNotOwned))
    );

    assert_eq!(contract_client.try_update_market(&1, &1), Err(Ok(MCError::MarketIsNotOwned)));
}

#[test]
fn test_update_pool_in_permissionless_market_fails() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, false);

    let token_address = register_random_sac(&e);

    let pool_address = contract_client.initialize_pool(&token_address, &None, &None);

    const NEW_SUPPLY_LIMIT: i128 = 100;

    let new_pool_config = PoolConfig {
        health_config: PoolHealthConfig { supply_limit: NEW_SUPPLY_LIMIT, ..Default::default() },
        ..Default::default()
    };

    assert_eq!(
        contract_client.try_queue_in_pool_config_update(&pool_address, &new_pool_config),
        Err(Ok(MCError::MarketIsNotOwned))
    );

    assert_eq!(contract_client.try_update_market(&1, &1), Err(Ok(MCError::MarketIsNotOwned)));
}

#[test]
fn test_update_market_status() {
    let TestMarketFixture { contract_client, users, usdc_pool_address, gold_pool_address, .. } =
        TestMarketFixture::new();
    let creditor = &users[0];
    let liquidity_provider = &users[1];

    let status = contract_client.get_global_state().status;
    assert_eq!(status, 0);

    contract_client.deposit_earn(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    assert!(contract_client.try_deposit(creditor, &gold_pool_address, &100).is_ok());
    assert!(contract_client.try_withdraw(creditor, &gold_pool_address, &1).is_ok());
    assert!(contract_client.try_borrow(creditor, &usdc_pool_address, &100).is_ok());
    assert!(contract_client.try_repay(creditor, &usdc_pool_address, &1).is_ok());

    contract_client.update_market_status(&1);
    let status = contract_client.get_global_state().status;
    assert_eq!(status, 1);

    assert!(contract_client.try_deposit(creditor, &gold_pool_address, &1).is_ok());
    assert!(contract_client.try_withdraw(creditor, &gold_pool_address, &1).is_ok());
    assert_eq!(
        contract_client.try_borrow(creditor, &usdc_pool_address, &1),
        Err(Ok(MCError::BorrowForbiddenOnMarket))
    );
    assert!(contract_client.try_repay(creditor, &usdc_pool_address, &1).is_ok());

    contract_client.update_market_status(&2);
    let status = contract_client.get_global_state().status;
    assert_eq!(status, 2);

    assert_eq!(
        contract_client.try_deposit(creditor, &gold_pool_address, &1),
        Err(Ok(MCError::DepositForbiddenOnMarket))
    );
    assert!(contract_client.try_withdraw(creditor, &gold_pool_address, &1).is_ok());
    assert_eq!(
        contract_client.try_borrow(creditor, &usdc_pool_address, &1),
        Err(Ok(MCError::BorrowForbiddenOnMarket))
    );
    assert!(contract_client.try_repay(creditor, &usdc_pool_address, &1).is_ok());

    contract_client.update_market_status(&3);
    let status = contract_client.get_global_state().status;
    assert_eq!(status, 3);

    assert_eq!(
        contract_client.try_deposit(creditor, &gold_pool_address, &1),
        Err(Ok(MCError::DepositForbiddenOnMarket))
    );
    assert_eq!(
        contract_client.try_withdraw(creditor, &gold_pool_address, &1),
        Err(Ok(MCError::MarketIsFrozen))
    );
    assert_eq!(
        contract_client.try_borrow(creditor, &usdc_pool_address, &1),
        Err(Ok(MCError::BorrowForbiddenOnMarket))
    );
    assert_eq!(
        contract_client.try_repay(creditor, &usdc_pool_address, &1),
        Err(Ok(MCError::MarketIsFrozen))
    );
}

#[test]
fn test_update_market_config() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    const MAX_POSITIONS: u32 = 2 * MAX_RESERVES;
    const MIN_COLLATERAL_VALUE: i128 = 10 * 10i128.pow(7);

    assert_eq!(
        contract_client.try_update_market(&(MAX_POSITIONS + 1), &0),
        Err(Ok(MCError::InvalidMarketUpdate))
    );
    assert_eq!(
        contract_client.try_update_market(&(MAX_POSITIONS), &-1),
        Err(Ok(MCError::InvalidMarketUpdate))
    );
    assert_eq!(
        contract_client.try_update_market(&(1), &MIN_COLLATERAL_VALUE),
        Err(Ok(MCError::InvalidMarketUpdate))
    );

    contract_client.update_market(&MAX_POSITIONS, &MIN_COLLATERAL_VALUE);

    let global_state = contract_client.get_global_state();
    let (new_min_collateral_value, new_max_positions) =
        (global_state.min_collateral_value, global_state.max_positions);

    assert_eq!(new_min_collateral_value, MIN_COLLATERAL_VALUE);
    assert_eq!(new_max_positions, MAX_POSITIONS);
}
