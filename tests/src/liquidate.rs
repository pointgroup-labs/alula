#![cfg(test)]

use market::{
    constants::{BPS_FACTOR, DEFAULT_CLOSE_FACTOR_BPS},
    error::MCError,
};
use soroban_sdk::{
    Address,
    testutils::{Address as _, Ledger},
};

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, get_obligation_borrowed, get_obligation_collateral,
    get_obligation_d_tokens_as_tokens, get_obligation_j_tokens_as_tokens,
    get_obligation_unpaid_interest,
};

struct LiquidationTest {
    test_fixture: TestMarketFixture<'static>,
    borrower: Address,
    liquidator: Address,
}

impl LiquidationTest {
    /// Creates a standard setup with healthy position
    fn new() -> Self {
        let test_fixture = TestMarketFixture::new();
        let borrower = test_fixture.users[0].clone();
        let lender = test_fixture.users[1].clone();
        let liquidator = test_fixture.users[2].clone();

        test_fixture.contract_client.deposit(
            &lender,
            &test_fixture.usdc_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT * 2),
        );
        test_fixture.contract_client.add_collateral(
            &borrower,
            &test_fixture.gold_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
        );

        test_fixture.contract_client.borrow(
            &borrower,
            &test_fixture.usdc_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 3), // Conservative 33% borrow ratio
        );

        Self {
            test_fixture,
            borrower,
            liquidator,
        }
    }

    /// Creates a risky position closer to liquidation threshold
    fn risky() -> Self {
        let test_fixture = TestMarketFixture::new();
        let borrower = test_fixture.users[0].clone();
        let lender = test_fixture.users[1].clone();
        let liquidator = test_fixture.users[2].clone();

        test_fixture.contract_client.deposit(
            &lender,
            &test_fixture.usdc_pool_address,
            &((3 * DEFAULT_DEPOSIT_AMOUNT) / 2),
        );

        let collateral = DEFAULT_DEPOSIT_AMOUNT;
        let borrow_amount = (DEFAULT_DEPOSIT_AMOUNT * 65) / 100; // 65% borrow ratio(default open LTV is 70%)

        test_fixture.contract_client.add_collateral(
            &borrower,
            &test_fixture.gold_pool_address,
            &collateral,
        );
        test_fixture.contract_client.borrow(
            &borrower,
            &test_fixture.usdc_pool_address,
            &borrow_amount,
        );

        Self {
            test_fixture,
            borrower,
            liquidator,
        }
    }

    fn risky_with_deposit_as_collateral() -> Self {
        let test_fixture = TestMarketFixture::new();
        let borrower = test_fixture.users[0].clone();
        let lender = test_fixture.users[1].clone();
        let liquidator = test_fixture.users[2].clone();

        test_fixture.contract_client.deposit(
            &lender,
            &test_fixture.usdc_pool_address,
            &(3 * DEFAULT_DEPOSIT_AMOUNT / 2),
        );

        let collateral = DEFAULT_DEPOSIT_AMOUNT;
        let borrow_amount = (DEFAULT_DEPOSIT_AMOUNT * 65) / 100; // 65% borrow ratio(default open LTV is 70%)

        test_fixture.contract_client.deposit(
            &borrower,
            &test_fixture.gold_pool_address,
            &collateral,
        );
        test_fixture.contract_client.borrow(
            &borrower,
            &test_fixture.usdc_pool_address,
            &borrow_amount,
        );

        Self {
            test_fixture,
            borrower,
            liquidator,
        }
    }

    fn make_unhealthy(&self) {
        self.test_fixture
            .e
            .ledger()
            .with_mut(|li| li.timestamp += 3 * 365 * 24 * 60 * 60);
    }

    fn collateral_amount(&self) -> i128 {
        get_obligation_collateral(
            &self.test_fixture.contract_client,
            &self.borrower,
            &self.test_fixture.gold_pool_address,
        )
        .unwrap()
    }

    fn total_supplied(&self) -> i128 {
        get_obligation_j_tokens_as_tokens(
            &self.test_fixture.e,
            &self.test_fixture.contract_client,
            &self.borrower,
            &self.test_fixture.gold_pool_address,
        )
        .unwrap()
    }

    fn initially_borrowed(&self) -> i128 {
        get_obligation_borrowed(
            &self.test_fixture.contract_client,
            &self.borrower,
            &self.test_fixture.usdc_pool_address,
        )
        .unwrap()
    }

    fn debt(&self) -> i128 {
        get_obligation_d_tokens_as_tokens(
            &self.test_fixture.e,
            &self.test_fixture.contract_client,
            &self.borrower,
            &self.test_fixture.usdc_pool_address,
        )
        .unwrap()
    }

    fn unpaid_interest(&self) -> i128 {
        get_obligation_unpaid_interest(
            &self.test_fixture.e,
            &self.test_fixture.contract_client,
            &self.borrower,
            &self.test_fixture.usdc_pool_address,
        )
        .unwrap()
    }

    fn liquidation_amount_from_percentage(&self, percentage: i128) -> i128 {
        (self.debt() * percentage) / 100
    }

    fn max_liquidation_amount(&self) -> i128 {
        (self.debt() * DEFAULT_CLOSE_FACTOR_BPS) / BPS_FACTOR
    }
}

// -- Basic Liquidation Tests --

#[test]
fn test_liquidate_healthy_position_fails() {
    let test = LiquidationTest::new();

    let result = test.test_fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &1,
    );

    assert_eq!(result, Err(Ok(MCError::LiquidatedPositionIsHealthy)));
}

#[test]
fn test_liquidate_zero() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let usdc_pool_before = test
        .test_fixture
        .contract_client
        .get_pool(&test.test_fixture.usdc_pool_address);

    let gold_pool_before = test
        .test_fixture
        .contract_client
        .get_pool(&test.test_fixture.gold_pool_address);

    let debt_before = test.debt();

    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &0,
    );

    let usdc_pool_after = test
        .test_fixture
        .contract_client
        .get_pool(&test.test_fixture.usdc_pool_address);

    let gold_pool_after = test
        .test_fixture
        .contract_client
        .get_pool(&test.test_fixture.gold_pool_address);

    let debt_after = test.debt();

    assert_eq!(debt_before, debt_after);
    assert_eq!(usdc_pool_before, usdc_pool_after);
    assert_eq!(gold_pool_before, gold_pool_after);
}

#[test]
fn test_liquidate_negative_amount() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let result = test.test_fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &-1,
    );

    assert_eq!(result, Err(Ok(MCError::NegativeAmount)));
}

#[test]
fn test_liquidate_self_fails() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let result = test.test_fixture.contract_client.try_liquidate(
        &test.borrower, // Same as borrower
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &test.liquidation_amount_from_percentage(10),
    );

    assert_eq!(result, Err(Ok(MCError::SelfLiquidation)));
}

#[test]
fn test_liquidate_nonexistent_user_fails() {
    let test = LiquidationTest::new();
    let fake_user = Address::generate(&test.test_fixture.e);

    let result = test.test_fixture.contract_client.try_liquidate(
        &test.liquidator,
        &fake_user,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &1,
    );

    assert_eq!(result, Err(Ok(MCError::ObligationDoesNotExist)));
}

// -- Close Factor Tests --

#[test]
fn test_liquidate_exceeds_close_factor_fails() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        gold_pool_address,
        ..
    } = TestMarketFixture::new();

    let borrower = &users[0];
    let loan_provider = &users[1];
    let liquidator = &users[2];

    contract_client.deposit(
        loan_provider,
        &usdc_pool_address,
        &((3 * DEFAULT_DEPOSIT_AMOUNT) / 2),
    );

    let minimal_collateral = DEFAULT_DEPOSIT_AMOUNT;
    contract_client.add_collateral(borrower, &gold_pool_address, &minimal_collateral);

    let max_borrow = i128::MAX;
    contract_client.borrow(borrower, &usdc_pool_address, &max_borrow);

    // - Accrue interest -

    e.ledger()
        .with_mut(|li| li.timestamp += 2 * 360 * 24 * 60 * 60); // 2 years

    let total_debt =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    let max_liquidation = (total_debt * DEFAULT_CLOSE_FACTOR_BPS) / BPS_FACTOR;
    let over_limit = max_liquidation + 10;

    assert_eq!(
        contract_client.try_liquidate(
            liquidator,
            borrower,
            &usdc_pool_address,
            &gold_pool_address,
            &over_limit,
        ),
        Err(Ok(MCError::LiquidationExceedsCloseFactor))
    );
}

#[test]
fn test_liquidate_at_exact_close_factor() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let debt_before = test.debt();
    let liquidation_amount = test.max_liquidation_amount();

    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &liquidation_amount,
    );

    let debt_after = test.debt();
    assert_eq!(debt_after, debt_before - liquidation_amount);
}

#[test]
fn test_liquidate_just_under_close_factor() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let debt_before = test.debt();
    let under_limit = test.max_liquidation_amount() - 1;

    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &under_limit,
    );

    let debt_after = test.debt();

    assert_eq!(debt_after, debt_before - under_limit);
}

// -- Successful Liquidation Tests --

#[test]
fn test_liquidate_collateral_successful() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let debt_before = test.debt();
    let collateral_before = test.collateral_amount();
    let liquidation_amount = test.liquidation_amount_from_percentage(10);

    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &liquidation_amount,
    );

    let debt_after = test.debt();
    let collateral_after = test.collateral_amount();

    // Debt should be reduced
    assert_eq!(debt_after, debt_before - liquidation_amount);

    // Collateral should be seized
    assert!(
        collateral_after < collateral_before,
        "Collateral should be reduced from {} to {}",
        collateral_before,
        collateral_after
    );
}

#[test]
fn test_liquidate_deposit_successful() {
    let test = LiquidationTest::risky_with_deposit_as_collateral();
    test.make_unhealthy();

    let debt_before = test.debt();
    let deposit_before = test.total_supplied();
    let liquidation_amount = test.liquidation_amount_from_percentage(10);

    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &liquidation_amount,
    );

    let debt_after = test.debt();
    let deposit_after = test.total_supplied();

    // Debt should be reduced
    assert_eq!(debt_after, debt_before - liquidation_amount);

    // Collateral should be seized
    assert!(
        deposit_after < deposit_before,
        "Deposit should be reduced from {} to {}",
        deposit_before,
        deposit_after
    );
}

#[test]
fn test_liquidate_health_factor_reduction() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let liquidation_amount = test.max_liquidation_amount();

    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &liquidation_amount,
    );

    assert_eq!(
        test.test_fixture.contract_client.try_liquidate(
            &test.liquidator,
            &test.borrower,
            &test.test_fixture.usdc_pool_address,
            &test.test_fixture.gold_pool_address,
            &10,
        ),
        Err(Ok(MCError::LiquidatedPositionIsHealthy))
    );
}

// -- Edge Cases --

#[test]
fn test_liquidate_same_pool_fails() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let result = test.test_fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.usdc_pool_address,
        &test.liquidation_amount_from_percentage(10),
    );

    assert_eq!(
        result,
        Err(Ok(MCError::LiquidationWithEqualCollateralAndDepositPools))
    );
}

#[test]
fn test_liquidate_multiple_small() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let initial_debt = test.debt();
    let small_amount = test.liquidation_amount_from_percentage(5);

    // Multiple small liquidations
    for i in 1..=3 {
        let result = test.test_fixture.contract_client.try_liquidate(
            &test.liquidator,
            &test.borrower,
            &test.test_fixture.usdc_pool_address,
            &test.test_fixture.gold_pool_address,
            &small_amount,
        );

        match result {
            Ok(_) => {
                let current_debt = test.debt();
                let expected = initial_debt - (small_amount * i);
                assert_eq!(current_debt, expected, "Liquidation {} failed", i);
            }
            Err(Ok(MCError::LiquidatedPositionIsHealthy)) => {
                // Position became healthy, this is expected
                break;
            }
            Err(Ok(error)) => {
                panic!("Unexpected error in liquidation {}: {:?}", i, error);
            }
            Err(Err(host_error)) => {
                panic!("Host error: {:?}", host_error);
            }
        }
    }
}

#[test]
fn test_liquidate_with_interest_accrual() {
    let test = LiquidationTest::new();

    // Start with healthy position, accrue interest to make it risky
    test.test_fixture
        .e
        .ledger()
        .with_mut(|li| li.timestamp += 40 * 365 * 24 * 60 * 60); // 40 years

    let debt = test.debt();
    let liquidation_amount = test.liquidation_amount_from_percentage(20);

    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &liquidation_amount,
    );

    let new_debt = test.debt();

    assert!(new_debt < debt, "Debt should be reduced");
}

#[test]
fn test_liquidate_unpaid_interest_only() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let initially_borrowed_before = test.initially_borrowed();
    let unpaid_interest = test.unpaid_interest();

    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &unpaid_interest,
    );

    let initially_borrowed_after = test.initially_borrowed();
    let new_unpaid_interest = test.unpaid_interest();
    let new_initially_borrowed = test.initially_borrowed();
    let new_debt = test.debt();

    assert_eq!(new_unpaid_interest, 0);
    assert_eq!(new_initially_borrowed, new_debt);
    assert_eq!(initially_borrowed_before, initially_borrowed_after);
}
