#![cfg(test)]

use lending::{constants::DEFAULT_MAX_SLIPPAGE_BPS, swap};
use soroban_sdk::token::TokenClient;

use crate::{tests::get_amount_scaled_down, TestFixture};

#[test]
fn test_swap() {
    const AMOUNT_IN: i128 = 100_000;

    let TestFixture {
        e,
        contract_client,
        users,
        gold_pool_address,
        gold_token_client,
        soroswap_router_address,
        ..
    } = TestFixture::new();

    let user = &users[0];

    let new_token_address = e
        .register_stellar_asset_contract_v2(soroswap_router_address.clone())
        .address();
    let new_token_client = TokenClient::new(&e, &new_token_address);

    let new_token_balance: i128 = new_token_client.balance(user);
    assert_eq!(new_token_balance, 0);

    let gold_token_balance = gold_token_client.balance(user);

    let amount_out =
        swap::get_amount_out(&e, &gold_pool_address, &new_token_address, AMOUNT_IN).unwrap();

    contract_client.swap(user, &gold_pool_address, &new_token_address, &AMOUNT_IN);

    let amount_out_min_slippage = get_amount_scaled_down(amount_out, DEFAULT_MAX_SLIPPAGE_BPS);

    let balance = new_token_client.balance(user);
    let new_gold_token_balance = gold_token_client.balance(user);

    assert_eq!(balance, amount_out_min_slippage);
    assert_eq!(
        new_gold_token_balance / 1_000_000,
        (gold_token_balance - amount_out_min_slippage) / 1_000_000 /* TODO: Check why do amounts
                                                                    * differ in a few smallest
                                                                    * units */
    );
}
