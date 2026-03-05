#![cfg(test)]

use market::{
    constants::{
        DEFAULT_INSOLVENCY_LTV_BPS, DEFAULT_MIN_COLLATERAL_VALUE_CENTS, MAX_RESERVES,
        POOL_STATUS_DEPOSIT_ENABLED,
    },
    error::MCError,
    pool::{PoolConfig, PoolFeeConfig, PoolHealthConfig, PoolStatus},
    storage::MarketStatus,
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

    contract_client.add_collateral(borrower, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.deposit_earn(
        liquidity_provider,
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    assert!(contract_client.try_borrow(borrower, &usdc_pool_address, &1, &None).is_ok());
    assert!(contract_client.try_deposit(creditor, &usdc_pool_address, &1, &None).is_ok());

    let pool_config_update_queue_in_period =
        contract_client.get_global_state().update_in_queue_period.unwrap();

    let new_pool_status = PoolStatus { flags: POOL_STATUS_DEPOSIT_ENABLED };
    let new_pool_config = PoolConfig { status: new_pool_status, ..Default::default() };

    contract_client.queue_in_pool_config_update(&usdc_pool_address, &new_pool_config);

    // - Move time -

    e.ledger().with_mut(|li| li.timestamp += pool_config_update_queue_in_period);

    contract_client.apply_pool_config_update(&usdc_pool_address);

    assert_eq!(
        contract_client.try_borrow(borrower, &usdc_pool_address, &1, &None),
        Err(Ok(MCError::OperationForbiddenOnPool))
    );
    assert!(contract_client.try_deposit(creditor, &usdc_pool_address, &1, &None).is_ok());

    let new_pool_config =
        PoolConfig { status: PoolStatus::new_all_disabled(), ..Default::default() };

    contract_client.queue_in_pool_config_update(&usdc_pool_address, &new_pool_config);

    // - Move time -

    e.ledger().with_mut(|li| li.timestamp += pool_config_update_queue_in_period);

    contract_client.apply_pool_config_update(&usdc_pool_address);

    assert_eq!(
        contract_client.try_borrow(borrower, &usdc_pool_address, &1, &None),
        Err(Ok(MCError::OperationForbiddenOnPool))
    );
    assert_eq!(
        contract_client.try_deposit(creditor, &usdc_pool_address, &1, &None),
        Err(Ok(MCError::OperationForbiddenOnPool))
    );
}

#[test]
fn test_cancel_pool_config_update() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    let pool_address = contract_client.initialize_pool(
        &token_address,
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

    let pool_address = contract_client.initialize_pool(&token_address, &None);

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
fn test_update_pool_status_instantaneously_in_owned_markets() {
    let TestMarketFixture {
        contract_client,
        full_contract_client,
        gold_pool_address,
        users,
        usdc_pool_address,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[0];
    let creditor = &users[1];

    contract_client.add_collateral(borrower, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.deposit_earn(
        liquidity_provider,
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    assert!(contract_client.try_borrow(borrower, &usdc_pool_address, &1, &None).is_ok());
    assert!(contract_client.try_deposit(creditor, &usdc_pool_address, &1, &None).is_ok());

    let new_pool_status_flags = POOL_STATUS_DEPOSIT_ENABLED;

    full_contract_client.update_pool_status(&usdc_pool_address, &new_pool_status_flags);

    assert_eq!(
        contract_client.try_borrow(borrower, &usdc_pool_address, &1, &None),
        Err(Ok(MCError::OperationForbiddenOnPool))
    );
    assert!(contract_client.try_deposit(creditor, &usdc_pool_address, &1, &None).is_ok());

    let new_pool_status_flags = 0;
    full_contract_client.update_pool_status(&usdc_pool_address, &new_pool_status_flags);

    assert_eq!(
        contract_client.try_borrow(borrower, &usdc_pool_address, &1, &None),
        Err(Ok(MCError::OperationForbiddenOnPool))
    );
    assert_eq!(
        contract_client.try_deposit(creditor, &usdc_pool_address, &1, &None),
        Err(Ok(MCError::OperationForbiddenOnPool))
    );
}

#[test]
fn test_update_market_status() {
    let TestMarketFixture { contract_client, users, usdc_pool_address, gold_pool_address, .. } =
        TestMarketFixture::new();
    let creditor = &users[0];
    let liquidity_provider = &users[1];

    let status = contract_client.get_global_state().status;
    assert_eq!(status, 0);

    contract_client.deposit_earn(
        liquidity_provider,
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    assert!(contract_client.try_deposit(creditor, &gold_pool_address, &100, &None).is_ok());
    assert!(contract_client.try_withdraw(creditor, &gold_pool_address, &1, &None).is_ok());
    assert!(contract_client.try_borrow(creditor, &usdc_pool_address, &50, &None).is_ok());
    assert!(contract_client.try_repay(creditor, &usdc_pool_address, &1, &None).is_ok());

    contract_client.update_market_status(&(MarketStatus::BorrowFrozen as u32));
    let status = contract_client.get_global_state().status;
    assert_eq!(status, MarketStatus::BorrowFrozen as u32);

    assert!(contract_client.try_deposit(creditor, &gold_pool_address, &1, &None).is_ok());
    assert!(contract_client.try_withdraw(creditor, &gold_pool_address, &1, &None).is_ok());
    assert_eq!(
        contract_client.try_borrow(creditor, &usdc_pool_address, &1, &None),
        Err(Ok(MCError::BorrowForbiddenOnMarket))
    );
    assert!(contract_client.try_repay(creditor, &usdc_pool_address, &1, &None).is_ok());

    contract_client.update_market_status(&(MarketStatus::DepositFrozen as u32));
    let status = contract_client.get_global_state().status;
    assert_eq!(status, MarketStatus::DepositFrozen as u32);

    assert_eq!(
        contract_client.try_deposit(creditor, &gold_pool_address, &1, &None),
        Err(Ok(MCError::DepositForbiddenOnMarket))
    );
    assert!(contract_client.try_withdraw(creditor, &gold_pool_address, &1, &None).is_ok());
    assert_eq!(
        contract_client.try_borrow(creditor, &usdc_pool_address, &1, &None),
        Err(Ok(MCError::BorrowForbiddenOnMarket))
    );
    assert!(contract_client.try_repay(creditor, &usdc_pool_address, &1, &None).is_ok());

    contract_client.update_market_status(&(MarketStatus::Frozen as u32));
    let status = contract_client.get_global_state().status;
    assert_eq!(status, MarketStatus::Frozen as u32);

    assert_eq!(
        contract_client.try_deposit(creditor, &gold_pool_address, &1, &None),
        Err(Ok(MCError::DepositForbiddenOnMarket))
    );
    assert_eq!(
        contract_client.try_withdraw(creditor, &gold_pool_address, &1, &None),
        Err(Ok(MCError::MarketIsFrozen))
    );
    assert_eq!(
        contract_client.try_borrow(creditor, &usdc_pool_address, &1, &None),
        Err(Ok(MCError::BorrowForbiddenOnMarket))
    );
    assert_eq!(
        contract_client.try_repay(creditor, &usdc_pool_address, &1, &None),
        Err(Ok(MCError::MarketIsFrozen))
    );
}

#[test]
fn test_update_market_config() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    const MAX_POSITIONS: u32 = MAX_RESERVES;
    const MIN_COLLATERAL_VALUE_CENTS: i128 = 10;

    assert_eq!(
        contract_client.try_update_market(&(MAX_POSITIONS + 1), &0),
        Err(Ok(MCError::InvalidMarketConfigOrUpdate))
    );
    assert_eq!(
        contract_client.try_update_market(&(MAX_POSITIONS), &-1),
        Err(Ok(MCError::InvalidInputAmount))
    );
    assert_eq!(
        contract_client.try_update_market(&(1), &MIN_COLLATERAL_VALUE_CENTS),
        Err(Ok(MCError::InvalidMarketConfigOrUpdate))
    );

    contract_client.update_market(&MAX_POSITIONS, &MIN_COLLATERAL_VALUE_CENTS);

    let global_state = contract_client.get_global_state();
    let (new_min_collateral_value_cents, new_max_positions) =
        (global_state.min_collateral_value_cents, global_state.max_positions);

    assert_eq!(new_min_collateral_value_cents, MIN_COLLATERAL_VALUE_CENTS);
    assert_eq!(new_max_positions, MAX_POSITIONS);
}

#[test]
fn test_anyone_cannot_freeze_market_via_controlled_insurance_fund() {
    use controlled_insurance_fund::{
        ControlledInsuranceFundContract, ControlledInsuranceFundContractClient,
    };
    use market::contract::{MarketContract, MarketContractClient};
    use soroban_sdk::{
        Address, Env, IntoVal, String,
        testutils::{Address as _, MockAuth, MockAuthInvoke},
    };

    let e = Env::default();

    // Actors
    let market_admin = Address::generate(&e);
    let fund_admin = Address::generate(&e);
    let attacker = Address::generate(&e);

    // Market dependencies required by constructor
    let oracle = Address::generate(&e);
    let deployer = Address::generate(&e);
    let swap_provider = Address::generate(&e);

    // Deploy ControlledInsuranceFund
    let cif_addr = e.register(ControlledInsuranceFundContract, (&fund_admin,));
    let cif = ControlledInsuranceFundContractClient::new(&e, &cif_addr);

    // Deploy Market with insurance_fund = ControlledInsuranceFund
    let name = String::from_str(&e, "test-market");
    let max_positions: u32 = MAX_RESERVES;
    let min_collateral_value_cents: i128 = DEFAULT_MIN_COLLATERAL_VALUE_CENTS;
    let insolvency_ltv_bps: i128 = DEFAULT_INSOLVENCY_LTV_BPS;
    let update_in_queue_period: Option<u64> = Some(1);

    let market_addr = e.register(
        MarketContract,
        (
            &name,
            &market_admin,
            &oracle,
            &swap_provider,
            &cif_addr,
            &deployer,
            &max_positions,
            &min_collateral_value_cents,
            &insolvency_ltv_bps,
            &update_in_queue_period,
        ),
    );
    let market = MarketContractClient::new(&e, &market_addr);

    // Setup: fund_admin sets the market address in the insurance fund.
    // This call is admin gated, so we mock auth only for this call
    let set_market_invoke = MockAuthInvoke {
        contract: &cif_addr,
        fn_name: "set_market",
        args: (&market_addr,).into_val(&e),
        sub_invokes: &[],
    };
    let set_market_auth = [MockAuth { address: &fund_admin, invoke: &set_market_invoke }];
    cif.mock_auths(&set_market_auth).set_market(&market_addr);

    // Setup: move the market to Active via market admin (admin only).
    // Owned markets may start Frozen depending on initialization
    let set_active_invoke = MockAuthInvoke {
        contract: &market_addr,
        fn_name: "update_market_status",
        args: (MarketStatus::Active as u32,).into_val(&e),
        sub_invokes: &[],
    };
    let set_active_auth = [MockAuth { address: &market_admin, invoke: &set_active_invoke }];
    market.mock_auths(&set_active_auth).update_market_status(&(MarketStatus::Active as u32));

    assert_eq!(market.get_global_state().status, MarketStatus::Active as u32);

    // Sanity: without auth, an unprivileged caller cannot use the admin only market entrypoint
    assert!(market.try_update_market_status(&(MarketStatus::Frozen as u32)).is_err());
    assert_eq!(market.get_global_state().status, MarketStatus::Active as u32);

    // Sanity: without auth, an unprivileged caller cannot use the admin only cif entrypoint
    let _ = attacker;
    assert!(cif.try_update_market_status(&(MarketStatus::Frozen as u32)).is_err());
}
