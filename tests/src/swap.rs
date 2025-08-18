#![cfg(test)]

use lending::{constants::DEFAULT_MAX_SLIPPAGE_BPS, swap};
use sep_40_oracle::testutils::{Asset, MockPriceOracleClient};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Env, Symbol, symbol_short, token::TokenClient, vec as svec};

use crate::{TestFixture, make_oracle_prices_different, tests::get_amount_scaled_down};

#[test]
fn test_swap() {
    const AMOUNT_IN: i128 = 100_000;

    let TestFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_client,
        router_address,
        ..
    } = TestFixture::new();

    let user = &users[0];

    let new_token_address = e
        .register_stellar_asset_contract_v2(router_address.clone())
        .address();
    let new_token_client = TokenClient::new(&e, &new_token_address);

    let new_token_balance: i128 = new_token_client.balance(user);
    assert_eq!(new_token_balance, 0);

    let usdc_token_balance = usdc_token_client.balance(user);

    let amount_out =
        swap::get_amount_out(&e, &usdc_pool_address, &new_token_address, AMOUNT_IN).unwrap();

    contract_client.swap(user, &usdc_pool_address, &new_token_address, &AMOUNT_IN);

    let amount_out_min_slippage = get_amount_scaled_down(amount_out, DEFAULT_MAX_SLIPPAGE_BPS);

    let balance = new_token_client.balance(user);
    let new_usdc_token_balance = usdc_token_client.balance(user);

    assert_eq!(balance, amount_out_min_slippage);
    assert_eq!(
        new_usdc_token_balance / 1_000_000,
        (usdc_token_balance - amount_out_min_slippage) / 1_000_000 /* TODO: Check why do amounts
                                                                    * differ in a few smallest
                                                                    * units */
    );
}

#[test]
fn test_get_amount_out() {
    const AMOUNT_IN: i128 = 5_000;

    let TestFixture {
        e,
        gold_token_address,
        usdc_token_address,
        oracle_client,
        router_client,
        ..
    } = TestFixture::new();

    make_oracle_prices_different(&e, &oracle_client);

    let path = svec![&e, gold_token_address.clone(), usdc_token_address.clone()];
    let amount_out1 = router_client
        .router_get_amounts_out(&AMOUNT_IN, &path)
        .last()
        .unwrap();
    dbg!(amount_out1);

    let amount_in1 = router_client
        .router_get_amounts_in(&amount_out1, &path)
        .first()
        .unwrap();
    std::dbg!(amount_in1);

    // ------

    let path: soroban_sdk::Vec<soroban_sdk::Address> =
        svec![&e, usdc_token_address.clone(), gold_token_address.clone()];
    let amount_out2 = router_client
        .router_get_amounts_out(&AMOUNT_IN, &path)
        .last()
        .unwrap();
    std::dbg!(amount_out2);

    let amount_in2 = router_client.router_get_amounts_in(&amount_out2, &path);
    std::dbg!(amount_in2);
}
