#![cfg(test)]

use market::{
    constants::*,
    pool::{PoolConfig, PoolHealthConfig},
};

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, MCError, TestMarketFixture,
    assert_approx_eq_rel, get_deposit_obligation, get_obligation_collateral,
    get_obligation_deposited, get_obligation_j_tokens, get_obligation_j_tokens_as_tokens,
    get_pool_total_available, get_pool_total_borrowed, get_pool_total_collateral,
    get_pool_total_j_tokens, get_pool_total_supply,
};

#[test]
fn test_withdraw() {
    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: BPS_FACTOR,
            ..Default::default()
        },
        ..Default::default()
    };
    let TestMarketFixture { e, contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new_with_pool_config(pool_config);
    let creditor = &users[0];

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    // Withdraw 50%
    contract_client.withdraw(creditor, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    let obligation_supplied =
        get_obligation_deposited(&contract_client, creditor, &gold_pool_address).unwrap();
    let obligation_j_tokens =
        get_obligation_j_tokens(&contract_client, creditor, &gold_pool_address).unwrap();
    let obligation_j_tokens_as_tokens =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address)
            .unwrap();

    assert_eq!(obligation_supplied, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(obligation_j_tokens, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(obligation_j_tokens_as_tokens, DEFAULT_DEPOSIT_AMOUNT / 2);

    let pool_total_supply = get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let pool_total_j_tokens = get_pool_total_j_tokens(&contract_client, &gold_pool_address);
    let pool_total_available = get_pool_total_available(&contract_client, &gold_pool_address);

    assert_eq!(pool_total_supply, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(pool_total_j_tokens, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(pool_total_available, DEFAULT_DEPOSIT_AMOUNT / 2);

    // Withdraw 50% again
    contract_client.withdraw(creditor, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    assert_eq!(
        get_obligation_deposited(&contract_client, creditor, &gold_pool_address),
        Err(MCError::ObligationDoesNotExist)
    );
    assert_eq!(
        get_obligation_j_tokens(&contract_client, creditor, &gold_pool_address),
        Err(MCError::ObligationDoesNotExist)
    );
    assert_eq!(
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address),
        Err(MCError::ObligationDoesNotExist)
    );

    let pool_total_supply = get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let pool_total_j_tokens = get_pool_total_j_tokens(&contract_client, &gold_pool_address);
    let pool_total_available = get_pool_total_available(&contract_client, &gold_pool_address);

    assert_eq!(pool_total_supply, 0);
    assert_eq!(pool_total_j_tokens, 0);
    assert_eq!(pool_total_available, 0);
}

#[test]
fn test_remove_collateral() {
    let TestMarketFixture { e, contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let collateral_provider = &users[0];

    contract_client.add_collateral(
        collateral_provider,
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
    );

    // Remove 50%
    contract_client.remove_collateral(
        collateral_provider,
        &gold_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
    );

    let obligation_collateral =
        get_obligation_collateral(&contract_client, collateral_provider, &gold_pool_address)
            .unwrap();
    let obligation_j_tokens =
        get_obligation_j_tokens(&contract_client, collateral_provider, &gold_pool_address).unwrap();
    let obligation_j_tokens_as_tokens = get_obligation_j_tokens_as_tokens(
        &e,
        &contract_client,
        collateral_provider,
        &gold_pool_address,
    )
    .unwrap();

    assert_eq!(obligation_collateral, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(obligation_j_tokens, 0);
    assert_eq!(obligation_j_tokens_as_tokens, 0);

    let pool_total_supply = get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let pool_total_j_tokens = get_pool_total_j_tokens(&contract_client, &gold_pool_address);
    let pool_total_available = get_pool_total_available(&contract_client, &gold_pool_address);
    let pool_total_collateral = get_pool_total_collateral(&contract_client, &gold_pool_address);

    assert_eq!(pool_total_collateral, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(pool_total_supply, 0);
    assert_eq!(pool_total_j_tokens, 0);
    assert_eq!(pool_total_available, 0);

    // Remove 50% again
    contract_client.remove_collateral(
        collateral_provider,
        &gold_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
    );
    // TODO: Investigate what happens if you withdraw here instead of removing collateral
    // contract_client.withdraw(collateral_provider, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT /
    // 2));

    assert_eq!(
        get_obligation_collateral(&contract_client, collateral_provider, &gold_pool_address),
        Err(MCError::ObligationDoesNotExist)
    );
    assert_eq!(
        get_obligation_j_tokens(&contract_client, collateral_provider, &gold_pool_address),
        Err(MCError::ObligationDoesNotExist)
    );
    assert_eq!(
        get_obligation_j_tokens_as_tokens(
            &e,
            &contract_client,
            collateral_provider,
            &gold_pool_address
        ),
        Err(MCError::ObligationDoesNotExist)
    );

    let pool_total_supply = get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let pool_total_j_tokens = get_pool_total_j_tokens(&contract_client, &gold_pool_address);
    let pool_total_available = get_pool_total_available(&contract_client, &gold_pool_address);
    let pool_total_collateral = get_pool_total_collateral(&contract_client, &gold_pool_address);

    assert_eq!(pool_total_collateral, 0);
    assert_eq!(pool_total_supply, 0);
    assert_eq!(pool_total_j_tokens, 0);
    assert_eq!(pool_total_available, 0);
}

#[test]
fn test_withdraw_zero() {
    let TestMarketFixture { contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let creditor = &users[0];

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligation_before =
        get_deposit_obligation(&contract_client, creditor, &gold_pool_address).unwrap();
    let pool_before = contract_client.get_pool(&gold_pool_address);

    contract_client.withdraw(creditor, &gold_pool_address, &0);

    let obligation_after =
        get_deposit_obligation(&contract_client, creditor, &gold_pool_address).unwrap();
    let pool_after = contract_client.get_pool(&gold_pool_address);

    assert_eq!(obligation_before, obligation_after);
    assert_eq!(pool_before, pool_after);
}

#[test]
fn test_remove_collateral_zero() {
    let TestMarketFixture { contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let collateral_provider = &users[0];

    contract_client.add_collateral(
        collateral_provider,
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
    );

    let obligation_before =
        get_deposit_obligation(&contract_client, collateral_provider, &gold_pool_address).unwrap();
    let pool_before = contract_client.get_pool(&gold_pool_address);

    contract_client.remove_collateral(collateral_provider, &gold_pool_address, &0);

    let obligation_after =
        get_deposit_obligation(&contract_client, collateral_provider, &gold_pool_address).unwrap();
    let pool_after = contract_client.get_pool(&gold_pool_address);

    assert_eq!(obligation_before, obligation_after);
    assert_eq!(pool_before, pool_after);
}

#[test]
fn test_withdraw_negative() {
    let TestMarketFixture { contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let creditor = &users[0];

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    assert_eq!(
        contract_client.try_withdraw(creditor, &gold_pool_address, &-1),
        Err(Ok(MCError::NegativeAmount))
    );
}

#[test]
fn test_remove_collateral_negative() {
    let TestMarketFixture { contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let collateral_provider = &users[0];

    contract_client.add_collateral(
        collateral_provider,
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
    );

    assert_eq!(
        contract_client.try_remove_collateral(collateral_provider, &gold_pool_address, &-1),
        Err(Ok(MCError::NegativeAmount))
    );
}

#[test]
fn test_withdraw_all_with_i128_max() {
    let TestMarketFixture { contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let creditor_1 = &users[0];
    let creditor_2 = &users[1];

    contract_client.deposit(creditor_1, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(creditor_2, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_total_supply_before =
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    contract_client.withdraw(creditor_1, &gold_pool_address, &i128::MAX);
    let pool_total_supply_after =
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();

    assert_eq!(pool_total_supply_after + DEFAULT_DEPOSIT_AMOUNT, pool_total_supply_before);
    assert_eq!(
        get_deposit_obligation(&contract_client, creditor_1, &gold_pool_address),
        Err(MCError::ObligationDoesNotExist)
    );
}

#[test]
fn test_remove_all_with_i128_max() {
    let TestMarketFixture { contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let creditor_1 = &users[0];
    let creditor_2 = &users[1];

    contract_client.add_collateral(creditor_1, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);
    contract_client.add_collateral(creditor_2, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    let pool_total_collateral_before =
        get_pool_total_collateral(&contract_client, &gold_pool_address);
    contract_client.remove_collateral(creditor_1, &gold_pool_address, &i128::MAX);
    let pool_total_collateral_after =
        get_pool_total_collateral(&contract_client, &gold_pool_address);

    assert_eq!(
        pool_total_collateral_after + DEFAULT_COLLATERAL_AMOUNT,
        pool_total_collateral_before
    );
    assert_eq!(
        get_deposit_obligation(&contract_client, creditor_1, &gold_pool_address),
        Err(MCError::ObligationDoesNotExist)
    );
}

#[test]
fn test_withdraw_exceeds_utilization_cap() {
    const UTILIZATION_RATIO_LIMIT_BPS: i128 = 9000; // 90%

    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: UTILIZATION_RATIO_LIMIT_BPS,
            ..Default::default()
        },
        ..Default::default()
    };

    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new_with_pool_config(pool_config);
    let creditor = &users[0];
    let borrower = &users[1];

    contract_client.add_collateral(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(creditor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    // Borrow 10% of USDC
    contract_client.borrow(borrower, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

    let pool_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_total_supply = get_pool_total_supply(&contract_client, &usdc_pool_address).unwrap();

    // Check that the utilization ratio is 10%
    assert_eq!(pool_total_supply / pool_borrowed, 10);
    // Try to withdraw more than UR cap allows
    assert_eq!(
        contract_client.try_withdraw(
            creditor,
            &usdc_pool_address,
            &(81 * DEFAULT_DEPOSIT_AMOUNT / 100), // 10% + 81% > 90%
        ),
        Err(Ok(MCError::PoolUtilizationRatioCapExceeded))
    );
    assert!(
        contract_client
            .try_withdraw(
                creditor,
                &usdc_pool_address,
                &(8 * DEFAULT_DEPOSIT_AMOUNT / 10), // 90%
            )
            .is_ok()
    );
}

#[test]
fn withdraw_up_to_open_ltv() {
    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let user = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(user, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(user, &usdc_pool_address, &((DEFAULT_DEPOSIT_AMOUNT) / 2));

    let obligation_j_tokens_before =
        get_obligation_j_tokens(&contract_client, user, &gold_pool_address).unwrap();
    // Try to withdraw more than default openLTV(70%) allows
    contract_client.withdraw(user, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    let obligation_j_tokens_after =
        get_obligation_j_tokens(&contract_client, user, &gold_pool_address).unwrap();

    assert_eq!(obligation_j_tokens_before, DEFAULT_DEPOSIT_AMOUNT);
    // Check that the required amount to back up the borrow remains
    assert_approx_eq_rel(
        // TODO: Investigate a bit deeper when checking maths
        obligation_j_tokens_after,
        (100 * (obligation_j_tokens_before / 2)) / 70,
        5,
    );
}

#[test]
fn remove_collateral_up_to_open_ltv() {
    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
    let user = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.add_collateral(user, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(user, &usdc_pool_address, &((DEFAULT_DEPOSIT_AMOUNT) / 2));

    let obligation_collateral_before =
        get_obligation_collateral(&contract_client, user, &gold_pool_address).unwrap();
    // Try to remove more than default openLTV(70%) allows
    contract_client.remove_collateral(user, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    let obligation_collateral_after =
        get_obligation_collateral(&contract_client, user, &gold_pool_address).unwrap();

    assert_eq!(obligation_collateral_before, DEFAULT_DEPOSIT_AMOUNT);
    // Check that the required amount to back up the borrow remains
    assert_approx_eq_rel(
        // TODO: Investigate a bit deeper when checking maths
        obligation_collateral_after,
        (BPS_FACTOR * (obligation_collateral_before / 2)) / DEFAULT_OPEN_LTV_BPS,
        5,
    );
}

// TODO: Add time passing test
