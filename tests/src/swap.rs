#![cfg(test)]

use market::swap;
use soroban_sdk::{Address, testutils::Address as _, vec as svec};

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, assert_approx_eq_rel, make_oracle_prices_different,
};

#[test]
fn test_swap_exact() {
    const AMOUNT_OUT: i128 = 5_000;
    const DELTA_BPS: i128 = 5; // 0.05 %

    let TestMarketFixture { e, gold_token_address, usdc_token_address, oracle_client, .. } =
        TestMarketFixture::new();

    make_oracle_prices_different(&e, &oracle_client);

    let gold_usdc_amount_in =
        swap::get_amount_in(&e, &gold_token_address, &usdc_token_address, AMOUNT_OUT).unwrap();
    let gold_usdc_amount_out =
        swap::get_amount_out(&e, &gold_token_address, &usdc_token_address, gold_usdc_amount_in)
            .unwrap();

    let usdc_gold_amount_in =
        swap::get_amount_in(&e, &usdc_token_address, &gold_token_address, AMOUNT_OUT).unwrap();
    let usdc_gold_amount_out =
        swap::get_amount_out(&e, &usdc_token_address, &gold_token_address, usdc_gold_amount_in)
            .unwrap();

    assert_approx_eq_rel(gold_usdc_amount_out, AMOUNT_OUT, DELTA_BPS);
    assert_approx_eq_rel(usdc_gold_amount_out, AMOUNT_OUT, DELTA_BPS);
}

#[test]
fn test_get_amount_out() {
    const AMOUNT_IN: i128 = 5_000;
    const DELTA_BPS: i128 = 100; // 1 %

    let TestMarketFixture {
        e,
        gold_token_address,
        usdc_token_address,
        oracle_client,
        router_client,
        ..
    } = TestMarketFixture::new();

    make_oracle_prices_different(&e, &oracle_client);

    let gold_usdc_path = svec![&e, gold_token_address.clone(), usdc_token_address.clone()];
    let usdc_gold_path = svec![&e, usdc_token_address.clone(), gold_token_address.clone()];

    let gold_usdc_amount_out =
        router_client.router_get_amounts_out(&AMOUNT_IN, &gold_usdc_path).last().unwrap();
    let gold_usdc_amount_in = router_client
        .router_get_amounts_in(&gold_usdc_amount_out, &gold_usdc_path)
        .first()
        .unwrap();

    let usdc_gold_amount_out =
        router_client.router_get_amounts_out(&AMOUNT_IN, &usdc_gold_path).last().unwrap();
    let usdc_gold_amount_in = router_client
        .router_get_amounts_in(&usdc_gold_amount_out, &usdc_gold_path)
        .first()
        .unwrap();

    // NB: Approximate check takes place because of the truncation that occurs in the whole numbers'
    // arithmetic
    assert_approx_eq_rel(gold_usdc_amount_in, AMOUNT_IN, DELTA_BPS);
    assert_approx_eq_rel(usdc_gold_amount_in, AMOUNT_IN, DELTA_BPS);
}

// TODO: Add a test for `swap_exact_tokens` and `get_amount_in` consistency after adding such a
// `swap` endpoint

#[test]
fn test_get_amount_in() {
    const AMOUNT_OUT: i128 = 5_000;
    const DELTA_BPS: i128 = 5; // 0.05 %

    let TestMarketFixture { e, gold_token_address, usdc_token_address, oracle_client, .. } =
        TestMarketFixture::new();

    make_oracle_prices_different(&e, &oracle_client);

    let gold_usdc_amount_in =
        swap::get_amount_in(&e, &gold_token_address, &usdc_token_address, AMOUNT_OUT).unwrap();
    let gold_usdc_amount_out =
        swap::get_amount_out(&e, &gold_token_address, &usdc_token_address, gold_usdc_amount_in)
            .unwrap();

    let usdc_gold_amount_in =
        swap::get_amount_in(&e, &usdc_token_address, &gold_token_address, AMOUNT_OUT).unwrap();
    let usdc_gold_amount_out =
        swap::get_amount_out(&e, &usdc_token_address, &gold_token_address, usdc_gold_amount_in)
            .unwrap();

    assert_approx_eq_rel(gold_usdc_amount_out, AMOUNT_OUT, DELTA_BPS);
    assert_approx_eq_rel(usdc_gold_amount_out, AMOUNT_OUT, DELTA_BPS);
}

// --- Proxy Swap ---

#[test]
fn test_proxy_swap_exact() {
    let TestMarketFixture {
        e,
        full_contract_client,
        usdc_token_address,
        usdc_token_client,
        gold_token_address,
        gold_token_client,
        users,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    let swap_provider = Address::generate(&e);
    let amount_out = full_contract_client.proxy_get_amount_out(
        &swap_provider,
        &usdc_token_address,
        &gold_token_address,
        &DEFAULT_DEPOSIT_AMOUNT,
    );

    let usdc_balance_before = usdc_token_client.balance(&user.address);
    let gold_balance_before = gold_token_client.balance(&user.address);

    full_contract_client.proxy_swap_exact(
        &swap_provider,
        &user.address,
        &usdc_token_address,
        &gold_token_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &amount_out,
    );

    let usdc_balance_after = usdc_token_client.balance(&user.address);
    let gold_balance_after = gold_token_client.balance(&user.address);

    let usdc_diff = usdc_balance_after.checked_sub(usdc_balance_before).unwrap();
    let gold_diff = gold_balance_after.checked_sub(gold_balance_before).unwrap();

    assert_eq!(usdc_diff, -DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(gold_diff, DEFAULT_DEPOSIT_AMOUNT);
}

#[test]
fn test_proxy_swap_for_exact() {
    let TestMarketFixture {
        e,
        full_contract_client,
        usdc_token_address,
        usdc_token_client,
        gold_token_address,
        gold_token_client,
        users,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];

    let swap_provider = Address::generate(&e);
    let amount_in = full_contract_client.proxy_get_amount_out(
        &swap_provider,
        &usdc_token_address,
        &gold_token_address,
        &DEFAULT_DEPOSIT_AMOUNT,
    );

    let usdc_balance_before = usdc_token_client.balance(&user.address);
    let gold_balance_before = gold_token_client.balance(&user.address);

    full_contract_client.proxy_swap_for_exact(
        &swap_provider,
        &user.address,
        &usdc_token_address,
        &gold_token_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &amount_in,
    );

    let usdc_balance_after = usdc_token_client.balance(&user.address);
    let gold_balance_after = gold_token_client.balance(&user.address);

    let usdc_diff = usdc_balance_after.checked_sub(usdc_balance_before).unwrap();
    let gold_diff = gold_balance_after.checked_sub(gold_balance_before).unwrap();

    assert_eq!(usdc_diff, -DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(gold_diff, DEFAULT_DEPOSIT_AMOUNT);
}
