#![cfg(test)]

use {crate::TestFixture, soroban_sdk::token::TokenClient};

#[test]
fn test_swap() {
    let TestFixture {
        e,
        contract_client,
        users,
        gold_pool_address,
        gold_token_client,
        soroswap_router_address,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();

    let new_token_address = e
        .register_stellar_asset_contract_v2(soroswap_router_address.clone())
        .address();
    let new_token_client = TokenClient::new(&e, &new_token_address);

    let new_token_balance: i128 = new_token_client.balance(&user);
    assert_eq!(new_token_balance, 0);

    let gold_token_balance = gold_token_client.balance(&user);

    contract_client.test_swap(&user, &gold_pool_address, &new_token_address, &100);

    let balance = new_token_client.balance(&user);
    let new_gold_token_balance = gold_token_client.balance(&user);

    assert_eq!(balance, 100);
    assert_eq!(new_gold_token_balance, gold_token_balance - 100);
}
