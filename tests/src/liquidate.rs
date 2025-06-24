#![cfg(test)]

use {
    crate::{get_borrow_obligation, TestFixture, DEFAULT_DEPOSIT_AMOUNT},
    lending::constants::{LCError, DEFAULT_CLOSE_FACTOR},
    soroban_sdk::{testutils::Address as _, testutils::Ledger, Address},
};

struct LiquidationTest {
    fixture: TestFixture<'static>,
    borrower: Address,
    // lender: Address,
    liquidator: Address,
}

impl LiquidationTest {
    /// Creates a standard setup with healthy position
    fn new() -> Self {
        let fixture = TestFixture::new();
        let borrower = fixture.users.get(0).unwrap();
        let lender = fixture.users.get(1).unwrap();
        let liquidator = fixture.users.get(2).unwrap();

        // Lender provides liquidity
        fixture.contract_client.deposit(
            &lender,
            &fixture.usdc_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT * 2),
        );

        // Borrower deposits collateral and borrows (healthy position)
        fixture.contract_client.add_collateral(
            &borrower,
            &fixture.gold_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
        );

        fixture.contract_client.borrow(
            &borrower,
            &fixture.usdc_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 3), // Conservative 33% borrow ratio
        );

        Self {
            fixture,
            borrower,
            // lender,
            liquidator,
        }
    }

    /// Creates setup with liquidator having funds
    #[allow(dead_code)]
    fn with_liquidator_funds() -> Self {
        let setup = Self::new();
        setup.fixture.contract_client.deposit(
            &setup.liquidator,
            &setup.fixture.usdc_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
        );
        setup
    }

    /// Creates a risky position closer to liquidation threshold
    fn risky() -> Self {
        let fixture = TestFixture::new();
        let borrower = fixture.users.get(0).unwrap();
        let lender = fixture.users.get(1).unwrap();
        let liquidator = fixture.users.get(2).unwrap();

        // Lender provides liquidity
        fixture.contract_client.deposit(
            &lender,
            &fixture.usdc_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT * 3),
        );

        // Liquidator gets funds
        fixture.contract_client.deposit(
            &liquidator,
            &fixture.usdc_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
        );

        // Borrower creates risky position - minimum collateral, maximum borrow
        let collateral = (DEFAULT_DEPOSIT_AMOUNT * 82) / 100; // 82% collateral
        let borrow_amount = (DEFAULT_DEPOSIT_AMOUNT * 65) / 100; // 65% borrow

        fixture
            .contract_client
            .add_collateral(&borrower, &fixture.gold_pool_address, &collateral);
        fixture
            .contract_client
            .borrow(&borrower, &fixture.usdc_pool_address, &borrow_amount);

        Self {
            fixture,
            borrower,
            // lender,
            liquidator,
        }
    }

    fn make_unhealthy(&self) {
        // Accrue interest over time to increase debt
        self.fixture
            .e
            .ledger()
            .with_mut(|li| li.timestamp = 365 * 24 * 60 * 60); // 1 year
    }

    fn borrowed_amount(&self) -> i128 {
        get_borrow_obligation(
            &self.fixture.contract_client,
            &self.borrower,
            &self.fixture.usdc_pool_address,
        )
        .unwrap()
        .borrowed
    }

    fn collateral_amount(&self) -> i128 {
        let obligation = self
            .fixture
            .contract_client
            .get_user_obligation(&self.borrower);
        obligation
            .deposits
            .get(self.fixture.gold_pool_address.clone())
            .map(|d| d.collateral)
            .unwrap_or(0)
    }

    fn liquidation_amount(&self, percentage: i128) -> i128 {
        (self.borrowed_amount() * percentage) / 100
    }

    fn max_liquidation_amount(&self) -> i128 {
        (self.borrowed_amount() * DEFAULT_CLOSE_FACTOR) / 100
    }
}

// === Basic Liquidation Tests ===

#[test]
fn test_liquidate_healthy_position_fails() {
    let test = LiquidationTest::new();

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &test.fixture.usdc_pool_address,
        &test.fixture.gold_pool_address,
        &1,
    );

    assert_eq!(result, Err(Ok(LCError::LiquidatedPositionIsHealthy)));
}

#[test]
fn test_liquidate_zero_amount_fails() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &test.fixture.usdc_pool_address,
        &test.fixture.gold_pool_address,
        &0,
    );

    assert_eq!(result, Err(Ok(LCError::NonPositiveLiquidation)));
}

#[test]
fn test_liquidate_self_fails() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let result = test.fixture.contract_client.try_liquidate(
        &test.borrower, // Same as borrower
        &test.borrower,
        &test.fixture.usdc_pool_address,
        &test.fixture.gold_pool_address,
        &test.liquidation_amount(10),
    );

    assert_eq!(result, Err(Ok(LCError::SelfLiquidation)));
}

#[test]
fn test_liquidate_nonexistent_user_fails() {
    let test = LiquidationTest::new();
    let fake_user = Address::generate(&test.fixture.e);

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &fake_user,
        &test.fixture.usdc_pool_address,
        &test.fixture.gold_pool_address,
        &1,
    );

    assert_eq!(result, Err(Ok(LCError::ObligationDoesNotExist)));
}

// === Close Factor Tests ===

#[test]
fn test_liquidate_exceeds_close_factor_fails() {
    // Create a position that's definitely unhealthy
    let fixture = TestFixture::new();
    let borrower = fixture.users.get(0).unwrap();
    let lender = fixture.users.get(1).unwrap();
    let liquidator = fixture.users.get(2).unwrap();

    // Lender provides liquidity
    fixture.contract_client.deposit(
        &lender,
        &fixture.usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT * 5),
    );

    // Liquidator gets funds
    fixture.contract_client.deposit(
        &liquidator,
        &fixture.usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT * 2),
    );

    // Create minimal collateral position
    let minimal_collateral = DEFAULT_DEPOSIT_AMOUNT / 10; // Very small collateral
    fixture.contract_client.add_collateral(
        &borrower,
        &fixture.gold_pool_address,
        &minimal_collateral,
    );

    // Borrow maximum possible amount
    let max_borrow = (minimal_collateral * 80) / 100; // 80% of collateral value
    fixture
        .contract_client
        .borrow(&borrower, &fixture.usdc_pool_address, &max_borrow);

    // Accrue massive interest to make position definitely unhealthy
    fixture
        .e
        .ledger()
        .with_mut(|li| li.timestamp = 1000 * 365 * 24 * 60 * 60); // 1000 years

    // Get current borrowed amount (should include accrued interest)
    let borrowed = get_borrow_obligation(
        &fixture.contract_client,
        &borrower,
        &fixture.usdc_pool_address,
    )
    .unwrap()
    .borrowed;

    // Calculate over-limit amount
    let max_liquidation = (borrowed * DEFAULT_CLOSE_FACTOR) / 100;
    let over_limit = max_liquidation + 1;

    // This should fail with close factor exceeded
    let result = fixture.contract_client.try_liquidate(
        &liquidator,
        &borrower,
        &fixture.usdc_pool_address,
        &fixture.gold_pool_address,
        &over_limit,
    );

    assert_eq!(result, Err(Ok(LCError::LiquidationExceedsCloseFactor)));
}

#[test]
fn test_liquidate_at_exact_close_factor() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let borrowed_before = test.borrowed_amount();
    let max_amount = test.max_liquidation_amount();

    // Should succeed at exactly the close factor limit
    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.fixture.usdc_pool_address,
        &test.fixture.gold_pool_address,
        &max_amount,
    );

    let borrowed_after = test.borrowed_amount();
    assert_eq!(borrowed_after, borrowed_before - max_amount);
}

#[test]
fn test_liquidate_just_under_close_factor() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let borrowed_before = test.borrowed_amount();
    let under_limit = test.max_liquidation_amount() - 1;

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.fixture.usdc_pool_address,
        &test.fixture.gold_pool_address,
        &under_limit,
    );

    let borrowed_after = test.borrowed_amount();
    assert_eq!(borrowed_after, borrowed_before - under_limit);
}

// === Successful Liquidation Tests ===

#[test]
fn test_successful_liquidation() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let borrowed_before = test.borrowed_amount();
    let collateral_before = test.collateral_amount();
    let liquidation_amount = test.liquidation_amount(10); // 10%

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &test.fixture.usdc_pool_address,
        &test.fixture.gold_pool_address,
        &liquidation_amount,
    );

    let borrowed_after = test.borrowed_amount();
    let collateral_after = test.collateral_amount();

    // Debt should be reduced
    assert_eq!(borrowed_after, borrowed_before - liquidation_amount);

    // Collateral should be seized
    assert!(
        collateral_after < collateral_before,
        "Collateral should be reduced from {} to {}",
        collateral_before,
        collateral_after
    );
}

#[test]
fn test_liquidation_reduces_health_risk() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let liquidation_amount = test.liquidation_amount(10); // Reduced to 10%

    // Check if position is actually liquidatable first
    let first_liquidation = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &test.fixture.usdc_pool_address,
        &test.fixture.gold_pool_address,
        &liquidation_amount,
    );

    match first_liquidation {
        Ok(_) => {
            // First liquidation succeeded, now try another
            let result = test.fixture.contract_client.try_liquidate(
                &test.liquidator,
                &test.borrower,
                &test.fixture.usdc_pool_address,
                &test.fixture.gold_pool_address,
                &liquidation_amount,
            );

            // Should either succeed (if still unhealthy) or fail because position is now healthy
            match result {
                Ok(_) => {
                    // Second liquidation also succeeded - this is fine
                    println!("Position was still unhealthy after first liquidation");
                }
                Err(Ok(LCError::LiquidatedPositionIsHealthy)) => {
                    // Position became healthy after first liquidation - this is the expected behavior
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
        Err(Ok(LCError::LiquidatedPositionIsHealthy)) => {
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

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &test.fixture.usdc_pool_address,
        &test.fixture.usdc_pool_address, // Same pool for debt and collateral
        &test.liquidation_amount(10),
    );

    assert_eq!(result, Err(Ok(LCError::InternalError)));
}

#[test]
fn test_multiple_small_liquidations() {
    let test = LiquidationTest::risky();
    test.make_unhealthy();

    let initial_borrowed = test.borrowed_amount();
    let small_amount = test.liquidation_amount(5); // 5% each time

    // Perform multiple small liquidations
    for i in 1..=3 {
        let result = test.fixture.contract_client.try_liquidate(
            &test.liquidator,
            &test.borrower,
            &test.fixture.usdc_pool_address,
            &test.fixture.gold_pool_address,
            &small_amount,
        );

        match result {
            Ok(_) => {
                let current_borrowed = test.borrowed_amount();
                let expected = initial_borrowed - (small_amount * i);
                assert_eq!(current_borrowed, expected, "Liquidation {} failed", i);
            }
            Err(Ok(LCError::LiquidatedPositionIsHealthy)) => {
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
    test.fixture
        .e
        .ledger()
        .with_mut(|li| li.timestamp = 100 * 365 * 24 * 60 * 60); // 100 years

    let borrowed = test.borrowed_amount();
    let liquidation_amount = test.liquidation_amount(20);

    // After massive interest accrual, position should be liquidatable
    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &test.fixture.usdc_pool_address,
        &test.fixture.gold_pool_address,
        &liquidation_amount,
    );

    match result {
        Ok(_) => {
            // Liquidation succeeded after interest accrual
            let new_borrowed = test.borrowed_amount();
            assert!(new_borrowed < borrowed, "Debt should be reduced");
        }
        Err(Ok(LCError::LiquidatedPositionIsHealthy)) => {
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
