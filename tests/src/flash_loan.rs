#![cfg(test)]

use {
    crate::{get_borrow_obligation, TestFixture, DEFAULT_DEPOSIT_AMOUNT},
    lending::{constants::LCError, flash_loan_utilizer},
    soroban_sdk::{testutils::Ledger, Address, String},
};

const FLASH_LOAN_CLIENT_ADDRESS: &str = "CACOK7HB7D7SRPMH3LYYOW77T6D4D2F7TR7UEVKY2TVSUDSRDM6DZVLK";

#[test]
fn test_flash_loan_liquidation() {
    let TestFixture {
        e,
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let flash_loan_taker_contract_address =
        Address::from_string(&String::from_str(&e, FLASH_LOAN_CLIENT_ADDRESS));
    e.register_at(
        &flash_loan_taker_contract_address,
        flash_loan_utilizer::WASM,
        (),
    );

    let user: Address = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    // Deposit gold to satisfy the health factor threshold
    contract_client.deposit(&user, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(3 * DEFAULT_DEPOSIT_AMOUNT));

    // Borrow maximum possible amount
    contract_client.borrow(
        &user,
        &usdc_pool_address,
        &((8 * DEFAULT_DEPOSIT_AMOUNT) / 10),
    );

    assert_eq!(
        contract_client.try_borrow(&user, &usdc_pool_address, &(1)),
        Err(Ok(LCError::HealthFactorIsLowerThanRequiredThreshold))
    );

    assert_eq!(
        get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
            .unwrap()
            .unpaid_interest,
        0
    );

    // Wait a borrowed amount to accrue
    e.ledger().with_mut(|li| li.timestamp = 24 * 60 * 60);

    // Check that the borrowed amount has indeed accrued
    assert!(
        get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
            .unwrap()
            .unpaid_interest
            > 0
    );

    let total_borrowed = get_borrow_obligation(&contract_client, &user, &usdc_pool_address)
        .unwrap()
        .total_borrowed()
        .unwrap();

    // Initiate a liquidation via flash loan

    // Register liquidatable
    // flash_loan_taker_contract_address.register_liquidatable

    contract_client.flash_loan(
        &flash_loan_taker_contract_address,
        &flash_loan_taker_contract_address,
        &usdc_pool_address,
        &total_borrowed,
    );
}
