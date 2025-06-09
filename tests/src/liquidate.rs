use crate::{get_borrow_obligation, get_deposit_obligation, TestFixture, DEFAULT_DEPOSIT_AMOUNT};

use {
    lending::constants::{LCError, DEFAULT_CLOSE_FACTOR},
    soroban_sdk::{testutils::Ledger, Address},
};

#[test]
fn test_liquidate() {
    let TestFixture {
        e,
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user: Address = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();
    let liquidator = users.get(2).unwrap();
    // Deposit gold to satisfy the health factor threshold
    contract_client.deposit(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    // Try to liquidate a healthy position
    assert_eq!(
        contract_client.try_liquidate(&liquidator, &user, &usdc_pool_address, &1),
        Err(Ok(LCError::LiquidatedPositionIsHealthy))
    );

    // Borrow maximum possible amount
    contract_client.borrow(
        &user,
        &usdc_pool_address,
        &((3 * DEFAULT_DEPOSIT_AMOUNT) / 10),
    );

    assert_eq!(
        contract_client.try_borrow(&user, &usdc_pool_address, &(1)),
        Err(Ok(LCError::HealthFactorIsLowerThanRequiredThreshold))
    );

    // Try to liquidate a least healthy position possible
    assert_eq!(
        contract_client.try_liquidate(&liquidator, &user, &usdc_pool_address, &1),
        Err(Ok(LCError::LiquidatedPositionIsHealthy))
    );

    let borrowed = get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .borrowed;
    let deposited = get_deposit_obligation(&contract_client, &user, &gold_pool_address)
        .unwrap()
        .deposited;

    // Wait an hour for borrowed amount to accrue
    e.ledger().with_mut(|li| li.timestamp = 60 * 60);

    let liquidatable_amount = ((DEFAULT_CLOSE_FACTOR - 1) * borrowed) / 100;
    let non_liquidatable_amount = ((DEFAULT_CLOSE_FACTOR + 1) * borrowed) / 100;

    // Liquidation which exceeds close factor percentage fails
    assert_eq!(
        contract_client.try_liquidate(
            &liquidator,
            &user,
            &usdc_pool_address,
            &non_liquidatable_amount,
        ),
        Err(Ok(LCError::LiquidationExceedsCloseFactor))
    );

    contract_client.liquidate(&liquidator, &user, &usdc_pool_address, &liquidatable_amount);

    let new_borrowed = get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .borrowed;
    let new_deposited = get_deposit_obligation(&contract_client, &user, &gold_pool_address)
        .unwrap()
        .deposited;

    // TODO: Check more specifically how liquidation affected obligation
    assert!(new_borrowed < borrowed);
    assert!(new_deposited < deposited);
}

// TODO:
// #[test]
// fn test_liquidate_multiple_collaterals() {}
