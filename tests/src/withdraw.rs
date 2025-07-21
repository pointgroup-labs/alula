#![cfg(test)]

use lending::constants::DEFAULT_LIQUIDATION_THRESHOLD;

use crate::{
    get_deposit_obligation, get_obligation_borrowed, get_obligation_collateral,
    get_obligation_tokens_from_shares, LCError, TestFixture, DEFAULT_COLLATERAL_AMOUNT,
    DEFAULT_DEPOSIT_AMOUNT,
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
fn test_remove_collateral_zero() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    contract_client.add_collateral(&user, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    let pool_before = contract_client.get_pool(&usdc_pool_address);
    let obligation_before = contract_client.get_user_obligation(&user);

    // Withdraw half
    contract_client.remove_collateral(&user, &usdc_pool_address, &0);

    let pool_after = contract_client.get_pool(&usdc_pool_address);
    let obligation_after = contract_client.get_user_obligation(&user);

    assert_eq!(pool_before, pool_after);
    assert_eq!(obligation_before, obligation_after);
}

#[test]
fn test_remove_collateral_negative() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    contract_client.add_collateral(&user, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    assert_eq!(
        contract_client.try_remove_collateral(&user, &usdc_pool_address, &-1),
        Err(Ok(LCError::NegativeCollateralRemoval))
    );
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
fn test_withdraw_all_with_i128_max() {
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

    let pool_deposited_before = contract_client
        .get_pool(&usdc_pool_address)
        .total_supply()
        .unwrap();

    contract_client.withdraw(&user, &usdc_pool_address, &i128::MAX);

    let pool_deposited_after = contract_client
        .get_pool(&usdc_pool_address)
        .total_supply()
        .unwrap();

    assert_eq!(
        pool_deposited_after + DEFAULT_DEPOSIT_AMOUNT,
        pool_deposited_before
    );
    assert_eq!(
        get_deposit_obligation(&contract_client, &user, &usdc_pool_address),
        Err(LCError::ObligationDoesNotExist)
    );
}

#[test]
fn test_withdraw_more_than_open_ltv_allows() {
    const MAX_BORROWING_AMOUNT: i128 =
        (DEFAULT_DEPOSIT_AMOUNT * DEFAULT_LIQUIDATION_THRESHOLD) / 100;

    let TestFixture {
        e,
        contract_client,
        contract_id,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Fill up the borrowing pool with liquidity
    contract_client.deposit(&user2, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    // Deposit gold as deposit that backs future borrows
    contract_client.deposit(&user, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligation = contract_client.get_user_obligation(&user);
    e.as_contract(&contract_id, || {
        let max_borrowing_amount = obligation
            .compute_max_healthy_borrow_added_amount(&e, &usdc_pool_address)
            .unwrap();

        assert_eq!(max_borrowing_amount, MAX_BORROWING_AMOUNT);
    });

    // Borrow half
    contract_client.borrow(&user, &usdc_pool_address, &(MAX_BORROWING_AMOUNT / 2));

    let borrowed_amount =
        get_obligation_borrowed(&contract_client, &user, &usdc_pool_address).unwrap();
    assert_eq!(borrowed_amount, MAX_BORROWING_AMOUNT / 2);

    let obligation = contract_client.get_user_obligation(&user);
    e.as_contract(&contract_id, || {
        let max_borrowing_amount = obligation
            .compute_max_healthy_borrow_added_amount(&e, &gold_pool_address)
            .unwrap();

        assert_eq!(max_borrowing_amount, MAX_BORROWING_AMOUNT / 2);
    });

    // Try withdraw all
    contract_client.withdraw(&user, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    // Check that there's a deposit left and it is backing the borrowed funds
    let deposit_amount =
        get_obligation_tokens_from_shares(&e, &contract_client, &user, &gold_pool_address).unwrap();

    // The deposit that backs borrowed funds must be present on the contract
    assert_eq!(
        deposit_amount,
        (borrowed_amount * 100) / DEFAULT_LIQUIDATION_THRESHOLD
    );
}

#[test]
fn test_remove_all_with_i128_max() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();

    contract_client.add_collateral(&user, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    let pool_collateral_before = contract_client
        .get_pool(&usdc_pool_address)
        .total_collateral;

    contract_client.remove_collateral(&user, &usdc_pool_address, &i128::MAX);

    let pool_collateral_after = contract_client
        .get_pool(&usdc_pool_address)
        .total_collateral;

    assert_eq!(
        pool_collateral_before - DEFAULT_COLLATERAL_AMOUNT,
        pool_collateral_after
    );
    assert_eq!(
        get_deposit_obligation(&contract_client, &user, &usdc_pool_address),
        Err(LCError::ObligationDoesNotExist)
    );
}

#[test]
fn test_remove_collateral_more_than_open_ltv_allows() {
    const MAX_BORROWING_AMOUNT: i128 =
        (DEFAULT_COLLATERAL_AMOUNT * DEFAULT_LIQUIDATION_THRESHOLD) / 100;

    let TestFixture {
        e,
        contract_client,
        contract_id,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit funds in a pool to borrow them later
    contract_client.deposit(&user2, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    // Add collateral
    contract_client.add_collateral(&user, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    let obligation = contract_client.get_user_obligation(&user);
    e.as_contract(&contract_id, || {
        let max_borrowing_amount = obligation
            .compute_max_healthy_borrow_added_amount(&e, &usdc_pool_address)
            .unwrap();

        assert_eq!(max_borrowing_amount, MAX_BORROWING_AMOUNT);
    });

    // Borrow half
    contract_client.borrow(&user, &usdc_pool_address, &(MAX_BORROWING_AMOUNT / 2));

    let borrowed_amount =
        get_obligation_borrowed(&contract_client, &user, &usdc_pool_address).unwrap();
    assert_eq!(borrowed_amount, MAX_BORROWING_AMOUNT / 2);

    let obligation = contract_client.get_user_obligation(&user);
    e.as_contract(&contract_id, || {
        let max_borrowing_amount = obligation
            .compute_max_healthy_borrow_added_amount(&e, &gold_pool_address)
            .unwrap();

        assert_eq!(max_borrowing_amount, MAX_BORROWING_AMOUNT / 2);
    });

    // Try to remove all collateral
    contract_client.remove_collateral(&user, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    // Check that there's a collateral left and it is backing the borrowed funds
    let collateral_amount =
        get_obligation_collateral(&contract_client, &user, &gold_pool_address).unwrap();

    // The collateral that backs borrowed funds must be present on the contract
    assert_eq!(
        collateral_amount,
        (borrowed_amount * 100) / DEFAULT_LIQUIDATION_THRESHOLD
    );
}
