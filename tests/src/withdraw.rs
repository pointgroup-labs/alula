#![cfg(test)]

use market::{
    constants::*,
    obligation::ObligationKey,
    pool::{PoolConfig, PoolHealthConfig},
};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::testutils::Ledger;

use crate::{
    DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, MCError, TestMarketFixture,
    assert_approx_eq_rel, get_deposit_position, get_obligation_collateral, get_obligation_d_tokens,
    get_obligation_j_tokens_as_tokens, get_obligation_originally_deposited,
    get_pool_operation_fees_sum, get_pool_total_available, get_pool_total_available_adjusted,
    get_pool_total_borrowed, get_pool_total_collateral, get_pool_total_supply,
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

    contract_client.deposit(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    // Withdraw 50%
    let creditor_balance_before = gold_token_client.balance(creditor);
    contract_client.withdraw(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
    );
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
    contract_client.withdraw(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
    );

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
        &ObligationKey::new(collateral_provider.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    // Remove 50%
    let creditor_balance_before = gold_token_client.balance(collateral_provider);
    contract_client.remove_collateral(
        &ObligationKey::new(collateral_provider.clone()),
        &gold_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
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
        &ObligationKey::new(collateral_provider.clone()),
        &gold_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
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

    contract_client.deposit(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(creditor.clone()),
            &gold_pool_address,
            &0,
            &None
        ),
        Err(Ok(MCError::InvalidInputAmount))
    );
}

#[test]
fn test_remove_collateral_zero() {
    let TestMarketFixture { contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let collateral_provider = &users[0];

    contract_client.add_collateral(
        &ObligationKey::new(collateral_provider.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    assert_eq!(
        contract_client.try_remove_collateral(
            &ObligationKey::new(collateral_provider.clone()),
            &gold_pool_address,
            &0,
            &None
        ),
        Err(Ok(MCError::InvalidInputAmount))
    );
}

#[test]
fn test_withdraw_negative() {
    let TestMarketFixture { contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let creditor = &users[0];

    contract_client.deposit(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    );

    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(creditor.clone()),
            &gold_pool_address,
            &-1,
            &None
        ),
        Err(Ok(MCError::InvalidInputAmount))
    );
}

#[test]
fn test_remove_collateral_negative() {
    let TestMarketFixture { contract_client, gold_pool_address, users, .. } =
        TestMarketFixture::new();
    let collateral_provider = &users[0];

    contract_client.add_collateral(
        &ObligationKey::new(collateral_provider.clone()),
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    );

    assert_eq!(
        contract_client.try_remove_collateral(
            &ObligationKey::new(collateral_provider.clone()),
            &gold_pool_address,
            &-1,
            &None
        ),
        Err(Ok(MCError::InvalidInputAmount))
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

    contract_client.deposit(
        &ObligationKey::new(creditor_1.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    ); // NB: A more general case when the creditor has more than 1 deposits
    contract_client.deposit(
        &ObligationKey::new(creditor_1.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(creditor_2.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    let pool_total_supply_before =
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    let creditor_balance_before = gold_token_client.balance(creditor_1);

    contract_client.withdraw(
        &ObligationKey::new(creditor_1.clone()),
        &gold_pool_address,
        &i128::MAX,
        &None,
    );

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

    contract_client.add_collateral(
        &ObligationKey::new(creditor_1.clone()),
        &usdc_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    ); // NB: A more general case when the collateral adder has more than 1 collaterals
    contract_client.add_collateral(
        &ObligationKey::new(creditor_1.clone()),
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    );
    contract_client.add_collateral(
        &ObligationKey::new(creditor_2.clone()),
        &gold_pool_address,
        &DEFAULT_COLLATERAL_AMOUNT,
        &None,
    );

    let pool_total_collateral_before =
        get_pool_total_collateral(&contract_client, &gold_pool_address);
    let creditor_balance_before = gold_token_client.balance(creditor_1);

    contract_client.remove_collateral(
        &ObligationKey::new(creditor_1.clone()),
        &gold_pool_address,
        &i128::MAX,
        &None,
    );

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

    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(creditor.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    // Borrow 10% of USDC
    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 10),
        &None,
    );

    let pool_borrowed = get_pool_total_borrowed(&contract_client, &usdc_pool_address);
    let pool_total_supply = get_pool_total_supply(&contract_client, &usdc_pool_address).unwrap();

    // Check that the utilization ratio is 10%
    assert_eq!(pool_total_supply / pool_borrowed, 10);

    contract_client.withdraw(
        &ObligationKey::new(creditor.clone()),
        &usdc_pool_address,
        &(89 * DEFAULT_DEPOSIT_AMOUNT / 100),
        &None,
    );
    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(creditor.clone()),
            &usdc_pool_address,
            &(88 * DEFAULT_DEPOSIT_AMOUNT / 100),
            &None
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
    let liquidity_provider = &users[1];

    let creditor_balance_1 = gold_token_client.balance(creditor);

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &gold_pool_address,
        &(10 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    contract_client.deposit(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(creditor.clone()),
        &usdc_pool_address,
        &((DEFAULT_DEPOSIT_AMOUNT) / 2),
        &None,
    );

    // Withdraw exceeding the healthy limit should fail
    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(creditor.clone()),
            &gold_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &None
        ),
        Err(Ok(MCError::UnhealthyOperation))
    );

    // Withdraw max healthy amount (capped via i128::MAX)
    contract_client.withdraw(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &i128::MAX,
        &None,
    );

    let creditor_balance_2 = gold_token_client.balance(creditor);
    assert!(creditor_balance_1 > creditor_balance_2);

    // Repay all debt
    contract_client.repay(
        &ObligationKey::new(creditor.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    // Withdraw all remaining
    contract_client.withdraw(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &i128::MAX,
        &None,
    );
    let creditor_balance_3 = gold_token_client.balance(creditor);

    assert_eq!(creditor_balance_1, creditor_balance_3);
    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(creditor.clone()),
            &gold_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::ObligationDoesNotExist))
    );
}

#[test]
fn test_withdraw_capping_behavior() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        gold_token_client,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(user.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    // -- No borrow: amount > all_deposit silently caps --

    let balance_before = gold_token_client.balance(user);
    contract_client.withdraw(
        &ObligationKey::new(user.clone()),
        &gold_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    let received = gold_token_client.balance(user).checked_sub(balance_before).unwrap();

    assert_eq!(received, DEFAULT_DEPOSIT_AMOUNT);
    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(user.clone()),
            &gold_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::ObligationDoesNotExist))
    );

    // -- With borrow: amount ≤ all_deposit but > max_healthy errors --

    contract_client.deposit(
        &ObligationKey::new(user.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(user.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 4),
        &None,
    );

    let healthy_withdraw = DEFAULT_DEPOSIT_AMOUNT / 4;
    contract_client.withdraw(
        &ObligationKey::new(user.clone()),
        &gold_pool_address,
        &healthy_withdraw,
        &None,
    );

    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(user.clone()),
            &gold_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &None
        ),
        Err(Ok(MCError::UnhealthyOperation))
    );

    // -- With borrow: amount > all_deposit still checks health --

    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(user.clone()),
            &gold_pool_address,
            &(10 * DEFAULT_DEPOSIT_AMOUNT),
            &None
        ),
        Err(Ok(MCError::UnhealthyOperation))
    );

    // -- With borrow: i128::MAX caps to max_healthy --

    let balance_before = gold_token_client.balance(user);
    contract_client.withdraw(
        &ObligationKey::new(user.clone()),
        &gold_pool_address,
        &i128::MAX,
        &None,
    );
    let received = gold_token_client.balance(user).checked_sub(balance_before).unwrap();

    assert!(received > 0);
    assert!(received < DEFAULT_DEPOSIT_AMOUNT);
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
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.add_collateral(
        &ObligationKey::new(collateral_adder.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );

    contract_client.borrow(
        &ObligationKey::new(collateral_adder.clone()),
        &usdc_pool_address,
        &((DEFAULT_DEPOSIT_AMOUNT) / 2),
        &None,
    );

    let obligation_collateral_before =
        get_obligation_collateral(&contract_client, collateral_adder, &gold_pool_address).unwrap();
    let creditor_balance_before = gold_token_client.balance(collateral_adder);

    // Try to remove more than default openLTV(70%) allows
    contract_client.remove_collateral(
        &ObligationKey::new(collateral_adder.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
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

    contract_client.deposit(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    // - Borrow up to utilization ratio cap -

    let borrow_amount: i128 = DEFAULT_DEPOSIT_AMOUNT
        .fixed_mul_ceil(DEFAULT_UTILIZATION_RATIO_LIMIT_BPS, BPS_FACTOR)
        .unwrap();

    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &borrow_amount,
        &None,
    );

    // - Try to withdraw remaining liquidity -

    let remaining_liquidity =
        get_pool_total_available_adjusted(&contract_client, &gold_pool_address).unwrap();
    let allowed_withdrawal =
        remaining_liquidity.fixed_mul_ceil(WITHDRAW_SCARCITY_LIMIT_BPS, BPS_FACTOR).unwrap();

    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(creditor.clone()),
            &gold_pool_address,
            &(allowed_withdrawal + 1),
            &None
        ),
        Err(Ok(MCError::WithdrawScarcityOverLimit))
    );

    contract_client.withdraw(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &allowed_withdrawal,
        &None,
    );

    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(creditor.clone()),
            &gold_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::ScarcityCooldownPeriod))
    );

    // - Move time -

    e.ledger().with_mut(|li| {
        li.timestamp += WITHDRAW_SCARCITY_COOLDOWN_SECONDS;
    });

    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(creditor.clone()),
            &gold_pool_address,
            &allowed_withdrawal,
            &None
        ),
        Err(Ok(MCError::WithdrawScarcityOverLimit)) /* Must fail now, since UR has increased due to a previous withdrawal */
    );

    assert!(
        contract_client
            .try_withdraw(
                &ObligationKey::new(creditor.clone()),
                &gold_pool_address,
                &(allowed_withdrawal / 10),
                &None
            )
            .is_ok()
    );
}

#[test]
fn test_withdraw_crosses_scarcity_limit() {
    const WITHDRAW_SCARCITY_LIMIT_BPS: i128 = 500; // 5%
    const WITHDRAW_SCARCITY_COOLDOWN_SECONDS: u64 = 10;

    let mut pool_config = PoolConfig::default();
    pool_config.health_config.withdraw_scarcity_limit_bps = WITHDRAW_SCARCITY_LIMIT_BPS;
    pool_config.health_config.withdraw_scarcity_cooldown_s = WITHDRAW_SCARCITY_COOLDOWN_SECONDS;

    let TestMarketFixture { contract_client, gold_pool_address, users, usdc_pool_address, .. } =
        TestMarketFixture::new_with_pool_config(pool_config);
    let creditor = &users[0];
    let borrower = &users[1];

    contract_client.deposit(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    // - Borrow up to 50% utilization ratio (Starts healthy) -
    let borrow_amount = DEFAULT_DEPOSIT_AMOUNT / 2;

    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &borrow_amount,
        &None,
    );

    // - Compute the max withdrawal that smoothly crosses into the scarcity zone -
    // Safe part (amount needed to reach exactly the limit)
    let safe_supply =
        borrow_amount.fixed_div_ceil(DEFAULT_UTILIZATION_RATIO_LIMIT_BPS, BPS_FACTOR).unwrap();
    let safe_withdrawal = DEFAULT_DEPOSIT_AMOUNT - safe_supply;

    // Scarcity allowance part (5% of the remaining unborrowed liquidity at threshold)
    let supply_after_safe = DEFAULT_DEPOSIT_AMOUNT - safe_withdrawal;
    let available_liquidity_at_threshold = supply_after_safe - borrow_amount;
    let scarcity_allowance = available_liquidity_at_threshold
        .fixed_mul_floor(WITHDRAW_SCARCITY_LIMIT_BPS, BPS_FACTOR)
        .unwrap();

    let max_allowed_withdrawal = safe_withdrawal + scarcity_allowance;

    // - Try to withdraw more than the combined limit -
    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(creditor.clone()),
            &gold_pool_address,
            &(max_allowed_withdrawal + 1),
            &None
        ),
        Err(Ok(MCError::WithdrawScarcityOverLimit))
    );

    // - Withdraw exactly the allowed combined limit -
    assert!(
        contract_client
            .try_withdraw(
                &ObligationKey::new(creditor.clone()),
                &gold_pool_address,
                &max_allowed_withdrawal,
                &None,
            )
            .is_ok()
    );

    // - Ensure cooldown is applied because the scarcity allowance was dipped into -
    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(creditor.clone()),
            &gold_pool_address,
            &1,
            &None
        ),
        Err(Ok(MCError::ScarcityCooldownPeriod))
    );
}

/// Reproduces an observed testnet pool state and verifies that the exact
/// amount reported by the `withdraw_scaricty_limit.py` simulator is actually
/// withdrawable on-chain (and that a single unit more is rejected).
///
/// Testnet state being reproduced (from the simulator's DEBUG dump):
///   supply    = 94_440_526_454
///   borrowed  = 83_534_950_390
///   util_limit = 8500 bps (85%), scarcity_limit = 5000 bps (50%)
///   current utilization = 8846 bps (88.46%) -> already ABOVE the limit
///   simulator's max allowed withdrawal = 5_452_788_032
#[test]
fn test_withdraw_scarcity_matches_simulator() {
    const UTILIZATION_RATIO_LIMIT_BPS: i128 = 8500; // 85%
    const WITHDRAW_SCARCITY_LIMIT_BPS: i128 = 5000; // 50%

    // Target (post-setup) pool state, taken verbatim from the testnet dump.
    const TARGET_SUPPLY: i128 = 94_440_526_454;
    const TARGET_BORROWED: i128 = 83_534_950_390;
    // The value the simulator says is the maximum allowed withdrawal.
    const SIMULATOR_MAX_WITHDRAWAL: i128 = 5_452_788_032;

    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: UTILIZATION_RATIO_LIMIT_BPS,
            withdraw_scarcity_limit_bps: WITHDRAW_SCARCITY_LIMIT_BPS,
            withdraw_scarcity_cooldown_s: 0,
            ..Default::default()
        },
        ..Default::default()
    };

    let TestMarketFixture { contract_client, gold_pool_address, users, usdc_pool_address, .. } =
        TestMarketFixture::new_with_pool_config(pool_config);
    let creditor = &users[0];
    let borrower = &users[1];

    // Initial supply so that borrowing `TARGET_BORROWED` yields a healthy 80%
    // utilization (below the 85% cap, so the borrow is accepted).
    let initial_deposit = TARGET_BORROWED.fixed_div_ceil(8000, BPS_FACTOR).unwrap();

    contract_client.deposit(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &initial_deposit,
        &None,
    );

    // Over-collateralize generously so the borrow is never LTV-constrained.
    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &(2 * initial_deposit),
        &None,
    );

    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &TARGET_BORROWED,
        &None,
    );

    // - Shrink supply down to the target via an intermediate scarcity withdrawal -
    // This drives utilization from 80% up to the target 88.46% (> limit).
    let intermediate_withdrawal = initial_deposit - TARGET_SUPPLY;
    contract_client.withdraw(
        &ObligationKey::new(creditor.clone()),
        &gold_pool_address,
        &intermediate_withdrawal,
        &None,
    );

    // - Verify we reproduced the exact testnet state -
    assert_eq!(
        get_pool_total_supply(&contract_client, &gold_pool_address).unwrap(),
        TARGET_SUPPLY,
        "reproduced total_supply must match testnet"
    );
    assert_eq!(
        get_pool_total_borrowed(&contract_client, &gold_pool_address),
        TARGET_BORROWED,
        "reproduced total_borrowed must match testnet"
    );

    // - Recompute the allowance exactly as the contract/simulator does -
    // Utilization is already above the limit, so the "safe" (up-to-limit)
    // portion is zero and the whole allowance is the scarcity drip.
    let available_liquidity = TARGET_SUPPLY - TARGET_BORROWED;
    let max_allowed_withdrawal =
        available_liquidity.fixed_mul_floor(WITHDRAW_SCARCITY_LIMIT_BPS, BPS_FACTOR).unwrap();

    // The on-chain math, the local computation, and the simulator all agree.
    assert_eq!(
        max_allowed_withdrawal, SIMULATOR_MAX_WITHDRAWAL,
        "recomputed allowance must equal the simulator's reported value"
    );

    // - One unit beyond the simulator's value must be rejected -
    assert_eq!(
        contract_client.try_withdraw(
            &ObligationKey::new(creditor.clone()),
            &gold_pool_address,
            &(SIMULATOR_MAX_WITHDRAWAL + 1),
            &None
        ),
        Err(Ok(MCError::WithdrawScarcityOverLimit))
    );

    // - Exactly the simulator's value is actually withdrawable -
    assert!(
        contract_client
            .try_withdraw(
                &ObligationKey::new(creditor.clone()),
                &gold_pool_address,
                &SIMULATOR_MAX_WITHDRAWAL,
                &None,
            )
            .is_ok()
    );
}

#[test]
fn test_simulate_withdraw_accrues_interest() {
    let TestMarketFixture {
        e,
        contract_client,
        gold_pool_address,
        usdc_pool_address,
        users,
        gold_token_client,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(user.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(user.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 4),
        &None,
    );

    e.ledger().with_mut(|li| li.timestamp += SECONDS_IN_YEAR);

    let creditor_balance_before = gold_token_client.balance(user);
    let fees_before = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);

    let simulated = contract_client.simulate_withdraw(
        &ObligationKey::new(user.clone()),
        &gold_pool_address,
        &i128::MAX,
        &None,
    );
    contract_client.withdraw(
        &ObligationKey::new(user.clone()),
        &gold_pool_address,
        &i128::MAX,
        &None,
    );

    let creditor_balance_after = gold_token_client.balance(user);
    let creditor_balance_diff =
        creditor_balance_after.checked_sub(creditor_balance_before).unwrap();
    let fees_after = get_pool_operation_fees_sum(&contract_client, &gold_pool_address);
    let fees_diff = fees_after.checked_sub(fees_before).unwrap();

    assert_eq!(creditor_balance_diff, simulated.withdrawer_to_receive);
    assert_eq!(fees_diff, simulated.operation_fees.fee_sum);
}

#[test]
fn test_withdraw_all_when_borrow_exists_but_max_healthy_exceeds_deposit() {
    let TestMarketFixture {
        contract_client,
        gold_pool_address,
        btc_pool_address,
        usdc_pool_address,
        users,
        gold_token_client,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];
    let liquidity_provider = &users[1];
    let user_key = &ObligationKey::new(user.clone());
    let lp_key = &ObligationKey::new(liquidity_provider.clone());

    contract_client.deposit(lp_key, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    // Deposit GOLD and add BTC collateral so the combined collateral value greatly exceeds
    // the borrow. max_healthy for the GOLD pool alone will exceed the user's GOLD deposit
    // because the surplus is computed globally across all deposit pools
    contract_client.deposit(user_key, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.add_collateral(user_key, &btc_pool_address, &DEFAULT_COLLATERAL_AMOUNT, &None);

    contract_client.borrow(user_key, &usdc_pool_address, &1, &None);

    let balance_before = gold_token_client.balance(user);

    // i128::MAX should withdraw the entire GOLD deposit, not an uncapped max_healthy value
    contract_client.withdraw(user_key, &gold_pool_address, &i128::MAX, &None);

    let received = gold_token_client.balance(user).checked_sub(balance_before).unwrap();
    assert_eq!(received, DEFAULT_DEPOSIT_AMOUNT);

    assert_eq!(
        get_deposit_position(&contract_client, user, &gold_pool_address),
        Err(MCError::DepositPositionDoesNotExist)
    );

    let pool_total_supply = get_pool_total_supply(&contract_client, &gold_pool_address).unwrap();
    assert_eq!(pool_total_supply, 0);

    // The borrow still exists
    assert!(get_obligation_d_tokens(&contract_client, user, &usdc_pool_address).unwrap() > 0);

    // Explicit amount == all_deposit should also succeed (not UnhealthyOperation)
    contract_client.deposit(user_key, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    contract_client.withdraw(user_key, &gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);

    assert_eq!(
        get_deposit_position(&contract_client, user, &gold_pool_address),
        Err(MCError::DepositPositionDoesNotExist)
    );
}
