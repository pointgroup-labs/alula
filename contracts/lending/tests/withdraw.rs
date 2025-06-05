mod common;
use common::{
    get_deposit_obligation, TestFixture, DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT,
};

use lending::constants::LCError;

#[test]
fn test_withdraw() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    contract_client.deposit(&user, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let deposit_obligation = get_deposit_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .deposited;
    let pool_deposited = contract_client
        .get_pool(&usdc_pool_address)
        .unwrap()
        .total_supply;

    assert_eq!(deposit_obligation, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_deposited, DEFAULT_DEPOSIT_AMOUNT);

    // Withdraw half
    contract_client.withdraw(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    let deposit_obligation = get_deposit_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .deposited;
    let pool_deposited = contract_client
        .get_pool(&usdc_pool_address)
        .unwrap()
        .total_supply;

    assert_eq!(deposit_obligation, (DEFAULT_DEPOSIT_AMOUNT / 2));
    assert_eq!(pool_deposited, (DEFAULT_DEPOSIT_AMOUNT / 2));

    // Withdraw half again
    contract_client.withdraw(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    assert_eq!(
        Err(Ok(LCError::ObligationDoesNotExist)),
        contract_client.try_get_user_obligation(&user)
    );

    let pool_deposited = contract_client
        .get_pool(&usdc_pool_address)
        .unwrap()
        .total_supply;

    assert_eq!(pool_deposited, 0);
}

#[test]
fn test_withdraw_collateral() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    contract_client.deposit_collateral(&user, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    let obligation_collateral = get_deposit_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .collateral;
    let pool_collateral = contract_client
        .get_pool(&usdc_pool_address)
        .unwrap()
        .total_collateral;

    assert_eq!(obligation_collateral, DEFAULT_COLLATERAL_AMOUNT);
    assert_eq!(pool_collateral, DEFAULT_COLLATERAL_AMOUNT);

    // Withdraw half
    contract_client.withdraw_collateral(
        &user,
        &usdc_pool_address,
        &(DEFAULT_COLLATERAL_AMOUNT / 2),
    );

    let obligation_collateral = get_deposit_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .collateral;
    let pool_collateral = contract_client
        .get_pool(&usdc_pool_address)
        .unwrap()
        .total_collateral;

    assert_eq!(obligation_collateral, (DEFAULT_COLLATERAL_AMOUNT / 2));
    assert_eq!(pool_collateral, (DEFAULT_COLLATERAL_AMOUNT / 2));

    // Withdraw half again
    contract_client.withdraw_collateral(
        &user,
        &usdc_pool_address,
        &(DEFAULT_COLLATERAL_AMOUNT / 2),
    );

    assert_eq!(
        Err(Ok(LCError::ObligationDoesNotExist)),
        contract_client.try_get_user_obligation(&user)
    );

    let pool_collateral = contract_client
        .get_pool(&usdc_pool_address)
        .unwrap()
        .total_collateral;

    assert_eq!(pool_collateral, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_withdraw_overbalance() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    contract_client.deposit(&user, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(&user2, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.withdraw(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT + 1));
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_withdraw_collateral_overbalance() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    contract_client.deposit_collateral(&user, &usdc_pool_address, &(DEFAULT_COLLATERAL_AMOUNT / 2));
    contract_client.deposit_collateral(
        &user2,
        &usdc_pool_address,
        &(DEFAULT_COLLATERAL_AMOUNT / 2),
    );

    contract_client.withdraw(
        &user,
        &usdc_pool_address,
        &((DEFAULT_COLLATERAL_AMOUNT / 2) + 1),
    );
}
