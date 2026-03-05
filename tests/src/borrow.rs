#![cfg(test)]

use market::{
    constants::*,
    contract::{MarketClient, MarketContract},
    error::MCError,
    obligation::ObligationKey,
    pool::{PoolConfig, PoolHealthConfig},
};
use sep_40_oracle::testutils::{Asset, MockPriceOracleClient, MockPriceOracleWASM};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, Env, String, Symbol, contract, contractimpl, contracttype,
    testutils::{Address as _, Ledger},
    token::TokenClient,
};

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, assert_approx_eq_abs,
    assert_approx_eq_rel, get_default_env, get_obligation_d_tokens_as_tokens,
    get_obligation_initially_borrowed, get_pool_fee_config, get_pool_total_available,
    get_pool_total_borrowed, setup_test_asset,
};

#[test]
fn test_borrow() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    // NB: GOLD is used as the main collateral in integration tests
    contract_client.deposit(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    // NB: USDC is used as the main borrowed token in integration tests
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let borrower_balance_before = usdc_token_client.balance(borrower);
    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    let borrower_balance_after = usdc_token_client.balance(borrower);

    let borrow_fee_bps = get_pool_fee_config(&contract_client, &usdc_pool_address).borrow_fee_bps;

    assert_eq!(
        borrower_balance_after.checked_sub(borrower_balance_before).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT
            .fixed_mul_ceil(BPS_FACTOR - borrow_fee_bps as i128, BPS_FACTOR)
            .unwrap()
    );

    let obligation_borrowed =
        get_obligation_initially_borrowed(&contract_client, borrower, &usdc_pool_address).unwrap();
    let obligation_d_tokens_as_tokens =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    assert_approx_eq_abs(obligation_borrowed, DEFAULT_DEPOSIT_AMOUNT, 2);
    assert_eq!(obligation_d_tokens_as_tokens, DEFAULT_DEPOSIT_AMOUNT);

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert_eq!(pool_total_borrowed, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_total_available, DEFAULT_DEPOSIT_AMOUNT);
}

#[test]
fn test_borrow_multiple_shareholders() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    let borrower_1 = &users[0];
    let borrower_2 = &users[1];
    let liquidity_provider = &users[2];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(3 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(borrower_1.clone()),
        &gold_pool_address,
        &(3 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(borrower_2.clone()),
        &gold_pool_address,
        &(3 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    contract_client.borrow(
        &ObligationKey::new(borrower_1.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    const BORROWER_2_BORROW_AMOUNT: i128 = (3 * DEFAULT_DEPOSIT_AMOUNT) / 2;
    contract_client.borrow(
        &ObligationKey::new(borrower_2.clone()),
        &usdc_pool_address,
        &BORROWER_2_BORROW_AMOUNT,
        &None,
    );

    let obligation_d_tokens_as_tokens_1 =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower_1, &usdc_pool_address)
            .unwrap();

    assert_eq!(obligation_d_tokens_as_tokens_1, DEFAULT_DEPOSIT_AMOUNT);
    let obligation_d_tokens_as_tokens_2 =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower_2, &usdc_pool_address)
            .unwrap();

    assert_eq!(obligation_d_tokens_as_tokens_2, BORROWER_2_BORROW_AMOUNT);

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    const BORROWED: i128 = DEFAULT_DEPOSIT_AMOUNT + BORROWER_2_BORROW_AMOUNT;

    assert_eq!(pool_total_borrowed, BORROWED);
    assert_eq!(pool_total_borrowed, BORROWED);
    assert_eq!(pool_total_available, (3 * DEFAULT_DEPOSIT_AMOUNT) - BORROWED);

    // -- Accrue debt on the pool --

    // - Wait 1 month -
    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR / 12;
    });
    contract_client.refresh_pool(&usdc_pool_address);

    // - Assert that the total debt has increased -

    let obligation_d_tokens_as_tokens_1 =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower_1, &usdc_pool_address)
            .unwrap();

    assert!(obligation_d_tokens_as_tokens_1 > DEFAULT_DEPOSIT_AMOUNT);

    let obligation_d_tokens_as_tokens_2 =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower_2, &usdc_pool_address)
            .unwrap();

    assert!(obligation_d_tokens_as_tokens_2 > BORROWER_2_BORROW_AMOUNT);

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert!(pool_total_borrowed > BORROWED);
    assert_eq!(pool_total_available, (3 * DEFAULT_DEPOSIT_AMOUNT) - BORROWED);
}
#[test]
fn test_borrow_exceeds_utilization_cap() {
    const UTILIZATION_RATIO_LIMIT_BPS: i128 = 9000; // 90%
    const BORROW_AMOUNT: i128 = (DEFAULT_DEPOSIT_AMOUNT * UTILIZATION_RATIO_LIMIT_BPS) / BPS_FACTOR;

    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: UTILIZATION_RATIO_LIMIT_BPS,
            ..Default::default()
        },
        ..Default::default()
    };

    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &(2 * &DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    let borrower_balance_before = usdc_token_client.balance(borrower);
    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &BORROW_AMOUNT,
        &None,
    );
    let borrower_balance_after = usdc_token_client.balance(borrower);

    let borrow_fee_bps = get_pool_fee_config(&contract_client, &usdc_pool_address).borrow_fee_bps;

    assert_eq!(
        borrower_balance_after.checked_sub(borrower_balance_before).unwrap(),
        BORROW_AMOUNT.fixed_mul_ceil(BPS_FACTOR - borrow_fee_bps as i128, BPS_FACTOR).unwrap()
    );

    assert_eq!(
        contract_client.try_borrow(
            &ObligationKey::new(borrower.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::PoolUtilizationRatioCapExceeded))
    );
}

#[test]
fn test_borrow_zero() {
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    assert_eq!(
        contract_client.try_borrow(
            &ObligationKey::new(borrower.clone()),
            &usdc_pool_address,
            &0,
            &None
        ),
        Err(Ok(MCError::InvalidInputAmount))
    );
}

#[test]
fn test_borrow_negative() {
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    assert_eq!(
        contract_client.try_borrow(
            &ObligationKey::new(borrower.clone()),
            &usdc_pool_address,
            &-1,
            &None
        ),
        Err(Ok(MCError::InvalidInputAmount))
    );
}

#[test]
fn test_deposit_exists() {
    let TestMarketFixture { contract_client, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let borrower = &users[0];

    contract_client.deposit(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    assert_eq!(
        contract_client.try_borrow(
            &ObligationKey::new(borrower.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::DepositPositionForAssetExists))
    );
}

#[test]
fn test_borrow_amount_is_reduced_to_satisfy_obligation_health() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    let borrower_balance_before = usdc_token_client.balance(borrower);
    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
    );
    let borrower_balance_after = usdc_token_client.balance(borrower);

    let borrow_fee_bps = get_pool_fee_config(&contract_client, &usdc_pool_address).borrow_fee_bps;

    assert!(
        borrower_balance_after.checked_sub(borrower_balance_before).unwrap()
            < DEFAULT_DEPOSIT_AMOUNT
                .fixed_mul_ceil(BPS_FACTOR - borrow_fee_bps as i128, BPS_FACTOR)
                .unwrap()
    );

    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &i128::MAX,
        &None,
    );

    let obligation_borrowed =
        get_obligation_initially_borrowed(&contract_client, borrower, &usdc_pool_address).unwrap();
    let obligation_d_tokens_as_tokens =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    const MAX_HEALTHY_BORROW_AMOUNT: i128 =
        (DEFAULT_OPEN_LTV_BPS * DEFAULT_DEPOSIT_AMOUNT) / BPS_FACTOR;

    assert_approx_eq_abs(obligation_borrowed, MAX_HEALTHY_BORROW_AMOUNT, 2);
    assert_eq!(obligation_d_tokens_as_tokens, MAX_HEALTHY_BORROW_AMOUNT);

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert_eq!(pool_total_borrowed, MAX_HEALTHY_BORROW_AMOUNT);
    assert_eq!(pool_total_borrowed, MAX_HEALTHY_BORROW_AMOUNT);
    assert_eq!(pool_total_available, DEFAULT_DEPOSIT_AMOUNT - MAX_HEALTHY_BORROW_AMOUNT);
}

#[test]
fn test_borrow_w_big_liability_factor() {
    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            liability_factor_bps: 2 * BPS_FACTOR, // 200%
            ..Default::default()
        },
        ..Default::default()
    };
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let borrower = &users[0];
    let liquidity_provider = &users[1];

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
    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &i128::MAX,
        &None,
    );

    let borrowed =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    assert!(borrowed < DEFAULT_COLLATERAL_AMOUNT / 2);
}

// -- Non SAC token borrows --

#[contracttype]
#[derive(Clone)]
enum MockTokenDataKey {
    Balance(Address),
    Decimals,
}

#[contract]
struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn __constructor(e: Env, decimals: u32) {
        e.storage().instance().set(&MockTokenDataKey::Decimals, &decimals);
    }

    pub fn mint(e: Env, to: Address, amount: i128) {
        let key = MockTokenDataKey::Balance(to);
        let balance: i128 = e.storage().persistent().get(&key).unwrap_or(0);

        e.storage().persistent().set(&key, &(balance + amount));
    }

    pub fn balance(e: Env, id: Address) -> i128 {
        e.storage().persistent().get(&MockTokenDataKey::Balance(id)).unwrap_or(0)
    }

    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        let from_key = MockTokenDataKey::Balance(from);
        let to_key = MockTokenDataKey::Balance(to);

        let from_balance: i128 = e.storage().persistent().get(&from_key).unwrap_or(0);
        let to_balance: i128 = e.storage().persistent().get(&to_key).unwrap_or(0);

        e.storage().persistent().set(&from_key, &(from_balance - amount));
        e.storage().persistent().set(&to_key, &(to_balance + amount));
    }

    pub fn decimals(e: Env) -> u32 {
        e.storage().instance().get(&MockTokenDataKey::Decimals).unwrap()
    }

    pub fn name(_e: Env) -> String {
        String::from_str(&_e, "MockToken")
    }

    pub fn symbol(_e: Env) -> String {
        String::from_str(&_e, "MOCK")
    }
}

/// Verifies that borrow amount limiting works correctly when collateral and borrowed
/// assets use different token decimals (e.g., 7-decimal SAC collateral vs 18-decimal
/// borrowed token). The protocol must properly normalize values through the oracle
/// to compute LTV regardless of per-asset decimal precision
#[test]
fn test_borrow_w_different_token_decimals() {
    const ORACLE_ADDRESS: &str = "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";
    const BORROWED_TOKEN_DECIMALS: u32 = 18;
    const ORACLE_PRICE_DECIMALS: u32 = 14;

    let e = get_default_env();
    e.mock_all_auths();

    let borrower = Address::generate(&e);
    let contract_admin = Address::generate(&e);
    let liquidity_provider = Address::generate(&e);

    // - Collateral asset: standard 7-decimal SAC -

    let collateral_admin = Address::generate(&e);
    let collateral_asset = setup_test_asset(
        &e,
        &collateral_admin,
        &vec![borrower.clone(), liquidity_provider.clone()],
    );

    // - Borrowed asset: 18-decimal mock token -

    let borrowed_token_address = e.register(MockToken, (BORROWED_TOKEN_DECIMALS,));
    let borrowed_token_client = TokenClient::new(&e, &borrowed_token_address);
    let mint_amount = i128::MAX / 1024;

    MockTokenClient::new(&e, &borrowed_token_address).mint(&borrower, &mint_amount);
    MockTokenClient::new(&e, &borrowed_token_address).mint(&liquidity_provider, &mint_amount);

    // - Oracle: both assets priced at $1 with 14 decimals -

    let oracle = Address::from_str(&e, ORACLE_ADDRESS);
    e.register_at(&oracle, MockPriceOracleWASM, ());
    let oracle_client = MockPriceOracleClient::new(&e, &oracle);

    // - Market contract -

    let router_address = Address::generate(&e);
    let insurance_fund = Address::generate(&e);
    let market_manager_address = Address::generate(&e);
    let contract_name = String::from_str(&e, "market_contract");

    let market_id = e.register(
        MarketContract,
        (
            contract_name,
            contract_admin.clone(),
            oracle.clone(),
            router_address,
            insurance_fund,
            market_manager_address,
            DEFAULT_MAX_POSITIONS,
            0i128,
            DEFAULT_INSOLVENCY_LTV_BPS,
            Some(DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS),
        ),
    );
    let market = MarketClient::new(&e, &market_id);
    market.update_market_status(&0);

    let collateral_pool = market.initialize_pool(&collateral_asset.token_address, &None);
    let borrowed_pool = market.initialize_pool(&borrowed_token_address, &None);

    // Oracle feeds prices for pool addresses (not token addresses)
    oracle_client.set_data(
        &contract_admin,
        &Asset::Other(Symbol::new(&e, "USD")),
        &soroban_sdk::vec![
            &e,
            Asset::Stellar(collateral_pool.clone()),
            Asset::Stellar(borrowed_pool.clone()),
        ],
        &ORACLE_PRICE_DECIMALS,
        &123,
    );
    oracle_client.set_price_stable(&soroban_sdk::vec![
        &e,
        10_i128.pow(ORACLE_PRICE_DECIMALS), // collateral: $1
        10_i128.pow(ORACLE_PRICE_DECIMALS), // borrowed: $1
    ]);

    // - Setup: deposit collateral (7 decimals) and liquidity (18 decimals) -

    let collateral_amount: i128 = 10_000 * 10_i128.pow(7); // 10,000 tokens in 7-decimal
    let liquidity_amount: i128 = 100_000 * 10_i128.pow(BORROWED_TOKEN_DECIMALS); // 100k tokens in 18-decimal

    market.deposit(
        &ObligationKey::new(borrower.clone()),
        &collateral_pool,
        &collateral_amount,
        &None,
    );
    market.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &borrowed_pool,
        &liquidity_amount,
        &None,
    );

    // - Borrow: request way more than LTV allows -

    let borrower_balance_before = borrowed_token_client.balance(&borrower);
    market.borrow(&ObligationKey::new(borrower.clone()), &borrowed_pool, &i128::MAX, &None);
    let borrower_balance_after = borrowed_token_client.balance(&borrower);

    let received = borrower_balance_after.checked_sub(borrower_balance_before).unwrap();

    // With equal prices and default open_ltv of 70%, the max borrow value is:
    //   collateral_value = 10,000 * $1 = $10,000
    //   max_borrow_value = $10,000 * 70% = $7,000
    //   max_borrow_amount = 7,000 tokens = 7,000 * 10^18 (in 18-decimal units)
    let expected_max_borrow = DEFAULT_OPEN_LTV_BPS * 10_i128.pow(BORROWED_TOKEN_DECIMALS);

    let obligation_d_tokens_as_tokens =
        get_obligation_d_tokens_as_tokens(&e, &market, &borrower, &borrowed_pool).unwrap();

    assert_approx_eq_rel(obligation_d_tokens_as_tokens, expected_max_borrow, 1);
    assert_approx_eq_rel(received, expected_max_borrow, 1);

    let pool_total_borrowed = get_pool_total_borrowed(&market, &borrowed_pool);
    assert_approx_eq_rel(pool_total_borrowed, expected_max_borrow, 1);
}
