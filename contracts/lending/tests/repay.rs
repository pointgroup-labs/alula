mod common;
use common::{get_borrow_obligation, TestFixture, DEFAULT_DEPOSIT_AMOUNT};

use soroban_sdk::{testutils::Ledger, Address};

#[test]
fn test_repay() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user: Address = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();
    // Deposit gold as a collateral to satisfy the health factor threshold
    contract_client.deposit_collateral(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // Borrow 50% of the deposited value
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    let obligation_borrowed = get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .borrowed;
    let pool_borrowed = contract_client
        .get_pool(&usdc_pool_address)
        .unwrap()
        .borrowed;

    assert_eq!(obligation_borrowed, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(pool_borrowed, DEFAULT_DEPOSIT_AMOUNT / 2);

    // Repay the half
    contract_client.repay(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    let obligation_borrowed = get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .borrowed;
    let pool_borrowed = contract_client
        .get_pool(&usdc_pool_address)
        .unwrap()
        .borrowed;

    assert_eq!(obligation_borrowed, DEFAULT_DEPOSIT_AMOUNT / 4);
    assert_eq!(pool_borrowed, DEFAULT_DEPOSIT_AMOUNT / 4);

    // Repay the rest
    contract_client.repay(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    let obligation = contract_client.get_user_obligation(&user);
    assert!(obligation.borrows.get(usdc_pool_address.clone()).is_none());

    let pool_borrowed = contract_client
        .get_pool(&usdc_pool_address)
        .unwrap()
        .borrowed;

    assert_eq!(pool_borrowed, 0);
}

#[test]
fn test_repay_with_interest_accrual() {
    let TestFixture {
        e,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user: Address = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();
    // Deposit gold as a collateral to satisfy the health factor threshold
    contract_client.deposit_collateral(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // Borrow 50% of the deposited value
    contract_client.borrow(
        &user,
        &usdc_pool_address,
        &(5 * DEFAULT_DEPOSIT_AMOUNT / 10),
    );

    let obligation_borrowed = get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .borrowed;
    let pool_borrowed = contract_client
        .get_pool(&usdc_pool_address)
        .unwrap()
        .borrowed;

    assert_eq!(obligation_borrowed, 5 * DEFAULT_DEPOSIT_AMOUNT / 10);
    assert_eq!(pool_borrowed, 5 * DEFAULT_DEPOSIT_AMOUNT / 10);

    // Wait for 5 hours to pass by
    e.ledger().with_mut(|li| li.timestamp = 60 * 60 * 5);

    let obligation_borrowed = get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .borrowed;

    // TODO: `pool_borrowed`` must likely increase as well, but what about `pool_deposited`?
    // If nobody repaid the interest and pool_deposited increased - pool doesn't contain the `pool_deposited` amount required...
    // let _pool_borrowed = contract_client
    //     .get_pool(&usdc_pool_address)
    //     .unwrap()
    //     .borrowed;

    assert!(obligation_borrowed > 5 * DEFAULT_DEPOSIT_AMOUNT / 10);

    let left = obligation_borrowed;

    contract_client.repay(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

    let obligation_borrowed = get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .borrowed;

    // Notice interest rate accrual
    assert_eq!(obligation_borrowed, left - (DEFAULT_DEPOSIT_AMOUNT / 10));

    // Wait another 15 hours to pass by
    e.ledger().with_mut(|li| li.timestamp = 60 * 60 * 15);

    let obligation_borrowed = get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .borrowed;

    // Notice interest rate accrual
    assert!(obligation_borrowed > left - (DEFAULT_DEPOSIT_AMOUNT / 10));

    // Repay everything
    contract_client.repay(&user, &usdc_pool_address, &obligation_borrowed);

    let obligation = contract_client.get_user_obligation(&user);
    assert!(obligation.borrows.is_empty());
}
