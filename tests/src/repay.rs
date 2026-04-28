#![cfg(test)]

use market::{
    constants::*,
    contract::{MarketClient, MarketContract},
    error::MCError,
    obligation::ObligationKey,
    pool::PoolConfig,
    storage::MarketInitParams,
};
use sep_40_oracle::testutils::{Asset, MockPriceOracleClient, MockPriceOracleWASM};
use soroban_sdk::{
    Address, Env, String, Symbol, contract, contractimpl, contracttype,
    testutils::{Address as _, Ledger},
    token::TokenClient,
};

use crate::{
    DEFAULT_BAD_DEBT_LOCK_D, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, assert_approx_eq_abs,
    get_default_env,
    get_obligation_d_tokens_as_tokens, get_obligation_initially_borrowed,
    get_obligation_unpaid_interest, get_pool_total_available, get_pool_total_borrowed, setup_test_asset,
};

#[contracttype]
#[derive(Clone)]
enum ZeroRejectTokenDataKey {
    Balance(Address),
    Decimals,
}

#[contract]
struct ZeroRejectToken;

#[contractimpl]
impl ZeroRejectToken {
    pub fn __constructor(e: Env, decimals: u32) {
        e.storage().instance().set(&ZeroRejectTokenDataKey::Decimals, &decimals);
    }

    pub fn mint(e: Env, to: Address, amount: i128) {
        let key = ZeroRejectTokenDataKey::Balance(to);
        let balance: i128 = e.storage().persistent().get(&key).unwrap_or(0);
        e.storage().persistent().set(&key, &(balance + amount));
    }

    pub fn balance(e: Env, id: Address) -> i128 {
        e.storage().persistent().get(&ZeroRejectTokenDataKey::Balance(id)).unwrap_or(0)
    }

    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        if amount == 0 {
            panic!("zero transfer rejected");
        }

        let from_key = ZeroRejectTokenDataKey::Balance(from);
        let to_key = ZeroRejectTokenDataKey::Balance(to);
        let from_balance: i128 = e.storage().persistent().get(&from_key).unwrap_or(0);
        let to_balance: i128 = e.storage().persistent().get(&to_key).unwrap_or(0);

        e.storage().persistent().set(&from_key, &(from_balance - amount));
        e.storage().persistent().set(&to_key, &(to_balance + amount));
    }

    pub fn decimals(e: Env) -> u32 {
        e.storage().instance().get(&ZeroRejectTokenDataKey::Decimals).unwrap()
    }

    pub fn name(e: Env) -> String {
        String::from_str(&e, "ZeroRejectToken")
    }

    pub fn symbol(e: Env) -> String {
        String::from_str(&e, "ZRT")
    }
}

#[test]
fn test_repay() {
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

    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    // Borrow 50% of the available
    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
    );

    // Repay the half of the debt

    let borrower_balance_before = usdc_token_client.balance(borrower);
    contract_client.repay(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 4),
        &None,
    );
    let borrower_balance_after = usdc_token_client.balance(borrower);

    assert_eq!(
        borrower_balance_before.checked_sub(borrower_balance_after).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT / 4
    );

    let obligation_borrowed =
        get_obligation_initially_borrowed(&contract_client, borrower, &usdc_pool_address).unwrap();
    let obligation_d_tokens_as_tokens =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    assert_eq!(obligation_borrowed, DEFAULT_DEPOSIT_AMOUNT / 4);
    assert_eq!(obligation_d_tokens_as_tokens, DEFAULT_DEPOSIT_AMOUNT / 4);

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert_eq!(pool_total_borrowed, DEFAULT_DEPOSIT_AMOUNT / 4);
    assert_eq!(pool_total_available, (3 * DEFAULT_DEPOSIT_AMOUNT) / 4);

    // Repay the rest
    contract_client.repay(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 4),
        &None,
    );

    assert_eq!(
        get_obligation_initially_borrowed(&contract_client, borrower, &usdc_pool_address),
        Err(MCError::BorrowPositionDoesNotExist)
    );

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert_eq!(pool_total_borrowed, 0);
    assert_eq!(pool_total_available, DEFAULT_DEPOSIT_AMOUNT);
}

#[test]
fn test_repay_zero() {
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
    );

    assert_eq!(
        contract_client.try_repay(
            &ObligationKey::new(borrower.clone()),
            &usdc_pool_address,
            &0,
            &None
        ),
        Err(Ok(MCError::InvalidInputAmount))
    );
}

#[test]
fn test_repay_exact_amount_fails_if_token_rejects_zero_transfer() {
    const ORACLE_ADDRESS: &str = "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";
    const TOKEN_DECIMALS: u32 = 7;
    const ORACLE_PRICE_DECIMALS: u32 = 14;

    let e = get_default_env();
    e.mock_all_auths();

    let borrower = Address::generate(&e);
    let liquidity_provider = Address::generate(&e);
    let collateral_admin = Address::generate(&e);
    let contract_admin = Address::generate(&e);

    let collateral_asset = setup_test_asset(&e, &collateral_admin, &vec![borrower.clone()]);

    let borrow_token_address = e.register(ZeroRejectToken, (TOKEN_DECIMALS,));
    let borrow_token_client = TokenClient::new(&e, &borrow_token_address);
    let borrow_token_mock = ZeroRejectTokenClient::new(&e, &borrow_token_address);
    borrow_token_mock.mint(&liquidity_provider, &(10 * DEFAULT_DEPOSIT_AMOUNT));

    let oracle = Address::from_str(&e, ORACLE_ADDRESS);
    e.register_at(&oracle, MockPriceOracleWASM, ());
    let oracle_client = MockPriceOracleClient::new(&e, &oracle);

    let insurance_fund = Address::generate(&e);
    let market_manager_address = Address::generate(&e);

    let market_id = e.register(
        MarketContract,
        (
            String::from_str(&e, "market_contract"),
            contract_admin.clone(),
            oracle.clone(),
            insurance_fund,
            market_manager_address,
            MarketInitParams {
                max_positions: DEFAULT_MAX_POSITIONS,
                min_collateral_value_cents: 0i128,
                insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                is_owned: true,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        ),
    );
    let market = MarketClient::new(&e, &market_id);
    market.update_market_status(&0);

    market.queue_in_pool_set(&collateral_asset.token_address, &PoolConfig::default());
    e.ledger().with_mut(|li| li.timestamp += DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);
    market.apply_pool_set(&collateral_asset.token_address);
    let collateral_pool = collateral_asset.token_address.clone();

    market.queue_in_pool_set(&borrow_token_address, &PoolConfig::default());
    e.ledger().with_mut(|li| li.timestamp += DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);
    market.apply_pool_set(&borrow_token_address);
    let borrow_pool = borrow_token_address.clone();

    oracle_client.set_data(
        &contract_admin,
        &Asset::Other(Symbol::new(&e, "USD")),
        &soroban_sdk::vec![&e, Asset::Stellar(collateral_pool.clone()), Asset::Stellar(borrow_pool.clone())],
        &ORACLE_PRICE_DECIMALS,
        &123,
    );
    oracle_client.set_price_stable(&soroban_sdk::vec![
        &e,
        10_i128.pow(ORACLE_PRICE_DECIMALS),
        10_i128.pow(ORACLE_PRICE_DECIMALS),
    ]);

    market.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &collateral_pool,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    market.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &borrow_pool,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    market.borrow(
        &ObligationKey::new(borrower.clone()),
        &borrow_pool,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
    );

    let exact_debt =
        get_obligation_d_tokens_as_tokens(&e, &market, &borrower, &borrow_pool).unwrap();
    let result = market.try_repay(
        &ObligationKey::new(borrower.clone()),
        &borrow_pool,
        &exact_debt,
        &None,
    );

    assert!(matches!(result, Err(Err(_))), "expected host error, got: {:?}", result);
    assert!(borrow_token_client.balance(&borrower) >= 0);
}

#[test]
fn test_repay_with_interest_accrual() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
    );

    // -- Move time --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR / 12;
    });
    contract_client.refresh_pool(&usdc_pool_address);

    let unpaid_interest =
        get_obligation_unpaid_interest(&e, &contract_client, borrower, &usdc_pool_address).unwrap();

    contract_client.repay(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
    );

    let remaining_debt =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    assert_approx_eq_abs(remaining_debt, unpaid_interest, 10);
}

#[test]
fn test_repay_unpaid_interest_only() {
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

    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
    );

    // -- Move time --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR / 12;
    });
    contract_client.refresh_pool(&usdc_pool_address);

    let obligation_unpaid_interest_before =
        get_obligation_unpaid_interest(&e, &contract_client, borrower, &usdc_pool_address).unwrap();

    let borrower_balance_before = usdc_token_client.balance(borrower);
    contract_client.repay(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &obligation_unpaid_interest_before,
        &None,
    );
    let borrower_balance_after = usdc_token_client.balance(borrower);

    assert_eq!(
        borrower_balance_before.checked_sub(borrower_balance_after).unwrap(),
        obligation_unpaid_interest_before
    );

    let obligation_unpaid_interest_after =
        get_obligation_unpaid_interest(&e, &contract_client, borrower, &usdc_pool_address).unwrap();

    assert_approx_eq_abs(obligation_unpaid_interest_after, 0, 1);
}

#[test]
fn test_repay_all_with_bigger_than_debt_value() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
    );

    let borrower_balance_before = usdc_token_client.balance(borrower);
    contract_client.repay(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(3 * DEFAULT_DEPOSIT_AMOUNT / 2), // x3 of borrowed amount
        &None,
    );
    let borrower_balance_after = usdc_token_client.balance(borrower);

    assert_eq!(
        borrower_balance_before.checked_sub(borrower_balance_after).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT / 2
    );

    assert_eq!(
        get_obligation_initially_borrowed(&contract_client, borrower, &usdc_pool_address),
        Err(MCError::BorrowPositionDoesNotExist)
    );

    let pool_total_available = get_pool_total_available(&contract_client, &usdc_pool_address);
    let pool_total_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);

    assert_eq!(pool_total_borrowed, 0);
    assert_eq!(pool_total_available, (2 * DEFAULT_DEPOSIT_AMOUNT));
}

#[test]
#[ignore]
fn test_consecutive_borrows_can_lead_to_unpaid_interest_become_negative() {
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let liquidity_provider = &users[0];
    let borrower_1 = &users[1];
    let borrower_2 = &users[2];
    let borrower_3 = &users[3];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100000000000 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    contract_client.add_collateral(
        &ObligationKey::new(borrower_1.clone()),
        &gold_pool_address,
        &7777777,
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(borrower_1.clone()),
        &usdc_pool_address,
        &i128::MAX,
        &None,
    );

    contract_client.add_collateral(
        &ObligationKey::new(borrower_2.clone()),
        &gold_pool_address,
        &177777,
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(borrower_2.clone()),
        &usdc_pool_address,
        &i128::MAX,
        &None,
    );

    contract_client.add_collateral(
        &ObligationKey::new(borrower_3.clone()),
        &gold_pool_address,
        &5325523,
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(borrower_3.clone()),
        &usdc_pool_address,
        &i128::MAX,
        &None,
    );

    // NB: Consecutive borrows can lead to 'unpaid_interest_becomes_negative' internal error when repaying the first borrow
    // right away. This is a consequence of generating an amount of dTokens with ceiling rounding to favour the protocol when borrowing
    assert_eq!(
        contract_client.try_repay(
            &ObligationKey::new(borrower_1.clone()),
            &usdc_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::InternalError))
    );
}
