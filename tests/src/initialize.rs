#![cfg(test)]

use market::{
    constants::DEFAULT_POOL_CONFIG_SEASONING_PERIOD_SECONDS,
    contract::{MarketContract, MarketContractClient},
    error::MCError,
    pool::{PoolConfig, PoolFeeConfig, PoolHealthConfig},
    storage::MarketStatus,
};
use soroban_sdk::{
    Address, BytesN, Env, symbol_short,
    testutils::{Address as _, Ledger},
};

use crate::{get_default_env, get_pool_fee_config};

#[test]
fn test_pool_initialize() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);
    let token_ticker = symbol_short!("TCK1");

    let pool_address_1 =
        contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);

    let all_pools = contract_client.get_all_pools();
    assert_eq!(all_pools.len(), 1);
    assert_eq!(all_pools.last().unwrap(), pool_address_1);

    let pool_address_2 = contract_client.initialize_pool(
        &token_address,
        &token_ticker,
        &Some(BytesN::from_array(&e, &[0; 32])),
        &None,
    );

    let all_pools = contract_client.get_all_pools();
    assert_eq!(all_pools.len(), 2);
    assert_eq!(all_pools.last().unwrap(), pool_address_2);
}

#[test]
fn test_pool_initialize_with_custom_config() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);
    let token_ticker = symbol_short!("TCK1");

    let pool_config = PoolConfig {
        health_config: PoolHealthConfig { utilization_ratio_limit_bps: 8000, ..Default::default() },
        ..Default::default()
    };

    let pool_address =
        contract_client.initialize_pool(&token_address, &token_ticker, &None, &Some(pool_config));

    let all_pools = contract_client.get_all_pools();
    assert_eq!(all_pools.len(), 1);
    assert_eq!(all_pools.last().unwrap(), pool_address);

    let pool = contract_client.get_pool(&pool_address);
    assert_eq!(pool.config, pool_config);
}

#[test]
fn test_pool_initialize_with_different_salt() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);
    let token_ticker = symbol_short!("TCK1");

    let salt = BytesN::from_array(&e, &[0; 32]);
    let salt2 = BytesN::from_array(&e, &[1; 32]);

    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt), &None);
    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt2), &None);
}

#[test]
fn test_pool_initialize_non_conflicting() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address_1 = register_random_sac(&e);
    let token_ticker_1 = symbol_short!("TCK1");

    let token_address_2 = register_random_sac(&e);
    let token_ticker_2 = symbol_short!("TCK2");

    let salt = BytesN::from_array(&e, &[0; 32]);

    let pool_address_1 =
        contract_client.initialize_pool(&token_address_1, &token_ticker_1, &None, &None);
    let pool_address_2 = contract_client.initialize_pool(
        &token_address_1,
        &token_ticker_1,
        &Some(salt.clone()),
        &None,
    );

    let pool_address_3 =
        contract_client.initialize_pool(&token_address_2, &token_ticker_2, &None, &None);
    let pool_address_4 =
        contract_client.initialize_pool(&token_address_2, &token_ticker_2, &Some(salt), &None);

    let all_pools = contract_client.get_all_pools();
    assert_eq!(
        all_pools,
        soroban_sdk::vec![&e, pool_address_1, pool_address_2, pool_address_3, pool_address_4]
    );
}

#[test]
fn test_pool_reinitialize_no_salt() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);
    let token_ticker = symbol_short!("TCK1");

    contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);

    assert_eq!(
        Err(Ok(MCError::PoolAlreadyExists)),
        contract_client.try_initialize_pool(&token_address, &token_ticker, &None, &None),
    );
}

#[test]
fn test_pool_reinitialize_with_salt() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);
    let token_ticker = symbol_short!("TCK1");

    let salt = BytesN::from_array(&e, &[0; 32]);

    contract_client.initialize_pool(&token_address, &token_ticker, &Some(salt.clone()), &None);

    assert_eq!(
        Err(Ok(MCError::PoolAlreadyExists)),
        contract_client.try_initialize_pool(&token_address, &token_ticker, &Some(salt), &None),
    );
}

#[test]
fn test_initialize_multiply_pair() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    // Initialize pools first
    let deposit_token_address = register_random_sac(&e);
    let deposit_token_ticker = symbol_short!("TCK1");

    let borrow_token_address = register_random_sac(&e);
    let borrow_token_ticker = symbol_short!("TCK2");

    let deposit_pool_address = contract_client.initialize_pool(
        &deposit_token_address,
        &deposit_token_ticker,
        &None,
        &None,
    );
    let borrow_pool_address =
        contract_client.initialize_pool(&borrow_token_address, &borrow_token_ticker, &None, &None);

    // Initialize a multiply pair
    contract_client.initialize_multiply_pair(&deposit_pool_address, &borrow_pool_address);

    let all_pairs = contract_client.get_all_multiply_pairs();
    let last_pair = all_pairs.last().unwrap();

    assert_eq!(last_pair.deposit_pool, deposit_pool_address);
    assert_eq!(last_pair.borrow_pool, borrow_pool_address);

    assert_eq!(
        contract_client.try_initialize_multiply_pair(&deposit_pool_address, &borrow_pool_address),
        Err(Ok(MCError::MultiplyPairAlreadyExists))
    );
}

#[test]
fn test_multiply_pair_with_inexistent_pool() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let deposit_pool_address = Address::generate(&e);
    let borrow_pool_address = Address::generate(&e);

    assert_eq!(
        contract_client.try_initialize_multiply_pair(&deposit_pool_address, &borrow_pool_address),
        Err(Ok(MCError::DepositPoolDoesNotExist))
    );

    let deposit_token_address = register_random_sac(&e);
    let deposit_token_ticker = symbol_short!("TCK1");

    let deposit_pool_address = contract_client.initialize_pool(
        &deposit_token_address,
        &deposit_token_ticker,
        &None,
        &None,
    );

    assert_eq!(
        contract_client.try_initialize_multiply_pair(&deposit_pool_address, &borrow_pool_address),
        Err(Ok(MCError::BorrowPoolDoesNotExist))
    );
}

// TODO: rename test module

#[test]
fn test_queue_in_pool_config_update() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);
    let token_ticker = symbol_short!("TCK1");

    let pool_address = contract_client.initialize_pool(
        &token_address,
        &token_ticker,
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
        Err(Ok(MCError::PoolConfigUpdateIsNotSeasonedYet))
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
    let token_ticker = symbol_short!("TCK1");

    let pool_address = contract_client.initialize_pool(
        &token_address,
        &token_ticker,
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
fn test_cancel_pool_config_update() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);
    let token_ticker = symbol_short!("TCK1");

    let pool_address = contract_client.initialize_pool(
        &token_address,
        &token_ticker,
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
        contract_client.get_pool_config_update(&pool_address).new_config.health_config.supply_limit,
        NEW_SUPPLY_LIMIT
    );

    contract_client.cancel_pool_config_update(&pool_address);

    assert_eq!(
        contract_client.try_get_pool_config_update(&pool_address),
        Err(Ok(MCError::PoolDoesNotHaveQueuedInConfigUpdate))
    );
}

#[test]
fn update_market_status_fails_for_permissionless_market() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, false);

    let token_address = register_random_sac(&e);
    let token_ticker = symbol_short!("TCK1");

    let pool_address = contract_client.initialize_pool(
        &token_address,
        &token_ticker,
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
fn update_market_status() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, false);

    let token_address = register_random_sac(&e);
    let token_ticker = symbol_short!("TCK1");

    let pool_address = contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);

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

// ---- Helpers ----

fn setup_market_client<'a>(e: &Env, is_owned: bool) -> MarketContractClient<'a> {
    let contract_name = soroban_sdk::String::from_str(e, "market_contract");
    let contract_admin = Address::generate(e);
    let oracle = Address::generate(e);

    let contract_id = e.register(
        MarketContract,
        (
            contract_name,
            contract_admin.clone(),
            oracle,
            contract_admin,
            0u32,
            0i128,
            if is_owned { Some(DEFAULT_POOL_CONFIG_SEASONING_PERIOD_SECONDS) } else { None },
        ),
    );

    let client = MarketContractClient::new(e, &contract_id);

    if is_owned {
        client.update_market_status(&MarketStatus::Active);
    }

    client
}

fn register_random_sac(e: &Env) -> Address {
    let token_admin = Address::generate(e);

    e.register_stellar_asset_contract_v2(token_admin).address()
}
