#![cfg(test)]

use market::swap;
use soroban_sdk::vec as svec;

use crate::{TestMarketFixture, assert_approx_eq_rel, make_oracle_prices_different};

#[test]
fn test_swap_equal_prices() {
    const AMOUNT_IN: i128 = 100_000;

    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_client,
        usdc_token_address,
        btc_pool_address,
        btc_token_client,
        btc_token_address,
        ..
    } = TestMarketFixture::new();

    let user = &users[0];

    let user_btc_balance_before = btc_token_client.balance(user);
    let user_usdc_balance_before = usdc_token_client.balance(user);

    let amount_out =
        swap::get_amount_out(&e, &btc_token_address, &usdc_token_address, AMOUNT_IN).unwrap();
    let received_amount =
        contract_client.swap(user, &btc_pool_address, &usdc_pool_address, &AMOUNT_IN);

    assert_eq!(received_amount, amount_out);

    let user_btc_balance_after = btc_token_client.balance(user);
    let user_usdc_balance_after = usdc_token_client.balance(user);

    let btc_balance_diff = user_btc_balance_after - user_btc_balance_before;
    let usdc_balance_diff = user_usdc_balance_after - user_usdc_balance_before;

    // BTC is swapped for USDC
    assert!(btc_balance_diff.is_negative() && usdc_balance_diff.is_positive());

    let abs_btc_balance_diff = -btc_balance_diff;

    assert_eq!(abs_btc_balance_diff, AMOUNT_IN);
    assert_eq!(usdc_balance_diff, amount_out);
}

#[test]
fn test_swap_different_prices() {
    const AMOUNT_IN: i128 = 100_000;

    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_client,
        usdc_token_address,
        gold_pool_address,
        gold_token_client,
        gold_token_address,
        oracle_client,
        ..
    } = TestMarketFixture::new();

    make_oracle_prices_different(&e, &oracle_client);

    let user = &users[0];

    let user_gold_balance_before = gold_token_client.balance(user);
    let user_usdc_balance_before = usdc_token_client.balance(user);

    let amount_out =
        swap::get_amount_out(&e, &gold_token_address, &usdc_token_address, AMOUNT_IN).unwrap();
    let received_amount =
        contract_client.swap(user, &gold_pool_address, &usdc_pool_address, &AMOUNT_IN);

    assert_eq!(received_amount, amount_out);

    let user_gold_balance_after = gold_token_client.balance(user);
    let user_usdc_balance_after = usdc_token_client.balance(user);

    let gold_balance_diff = user_gold_balance_after - user_gold_balance_before;
    let usdc_balance_diff = user_usdc_balance_after - user_usdc_balance_before;

    // GOLD is swapped for USDC
    assert!(gold_balance_diff.is_negative() && usdc_balance_diff.is_positive());

    let abs_gold_balance_diff = -gold_balance_diff;

    assert_eq!(abs_gold_balance_diff, AMOUNT_IN);
    assert_eq!(usdc_balance_diff, amount_out);
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
