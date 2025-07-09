#![cfg(test)]

use crate::{
    get_deposit_obligation, LCError, TestFixture, DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT,
};

#[test]
fn test_withdraw_zero() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    contract_client.deposit(&user, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligation_shares = get_deposit_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .shares;
    let pool_shares = contract_client.get_pool(&usdc_pool_address).total_shares;

    assert_eq!(obligation_shares, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_shares, DEFAULT_DEPOSIT_AMOUNT);

    let usdc_pool_before = contract_client.get_pool(&usdc_pool_address);
    let gold_pool_before = contract_client.get_pool(&usdc_pool_address);
    let obligation_before = contract_client.get_user_obligation(&user);

    contract_client.withdraw(&user, &usdc_pool_address, &0);
    let usdc_pool_after = contract_client.get_pool(&usdc_pool_address);
    let gold_pool_after = contract_client.get_pool(&usdc_pool_address);
    let obligation_after = contract_client.get_user_obligation(&user);

    assert_eq!(obligation_before, obligation_after);
    assert_eq!(usdc_pool_before, usdc_pool_after);
    assert_eq!(gold_pool_before, gold_pool_after);
}

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

    let obligation_shares = get_deposit_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .shares;
    let pool_shares = contract_client.get_pool(&usdc_pool_address).total_shares;

    assert_eq!(obligation_shares, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(pool_shares, DEFAULT_DEPOSIT_AMOUNT);

    // Withdraw half
    contract_client.withdraw(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    let obligation_shares = get_deposit_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .shares;
    let pool_shares = contract_client.get_pool(&usdc_pool_address).total_shares;

    assert_eq!(obligation_shares, (DEFAULT_DEPOSIT_AMOUNT / 2));
    assert_eq!(pool_shares, (DEFAULT_DEPOSIT_AMOUNT / 2));

    // Withdraw half again
    contract_client.withdraw(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    assert_eq!(
        Err(Ok(LCError::ObligationDoesNotExist)),
        contract_client.try_get_user_obligation(&user)
    );

    let pool_shares = contract_client.get_pool(&usdc_pool_address).total_shares;

    assert_eq!(pool_shares, 0);
}

#[test]
fn test_remove_collateral() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    contract_client.add_collateral(&user, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    let obligation_collateral = get_deposit_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .collateral;
    let pool_collateral = contract_client
        .get_pool(&usdc_pool_address)
        .total_collateral;

    assert_eq!(obligation_collateral, DEFAULT_COLLATERAL_AMOUNT);
    assert_eq!(pool_collateral, DEFAULT_COLLATERAL_AMOUNT);

    // Withdraw half
    contract_client.remove_collateral(&user, &usdc_pool_address, &(DEFAULT_COLLATERAL_AMOUNT / 2));

    let obligation_collateral = get_deposit_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .collateral;
    let pool_collateral = contract_client
        .get_pool(&usdc_pool_address)
        .total_collateral;

    assert_eq!(obligation_collateral, (DEFAULT_COLLATERAL_AMOUNT / 2));
    assert_eq!(pool_collateral, (DEFAULT_COLLATERAL_AMOUNT / 2));

    // Withdraw half again
    contract_client.remove_collateral(&user, &usdc_pool_address, &(DEFAULT_COLLATERAL_AMOUNT / 2));

    assert_eq!(
        Err(Ok(LCError::ObligationDoesNotExist)),
        contract_client.try_get_user_obligation(&user)
    );

    let pool_collateral = contract_client
        .get_pool(&usdc_pool_address)
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
fn test_remove_collateral_overbalance() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    contract_client.add_collateral(&user, &usdc_pool_address, &(DEFAULT_COLLATERAL_AMOUNT / 2));
    contract_client.add_collateral(&user2, &usdc_pool_address, &(DEFAULT_COLLATERAL_AMOUNT / 2));

    contract_client.remove_collateral(
        &user,
        &usdc_pool_address,
        &((DEFAULT_COLLATERAL_AMOUNT / 2) + 1),
    );
}
