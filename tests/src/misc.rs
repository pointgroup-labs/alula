#![cfg(test)]

use controlled_insurance_fund::storage::DataKey;
use market::{
    constants::{BPS_FACTOR, LEVERAGE_SCALE, SECONDS_IN_YEAR},
    error::MCError,
    obligation::ObligationKey,
    utils::{MarketData, PoolData},
};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, Env, map as smap,
    testutils::{Address as _, Ledger},
};

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, assert_approx_eq_abs,
    get_obligation_d_tokens, get_obligation_j_tokens, get_obligation_unpaid_interest,
};

fn wait(e: &Env, am: u64) {
    e.ledger().with_mut(|li| {
        li.timestamp += am;
    });
}

#[test]
fn test_computed_interest_is_negative_reproduced() {
    let TestMarketFixture {
        e, users, contract_client, usdc_pool_address, gold_pool_address, ..
    } = TestMarketFixture::new();

    let maksym = &users[0];
    let k1 = &users[1];
    let maksym2 = &users[2];
    let k2 = &users[3];

    contract_client.deposit(maksym, &gold_pool_address, &100000000000, &None);
    wait(&e, 15);
    contract_client.deposit(maksym2, &gold_pool_address, &1000000000, &None);
    wait(&e, 60 + 39);
    contract_client.get_market_data();
    wait(&e, 25);
    contract_client.get_pool_data(&usdc_pool_address);
    wait(&e, 35);
    contract_client.deposit(maksym2, &usdc_pool_address, &1000000, &None);
    wait(&e, 10);
    contract_client.borrow(maksym, &usdc_pool_address, &10000, &None);
    wait(&e, 10);
    contract_client.get_market_data();
    wait(&e, (2 * 60) + 20);
    contract_client.get_market_data();
    wait(&e, (3 * 60 * 60) + 35 * 60);
    contract_client.deposit(k1, &gold_pool_address, &110000000, &None);
    wait(&e, 30);
    contract_client.deposit(k1, &usdc_pool_address, &220000000, &None);
    wait(&e, 60 * 60 + 60 * 5 + 29);
    contract_client.withdraw(k1, &usdc_pool_address, &220000000, &None);
    wait(&e, 60 * 2 + 5);
    contract_client.deposit(k2, &usdc_pool_address, &1000000000, &None);
    wait(&e, 35);
    contract_client.add_collateral(k1, &gold_pool_address, &1000000, &None);
    contract_client.borrow(k1, &usdc_pool_address, &77367249, &None);
    wait(&e, 27 * 60);
    contract_client.deposit(k1, &gold_pool_address, &120000000, &None);
    wait(&e, (2 * 60) + 20);
    contract_client.deposit(k1, &gold_pool_address, &10000000, &None);
    wait(&e, (10 * 60) + 36);
    contract_client.deposit(k1, &gold_pool_address, &10000000, &None);
    wait(&e, 60 + 45);
    contract_client.deposit(k1, &gold_pool_address, &10000000, &None);
    wait(&e, (2 * 60) + 31);
    contract_client.deposit(k1, &gold_pool_address, &550000000, &None);
    wait(&e, 15);
    contract_client.deposit(k1, &gold_pool_address, &2220000000, &None);
    wait(&e, 25);
    contract_client.deposit(k1, &gold_pool_address, &670000000, &None);
    wait(&e, 20);
    contract_client.deposit(k1, &gold_pool_address, &1000000000, &None);
    wait(&e, (3 * 60) + 25);
    contract_client.withdraw(k1, &gold_pool_address, &i128::MAX, &None);
    wait(&e, (28 * 60) + 42);
    contract_client.deposit(k1, &gold_pool_address, &23330000000, &None);
    wait(&e, (4 * 60) + 15);
    contract_client.deposit(k1, &gold_pool_address, &220000000, &None);
    wait(&e, (15 * 60) + 10);
    contract_client.withdraw(k2, &usdc_pool_address, &105000000, &None);
    wait(&e, (12 * 60) + 5);
    contract_client.withdraw(k2, &usdc_pool_address, &773598393, &None);
    wait(&e, 20);
    contract_client.repay(k1, &usdc_pool_address, &81237000, &None);
    wait(&e, 30);
    contract_client.borrow(k2, &gold_pool_address, &53141657, &None);
    wait(&e, (10 * 60) + 41);
    contract_client.deposit(k2, &usdc_pool_address, &110000000, &None);
    wait(&e, 45);
    contract_client.repay(k2, &gold_pool_address, &55798739, &None);
    wait(&e, 25);
    contract_client.withdraw(k2, &usdc_pool_address, &195847114, &None);
    wait(&e, 20);
    contract_client.add_collateral(k2, &usdc_pool_address, &440000000, &None);
    wait(&e, (10 * 60) + 10);
    contract_client.remove_collateral(k2, &usdc_pool_address, &440000000, &None);
    wait(&e, 35);
    contract_client.deposit(k1, &usdc_pool_address, &110000000, &None);
    wait(&e, 20);
    contract_client.withdraw(k1, &usdc_pool_address, &115500801, &None);
    wait(&e, 13 * 60 * 60);
}

#[test]
fn test_computed_interest_is_negative_reproduced_2() {
    let TestMarketFixture {
        e, users, contract_client, usdc_pool_address, gold_pool_address, ..
    } = TestMarketFixture::new();

    let maksym = &users[0];
    let k1 = &users[1];

    contract_client.deposit(maksym, &gold_pool_address, &10000000000, &None);
    wait(&e, 40);
    contract_client.add_collateral(maksym, &usdc_pool_address, &1000000000, &None);
    wait(&e, 55);
    contract_client.remove_collateral(maksym, &usdc_pool_address, &1000000000, &None);
    wait(&e, 20);
    contract_client.withdraw(maksym, &gold_pool_address, &10500000000, &None);
    wait(&e, 25);
    contract_client.deposit(maksym, &usdc_pool_address, &21000000000, &None);
    wait(&e, 50);
    contract_client.deposit(k1, &gold_pool_address, &20000000000i128, &None);
    wait(&e, 60);
    contract_client.borrow(maksym, &gold_pool_address, &6979216852i128, &None);
    wait(&e, 20);
    contract_client.borrow(maksym, &gold_pool_address, &7328177694, &None);
    wait(&e, 20);
    contract_client.deposit_with_leverage(
        maksym,
        &gold_pool_address,
        &usdc_pool_address,
        &true,
        &1000000000,
        &237,
        &None,
    );
    wait(&e, 5 * 60 + 26);
    contract_client.withdraw_from_leveraged(
        maksym,
        &gold_pool_address,
        &usdc_pool_address,
        &1000000000,
        &None,
    );
}

#[test]
fn test_obligation_does_not_exist_prior_anything() {
    let TestMarketFixture { users, contract_client, .. } = TestMarketFixture::new();

    let user = &users[0];
    let obligation = contract_client.try_get_user_obligation(user);

    assert_eq!(obligation, Err(Ok(MCError::ObligationDoesNotExist)));
}

#[test]
fn test_pool_with_random_address_does_not_exist() {
    let TestMarketFixture { e, contract_client, .. } = TestMarketFixture::new();

    let rand_addr = Address::generate(&e);
    let res = contract_client.try_get_pool(&rand_addr);

    assert_eq!(res, Err(Ok(MCError::PoolDoesNotExist)));
}

#[test]
fn test_pool_is_empty_prior_anything() {
    let TestMarketFixture { contract_client, usdc_pool_address, .. } = TestMarketFixture::new();

    let pool = contract_client.get_pool(&usdc_pool_address);

    assert_eq!(pool.total_available, 0);
    assert_eq!(pool.total_borrowed, 0);
    assert_eq!(pool.total_j_tokens, 0);
    assert_eq!(pool.total_d_tokens, 0);
    assert_eq!(pool.total_collateral, 0);
}

#[test]
fn test_obligations_list_contains_unique_obligations() {
    let TestMarketFixture { contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let liquidity_provider = &users[0];
    let creditor = &users[1];

    contract_client.deposit(
        liquidity_provider,
        &gold_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let obligations = contract_client.get_all_obligations();
    assert_eq!(obligations.len(), 2);
    assert!(obligations.contains(ObligationKey::new(creditor.clone())));

    contract_client.withdraw(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let obligations = contract_client.get_all_obligations();
    assert_eq!(obligations.len(), 1);
    assert!(!obligations.contains(ObligationKey::new(creditor.clone())));
}

#[test]
fn test_too_many_positions() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        btc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    contract_client.update_market(&2, &1);

    contract_client.add_collateral(user, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.add_collateral(user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2), &None);

    assert_eq!(
        contract_client.try_add_collateral(
            user,
            &btc_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 2),
            &None
        ),
        Err(Ok(MCError::TooManyPositions))
    );

    contract_client.remove_collateral(user, &gold_pool_address, &i128::MAX, &None);

    assert!(
        contract_client
            .try_add_collateral(user, &btc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2), &None)
            .is_ok()
    );

    assert_eq!(
        contract_client.try_add_collateral(
            user,
            &gold_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 2),
            &None
        ),
        Err(Ok(MCError::TooManyPositions))
    );

    contract_client.update_market(&3, &1);

    assert!(
        contract_client
            .try_add_collateral(user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2), &None)
            .is_ok()
    );
}

#[test]
fn test_unable_to_borrow_and_deposit_the_same_asset() {
    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let liquidity_provider = &users[0];
    let user_1 = &users[1];
    let user_2 = &users[2];

    contract_client.add_collateral(user_1, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    assert_eq!(
        contract_client.try_borrow(
            user_1,
            &usdc_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 2),
            &None
        ),
        Err(Ok(MCError::DepositPositionForAssetExists))
    );

    contract_client.deposit(liquidity_provider, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(user_2, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.borrow(user_2, &gold_pool_address, &i128::MAX, &None);
    assert_eq!(
        contract_client.try_deposit(user_2, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None),
        Err(Ok(MCError::BorrowPositionForAssetExists))
    );
}

#[test]
fn test_get_pool_data() {
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let creditor = &users[0];
    let debtor = &users[1];

    let PoolData { apy, j_token_rate_floor_bps, d_token_rate_ceil_bps, .. } =
        contract_client.get_pool_data(&usdc_pool_address);

    assert_eq!(j_token_rate_floor_bps, 0);
    assert_eq!(d_token_rate_ceil_bps, 0);
    assert_eq!(apy.supply_bps, 0);
    assert!(apy.supply_bps <= apy.borrow_bps);

    contract_client.deposit(creditor, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT), &None);
    contract_client.add_collateral(
        debtor,
        &gold_pool_address,
        &(2 * DEFAULT_COLLATERAL_AMOUNT),
        &None,
    );
    contract_client.borrow(debtor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let PoolData { apy, j_token_rate_floor_bps, d_token_rate_ceil_bps, .. } =
        contract_client.get_pool_data(&usdc_pool_address);

    assert!(j_token_rate_floor_bps.is_positive());
    assert!(d_token_rate_ceil_bps.is_positive());
    assert_ne!(apy.supply_bps, 0);
    assert!(apy.supply_bps <= apy.borrow_bps);

    let user_j_tokens =
        get_obligation_j_tokens(&contract_client, creditor, &usdc_pool_address).unwrap();
    let tokens_from_j_tokens =
        user_j_tokens.fixed_mul_ceil(j_token_rate_floor_bps, BPS_FACTOR).unwrap();

    let user_d_tokens =
        get_obligation_d_tokens(&contract_client, debtor, &usdc_pool_address).unwrap();
    let tokens_from_d_tokens =
        user_d_tokens.fixed_mul_floor(d_token_rate_ceil_bps, BPS_FACTOR).unwrap();

    assert_eq!(tokens_from_j_tokens, 2 * DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(tokens_from_d_tokens, DEFAULT_DEPOSIT_AMOUNT);
}

#[test]
fn test_get_market_data() {
    let TestMarketFixture { contract_client, .. } = TestMarketFixture::new();

    let market_data = contract_client.get_market_data();
    let MarketData { pools_data, global_state, .. } = market_data;

    assert!(global_state.update_in_queue_period.is_some());

    for pool_data in pools_data.iter() {
        let PoolData { apy, j_token_rate_floor_bps, d_token_rate_ceil_bps, .. } = pool_data;

        assert_eq!(j_token_rate_floor_bps, 0);
        assert_eq!(d_token_rate_ceil_bps, 0);
        assert_eq!(apy.supply_bps, 0);
        assert!(apy.supply_bps <= apy.borrow_bps);
    }
}

#[test]
fn test_refresh_pool() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    let liquidity_provider = &users[0];
    let debtor = &users[1];

    contract_client.deposit(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10), &None);

    let pool_before = contract_client.get_pool(&usdc_pool_address);

    // -- Move time --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR;
    });

    let pool_after = contract_client.get_pool(&usdc_pool_address);
    assert_eq!(pool_before, pool_after);

    contract_client.refresh_pool(&usdc_pool_address);

    let pool_after_w_refresh = contract_client.get_pool(&usdc_pool_address);
    assert_ne!(pool_before, pool_after_w_refresh);
}

#[test]
fn test_refresh_obligation() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        btc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let liquidity_provider = &users[0];
    let debtor = &users[1];

    contract_client.deposit(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10), &None);

    contract_client.deposit(liquidity_provider, &btc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.borrow(debtor, &btc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10), &None);

    let usdc_pool_before = contract_client.get_pool(&usdc_pool_address);
    let btc_pool_before = contract_client.get_pool(&btc_pool_address);

    // -- Move time --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR;
    });

    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    let btc_pool_after = contract_client.get_pool(&btc_pool_address);
    assert_eq!(usdc_pool_before, usdc_pool_after);
    assert_eq!(btc_pool_before, btc_pool_after);

    contract_client.refresh_obligation(debtor);

    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    let btc_pool_after = contract_client.get_pool(&btc_pool_address);
    assert_ne!(usdc_pool_before, usdc_pool_after);
    assert_ne!(btc_pool_before, btc_pool_after);
}

#[test]
fn test_refresh_earn_obligation() {
    let TestMarketFixture {
        e,
        contract_client,
        usdc_pool_address,
        btc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let creditor = &users[0];
    let debtor = &users[1];

    contract_client.deposit_earn(creditor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10), &None);

    contract_client.deposit_earn(creditor, &btc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.borrow(debtor, &btc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10), &None);

    let usdc_pool_before = contract_client.get_pool(&usdc_pool_address);
    let btc_pool_before = contract_client.get_pool(&btc_pool_address);

    // -- Move time --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR;
    });

    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    let btc_pool_after = contract_client.get_pool(&btc_pool_address);
    assert_eq!(usdc_pool_before, usdc_pool_after);
    assert_eq!(btc_pool_before, btc_pool_after);

    contract_client.refresh_earn_obligation(creditor);

    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    let btc_pool_after = contract_client.get_pool(&btc_pool_address);
    assert_ne!(usdc_pool_before, usdc_pool_after);
    assert_ne!(btc_pool_before, btc_pool_after);
}

#[test]
fn test_refresh_multiply_pair_obligation() {
    const LEVERAGE: u32 = 3;
    const LEVERAGE_MULTIPLIER: u32 = LEVERAGE * LEVERAGE_SCALE;

    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    let liquidity_provider = &users[0];
    let looper = &users[1];

    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &true,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
        &None,
    );

    let usdc_pool_before = contract_client.get_pool(&usdc_pool_address);

    // -- Move time --

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR;
    });

    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    assert_eq!(usdc_pool_before, usdc_pool_after);

    contract_client.refresh_multiply_pair_obligation(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
    );

    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    assert_ne!(usdc_pool_before, usdc_pool_after);
}

#[test]
fn test_transfer_admin() {
    let TestMarketFixture { e, contract_id, full_contract_client, contract_admin, .. } =
        TestMarketFixture::new();
    let new_admin = Address::generate(&e);
    assert_ne!(new_admin, contract_admin);

    e.as_contract(&contract_id, || {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        assert_eq!(admin, contract_admin);
    });

    full_contract_client.propose_new_admin(&new_admin);

    e.as_contract(&contract_id, || {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        assert_eq!(admin, contract_admin);
    });

    full_contract_client.accept_proposed_admin();

    e.as_contract(&contract_id, || {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        assert_eq!(admin, new_admin);
    });
}

#[test]
fn test_require_available_accounts_for_take_rate_fees() {
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new();
    let liquidity_provider = &users[0];
    let borrower = &users[1];
    let beneficiary = users[2].clone();

    contract_client.deposit(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(borrower, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.borrow(borrower, &usdc_pool_address, &i128::MAX, &None);

    contract_client
        .set_take_rate_fees_beneficiaries(&usdc_pool_address, &smap![&e, (beneficiary, 10_000)]);

    // - Move time -

    e.ledger().with_mut(|li| {
        li.timestamp += SECONDS_IN_YEAR / 12;
    });

    contract_client.refresh_obligation(borrower);
    let usdc_pool_data = contract_client.get_pool_data(&usdc_pool_address);

    // - Verify that the interest rate has accrued and take rate fees are accounted for -

    let unpaid_interest =
        get_obligation_unpaid_interest(&e, &contract_client, borrower, &usdc_pool_address).unwrap();
    let take_rate_fees_sum = usdc_pool_data.pool.take_rate_fees_sum;

    assert!(unpaid_interest > 0);
    assert!(take_rate_fees_sum > 0);

    assert_approx_eq_abs(unpaid_interest / 10, take_rate_fees_sum, 1);

    // - Verify that the interest rate has accrued and that the rate fees are accounted for -

    let (total_available, total_available_adjusted) =
        (usdc_pool_data.pool.total_available, usdc_pool_data.total_available_adjusted);
    assert_eq!(total_available, total_available_adjusted.checked_add(take_rate_fees_sum).unwrap());

    let flash_loan_callback = Address::generate(&e);

    assert_ne!(
        contract_client.try_flash_loan(
            &flash_loan_callback,
            borrower,
            &usdc_pool_address,
            &total_available_adjusted
        ),
        Err(Ok(MCError::NotEnoughPoolFunds))
    );
    assert_eq!(
        contract_client.try_flash_loan(
            &flash_loan_callback,
            borrower,
            &usdc_pool_address,
            &(total_available_adjusted + 1)
        ),
        Err(Ok(MCError::NotEnoughPoolFunds))
    );
}

#[test]
fn test_referrer_fee_is_charged_and_referrer_receives_it() {
    use soroban_sdk::{Address, Map};

    let TestMarketFixture {
        e, contract_client, gold_pool_address, users, gold_token_client, ..
    } = TestMarketFixture::new();

    let depositor = &users[0];
    let referrer = Address::generate(&e);

    // Configure fees: 20% deposit fee, referrer share 50%
    let pool_before = contract_client.get_pool(&gold_pool_address);
    let mut new_cfg = pool_before.config.clone();

    let deposit_fee_bps: u32 = 2_000;
    new_cfg.fee_config.deposit_fee_bps = deposit_fee_bps;

    let mut referrers: Map<Address, u32> = Map::new(&e);
    referrers.set(referrer.clone(), 5_000);
    new_cfg.fee_config.referrers = Some(referrers);

    contract_client.queue_in_pool_config_update(&gold_pool_address, &new_cfg);

    // Advance time so the queued config becomes eligible to apply
    let gs = contract_client.get_global_state();
    if let Some(period) = gs.update_in_queue_period
        && period > 0
    {
        e.ledger().with_mut(|li| li.timestamp += period + 1);
    }
    contract_client.apply_pool_config_update(&gold_pool_address);

    // Sanity: config applied
    let pool_after_cfg = contract_client.get_pool(&gold_pool_address);
    assert_eq!(pool_after_cfg.config.fee_config.deposit_fee_bps, deposit_fee_bps);

    let ref_before = gold_token_client.balance(&referrer);

    let amount: i128 = 1_000_000;
    contract_client.deposit(depositor, &gold_pool_address, &amount, &Some(referrer.clone()));

    // Sanity: referrer fee expected to be nonzero (otherwise test is meaningless)
    let total_fee = (amount * deposit_fee_bps as i128) / BPS_FACTOR;
    let expected_referrer_fee = total_fee / 2;
    assert!(expected_referrer_fee > 0);

    let ref_after = gold_token_client.balance(&referrer);
    assert_eq!(ref_after.checked_sub(ref_before).unwrap(), expected_referrer_fee);
}

#[test]
fn test_repeated_deposits_do_not_inflate_positions_count() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        btc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let creditor = &users[0];

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    assert_eq!(contract_client.get_user_obligation(creditor).positions_count, 1);

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    assert_eq!(contract_client.get_user_obligation(creditor).positions_count, 1);

    contract_client.add_collateral(creditor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    assert_eq!(contract_client.get_user_obligation(creditor).positions_count, 1);

    contract_client.deposit(creditor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    assert_eq!(contract_client.get_user_obligation(creditor).positions_count, 2);

    contract_client.deposit(creditor, &btc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    assert_eq!(contract_client.get_user_obligation(creditor).positions_count, 3);

    contract_client.deposit(creditor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(creditor, &btc_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    assert_eq!(contract_client.get_user_obligation(creditor).positions_count, 3);
}

#[test]
fn test_repeated_borrows_do_not_inflate_positions_count() {
    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT), &None);
    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    contract_client.borrow(borrower, &usdc_pool_address, &1000, &None);
    assert_eq!(contract_client.get_user_obligation(borrower).positions_count, 2);

    contract_client.borrow(borrower, &usdc_pool_address, &1000, &None);
    assert_eq!(contract_client.get_user_obligation(borrower).positions_count, 2);
}

#[test]
fn test_positions_count_decrements_on_withdraw_and_remove_collateral() {
    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let creditor = &users[0];

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(creditor, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    contract_client.add_collateral(creditor, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);
    assert_eq!(contract_client.get_user_obligation(creditor).positions_count, 2);

    contract_client.withdraw(creditor, &gold_pool_address, &i128::MAX, &None);
    assert_eq!(contract_client.get_user_obligation(creditor).positions_count, 1);

    contract_client.remove_collateral(creditor, &usdc_pool_address, &i128::MAX, &None);
    assert!(contract_client.try_get_user_obligation(creditor).is_err());
}

#[test]
fn test_positions_count_decrements_on_repay() {
    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let borrower = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT), &None);
    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    contract_client.borrow(borrower, &usdc_pool_address, &1000, &None);
    contract_client.borrow(borrower, &usdc_pool_address, &1000, &None);
    assert_eq!(contract_client.get_user_obligation(borrower).positions_count, 2);

    contract_client.repay(borrower, &usdc_pool_address, &2000, &None);
    assert_eq!(contract_client.get_user_obligation(borrower).positions_count, 1);

    contract_client.withdraw(borrower, &gold_pool_address, &i128::MAX, &None);
    assert!(contract_client.try_get_user_obligation(borrower).is_err());
}
