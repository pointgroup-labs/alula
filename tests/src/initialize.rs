#![cfg(test)]

use market::{
    error::MCError,
    pool::{PoolConfig, PoolFeeConfig, PoolHealthConfig},
};

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
