#![cfg(test)]

use market::request::{Request, StandardRequest};
use soroban_sdk::vec as svec;

use crate::{DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture};

#[test]
fn test_empty_batching() {
    let TestMarketFixture {
        e,
        contract_client,
        contract_id,
        usdc_pool_address,
        btc_pool_address,
        gold_pool_address,
        users,
        usdc_token_client,
        btc_token_client,
        gold_token_client,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    let tuple_before = (
        contract_client.get_pool(&usdc_pool_address),
        contract_client.get_pool(&btc_pool_address),
        contract_client.get_pool(&gold_pool_address),
        usdc_token_client.balance(&contract_id),
        btc_token_client.balance(&contract_id),
        gold_token_client.balance(&contract_id),
        usdc_token_client.balance(&user.address),
        btc_token_client.balance(&user.address),
        gold_token_client.balance(&user.address),
    );

    contract_client.submit_requests_batch(user, &svec![&e], &None);

    let tuple_after = (
        contract_client.get_pool(&usdc_pool_address),
        contract_client.get_pool(&btc_pool_address),
        contract_client.get_pool(&gold_pool_address),
        usdc_token_client.balance(&contract_id),
        btc_token_client.balance(&contract_id),
        gold_token_client.balance(&contract_id),
        usdc_token_client.balance(&user.address),
        btc_token_client.balance(&user.address),
        gold_token_client.balance(&user.address),
    );

    assert_eq!(tuple_before, tuple_after);
}

#[test]
fn test_simple_batching() {
    let TestMarketFixture {
        e,
        contract_client,
        contract_id,
        usdc_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    let request_0 = Request::Deposit(StandardRequest {
        pool_address: usdc_pool_address.clone(),
        amount: DEFAULT_DEPOSIT_AMOUNT,
    });

    let pool_supply_1 = contract_client.get_pool(&usdc_pool_address).total_supply().unwrap();
    let contract_balance_1 = usdc_token_client.balance(&contract_id);
    let user_balance_1 = usdc_token_client.balance(&user.address);

    let simple_batch = svec![&e, request_0];

    contract_client.submit_requests_batch(user, &simple_batch, &None);

    let pool_supply_2 = contract_client.get_pool(&usdc_pool_address).total_supply().unwrap();
    let contract_balance_2 = usdc_token_client.balance(&contract_id);
    let user_balance_2 = usdc_token_client.balance(&user.address);

    let pool_supply_diff = pool_supply_2.checked_sub(pool_supply_1).unwrap();
    let contract_balance_diff = contract_balance_2.checked_sub(contract_balance_1).unwrap();
    let user_balance_diff = user_balance_2.checked_sub(user_balance_1).unwrap();

    assert_eq!(pool_supply_diff, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(contract_balance_diff, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(user_balance_diff, DEFAULT_DEPOSIT_AMOUNT.checked_neg().unwrap());
}

#[test]
fn test_complex_batching() {
    let TestMarketFixture {
        e,
        contract_client,
        contract_id,
        usdc_pool_address,
        gold_pool_address,
        users,
        usdc_token_client,
        gold_token_client,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];
    let liquidity_provider = &users[1];
    contract_client.deposit(
        liquidity_provider,
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    // -- Deposit => Withdraw --

    let tuple_before = (
        contract_client.get_pool(&usdc_pool_address).total_supply().unwrap(),
        usdc_token_client.balance(&contract_id),
        usdc_token_client.balance(&user.address),
    );
    let deposit_r = Request::Deposit(StandardRequest {
        amount: DEFAULT_DEPOSIT_AMOUNT,
        pool_address: usdc_pool_address.clone(),
    });
    let withdraw_r = Request::Withdraw(StandardRequest {
        amount: DEFAULT_DEPOSIT_AMOUNT,
        pool_address: usdc_pool_address.clone(),
    });
    let r_0 = svec![&e, deposit_r, withdraw_r];

    contract_client.submit_requests_batch(user, &r_0, &None);

    let tuple_after = (
        contract_client.get_pool(&usdc_pool_address).total_supply().unwrap(),
        usdc_token_client.balance(&contract_id),
        usdc_token_client.balance(&user.address),
    );

    assert_eq!(tuple_before, tuple_after);

    // -- AddCollateral => Borrow => Repay => RemoveCollateral --

    let tuple_before = (
        contract_client.get_pool(&usdc_pool_address).total_supply().unwrap(),
        usdc_token_client.balance(&contract_id),
        usdc_token_client.balance(&user.address),
        contract_client.get_pool(&gold_pool_address).total_supply().unwrap(),
        gold_token_client.balance(&contract_id),
        gold_token_client.balance(&user.address),
    );
    let add_collateral_r = Request::AddCollateral(StandardRequest {
        amount: DEFAULT_DEPOSIT_AMOUNT,
        pool_address: gold_pool_address.clone(),
    });
    let borrow_r = Request::Borrow(StandardRequest {
        amount: DEFAULT_DEPOSIT_AMOUNT,
        pool_address: usdc_pool_address.clone(),
    });
    let repay_r = Request::Repay(StandardRequest {
        amount: DEFAULT_DEPOSIT_AMOUNT,
        pool_address: usdc_pool_address.clone(),
    });
    let remove_collateral_r = Request::RemoveCollateral(StandardRequest {
        amount: DEFAULT_COLLATERAL_AMOUNT,
        pool_address: gold_pool_address.clone(),
    });
    let r_1 = svec![&e, add_collateral_r, borrow_r, repay_r, remove_collateral_r];

    contract_client.submit_requests_batch(user, &r_1, &None);

    let tuple_after = (
        contract_client.get_pool(&usdc_pool_address).total_supply().unwrap(),
        usdc_token_client.balance(&contract_id),
        usdc_token_client.balance(&user.address),
        contract_client.get_pool(&gold_pool_address).total_supply().unwrap(),
        gold_token_client.balance(&contract_id),
        gold_token_client.balance(&user.address),
    );

    assert_eq!(tuple_before, tuple_after);
}
