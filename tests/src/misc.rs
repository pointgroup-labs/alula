#![cfg(test)]

use controlled_insurance_fund::storage::DataKey;
use market::{
    constants::{BPS_FACTOR, LEVERAGE_SCALE, SECONDS_IN_YEAR},
    error::MCError,
    request::{Request, StandardRequest, SwapExactTokensRequest},
    utils::{MarketData, PoolData},
};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, Env,
    testutils::{Address as _, Ledger},
    vec as svec,
};

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, assert_approx_eq_abs,
    assert_approx_eq_rel, get_obligation_d_tokens, get_obligation_j_tokens,
    get_obligation_received_interest,
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
    contract_client.withdraw(k1, &gold_pool_address, &4819579912, &None);
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
    contract_client.deposit(maksym, &usdc_pool_address, &10000000000, &None);
    wait(&e, 50);
    contract_client.deposit(k1, &gold_pool_address, &10000000000i128, &None);
    wait(&e, 60);
    contract_client.borrow(maksym, &gold_pool_address, &6979216852i128, &None);
    wait(&e, 20);
    contract_client.borrow(maksym, &gold_pool_address, &7328177694, &None);
    wait(&e, 20);
    contract_client.deposit_with_leverage(
        &maksym.address,
        &gold_pool_address,
        &usdc_pool_address,
        &true,
        &1000000000,
        &237,
        &None,
    );
    wait(&e, 5 * 60 + 26);
    contract_client.withdraw_from_leveraged(
        &maksym.address,
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
fn test_reset_storage_removes_obligations() {
    let TestMarketFixture { contract_client, usdc_pool_address, users, .. } =
        TestMarketFixture::new();

    let user1 = &users[0];
    let user2 = &users[2];

    assert!(contract_client.get_all_obligations().is_empty());

    contract_client.deposit(user1, &usdc_pool_address, &1000, &None);
    contract_client.deposit(user2, &usdc_pool_address, &1000, &None);

    assert!(contract_client.try_get_user_obligation(user1).is_ok());
    assert!(contract_client.try_get_user_obligation(user2).is_ok());
    assert_eq!(contract_client.get_all_obligations().len(), 2);

    contract_client.reset_storage();

    assert!(contract_client.try_get_user_obligation(user1).is_err());
    assert!(contract_client.try_get_user_obligation(user2).is_err());
    assert!(contract_client.get_all_obligations().is_empty());
}

#[test]
fn test_reset_storage_removes_pool() {
    let TestMarketFixture { contract_client, .. } = TestMarketFixture::new();

    assert_eq!(contract_client.get_all_pools().len(), 3); // NB: 3 pools are set initially

    contract_client.reset_storage();

    assert!(contract_client.get_all_pools().is_empty());
}

#[test]
fn test_reset_storage_removes_multiply_pairs() {
    let TestMarketFixture { contract_client, usdc_pool_address, btc_pool_address, .. } =
        TestMarketFixture::new();

    assert_eq!(contract_client.get_all_multiply_pairs().len(), 1); // NB: 1 pair is set initially

    contract_client.initialize_multiply_pair(&usdc_pool_address, &btc_pool_address);

    assert_eq!(contract_client.get_all_multiply_pairs().len(), 2);

    contract_client.reset_storage();

    assert!(contract_client.get_all_multiply_pairs().is_empty());
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
    assert!(obligations.contains(creditor));

    contract_client.withdraw(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let obligations = contract_client.get_all_obligations();
    assert_eq!(obligations.len(), 1);
    assert!(!obligations.contains(creditor));
}

#[test]
fn test_bootstrap_pool() {
    let TestMarketFixture {
        e,
        contract_client,
        gold_pool_address,
        gold_token_client,
        contract_id,
        users,
        ..
    } = TestMarketFixture::new();
    let liquidity_provider = &users[0];
    let creditor_1 = &users[1];
    let creditor_2 = &users[2];

    contract_client.deposit(creditor_1, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.deposit(creditor_2, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    // -- Move time --

    e.ledger().with_mut(|li| li.timestamp += SECONDS_IN_YEAR);
    contract_client.refresh_pool(&gold_pool_address);

    // -- Assert no received interest has accrued due to 0% utilization --

    let received_interest_1 =
        get_obligation_received_interest(&e, &contract_client, creditor_1, &gold_pool_address)
            .unwrap();
    let received_interest_2 =
        get_obligation_received_interest(&e, &contract_client, creditor_2, &gold_pool_address)
            .unwrap();

    assert_approx_eq_abs(received_interest_1, 0, 2);
    assert_approx_eq_abs(received_interest_2, 0, 2);

    // -- Bootstrap pool --

    gold_token_client.approve(
        &liquidity_provider.address,
        &contract_id,
        &DEFAULT_DEPOSIT_AMOUNT,
        &(e.ledger().sequence()),
    );
    contract_client.bootstrap_pool(
        &gold_pool_address,
        &liquidity_provider.address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &e.ledger().timestamp(),
        &(e.ledger().timestamp() + SECONDS_IN_YEAR),
    );

    // -- Assert half of bootstrapped value has accrued --

    e.ledger().with_mut(|li| li.timestamp += SECONDS_IN_YEAR / 2);
    contract_client.refresh_pool(&gold_pool_address);

    let received_interest_1 =
        get_obligation_received_interest(&e, &contract_client, creditor_1, &gold_pool_address)
            .unwrap();
    let received_interest_2 =
        get_obligation_received_interest(&e, &contract_client, creditor_2, &gold_pool_address)
            .unwrap();

    assert_approx_eq_rel(received_interest_1, received_interest_2, 1);
    assert_approx_eq_rel(
        received_interest_1.checked_add(received_interest_2).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT / 2,
        10,
    );

    // -- Wait till the bootstrap period ends --

    e.ledger().with_mut(|li| li.timestamp += SECONDS_IN_YEAR / 2);
    contract_client.refresh_pool(&gold_pool_address);

    let received_interest_1 =
        get_obligation_received_interest(&e, &contract_client, creditor_1, &gold_pool_address)
            .unwrap();
    let received_interest_2 =
        get_obligation_received_interest(&e, &contract_client, creditor_2, &gold_pool_address)
            .unwrap();

    assert_eq!(received_interest_1, received_interest_2);
    assert_approx_eq_rel(
        received_interest_1.checked_add(received_interest_2).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT,
        10,
    );

    // -- Assert no bootstrap takes place after draining --

    e.ledger().with_mut(|li| li.timestamp += SECONDS_IN_YEAR / 2);
    contract_client.refresh_pool(&gold_pool_address);

    let received_interest_1 =
        get_obligation_received_interest(&e, &contract_client, creditor_1, &gold_pool_address)
            .unwrap();
    let received_interest_2 =
        get_obligation_received_interest(&e, &contract_client, creditor_2, &gold_pool_address)
            .unwrap();

    assert_eq!(received_interest_1, received_interest_2);
    assert_approx_eq_rel(
        received_interest_1.checked_add(received_interest_2).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT,
        10,
    );
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
    contract_client.borrow(user_2, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
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
        &looper.address,
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
        &looper.address,
        &gold_pool_address,
        &usdc_pool_address,
    );

    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    assert_ne!(usdc_pool_before, usdc_pool_after);
}

#[test]
fn transfer_admin() {
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
fn test_updating_positions_amounts_does_not_affect_positions_count() {
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let creditor = &users[0];
    let debtor = &users[1];

    contract_client.deposit(creditor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    let creditor_obligation = contract_client.get_user_obligation(creditor);
    assert_eq!(creditor_obligation.positions_count, 1);

    contract_client.deposit(creditor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    let creditor_obligation = contract_client.get_user_obligation(creditor);
    assert_eq!(creditor_obligation.positions_count, 1);

    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    let debtor_obligation = contract_client.get_user_obligation(debtor);
    assert_eq!(debtor_obligation.positions_count, 1);

    contract_client.add_collateral(debtor, &gold_pool_address, &1, &None);
    let debtor_obligation = contract_client.get_user_obligation(debtor);
    assert_eq!(debtor_obligation.positions_count, 1);

    contract_client.borrow(debtor, &usdc_pool_address, &1, &None);
    let debtor_obligation = contract_client.get_user_obligation(debtor);
    assert_eq!(debtor_obligation.positions_count, 2);

    contract_client.borrow(debtor, &usdc_pool_address, &1, &None);
    let debtor_obligation = contract_client.get_user_obligation(debtor);
    assert_eq!(debtor_obligation.positions_count, 2);

    contract_client.repay(debtor, &usdc_pool_address, &1, &None);
    let debtor_obligation = contract_client.get_user_obligation(debtor);
    assert_eq!(debtor_obligation.positions_count, 2);

    contract_client.repay(debtor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    let debtor_obligation = contract_client.get_user_obligation(debtor);
    assert_eq!(debtor_obligation.positions_count, 1);

    contract_client.remove_collateral(debtor, &gold_pool_address, &1, &None);
    let debtor_obligation = contract_client.get_user_obligation(debtor);
    assert_eq!(debtor_obligation.positions_count, 1);

    contract_client.withdraw(creditor, &usdc_pool_address, &1, &None);
    let creditor_obligation = contract_client.get_user_obligation(creditor);
    assert_eq!(creditor_obligation.positions_count, 1);
}

#[test]
fn test_collect_dust() {
    let TestMarketFixture {
        contract_admin,
        contract_id,
        full_contract_client,
        usdc_token_client,
        gold_token_client,
        btc_token_client,
        users,
        ..
    } = TestMarketFixture::new();
    let donor_address = &users[0].address;

    let admin_usdc_balance_pre = usdc_token_client.balance(&contract_admin);
    let admin_gold_balance_pre = gold_token_client.balance(&contract_admin);
    let admin_btc_balance_pre = btc_token_client.balance(&contract_admin);

    full_contract_client.collect_pools_excessive_tokens();

    let admin_usdc_balance_post = usdc_token_client.balance(&contract_admin);
    let admin_gold_balance_post = gold_token_client.balance(&contract_admin);
    let admin_btc_balance_post = btc_token_client.balance(&contract_admin);

    assert_eq!(admin_usdc_balance_pre, admin_usdc_balance_post);
    assert_eq!(admin_gold_balance_pre, admin_gold_balance_post);
    assert_eq!(admin_btc_balance_pre, admin_btc_balance_post);

    usdc_token_client.transfer(donor_address, &contract_id, &1);
    gold_token_client.transfer(donor_address, &contract_id, &2);
    btc_token_client.transfer(donor_address, &contract_id, &3);

    full_contract_client.collect_pools_excessive_tokens();

    let admin_usdc_balance_post = usdc_token_client.balance(&contract_admin);
    let admin_gold_balance_post = gold_token_client.balance(&contract_admin);
    let admin_btc_balance_post = btc_token_client.balance(&contract_admin);

    let usdc_diff = admin_usdc_balance_post - admin_usdc_balance_pre;
    let gold_diff = admin_gold_balance_post - admin_gold_balance_pre;
    let btc_diff = admin_btc_balance_post - admin_btc_balance_pre;

    assert_eq!(usdc_diff, 1);
    assert_eq!(gold_diff, 2);
    assert_eq!(btc_diff, 3);
}

#[test]
fn test_collect_excessive_tokens() {
    let TestMarketFixture { users, .. } = TestMarketFixture::new();
    let _donor = &users[0];
}

#[test]
fn test_leverage_w_new_flash_loan() {
    let TestMarketFixture {
        e,

        full_contract_client,
        usdc_token_client,
        gold_token_client,
        gold_token_address,
        usdc_pool_address,

        users,
        ..
    } = TestMarketFixture::new();
    let liquidity_provider = &users[0];
    let looper = &users[1];

    full_contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let usdc_balance_before = usdc_token_client.balance(&looper.address);
    let gold_balance_before = gold_token_client.balance(&looper.address);

    let flash_borrow_request = Request::FlashBorrow(StandardRequest {
        amount: DEFAULT_DEPOSIT_AMOUNT,
        pool_address: usdc_pool_address.clone(),
    });

    let swap_provider = Address::generate(&e);
    let swap_request = Request::SwapExactTokens(SwapExactTokensRequest {
        swap_provider,
        token_in: usdc_pool_address.clone(),
        token_out: gold_token_address.clone(),
        amount_in: DEFAULT_DEPOSIT_AMOUNT,
        min_amount_out: 1,
    });

    let requests = svec![&e, flash_borrow_request, swap_request];

    full_contract_client.submit_requests_batch(looper, &requests, &None);

    let usdc_balance_after = usdc_token_client.balance(&looper.address);
    let gold_balance_after = gold_token_client.balance(&looper.address);
}
