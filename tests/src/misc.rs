#![cfg(test)]

use market::{
    constants::{BPS_FACTOR, LEVERAGE_SCALE, SECONDS_IN_YEAR},
    error::MCError,
    misc::{MarketData, PoolData},
    obligation::ObligationKey,
};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address,
    testutils::{Address as _, Ledger},
};

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, get_obligation_d_tokens,
    get_obligation_j_tokens,
};

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

    contract_client.deposit(user1, &usdc_pool_address, &1000);
    contract_client.deposit(user2, &usdc_pool_address, &1000);

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

    contract_client.deposit(liquidity_provider, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligations = contract_client.get_all_obligations();
    assert_eq!(obligations.len(), 2);
    assert!(obligations.contains(ObligationKey::new(creditor.clone())));

    contract_client.withdraw(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligations = contract_client.get_all_obligations();
    assert_eq!(obligations.len(), 1);
    assert!(!obligations.contains(ObligationKey::new(creditor.clone())));
}

// #[test]
// fn test_bootstrap_pool() {
//     let TestMarketFixture {
//         e,
//         contract_client,
//         gold_pool_address,
//         gold_token_client,
//         contract_id,
//         users,
//         ..
//     } = TestMarketFixture::new();
//     let liquidity_provider = &users[0];
//     let creditor_1 = &users[1];
//     let creditor_2 = &users[2];

//     contract_client.deposit(creditor_1, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
//     contract_client.deposit(creditor_2, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

//     // -- Move time --

//     e.ledger().with_mut(|li| li.timestamp += SECONDS_IN_YEAR);
//     contract_client.refresh_pool(&gold_pool_address);

//     // -- Assert no received interest has accrued due to 0% utilization --

//     let received_interest_1 =
//         get_obligation_received_interest(&e, &contract_client, creditor_1, &gold_pool_address)
//             .unwrap();
//     let received_interest_2 =
//         get_obligation_received_interest(&e, &contract_client, creditor_2, &gold_pool_address)
//             .unwrap();

//     assert_eq!(received_interest_1, 0);
//     assert_eq!(received_interest_2, 0);

//     // -- Bootstrap pool --

//     gold_token_client.approve(
//         liquidity_provider,
//         &contract_id,
//         &DEFAULT_DEPOSIT_AMOUNT,
//         &(e.ledger().sequence()),
//     );

//     contract_client.bootstrap_pool(
//         &gold_pool_address,
//         liquidity_provider,
//         &DEFAULT_DEPOSIT_AMOUNT,
//         &e.ledger().timestamp(),
//         &(e.ledger().timestamp() + SECONDS_IN_YEAR),
//     );

//     // -- Assert half of bootstrapped value has accrued --

//     e.ledger().with_mut(|li| li.timestamp += SECONDS_IN_YEAR / 4);
//     contract_client.refresh_pool(&gold_pool_address);

//     let received_interest_1 =
//         get_obligation_received_interest(&e, &contract_client, creditor_1, &gold_pool_address)
//             .unwrap();
//     let received_interest_2 =
//         get_obligation_received_interest(&e, &contract_client, creditor_2, &gold_pool_address)
//             .unwrap();

//     // TODO: Fix later

//     // assert_eq!(received_interest_1, received_interest_2);
//     // assert_eq!(
//     //     received_interest_1.checked_add(received_interest_2).unwrap(),
//     //     DEFAULT_DEPOSIT_AMOUNT / 2
//     // );
// }

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

    contract_client.add_collateral(user, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);
    contract_client.add_collateral(user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    assert_eq!(
        contract_client.try_add_collateral(user, &btc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2)),
        Err(Ok(MCError::TooManyPositions))
    );

    contract_client.remove_collateral(user, &gold_pool_address, &i128::MAX);

    assert!(
        contract_client
            .try_add_collateral(user, &btc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2))
            .is_ok()
    );

    assert_eq!(
        contract_client.try_add_collateral(user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2)),
        Err(Ok(MCError::TooManyPositions))
    );

    contract_client.update_market(&3, &1);

    assert!(
        contract_client
            .try_add_collateral(user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2))
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

    contract_client.add_collateral(user_1, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT);
    assert_eq!(
        contract_client.try_borrow(user_1, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2)),
        Err(Ok(MCError::DepositPositionForAssetExists))
    );

    contract_client.deposit(liquidity_provider, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.add_collateral(user_2, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT);
    contract_client.borrow(user_2, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(
        contract_client.try_deposit(user_2, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT),
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

    contract_client.deposit(creditor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

    let PoolData { apy, j_token_rate_floor_bps, d_token_rate_ceil_bps, .. } =
        contract_client.get_pool_data(&usdc_pool_address);

    assert!(j_token_rate_floor_bps > 0);
    assert!(d_token_rate_ceil_bps > 0);
    assert!(apy.supply_bps > 0);
    assert!(apy.supply_bps <= apy.borrow_bps);

    let user_j_tokens =
        get_obligation_j_tokens(&contract_client, creditor, &usdc_pool_address).unwrap();
    let tokens_from_j_tokens =
        user_j_tokens.fixed_mul_ceil(j_token_rate_floor_bps, BPS_FACTOR).unwrap();

    let user_d_tokens =
        get_obligation_d_tokens(&contract_client, debtor, &usdc_pool_address).unwrap();
    let tokens_from_d_tokens =
        user_d_tokens.fixed_mul_floor(d_token_rate_ceil_bps, BPS_FACTOR).unwrap();

    assert_eq!(tokens_from_j_tokens, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(tokens_from_d_tokens, DEFAULT_DEPOSIT_AMOUNT / 10);
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

    contract_client.deposit(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

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

    contract_client.deposit(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

    contract_client.deposit(liquidity_provider, &btc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);
    contract_client.borrow(debtor, &btc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

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

    contract_client.deposit_into_earn_obligation(
        creditor,
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
    );
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

    contract_client.deposit_into_earn_obligation(
        creditor,
        &btc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
    );
    contract_client.add_collateral(debtor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);
    contract_client.borrow(debtor, &btc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

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
    );

    contract_client.deposit_with_leverage(
        looper,
        &gold_pool_address,
        &usdc_pool_address,
        &true,
        &DEFAULT_DEPOSIT_AMOUNT,
        &LEVERAGE_MULTIPLIER,
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
