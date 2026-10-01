#![cfg(test)]

use market::{
    constants::{
        DEFAULT_BAD_DEBT_LOCK_D, DEFAULT_INSOLVENCY_LTV_BPS,
        DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS, MAX_RESERVES,
    },
    contract::{MarketClient, MarketContract},
    error::MCError,
    pool::{PoolConfig, PoolFeeConfig, PoolHealthConfig},
    storage::MarketInitParams,
};
use soroban_sdk::{
    Address, String, contract, contractimpl,
    testutils::{Address as _, Ledger},
};

use crate::{get_default_env, register_random_sac, setup_market_client};

#[test]
fn test_pool_set_new() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    contract_client.queue_in_pool_set(&token_address, &PoolConfig::default());

    e.ledger().with_mut(|li| li.timestamp += DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);

    contract_client.apply_pool_set(&token_address);

    let all_pools = contract_client.get_all_pools();
    assert_eq!(all_pools.len(), 1);
    assert_eq!(all_pools.last().unwrap(), token_address);
}

#[test]
fn test_pool_set_new_with_custom_config() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    let pool_config = PoolConfig {
        health_config: PoolHealthConfig { utilization_ratio_limit_bps: 8000, ..Default::default() },
        ..Default::default()
    };

    contract_client.queue_in_pool_set(&token_address, &pool_config);

    e.ledger().with_mut(|li| li.timestamp += DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);

    contract_client.apply_pool_set(&token_address);

    let all_pools = contract_client.get_all_pools();
    assert_eq!(all_pools.len(), 1);
    assert_eq!(all_pools.last().unwrap(), token_address.clone());

    let pool = contract_client.get_pool(&token_address);
    assert_eq!(pool.config, pool_config);
}

#[test]
fn test_queue_pool_set_for_existing_pool_on_unowned_market_fails() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, false);

    let token_address = register_random_sac(&e);

    contract_client.queue_in_pool_set(&token_address, &PoolConfig::default());

    e.ledger().with_mut(|li| li.timestamp += DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);

    contract_client.apply_pool_set(&token_address);

    let new_pool_config = PoolConfig {
        health_config: PoolHealthConfig { supply_limit: 100, ..Default::default() },
        ..Default::default()
    };

    assert_eq!(
        contract_client.try_queue_in_pool_set(&token_address, &new_pool_config),
        Err(Ok(MCError::MarketIsNotOwned))
    );
}

#[test]
fn test_pool_set_rejects_100_percent_fee() {
    let e = get_default_env();
    let contract_client = setup_market_client(&e, true);

    let token_address = register_random_sac(&e);

    let pool_config = PoolConfig {
        fee_config: PoolFeeConfig { borrow_fee_bps: 10_000, ..Default::default() },
        ..Default::default()
    };

    assert_eq!(
        contract_client.try_queue_in_pool_set(&token_address, &pool_config),
        Err(Ok(MCError::InvalidLoanPoolConfig))
    );
}

#[test]
#[should_panic]
fn test_constructor_rejects_invalid_min_collateral_value_cents() {
    let e = get_default_env();

    e.register(
        MarketContract,
        (
            &String::from_str(&e, "test"),
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            MarketInitParams {
                max_positions: MAX_RESERVES,
                min_collateral_value_cents: 10_001,
                insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                is_owned: false,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        ),
    );
}

#[test]
fn test_constructor_accepts_boundary_min_collateral_value_cents() {
    let e = get_default_env();

    let market_addr = e.register(
        MarketContract,
        (
            &String::from_str(&e, "test"),
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            MarketInitParams {
                max_positions: MAX_RESERVES,
                min_collateral_value_cents: 10_000,
                insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                is_owned: false,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        ),
    );

    let client = MarketClient::new(&e, &market_addr);
    assert_eq!(client.get_global_state().min_collateral_value_cents, 10_000);
}

/// **A market cannot be constructed with a timelock that defeats its own queue.** With a period of
/// `0`, `apply_market_update` and `apply_pool_set` succeed in the ledger the change was queued in.
#[test]
#[should_panic]
fn test_constructor_rejects_a_zero_update_in_queue_period() {
    let e = get_default_env();

    e.register(
        MarketContract,
        (
            &String::from_str(&e, "test"),
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            MarketInitParams {
                max_positions: MAX_RESERVES,
                min_collateral_value_cents: 0,
                insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                update_in_queue_period: 0,
                is_owned: false,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        ),
    );
}

/// The other end: a period near `u64::MAX` overflows the applicability test on any real ledger and jams
/// both applies permanently, so it is refused too.
#[test]
#[should_panic]
fn test_constructor_rejects_an_unapplicable_update_in_queue_period() {
    let e = get_default_env();

    e.register(
        MarketContract,
        (
            &String::from_str(&e, "test"),
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            MarketInitParams {
                max_positions: MAX_RESERVES,
                min_collateral_value_cents: 0,
                insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                update_in_queue_period: u64::MAX,
                is_owned: false,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        ),
    );
}

/// The floor itself is accepted, so the bound is a bound and not a ban.
#[test]
fn test_constructor_accepts_the_minimum_update_in_queue_period() {
    let e = get_default_env();

    e.register(
        MarketContract,
        (
            &String::from_str(&e, "test"),
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            MarketInitParams {
                max_positions: MAX_RESERVES,
                min_collateral_value_cents: 0,
                insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                update_in_queue_period: market::constants::MIN_UPDATE_IN_QUEUE_SECONDS,
                is_owned: false,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        ),
    );
}

/// A feed whose scale overflows `i128::pow(10, decimals)`: the host traps on every priced
/// operation, and the oracle address cannot be changed after construction.
#[contract]
pub struct OverScaledOracle;

#[contractimpl]
impl OverScaledOracle {
    pub fn decimals(_e: soroban_sdk::Env) -> u32 {
        39
    }
}

#[test]
#[should_panic]
fn test_constructor_rejects_a_feed_whose_scale_overflows() {
    let e = get_default_env();
    let oracle = e.register(OverScaledOracle, ());

    e.register(
        MarketContract,
        (
            &String::from_str(&e, "test"),
            &Address::generate(&e),
            &oracle,
            &Address::generate(&e),
            &Address::generate(&e),
            MarketInitParams {
                max_positions: MAX_RESERVES,
                min_collateral_value_cents: 0,
                insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                is_owned: false,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        ),
    );
}

/// A feed that does not answer yet is **accepted**: the fixture itself constructs the market before
/// configuring the feed, and that state is recoverable from the feed's side.
#[test]
fn test_constructor_accepts_an_oracle_that_does_not_answer_yet() {
    let e = get_default_env();

    e.register(
        MarketContract,
        (
            &String::from_str(&e, "test"),
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            &Address::generate(&e),
            MarketInitParams {
                max_positions: MAX_RESERVES,
                min_collateral_value_cents: 0,
                insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                is_owned: false,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        ),
    );
}
