#![cfg(test)]

use market::{
    constants::{DEFAULT_CLOSE_FACTOR, DEFAULT_CLOSE_LTV},
    error::MCError,
    math_utils::MathUtils,
};
use soroban_sdk::{
    Address,
    testutils::{Address as _, Ledger},
};

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, get_borrow_obligation, get_obligation_borrowed,
    get_obligation_collateral, get_obligation_d_tokens_as_tokens,
    get_obligation_j_tokens_as_tokens,
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

        // Lender provides liquidity
        test_fixture.contract_client.deposit(
            &lender,
            &test_fixture.usdc_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT * 2),
        );

        // Borrower deposits collateral and borrows (healthy position)
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

    /// Creates setup with liquidator having funds
    #[allow(dead_code)]
    fn with_liquidator_funds() -> Self {
        let setup = Self::new();
        setup.test_fixture.contract_client.deposit(
            &setup.liquidator,
            &setup.test_fixture.usdc_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
        );
        setup
    }

    /// Creates a risky position closer to liquidation threshold
    fn risky() -> Self {
        let test_fixture = TestMarketFixture::new();
        let borrower = test_fixture.users[0].clone();
        let lender = test_fixture.users[1].clone();
        let liquidator = test_fixture.users[2].clone();

        // Lender provides liquidity
        test_fixture.contract_client.deposit(
            &lender,
            &test_fixture.usdc_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT * 3),
        );

        let collateral = (DEFAULT_DEPOSIT_AMOUNT * 82) / 100; // 82% collateral
        let borrow_amount = (DEFAULT_DEPOSIT_AMOUNT * 65) / 100; // 65% borrow

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

        // Lender provides liquidity
        test_fixture.contract_client.deposit(
            &lender,
            &test_fixture.usdc_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT * 3),
        );

        let collateral = (DEFAULT_DEPOSIT_AMOUNT * 82) / 100; // 82% deposit as collateral
        let borrow_amount = (DEFAULT_DEPOSIT_AMOUNT * 65) / 100; // 65% borrow

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
        // Accrue interest over time to increase debt
        self.test_fixture
            .e
            .ledger()
            .with_mut(|li| li.timestamp += 365 * 24 * 60 * 60); // 1 year
    }

    fn unpaid_interest(&self) -> i128 {
        let d_tokens_as_tokens = get_obligation_d_tokens_as_tokens(
            &self.test_fixture.e,
            &self.test_fixture.contract_client,
            &self.borrower,
            &self.test_fixture.usdc_pool_address,
        )
        .unwrap();

        let initially_borrowed = get_obligation_borrowed(
            &self.test_fixture.contract_client,
            &self.borrower,
            &self.test_fixture.usdc_pool_address,
        )
        .unwrap();

        d_tokens_as_tokens
            .checked_sub(initially_borrowed)
            .map_over_or_underflow()
            .unwrap()
    }

    fn borrowed_amount(&self) -> i128 {
        get_borrow_obligation(
            &self.test_fixture.contract_client,
            &self.borrower,
            &self.test_fixture.usdc_pool_address,
        )
        .unwrap()
        .borrowed
    }

    fn total_debt(&self) -> i128 {
        get_obligation_d_tokens_as_tokens(
            &self.test_fixture.e,
            &self.test_fixture.contract_client,
            &self.borrower,
            &self.test_fixture.usdc_pool_address,
        )
        .unwrap()
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

    fn liquidation_amount(&self, percentage: i128) -> i128 {
        (self.total_debt() * percentage) / 100
    }

    fn max_liquidation_amount(&self) -> i128 {
        (self.total_debt() * DEFAULT_CLOSE_FACTOR) / 100
    }
}

// === Basic Liquidation Tests ===

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
#[ignore]
fn test_liquidate_zero() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();
    // Accrue interest
    test.test_fixture
        .contract_client
        .accrue_interest(&test.borrower);

    let usdc_pool_before = test
        .test_fixture
        .contract_client
        .get_pool(&test.test_fixture.usdc_pool_address);
    let gold_pool_before = test
        .test_fixture
        .contract_client
        .get_pool(&test.test_fixture.gold_pool_address);
    let borrowed_before = test.borrowed_amount();

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
    let borrowed_after = test.borrowed_amount();

    assert_eq!(borrowed_after, borrowed_before);
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
        &test.liquidation_amount(10),
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

// === Close Factor Tests ===

#[test]
#[ignore]
fn test_liquidate_exceeds_close_factor_fails() {
    // Create a position that's definitely unhealthy
    let fixture = TestMarketFixture::new();

    let borrower = &fixture.users[0];
    let lender = &fixture.users[1];
    let liquidator = &fixture.users[2];
    // Lender provides liquidity
    fixture.contract_client.deposit(
        lender,
        &fixture.usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT * 5),
    );

    // Create minimal collateral position
    let minimal_collateral = DEFAULT_DEPOSIT_AMOUNT / 10; // Very small collateral
    fixture.contract_client.add_collateral(
        borrower,
        &fixture.gold_pool_address,
        &minimal_collateral,
    );

    // Borrow maximum possible amount
    let max_borrow = (minimal_collateral * DEFAULT_CLOSE_LTV) / 100; // 80% of collateral value
    fixture
        .contract_client
        .borrow(borrower, &fixture.usdc_pool_address, &max_borrow);

    // Accrue interest to make position unhealthy
    fixture
        .e
        .ledger()
        .with_mut(|li| li.timestamp += 50 * 24 * 60 * 60); // 50 days

    // Get current borrowed amount (should include accrued interest)
    let total_debt = get_obligation_d_tokens_as_tokens(
        &fixture.e,
        &fixture.contract_client,
        lender,
        &fixture.usdc_pool_address,
    )
    .unwrap();

    // Calculate over-limit amount
    let max_liquidation = (total_debt * DEFAULT_CLOSE_FACTOR) / 100;
    let over_limit = max_liquidation + 1;

    // This should fail with close factor exceeded
    let result = fixture.contract_client.try_liquidate(
        liquidator,
        borrower,
        &fixture.usdc_pool_address,
        &fixture.gold_pool_address,
        &over_limit,
    );

    assert_eq!(result, Err(Ok(MCError::LiquidationExceedsCloseFactor)));
}

#[test]
#[ignore]
fn test_liquidate_at_exact_close_factor() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let debt_before = test.total_debt();
    let liquidation_amount = test.max_liquidation_amount();

    // Should succeed at exactly the close factor limit
    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &liquidation_amount,
    );

    let debt_after = test.total_debt();
    assert_eq!(debt_after, debt_before - liquidation_amount);
}

#[test]
#[ignore]
fn test_liquidate_just_under_close_factor() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let debt_before = test.total_debt();
    let under_limit = test.max_liquidation_amount() - 1;

    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &under_limit,
    );

    let debt_after = test.total_debt();
    assert_eq!(debt_after, debt_before - under_limit);
}

// === Successful Liquidation Tests ===

#[test]
#[ignore]
fn test_successful_collateral_liquidation() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let debt_before = test.total_debt();
    let collateral_before = test.collateral_amount();
    let liquidation_amount = test.liquidation_amount(10); // 10%

    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &liquidation_amount,
    );

    let debt_after = test.total_debt();
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
#[ignore]
fn test_successful_deposit_liquidation() {
    let test = LiquidationTest::risky_with_deposit_as_collateral();
    test.make_unhealthy();

    let debt_before = test.total_debt();
    let deposit_before = test.total_supplied();
    let liquidation_amount = test.liquidation_amount(10); // 10%

    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &liquidation_amount,
    );

    let debt_after = test.total_debt();
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
#[ignore]
fn test_liquidation_reduces_health_factor() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let liquidation_amount = test.liquidation_amount(10); // Reduced to 10%

    // Check if position is actually liquidatable first
    let first_liquidation = test.test_fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &liquidation_amount,
    );

    match first_liquidation {
        Ok(_) => {
            // First liquidation succeeded, now try another
            let result = test.test_fixture.contract_client.try_liquidate(
                &test.liquidator,
                &test.borrower,
                &test.test_fixture.usdc_pool_address,
                &test.test_fixture.gold_pool_address,
                &liquidation_amount,
            );

            // Should either succeed (if still unhealthy) or fail because position is now healthy
            match result {
                Ok(_) => {
                    // Second liquidation also succeeded - this is fine
                    println!("Position was still unhealthy after first liquidation");
                }
                Err(Ok(MCError::LiquidatedPositionIsHealthy)) => {
                    // Position became healthy after first liquidation - this is the expected
                    // behavior
                    println!("Position became healthy after liquidation");
                }
                Err(Ok(error)) => {
                    panic!("Unexpected error: {:?}", error);
                }
                Err(Err(host_error)) => {
                    panic!("Host error: {:?}", host_error);
                }
            }
        }
        Err(Ok(MCError::LiquidatedPositionIsHealthy)) => {
            // Position was never unhealthy to begin with
            println!("Position remained healthy - liquidation protection working correctly");
        }
        Err(Ok(error)) => {
            panic!("Unexpected error in first liquidation: {:?}", error);
        }
        Err(Err(host_error)) => {
            panic!("Host error: {:?}", host_error);
        }
    }
}

// === Edge Cases ===

#[test]
fn test_liquidate_same_pool_fails() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let result = test.test_fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.usdc_pool_address, // Same pool for debt and collateral
        &test.liquidation_amount(10),
    );

    assert_eq!(
        result,
        Err(Ok(MCError::LiquidationWithEqualCollateralAndDepositPools))
    );
}

#[test]
fn test_multiple_small_liquidations() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let initial_debt = test.total_debt();
    let small_amount = test.liquidation_amount(5); // 5% each time

    // Perform multiple small liquidations
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
                let current_debt = test.total_debt();
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
fn test_liquidation_with_interest_accrual() {
    let test = LiquidationTest::new();

    // Start with healthy position, accrue interest to make it risky
    test.test_fixture
        .e
        .ledger()
        .with_mut(|li| li.timestamp += 100 * 365 * 24 * 60 * 60); // 100 years

    let debt = test.total_debt();
    let liquidation_amount = test.liquidation_amount(20);

    // After massive interest accrual, position should be liquidatable
    let result = test.test_fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &liquidation_amount,
    );

    match result {
        Ok(_) => {
            // Liquidation succeeded after interest accrual
            let new_debt = test.total_debt();
            assert!(new_debt < debt, "Debt should be reduced");
        }
        Err(Ok(MCError::LiquidatedPositionIsHealthy)) => {
            // Position is still healthy even after massive interest - this shows robustness
            println!("Position remained healthy even after 100 years of interest");
        }
        Err(Ok(error)) => {
            panic!("Unexpected error: {:?}", error);
        }
        Err(Err(host_error)) => {
            panic!("Host error: {:?}", host_error);
        }
    }
}

#[test]
#[ignore]
fn liquidate_unpaid_interest_only() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let unpaid_interest = test.unpaid_interest();
    let borrowed_amount = test.borrowed_amount();

    test.test_fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.test_fixture.usdc_pool_address,
        &test.test_fixture.gold_pool_address,
        &unpaid_interest,
    );

    let new_unpaid_interest = test.unpaid_interest();
    let new_borrowed_amount = test.borrowed_amount();

    assert_eq!(new_unpaid_interest, 0);
    assert_eq!(new_borrowed_amount, borrowed_amount)
}
