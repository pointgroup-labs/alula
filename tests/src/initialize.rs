#![cfg(test)]

use market::{
    error::MCError,
    pool::{PoolConfig, PoolFeeConfig, PoolHealthConfig},
};
use soroban_sdk::{Address, testutils::Address as _};

use crate::{get_default_env, register_random_sac, setup_market_client};

#[test]
fn test_pool_initialize() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    let pool_address = contract_client.initialize_pool(&token_address, &None);

    let all_pools = contract_client.get_all_pools();
    assert_eq!(all_pools.len(), 1);
    assert_eq!(all_pools.last().unwrap(), pool_address);
}

#[test]
fn test_pool_initialize_with_custom_config() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    let pool_config = PoolConfig {
        health_config: PoolHealthConfig { utilization_ratio_limit_bps: 8000, ..Default::default() },
        ..Default::default()
    };

    let pool_address = contract_client.initialize_pool(&token_address, &Some(pool_config.clone()));

    let all_pools = contract_client.get_all_pools();
    assert_eq!(all_pools.len(), 1);
    assert_eq!(all_pools.last().unwrap(), pool_address);

    let pool = contract_client.get_pool(&pool_address);
    assert_eq!(pool.config, pool_config);
}

#[test]
fn test_pool_reinitialize_fails() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    contract_client.initialize_pool(&token_address, &None);

    assert_eq!(
        Err(Ok(MCError::InvalidInitialization)),
        contract_client.try_initialize_pool(&token_address, &None),
    );
}

#[test]
fn test_initialize_multiply_pair() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    // Initialize pools first
    let deposit_token_address = register_random_sac(&e);
    let borrow_token_address = register_random_sac(&e);

    let deposit_pool_address = contract_client.initialize_pool(&deposit_token_address, &None);
    let borrow_pool_address = contract_client.initialize_pool(&borrow_token_address, &None);

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
fn test_initialize_multiply_pair_with_same_pools_fails() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let deposit_token_address = register_random_sac(&e);

    let deposit_pool_address = contract_client.initialize_pool(&deposit_token_address, &None);

    assert_eq!(
        contract_client.try_initialize_multiply_pair(&deposit_pool_address, &deposit_pool_address),
        Err(Ok(MCError::InvalidInitialization))
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

    let deposit_pool_address = contract_client.initialize_pool(&deposit_token_address, &None);

    assert_eq!(
        contract_client.try_initialize_multiply_pair(&deposit_pool_address, &borrow_pool_address),
        Err(Ok(MCError::BorrowPoolDoesNotExist))
    );
}

#[test]
fn test_pool_initialize_rejects_100_percent_fee() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    let pool_config = PoolConfig {
        fee_config: PoolFeeConfig { borrow_fee_bps: 10_000, ..Default::default() },
        ..Default::default()
    };

    assert_eq!(
        contract_client.try_initialize_pool(&token_address, &Some(pool_config)),
        Err(Ok(MCError::InvalidLoanPoolConfig))
    );
}
