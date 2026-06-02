#![cfg(test)]

use market::{
    constants::{
        DEFAULT_BAD_DEBT_LOCK_D, DEFAULT_INSOLVENCY_LTV_BPS, DEFAULT_MIN_COLLATERAL_VALUE_CENTS,
        MAX_BAD_DEBT_LOCK_D, MAX_RESERVES, POOL_STATUS_DEPOSIT_ENABLED,
    },
    error::MCError,
    obligation::ObligationKey,
    pool::{PoolConfig, PoolFeeConfig, PoolHealthConfig, PoolStatus},
    storage::MarketStatus,
};
use soroban_sdk::testutils::Ledger;

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, get_default_env,
    get_pool_fee_config, register_random_sac, setup_market_client,
};

#[test]
fn test_queue_in_pool_set_for_existing_pool() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    contract_client.queue_in_pool_set(&token_address, &PoolConfig::default());

    let update_in_queue_period = contract_client.get_global_state().update_in_queue_period;

    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);
    contract_client.apply_pool_set(&token_address);

    let pool_address = token_address.clone();

    assert_eq!(
        contract_client.try_cancel_pool_set(&pool_address),
        Err(Ok(MCError::PoolDoesNotHaveQueuedPoolSet))
    );

    let before_borrow_fee_bps = get_pool_fee_config(&contract_client, &pool_address).borrow_fee_bps;

    const NEW_BORROW_FEE_BPS: u32 = 1000;
    let new_pool_config = PoolConfig {
        fee_config: PoolFeeConfig { borrow_fee_bps: NEW_BORROW_FEE_BPS, ..Default::default() },
        ..Default::default()
    };

    contract_client.queue_in_pool_set(&pool_address, &new_pool_config);

    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period - 1);

    assert_eq!(
        contract_client.try_apply_pool_set(&pool_address),
        Err(Ok(MCError::PoolSetIsNotYetApplicable))
    );

    e.ledger().with_mut(|li| li.timestamp += 1);

    contract_client.apply_pool_set(&pool_address);

    let after_borrow_fee_bps = get_pool_fee_config(&contract_client, &pool_address).borrow_fee_bps;

    assert_ne!(before_borrow_fee_bps, NEW_BORROW_FEE_BPS);
    assert_eq!(after_borrow_fee_bps, NEW_BORROW_FEE_BPS);
}

#[test]
fn test_queue_in_invalid_pool_set() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    const NEW_SUPPLY_LIMIT: i128 = -1;

    let new_pool_config = PoolConfig {
        health_config: PoolHealthConfig { supply_limit: NEW_SUPPLY_LIMIT, ..Default::default() },
        ..Default::default()
    };

    assert_eq!(
        contract_client.try_queue_in_pool_set(&token_address, &new_pool_config),
        Err(Ok(MCError::InvalidLoanPoolConfig))
    );
}

#[test]
fn test_queue_in_disable_borrowing_pool_set() {
    let TestMarketFixture {
        e, contract_client, gold_pool_address, users, usdc_pool_address, ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[2];
    let creditor = &users[1];

    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    assert!(
        contract_client
            .try_borrow(&ObligationKey::new(borrower.clone()), &usdc_pool_address, &1, &None)
            .is_ok()
    );
    assert!(
        contract_client
            .try_deposit(&ObligationKey::new(creditor.clone()), &usdc_pool_address, &1, &None)
            .is_ok()
    );

    let update_in_queue_period = contract_client.get_global_state().update_in_queue_period;

    let new_pool_status = PoolStatus { flags: POOL_STATUS_DEPOSIT_ENABLED };
    let new_pool_config = PoolConfig { status: new_pool_status, ..Default::default() };

    contract_client.queue_in_pool_set(&usdc_pool_address, &new_pool_config);

    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);

    contract_client.apply_pool_set(&usdc_pool_address);

    assert_eq!(
        contract_client.try_borrow(
            &ObligationKey::new(borrower.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::OperationForbiddenOnPool))
    );
    assert!(
        contract_client
            .try_deposit(&ObligationKey::new(creditor.clone()), &usdc_pool_address, &1, &None)
            .is_ok()
    );

    let new_pool_config =
        PoolConfig { status: PoolStatus::new_all_disabled(), ..Default::default() };

    contract_client.queue_in_pool_set(&usdc_pool_address, &new_pool_config);

    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);

    contract_client.apply_pool_set(&usdc_pool_address);

    assert_eq!(
        contract_client.try_borrow(
            &ObligationKey::new(borrower.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::OperationForbiddenOnPool))
    );
    assert_eq!(
        contract_client.try_deposit(
            &ObligationKey::new(creditor.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::OperationForbiddenOnPool))
    );
}

#[test]
fn test_cancel_pool_set() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    contract_client.queue_in_pool_set(&token_address, &PoolConfig::default());

    let update_in_queue_period = contract_client.get_global_state().update_in_queue_period;
    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);
    contract_client.apply_pool_set(&token_address);

    let pool_address = token_address.clone();

    const NEW_SUPPLY_LIMIT: i128 = 100;

    let new_pool_config = PoolConfig {
        health_config: PoolHealthConfig { supply_limit: NEW_SUPPLY_LIMIT, ..Default::default() },
        ..Default::default()
    };

    assert_eq!(
        contract_client.try_cancel_pool_set(&pool_address),
        Err(Ok(MCError::PoolDoesNotHaveQueuedPoolSet))
    );

    contract_client.queue_in_pool_set(&pool_address, &new_pool_config);

    assert_eq!(
        contract_client.get_queued_pool_set(&pool_address).new_config.health_config.supply_limit,
        NEW_SUPPLY_LIMIT
    );

    contract_client.cancel_pool_set(&pool_address);

    assert_eq!(
        contract_client.try_get_queued_pool_set(&pool_address),
        Err(Ok(MCError::PoolDoesNotHaveQueuedPoolSet))
    );
}

#[test]
fn test_update_market_fails_for_permissionless_market() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, false);

    let token_address = register_random_sac(&e);

    contract_client.queue_in_pool_set(&token_address, &PoolConfig::default());

    let update_in_queue_period = contract_client.get_global_state().update_in_queue_period;
    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);
    contract_client.apply_pool_set(&token_address);

    let pool_address = token_address.clone();

    const NEW_SUPPLY_LIMIT: i128 = 100;

    let new_pool_config = PoolConfig {
        health_config: PoolHealthConfig { supply_limit: NEW_SUPPLY_LIMIT, ..Default::default() },
        ..Default::default()
    };

    assert_eq!(
        contract_client.try_queue_in_pool_set(&pool_address, &new_pool_config),
        Err(Ok(MCError::MarketIsNotOwned))
    );

    assert_eq!(
        contract_client.try_queue_in_market_update(&1, &1, &DEFAULT_BAD_DEBT_LOCK_D),
        Err(Ok(MCError::MarketIsNotOwned))
    );
}

#[test]
fn test_update_pool_in_permissionless_market_fails() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, false);

    let token_address = register_random_sac(&e);

    contract_client.queue_in_pool_set(&token_address, &PoolConfig::default());

    let update_in_queue_period = contract_client.get_global_state().update_in_queue_period;
    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);
    contract_client.apply_pool_set(&token_address);

    let pool_address = token_address.clone();

    const NEW_SUPPLY_LIMIT: i128 = 100;

    let new_pool_config = PoolConfig {
        health_config: PoolHealthConfig { supply_limit: NEW_SUPPLY_LIMIT, ..Default::default() },
        ..Default::default()
    };

    assert_eq!(
        contract_client.try_queue_in_pool_set(&pool_address, &new_pool_config),
        Err(Ok(MCError::MarketIsNotOwned))
    );

    assert_eq!(
        contract_client.try_queue_in_market_update(&1, &1, &DEFAULT_BAD_DEBT_LOCK_D),
        Err(Ok(MCError::MarketIsNotOwned))
    );
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
    let liquidity_provider = &users[2];
    let creditor = &users[1];

    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    assert!(
        contract_client
            .try_borrow(&ObligationKey::new(borrower.clone()), &usdc_pool_address, &1, &None)
            .is_ok()
    );
    assert!(
        contract_client
            .try_deposit(&ObligationKey::new(creditor.clone()), &usdc_pool_address, &1, &None)
            .is_ok()
    );

    let new_pool_status_flags = POOL_STATUS_DEPOSIT_ENABLED;

    full_contract_client.update_pool_status(&usdc_pool_address, &new_pool_status_flags);

    assert_eq!(
        contract_client.try_borrow(
            &ObligationKey::new(borrower.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::OperationForbiddenOnPool))
    );
    assert!(
        contract_client
            .try_deposit(&ObligationKey::new(creditor.clone()), &usdc_pool_address, &1, &None)
            .is_ok()
    );

    let new_pool_status_flags = 0;
    full_contract_client.update_pool_status(&usdc_pool_address, &new_pool_status_flags);

    assert_eq!(
        contract_client.try_borrow(
            &ObligationKey::new(borrower.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::OperationForbiddenOnPool))
    );
    assert_eq!(
        contract_client.try_deposit(
            &ObligationKey::new(creditor.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
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

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    assert!(
        contract_client
            .try_deposit(&ObligationKey::new(creditor.clone()), &gold_pool_address, &100, &None)
            .is_ok()
    );
    assert!(
        contract_client
            .try_withdraw(&ObligationKey::new(creditor.clone()), &gold_pool_address, &1, &None)
            .is_ok()
    );
    assert!(
        contract_client
            .try_borrow(&ObligationKey::new(creditor.clone()), &usdc_pool_address, &50, &None)
            .is_ok()
    );
    assert!(
        contract_client
            .try_repay(&ObligationKey::new(creditor.clone()), &usdc_pool_address, &1, &None)
            .is_ok()
    );

    contract_client.update_market_status(&(MarketStatus::BorrowFrozen as u32));
    let status = contract_client.get_global_state().status;
    assert_eq!(status, MarketStatus::BorrowFrozen as u32);

    assert!(
        contract_client
            .try_deposit(&ObligationKey::new(creditor.clone()), &gold_pool_address, &1, &None)
            .is_ok()
    );
    assert!(
        contract_client
            .try_withdraw(&ObligationKey::new(creditor.clone()), &gold_pool_address, &1, &None)
            .is_ok()
    );
    assert_eq!(
        contract_client.try_borrow(
            &ObligationKey::new(creditor.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::BorrowForbiddenOnMarket))
    );
    assert!(
        contract_client
            .try_repay(&ObligationKey::new(creditor.clone()), &usdc_pool_address, &1, &None)
            .is_ok()
    );

    contract_client.update_market_status(&(MarketStatus::DepositFrozen as u32));
    let status = contract_client.get_global_state().status;
    assert_eq!(status, MarketStatus::DepositFrozen as u32);

    assert_eq!(
        contract_client.try_deposit(
            &ObligationKey::new(creditor.clone()),
            &gold_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::DepositForbiddenOnMarket))
    );
    assert!(
        contract_client
            .try_withdraw(&ObligationKey::new(creditor.clone()), &gold_pool_address, &1, &None)
            .is_ok()
    );
    assert_eq!(
        contract_client.try_borrow(
            &ObligationKey::new(creditor.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::BorrowForbiddenOnMarket))
    );
    assert!(
        contract_client
            .try_repay(&ObligationKey::new(creditor.clone()), &usdc_pool_address, &1, &None)
            .is_ok()
    );

    contract_client.update_market_status(&(MarketStatus::Frozen as u32));
    let status = contract_client.get_global_state().status;
    assert_eq!(status, MarketStatus::Frozen as u32);

    assert_eq!(
        contract_client.try_deposit(
            &ObligationKey::new(creditor.clone()),
            &gold_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::DepositForbiddenOnMarket))
    );
    assert!(
        contract_client
            .try_withdraw(&ObligationKey::new(creditor.clone()), &gold_pool_address, &1, &None)
            .is_ok()
    );
    assert_eq!(
        contract_client.try_borrow(
            &ObligationKey::new(creditor.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::BorrowForbiddenOnMarket))
    );
    assert!(
        contract_client
            .try_repay(&ObligationKey::new(creditor.clone()), &usdc_pool_address, &1, &None)
            .is_ok()
    );
}

#[test]
fn test_queue_in_market_config_update() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    const MAX_POSITIONS: u32 = MAX_RESERVES;
    const MIN_COLLATERAL_VALUE_CENTS: i128 = 10;

    assert_eq!(
        contract_client.try_queue_in_market_update(
            &(MAX_POSITIONS + 1),
            &0,
            &DEFAULT_BAD_DEBT_LOCK_D,
        ),
        Err(Ok(MCError::InvalidMarketConfigOrUpdate))
    );
    assert_eq!(
        contract_client
            .try_queue_in_market_update(&(MAX_POSITIONS), &-1, &DEFAULT_BAD_DEBT_LOCK_D,),
        Err(Ok(MCError::InvalidInputAmount))
    );
    assert_eq!(
        contract_client.try_queue_in_market_update(
            &(1),
            &MIN_COLLATERAL_VALUE_CENTS,
            &DEFAULT_BAD_DEBT_LOCK_D,
        ),
        Err(Ok(MCError::InvalidMarketConfigOrUpdate))
    );

    contract_client.queue_in_market_update(
        &MAX_POSITIONS,
        &MIN_COLLATERAL_VALUE_CENTS,
        &DEFAULT_BAD_DEBT_LOCK_D,
    );

    let update_in_queue_period = contract_client.get_global_state().update_in_queue_period;

    assert_eq!(
        contract_client.try_apply_market_update(),
        Err(Ok(MCError::MarketConfigUpdateIsNotYetApplicable))
    );

    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period - 1);

    assert_eq!(
        contract_client.try_apply_market_update(),
        Err(Ok(MCError::MarketConfigUpdateIsNotYetApplicable))
    );

    e.ledger().with_mut(|li| li.timestamp += 1);

    contract_client.apply_market_update();

    let global_state = contract_client.get_global_state();
    let (new_min_collateral_value_cents, new_max_positions) =
        (global_state.min_collateral_value_cents, global_state.max_positions);

    assert_eq!(new_min_collateral_value_cents, MIN_COLLATERAL_VALUE_CENTS);
    assert_eq!(new_max_positions, MAX_POSITIONS);
}

#[test]
fn test_cancel_market_config_update() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    assert_eq!(
        contract_client.try_cancel_market_update(),
        Err(Ok(MCError::MarketDoesNotHaveQueuedInConfigUpdate))
    );

    const MAX_POSITIONS: u32 = MAX_RESERVES;
    const MIN_COLLATERAL_VALUE_CENTS: i128 = 10;

    contract_client.queue_in_market_update(
        &MAX_POSITIONS,
        &MIN_COLLATERAL_VALUE_CENTS,
        &DEFAULT_BAD_DEBT_LOCK_D,
    );

    let queued_update = contract_client.get_market_queued_in_update();
    assert_eq!(queued_update.new_max_positions, MAX_POSITIONS);
    assert_eq!(queued_update.new_min_collateral_value_cents, MIN_COLLATERAL_VALUE_CENTS);

    contract_client.cancel_market_update();

    assert_eq!(
        contract_client.try_get_market_queued_in_update(),
        Err(Ok(MCError::MarketDoesNotHaveQueuedInConfigUpdate))
    );
}

#[test]
fn test_queue_in_market_config_update_fails_when_already_queued() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    contract_client.queue_in_market_update(&MAX_RESERVES, &10, &DEFAULT_BAD_DEBT_LOCK_D);

    assert_eq!(
        contract_client.try_queue_in_market_update(&MAX_RESERVES, &20, &DEFAULT_BAD_DEBT_LOCK_D,),
        Err(Ok(MCError::MarketAlreadyContainsQueuedInConfigUpdate))
    );
}

#[test]
fn test_anyone_cannot_freeze_market_via_controlled_insurance_fund() {
    use controlled_insurance_fund::{
        ControlledInsuranceFundContract, ControlledInsuranceFundContractClient,
    };
    use market::{
        contract::{MarketContract, MarketContractClient},
        storage::MarketInitParams,
    };
    use soroban_sdk::{
        Address, Env, IntoVal, String,
        testutils::{Address as _, MockAuth, MockAuthInvoke},
    };

    let e = Env::default();

    let market_admin = Address::generate(&e);
    let fund_admin = Address::generate(&e);
    let attacker = Address::generate(&e);

    let oracle = Address::generate(&e);
    let deployer = Address::generate(&e);

    let cif_addr = e.register(ControlledInsuranceFundContract, (&fund_admin,));
    let cif = ControlledInsuranceFundContractClient::new(&e, &cif_addr);

    let name = String::from_str(&e, "test-market");
    let max_positions: u32 = MAX_RESERVES;
    let min_collateral_value_cents: i128 = DEFAULT_MIN_COLLATERAL_VALUE_CENTS;
    let insolvency_ltv_bps: i128 = DEFAULT_INSOLVENCY_LTV_BPS;
    let update_in_queue_period: u64 = 1;
    let is_owned: bool = true;

    let market_addr = e.register(
        MarketContract,
        (
            &name,
            &market_admin,
            &oracle,
            &cif_addr,
            &deployer,
            MarketInitParams {
                max_positions,
                min_collateral_value_cents,
                insolvency_ltv_bps,
                update_in_queue_period,
                is_owned,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        ),
    );
    let market = MarketContractClient::new(&e, &market_addr);

    let set_market_invoke = MockAuthInvoke {
        contract: &cif_addr,
        fn_name: "set_market",
        args: (&market_addr,).into_val(&e),
        sub_invokes: &[],
    };
    let set_market_auth = [MockAuth { address: &fund_admin, invoke: &set_market_invoke }];
    cif.mock_auths(&set_market_auth).set_market(&market_addr);

    let set_active_invoke = MockAuthInvoke {
        contract: &market_addr,
        fn_name: "update_market_status",
        args: (MarketStatus::Active as u32,).into_val(&e),
        sub_invokes: &[],
    };
    let set_active_auth = [MockAuth { address: &market_admin, invoke: &set_active_invoke }];
    market.mock_auths(&set_active_auth).update_market_status(&(MarketStatus::Active as u32));

    assert_eq!(market.get_global_state().status, MarketStatus::Active as u32);

    assert!(market.try_update_market_status(&(MarketStatus::Frozen as u32)).is_err());
    assert_eq!(market.get_global_state().status, MarketStatus::Active as u32);

    let _ = attacker;
    assert!(cif.try_update_market_status(&(MarketStatus::Frozen as u32)).is_err());
}

#[test]
fn test_queue_in_new_pool_set() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    assert!(contract_client.try_get_pool(&token_address).is_err());

    contract_client.queue_in_pool_set(&token_address, &PoolConfig::default());

    assert!(contract_client.try_get_pool(&token_address).is_err());

    let update_in_queue_period = contract_client.get_global_state().update_in_queue_period;
    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);

    contract_client.apply_pool_set(&token_address);

    assert!(contract_client.try_get_pool(&token_address).is_ok());
}

#[test]
fn test_apply_pool_set_permissionless() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, false);

    let token_address = register_random_sac(&e);

    contract_client.queue_in_pool_set(&token_address, &PoolConfig::default());

    let update_in_queue_period = contract_client.get_global_state().update_in_queue_period;
    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);

    contract_client.apply_pool_set(&token_address);

    assert!(contract_client.try_get_pool(&token_address).is_ok());
}

// -- Security regression tests --

#[test]
fn test_apply_pool_set_is_permissionless_after_queue() {
    use market::{
        contract::{MarketClient, MarketContract},
        storage::MarketInitParams,
    };
    use soroban_sdk::{
        Address, Env, IntoVal, String,
        testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    };

    let e = Env::default();

    let admin = Address::generate(&e);
    let attacker = Address::generate(&e);
    let oracle = Address::generate(&e);
    let deployer = Address::generate(&e);
    let insurance_fund = Address::generate(&e);

    let market_addr = e.register(
        MarketContract,
        (
            &String::from_str(&e, "test"),
            &admin,
            &oracle,
            &insurance_fund,
            &deployer,
            MarketInitParams {
                max_positions: MAX_RESERVES,
                min_collateral_value_cents: 0i128,
                insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                update_in_queue_period: 100,
                is_owned: true,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        ),
    );

    let market = MarketClient::new(&e, &market_addr);

    let activate_invoke = MockAuthInvoke {
        contract: &market_addr,
        fn_name: "update_market_status",
        args: (0u32,).into_val(&e),
        sub_invokes: &[],
    };
    market
        .mock_auths(&[MockAuth { address: &admin, invoke: &activate_invoke }])
        .update_market_status(&0);

    let token_address = register_random_sac(&e);

    let queue_invoke = MockAuthInvoke {
        contract: &market_addr,
        fn_name: "queue_in_pool_set",
        args: (&token_address, PoolConfig::default()).into_val(&e),
        sub_invokes: &[],
    };
    market
        .mock_auths(&[MockAuth { address: &admin, invoke: &queue_invoke }])
        .queue_in_pool_set(&token_address, &PoolConfig::default());

    e.ledger().with_mut(|li| li.timestamp += 100);

    // Attacker (non-admin) drives `apply_pool_set` after the timelock has
    // elapsed. This MUST succeed: the payload was admin-authenticated at
    // queue time, and `apply_*` is just transport. The attacker can't change
    // what gets applied.
    let apply_invoke = MockAuthInvoke {
        contract: &market_addr,
        fn_name: "apply_pool_set",
        args: (&token_address,).into_val(&e),
        sub_invokes: &[],
    };
    market
        .mock_auths(&[MockAuth { address: &attacker, invoke: &apply_invoke }])
        .apply_pool_set(&token_address);

    // The queued config is now live — exactly what the admin queued.
    assert!(market.try_get_pool(&token_address).is_ok());
}

#[test]
fn test_apply_market_update_is_permissionless_when_owned() {
    use market::{
        contract::{MarketClient, MarketContract},
        storage::MarketInitParams,
    };
    use soroban_sdk::{
        Address, Env, IntoVal, String,
        testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    };

    let e = Env::default();

    let admin = Address::generate(&e);
    let attacker = Address::generate(&e);

    let market_addr = e.register(
        MarketContract,
        (
            &String::from_str(&e, "test"),
            &admin,
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            MarketInitParams {
                max_positions: MAX_RESERVES,
                min_collateral_value_cents: 0i128,
                insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                update_in_queue_period: 100,
                is_owned: true,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        ),
    );

    let market = MarketClient::new(&e, &market_addr);

    let activate_invoke = MockAuthInvoke {
        contract: &market_addr,
        fn_name: "update_market_status",
        args: (0u32,).into_val(&e),
        sub_invokes: &[],
    };
    market
        .mock_auths(&[MockAuth { address: &admin, invoke: &activate_invoke }])
        .update_market_status(&0);

    let queue_invoke = MockAuthInvoke {
        contract: &market_addr,
        fn_name: "queue_in_market_update",
        args: (MAX_RESERVES, 10i128, DEFAULT_BAD_DEBT_LOCK_D).into_val(&e),
        sub_invokes: &[],
    };
    market
        .mock_auths(&[MockAuth { address: &admin, invoke: &queue_invoke }])
        .queue_in_market_update(&MAX_RESERVES, &10, &DEFAULT_BAD_DEBT_LOCK_D);

    e.ledger().with_mut(|li| li.timestamp += 100);

    // After the timelock, anyone — including a non-admin — can drive
    // `apply_market_update`. The payload was authenticated at queue time, so
    // the attacker has no leverage over what gets applied; they're merely the
    // transport. The `require_owned` check still applies (covered by the
    // companion test below).
    let apply_invoke = MockAuthInvoke {
        contract: &market_addr,
        fn_name: "apply_market_update",
        args: ().into_val(&e),
        sub_invokes: &[],
    };
    market
        .mock_auths(&[MockAuth { address: &attacker, invoke: &apply_invoke }])
        .apply_market_update();
}

#[test]
fn test_get_queued_pool_set_is_public_readable() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    contract_client.queue_in_pool_set(&token_address, &PoolConfig::default());

    let queued = contract_client.get_queued_pool_set(&token_address);
    assert_eq!(queued.new_config, PoolConfig::default());
}

#[test]
fn test_cancel_pool_set_for_nonexistent_pool_only_requires_admin() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, false);

    let token_address = register_random_sac(&e);

    contract_client.queue_in_pool_set(&token_address, &PoolConfig::default());
    contract_client.cancel_pool_set(&token_address);

    assert_eq!(
        contract_client.try_get_queued_pool_set(&token_address),
        Err(Ok(MCError::PoolDoesNotHaveQueuedPoolSet))
    );
}

#[test]
fn test_queue_pool_set_duplicate_fails() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    contract_client.queue_in_pool_set(&token_address, &PoolConfig::default());

    assert_eq!(
        contract_client.try_queue_in_pool_set(&token_address, &PoolConfig::default()),
        Err(Ok(MCError::PoolAlreadyContainsQueuedPoolSet))
    );
}

#[test]
fn test_repay_and_withdraw_allowed_when_frozen() {
    let TestMarketFixture { contract_client, gold_pool_address, users, usdc_pool_address, .. } =
        TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[2];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    );
    contract_client.borrow(&ObligationKey::new(borrower.clone()), &usdc_pool_address, &100, &None);

    contract_client.update_market_status(&(MarketStatus::Frozen as u32));

    assert!(
        contract_client
            .try_repay(&ObligationKey::new(borrower.clone()), &usdc_pool_address, &50, &None)
            .is_ok(),
        "Repay should be allowed when market is frozen"
    );

    assert!(
        contract_client
            .try_withdraw(
                &ObligationKey::new(liquidity_provider.clone()),
                &usdc_pool_address,
                &1,
                &None
            )
            .is_ok(),
        "Withdraw should be allowed when market is frozen"
    );
}

#[test]
fn test_submit_requests_batch_allowed_when_frozen() {
    use market::request::{Request, StandardRequest};
    let TestMarketFixture {
        e, contract_client, gold_pool_address, users, usdc_pool_address, ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[2];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    );
    contract_client.borrow(&ObligationKey::new(borrower.clone()), &usdc_pool_address, &100, &None);

    contract_client.update_market_status(&(MarketStatus::Frozen as u32));

    contract_client.submit_requests_batch(
        &ObligationKey::new(borrower.clone()),
        &soroban_sdk::vec![
            &e,
            Request::Repay(StandardRequest { pool_address: usdc_pool_address.clone(), amount: 50 })
        ],
        &None,
    );
}

#[test]
fn test_min_collateral_value_cents_validation_on_market_update() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    assert_eq!(
        contract_client.try_queue_in_market_update(
            &MAX_RESERVES,
            &10_001,
            &DEFAULT_BAD_DEBT_LOCK_D,
        ),
        Err(Ok(MCError::InvalidMarketConfigOrUpdate)),
        "min_collateral_value_cents should be capped at MAX_COLLATERAL_VALUE_CENTS (10_000)"
    );

    assert!(
        contract_client
            .try_queue_in_market_update(&MAX_RESERVES, &10_000, &DEFAULT_BAD_DEBT_LOCK_D)
            .is_ok(),
        "10_000 cents ($100) should be accepted"
    );
}

#[test]
fn test_bad_debt_lock_d_validation_on_market_update() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    assert_eq!(
        contract_client.try_queue_in_market_update(&MAX_RESERVES, &0, &(MAX_BAD_DEBT_LOCK_D + 1),),
        Err(Ok(MCError::InvalidMarketConfigOrUpdate)),
        "Value exceeding MAX_BAD_DEBT_LOCK_D should be rejected"
    );

    assert!(
        contract_client.try_queue_in_market_update(&MAX_RESERVES, &0, &0).is_ok(),
        "0 (no lock) should be accepted"
    );
}

#[test]
fn test_apply_market_update_sets_bad_debt_lock_d() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    const NEW_LOCK_DURATION: u64 = 6 * 3600; // 6 hours

    contract_client.queue_in_market_update(&MAX_RESERVES, &0, &NEW_LOCK_DURATION);

    let queued = contract_client.get_market_queued_in_update();
    assert_eq!(queued.new_bad_debt_lock_d, NEW_LOCK_DURATION);

    let update_in_queue_period = contract_client.get_global_state().update_in_queue_period;
    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);
    contract_client.apply_market_update();

    // Queue a second update with a different value to confirm the first took effect
    contract_client.queue_in_market_update(&MAX_RESERVES, &0, &0);
    let queued2 = contract_client.get_market_queued_in_update();
    assert_eq!(queued2.new_bad_debt_lock_d, 0);

    e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);
    contract_client.apply_market_update();
}

#[test]
fn test_remove_and_add_collateral_allowed_when_frozen() {
    let TestMarketFixture { contract_client, gold_pool_address, users, usdc_pool_address, .. } =
        TestMarketFixture::new();
    let depositor = &users[0];
    let liquidity_provider = &users[2];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.add_collateral(
        &ObligationKey::new(depositor.clone()),
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    );

    contract_client.update_market_status(&(MarketStatus::Frozen as u32));

    contract_client.remove_collateral(
        &ObligationKey::new(depositor.clone()),
        &gold_pool_address,
        &1,
        &None,
    );

    contract_client.add_collateral(
        &ObligationKey::new(depositor.clone()),
        &gold_pool_address,
        &1,
        &None,
    );

    contract_client.withdraw(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &1,
        &None,
    );
}

#[test]
fn test_frozen_by_admin_blocks_deposit_and_borrow() {
    let TestMarketFixture { contract_client, gold_pool_address, users, usdc_pool_address, .. } =
        TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[2];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    );
    contract_client.borrow(&ObligationKey::new(borrower.clone()), &usdc_pool_address, &100, &None);

    contract_client.update_market_status(&(MarketStatus::FrozenByAdmin as u32));

    assert_eq!(
        contract_client.try_deposit(
            &ObligationKey::new(liquidity_provider.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::DepositForbiddenOnMarket))
    );
    assert_eq!(
        contract_client.try_borrow(
            &ObligationKey::new(borrower.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::BorrowForbiddenOnMarket))
    );

    contract_client.repay(&ObligationKey::new(borrower.clone()), &usdc_pool_address, &50, &None);

    contract_client.withdraw(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &1,
        &None,
    );
}
