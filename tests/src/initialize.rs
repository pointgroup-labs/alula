#![cfg(test)]

use market::{
    error::MCError,
    pool::{PoolConfig, PoolHealthConfig},
};
use soroban_sdk::{Address, BytesN, testutils::Address as _};

use crate::{get_default_env, register_random_sac, setup_market_client};

#[test]
fn test_pool_initialize() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    let pool_address_1 = contract_client.initialize_pool(&token_address, &None, &None);

    let all_pools = contract_client.get_all_pools();
    assert_eq!(all_pools.len(), 1);
    assert_eq!(all_pools.last().unwrap(), pool_address_1);

    let pool_address_2 = contract_client.initialize_pool(
        &token_address,
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

    let pool_config = PoolConfig {
        health_config: PoolHealthConfig { utilization_ratio_limit_bps: 8000, ..Default::default() },
        ..Default::default()
    };

    let pool_address =
        contract_client.initialize_pool(&token_address, &None, &Some(pool_config.clone()));

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

    let salt = BytesN::from_array(&e, &[0; 32]);
    let salt2 = BytesN::from_array(&e, &[1; 32]);

    contract_client.initialize_pool(&token_address, &Some(salt), &None);
    contract_client.initialize_pool(&token_address, &Some(salt2), &None);
}

#[test]
fn test_pool_initialize_non_conflicting() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address_1 = register_random_sac(&e);
    let token_address_2 = register_random_sac(&e);

    let salt = BytesN::from_array(&e, &[0; 32]);

    let pool_address_1 = contract_client.initialize_pool(&token_address_1, &None, &None);
    let pool_address_2 =
        contract_client.initialize_pool(&token_address_1, &Some(salt.clone()), &None);

    let pool_address_3 = contract_client.initialize_pool(&token_address_2, &None, &None);
    let pool_address_4 = contract_client.initialize_pool(&token_address_2, &Some(salt), &None);

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

    contract_client.initialize_pool(&token_address, &None, &None);

    assert_eq!(
        Err(Ok(MCError::PoolAlreadyExists)),
        contract_client.try_initialize_pool(&token_address, &None, &None),
    );
}

#[test]
fn test_pool_reinitialize_with_salt() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    let salt = BytesN::from_array(&e, &[0; 32]);

    contract_client.initialize_pool(&token_address, &Some(salt.clone()), &None);

    assert_eq!(
        Err(Ok(MCError::PoolAlreadyExists)),
        contract_client.try_initialize_pool(&token_address, &Some(salt), &None),
    );
}

#[test]
fn test_initialize_multiply_pair() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    // Initialize pools first
    let deposit_token_address = register_random_sac(&e);
    let borrow_token_address = register_random_sac(&e);

    let deposit_pool_address =
        contract_client.initialize_pool(&deposit_token_address, &None, &None);
    let borrow_pool_address = contract_client.initialize_pool(&borrow_token_address, &None, &None);

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

    let deposit_pool_address =
        contract_client.initialize_pool(&deposit_token_address, &None, &None);

    assert_eq!(
        contract_client.try_initialize_multiply_pair(&deposit_pool_address, &borrow_pool_address),
        Err(Ok(MCError::BorrowPoolDoesNotExist))
    );
}
