#![cfg(test)]

use market::{
    constants::*,
    pool::{PoolConfig, PoolHealthConfig},
};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::testutils::Ledger;

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, MCError, TestMarketFixture,
    assert_approx_eq_rel, get_deposit_position, get_obligation_collateral,
    get_obligation_j_tokens_as_tokens, get_obligation_originally_deposited,
    get_pool_total_available, get_pool_total_borrowed, get_pool_total_collateral,
    get_pool_total_supply,
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
    let TestMarketFixture {
        e, contract_client, gold_pool_address, users, gold_token_client, ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let creditor = &users[0];

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    // Withdraw 50%
    let creditor_balance_before = gold_token_client.balance(creditor);
    contract_client.withdraw(creditor, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));
    let creditor_balance_after = gold_token_client.balance(creditor);

    assert_eq!(
        creditor_balance_after.checked_sub(creditor_balance_before).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT / 2
    );

    let obligation_supplied =
        get_obligation_originally_deposited(&contract_client, creditor, &gold_pool_address)
            .unwrap();
    let obligation_j_tokens_as_tokens =
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address)
            .unwrap();

    assert_eq!(obligation_supplied, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(obligation_j_tokens_as_tokens, DEFAULT_DEPOSIT_AMOUNT / 2);

    let pool_total_supply = get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let pool_total_available = get_pool_total_available(&contract_client, &gold_pool_address);

    assert_eq!(pool_total_supply, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(pool_total_available, DEFAULT_DEPOSIT_AMOUNT / 2);

    // Withdraw 50% again
    contract_client.withdraw(creditor, &gold_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    assert_eq!(
        get_obligation_originally_deposited(&contract_client, creditor, &gold_pool_address),
        Err(MCError::ObligationDoesNotExist)
    );
    assert_eq!(
        get_obligation_j_tokens_as_tokens(&e, &contract_client, creditor, &gold_pool_address),
        Err(MCError::ObligationDoesNotExist)
    );

    let pool_total_supply = get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let pool_total_available = get_pool_total_available(&contract_client, &gold_pool_address);

    assert_eq!(pool_total_supply, 0);
    assert_eq!(pool_total_available, 0);
}

#[test]
fn test_remove_collateral() {
    let TestMarketFixture {
        e, contract_client, gold_pool_address, users, gold_token_client, ..
    } = TestMarketFixture::new();
    let collateral_provider = &users[0];

    contract_client.add_collateral(
        collateral_provider,
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
    );

    // Remove 50%
    let creditor_balance_before = gold_token_client.balance(collateral_provider);
    contract_client.remove_collateral(
        collateral_provider,
        &gold_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
    );
    let creditor_balance_after = gold_token_client.balance(collateral_provider);

    assert_eq!(
        creditor_balance_after.checked_sub(creditor_balance_before).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT / 2
    );

    let obligation_collateral =
        get_obligation_collateral(&contract_client, collateral_provider, &gold_pool_address)
            .unwrap();
    let obligation_j_tokens_as_tokens = get_obligation_j_tokens_as_tokens(
        &e,
        &contract_client,
        collateral_provider,
        &gold_pool_address,
    )
    .unwrap();

    assert_eq!(obligation_collateral, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(obligation_j_tokens_as_tokens, 0);

    let pool_total_supply = get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let pool_total_available = get_pool_total_available(&contract_client, &gold_pool_address);
    let pool_total_collateral = get_pool_total_collateral(&contract_client, &gold_pool_address);

    assert_eq!(pool_total_collateral, DEFAULT_DEPOSIT_AMOUNT / 2);
    assert_eq!(pool_total_supply, 0);
    assert_eq!(pool_total_available, 0);

    // Remove 50% again
    contract_client.remove_collateral(
        collateral_provider,
        &gold_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
    );

    assert_eq!(
        get_obligation_collateral(&contract_client, collateral_provider, &gold_pool_address),
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
    let pool_total_available = get_pool_total_available(&contract_client, &gold_pool_address);
    let pool_total_collateral = get_pool_total_collateral(&contract_client, &gold_pool_address);

    assert_eq!(pool_total_collateral, 0);
    assert_eq!(pool_total_supply, 0);
    assert_eq!(pool_total_available, 0);
}

#[test]
fn test_withdraw_zero() {
    let TestMarketFixture { contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let creditor = &users[0];

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let obligation_before =
        get_deposit_position(&contract_client, creditor, &gold_pool_address).unwrap();
    let pool_before = contract_client.get_pool(&gold_pool_address);

    contract_client.withdraw(creditor, &gold_pool_address, &0);

    let obligation_after =
        get_deposit_position(&contract_client, creditor, &gold_pool_address).unwrap();
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
        get_deposit_position(&contract_client, collateral_provider, &gold_pool_address).unwrap();
    let pool_before = contract_client.get_pool(&gold_pool_address);

    contract_client.remove_collateral(collateral_provider, &gold_pool_address, &0);

    let obligation_after =
        get_deposit_position(&contract_client, collateral_provider, &gold_pool_address).unwrap();
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
        Err(Ok(MCError::NegativeInputAmount))
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
        Err(Ok(MCError::NegativeInputAmount))
    );
}

#[test]
fn test_withdraw_all_with_i128_max() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        gold_token_client,
        usdc_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let creditor_1 = &users[0];
    let creditor_2 = &users[1];

    contract_client.deposit(creditor_1, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT); // NB: A more general case when the creditor has more than 1 deposits
    contract_client.deposit(creditor_1, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(creditor_2, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_total_supply_before =
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let creditor_balance_before = gold_token_client.balance(creditor_1);

    contract_client.withdraw(creditor_1, &gold_pool_address, &i128::MAX);

    let pool_total_supply_after =
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let creditor_balance_after = gold_token_client.balance(creditor_1);

    assert_eq!(
        creditor_balance_after.checked_sub(creditor_balance_before).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT
    );

    assert_eq!(pool_total_supply_after + DEFAULT_DEPOSIT_AMOUNT, pool_total_supply_before);
    assert_eq!(
        get_deposit_position(&contract_client, creditor_1, &gold_pool_address),
        Err(MCError::DepositPositionDoesNotExist)
    );
}

#[test]
fn test_remove_all_with_i128_max() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        gold_token_client,
        ..
    } = TestMarketFixture::new();
    let creditor_1 = &users[0];
    let creditor_2 = &users[1];

    contract_client.add_collateral(creditor_1, &usdc_pool_address, &DEFAULT_COLLATERAL_AMOUNT); // NB: A more general case when the collateral adder has more than 1 collaterals
    contract_client.add_collateral(creditor_1, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);
    contract_client.add_collateral(creditor_2, &gold_pool_address, &DEFAULT_COLLATERAL_AMOUNT);

    let pool_total_collateral_before =
        get_pool_total_collateral(&contract_client, &gold_pool_address);
    let creditor_balance_before = gold_token_client.balance(creditor_1);

    contract_client.remove_collateral(creditor_1, &gold_pool_address, &i128::MAX);

    let pool_total_collateral_after =
        get_pool_total_collateral(&contract_client, &gold_pool_address);
    let creditor_balance_after = gold_token_client.balance(creditor_1);

    assert_eq!(
        creditor_balance_after.checked_sub(creditor_balance_before).unwrap(),
        DEFAULT_COLLATERAL_AMOUNT
    );

    assert_eq!(
        pool_total_collateral_after + DEFAULT_COLLATERAL_AMOUNT,
        pool_total_collateral_before
    );
    assert_eq!(
        get_deposit_position(&contract_client, creditor_1, &gold_pool_address),
        Err(MCError::DepositPositionDoesNotExist)
    );
}

#[test]
fn test_withdraw_exceeds_utilization_cap() {
    let TestMarketFixture { contract_client, gold_pool_address, usdc_pool_address, users, .. } =
        TestMarketFixture::new();
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

    contract_client.withdraw(creditor, &usdc_pool_address, &(89 * DEFAULT_DEPOSIT_AMOUNT / 100));
    assert_eq!(
        contract_client.try_withdraw(
            creditor,
            &usdc_pool_address,
            &(88 * DEFAULT_DEPOSIT_AMOUNT / 100)
        ),
        Err(Ok(MCError::NotEnoughPoolFunds))
    );
}

#[test]
fn withdraw_up_to_open_ltv() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        gold_token_client,
        ..
    } = TestMarketFixture::new();
    let creditor = &users[0];
    let loan_provider = &users[1];

    let creditor_balance_1 = gold_token_client.balance(creditor);

    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.deposit(loan_provider, &gold_pool_address, &(10 * DEFAULT_DEPOSIT_AMOUNT));

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.borrow(creditor, &usdc_pool_address, &((DEFAULT_DEPOSIT_AMOUNT) / 2));
    // Try to withdraw more than default openLTV(70%) allows
    contract_client.withdraw(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let creditor_balance_2 = gold_token_client.balance(creditor);
    assert!(creditor_balance_1 > creditor_balance_2);

    // Repay all debt
    contract_client.repay(creditor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    // Withdraw all
    contract_client.withdraw(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    let creditor_balance_3 = gold_token_client.balance(creditor);

    assert_eq!(creditor_balance_1, creditor_balance_3);
    assert_eq!(
        contract_client.try_withdraw(creditor, &gold_pool_address, &1),
        Err(Ok(MCError::ObligationDoesNotExist))
    );
}

#[test]
fn remove_collateral_up_to_open_ltv() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        gold_token_client,
        ..
    } = TestMarketFixture::new();
    let collateral_adder = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.add_collateral(collateral_adder, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    contract_client.borrow(collateral_adder, &usdc_pool_address, &((DEFAULT_DEPOSIT_AMOUNT) / 2));

    let obligation_collateral_before =
        get_obligation_collateral(&contract_client, collateral_adder, &gold_pool_address).unwrap();
    let creditor_balance_before = gold_token_client.balance(collateral_adder);

    // Try to remove more than default openLTV(70%) allows
    contract_client.remove_collateral(
        collateral_adder,
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
    );
    let obligation_collateral_after =
        get_obligation_collateral(&contract_client, collateral_adder, &gold_pool_address).unwrap();
    let creditor_balance_after = gold_token_client.balance(collateral_adder);

    assert_approx_eq_rel(
        creditor_balance_after.checked_sub(creditor_balance_before).unwrap(),
        DEFAULT_DEPOSIT_AMOUNT
            .checked_sub(BPS_FACTOR * (DEFAULT_DEPOSIT_AMOUNT / 2) / DEFAULT_OPEN_LTV_BPS)
            .unwrap(),
        5,
    );

    assert_eq!(obligation_collateral_before, DEFAULT_DEPOSIT_AMOUNT);
    // Check that the required amount to back up the borrow remains
    assert_approx_eq_rel(
        // TODO: Investigate a bit deeper when checking maths
        obligation_collateral_after,
        (BPS_FACTOR * (obligation_collateral_before / 2)) / DEFAULT_OPEN_LTV_BPS,
        5,
    );
}

#[test]
fn test_withdraw_scarcity_over_limit() {
    const WITHDRAW_SCARCITY_LIMIT_BPS: i128 = 400; // 4%
    const WITHDRAW_SCARCITY_COOLDOWN_SECONDS: u64 = 10;

    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            withdraw_scarcity_limit_bps: WITHDRAW_SCARCITY_LIMIT_BPS,
            withdraw_scarcity_cooldown_s: WITHDRAW_SCARCITY_COOLDOWN_SECONDS,
            ..Default::default()
        },
        ..Default::default()
    };

    let TestMarketFixture {
        e, contract_client, gold_pool_address, users, usdc_pool_address, ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let creditor = &users[0];
    let borrower = &users[1];

    contract_client.deposit(creditor, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT);
    contract_client.add_collateral(borrower, &usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));

    // - Borrow up to utilization ratio cap -

    let borrow_amount: i128 = DEFAULT_DEPOSIT_AMOUNT
        .fixed_mul_ceil(DEFAULT_UTILIZATION_RATIO_LIMIT_BPS, BPS_FACTOR)
        .unwrap();

    contract_client.borrow(borrower, &gold_pool_address, &borrow_amount);

    // - Try to withdraw remaining liquidity -

    let allowed_withdrawal =
        DEFAULT_DEPOSIT_AMOUNT.fixed_mul_ceil(WITHDRAW_SCARCITY_LIMIT_BPS, BPS_FACTOR).unwrap();

    assert_eq!(
        contract_client.try_withdraw(creditor, &gold_pool_address, &(allowed_withdrawal + 1)),
        Err(Ok(MCError::WithdrawScarcityOverLimit))
    );

    contract_client.withdraw(creditor, &gold_pool_address, &allowed_withdrawal);

    assert_eq!(
        contract_client.try_withdraw(creditor, &gold_pool_address, &1),
        Err(Ok(MCError::ScarcityCooldownPeriod))
    );

    // - Move time -

    e.ledger().with_mut(|li| {
        li.timestamp += WITHDRAW_SCARCITY_COOLDOWN_SECONDS;
    });

    assert_eq!(
        contract_client.try_withdraw(creditor, &gold_pool_address, &allowed_withdrawal),
        Err(Ok(MCError::WithdrawScarcityOverLimit)) /* Must fail now, since UR has increased due to a previous withdrawal */
    );

    assert!(
        contract_client
            .try_withdraw(creditor, &gold_pool_address, &(allowed_withdrawal / 10))
            .is_ok()
    );
}
