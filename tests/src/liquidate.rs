#![cfg(test)]

use market::{
    constants::BPS_FACTOR,
    error::MCError,
    pool::{PoolConfig, PoolHealthConfig},
};
use soroban_sdk::{
    Address,
    testutils::{Address as _, Ledger},
};

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, compute_unparameterized_ltv_bps,
    get_obligation_collateral, get_obligation_d_tokens_as_tokens,
    get_obligation_initially_borrowed, get_obligation_j_tokens_as_tokens,
    get_obligation_unpaid_interest,
};

struct LiquidationTest {
    fixture: TestMarketFixture<'static>,
    borrower: Address,
    liquidator: Address,
    borrow_pool_address: Address,
    collateral_pool_address: Address,
}

impl LiquidationTest {
    /// Creates a standard setup with healthy position
    fn new() -> Self {
        let fixture = TestMarketFixture::new();
        let (borrow_pool_address, collateral_pool_address) =
            (fixture.usdc_pool_address.clone(), fixture.gold_pool_address.clone());

        let borrower = fixture.users[0].clone();
        let liquidity_provider = fixture.users[1].clone();
        let liquidator = fixture.users[2].clone();

        fixture.contract_client.deposit(
            &liquidity_provider,
            &borrow_pool_address,
            &(2 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );
        fixture.contract_client.add_collateral(
            &borrower,
            &collateral_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &None,
        );

        fixture.contract_client.borrow(
            &borrower,
            &borrow_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 3), // Conservative 33% borrow ratio
            &None,
        );

        Self { fixture, borrower, liquidator, borrow_pool_address, collateral_pool_address }
    }

    /// Creates a risky position closer to liquidation threshold
    fn risky() -> Self {
        let pool_config = PoolConfig {
            health_config: PoolHealthConfig {
                liability_factor_bps: (BPS_FACTOR * 11) / 10,
                ..Default::default()
            },
            ..Default::default()
        };
        let fixture = TestMarketFixture::new_with_pool_config(pool_config);
        let (borrow_pool_address, collateral_pool_address) =
            (fixture.usdc_pool_address.clone(), fixture.gold_pool_address.clone());

        let borrower = fixture.users[0].clone();
        let liquidity_provider = fixture.users[1].clone();
        let liquidator = fixture.users[2].clone();

        fixture.contract_client.deposit(
            &liquidity_provider,
            &borrow_pool_address,
            &((3 * DEFAULT_DEPOSIT_AMOUNT) / 2),
            &None,
        );

        let collateral = DEFAULT_DEPOSIT_AMOUNT;
        let borrow_amount = (DEFAULT_DEPOSIT_AMOUNT * 65) / 100; // 65% borrow ratio(default open LTV is 70%)

        fixture.contract_client.add_collateral(
            &borrower,
            &collateral_pool_address,
            &collateral,
            &None,
        );
        fixture.contract_client.borrow(&borrower, &borrow_pool_address, &borrow_amount, &None);

        Self { fixture, borrower, liquidator, borrow_pool_address, collateral_pool_address }
    }

    fn risky_with_deposit_as_collateral() -> Self {
        let fixture = TestMarketFixture::new();
        let (borrow_pool_address, collateral_pool_address) =
            (fixture.usdc_pool_address.clone(), fixture.gold_pool_address.clone());

        let borrower = fixture.users[0].clone();
        let liquidity_provider = fixture.users[1].clone();
        let liquidator = fixture.users[2].clone();

        fixture.contract_client.deposit(
            &liquidity_provider,
            &borrow_pool_address,
            &(2 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );

        fixture.contract_client.deposit(
            &borrower,
            &collateral_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &None,
        );
        fixture.contract_client.borrow(
            &borrower,
            &fixture.usdc_pool_address,
            &((DEFAULT_DEPOSIT_AMOUNT * 65) / 100), // 65% borrow ratio(default open LTV is 70%),
            &None,
        );

        Self { fixture, borrower, liquidator, borrow_pool_address, collateral_pool_address }
    }

    fn risky_with_both_as_a_collateral() -> Self {
        let fixture = TestMarketFixture::new();
        let (borrow_pool_address, collateral_pool_address) =
            (fixture.usdc_pool_address.clone(), fixture.gold_pool_address.clone());

        let borrower = fixture.users[0].clone();
        let liquidity_provider = fixture.users[1].clone();
        let liquidator = fixture.users[2].clone();

        fixture.contract_client.deposit(
            &liquidity_provider,
            &borrow_pool_address,
            &(2 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );

        fixture.contract_client.add_collateral(
            &borrower,
            &collateral_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 2),
            &None,
        );
        fixture.contract_client.deposit(
            &borrower,
            &collateral_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 2),
            &None,
        );
        fixture.contract_client.borrow(
            &borrower,
            &fixture.usdc_pool_address,
            &((DEFAULT_DEPOSIT_AMOUNT * 65) / 100), // 65% borrow ratio(default open LTV is 70%),
            &None,
        );

        Self { fixture, borrower, liquidator, borrow_pool_address, collateral_pool_address }
    }

    fn wait_n_years(&self, n: u64) {
        self.fixture.e.ledger().with_mut(|li| li.timestamp += n * 365 * 24 * 60 * 60);

        self.fixture.contract_client.refresh_pool(&self.borrow_pool_address);
        self.fixture.contract_client.refresh_pool(&self.collateral_pool_address);
    }

    fn ltv(&self) -> i128 {
        compute_unparameterized_ltv_bps(
            &self.fixture.e,
            &self.fixture.contract_client,
            &self.borrower,
        )
        .unwrap()
    }

    fn collateral(&self) -> i128 {
        get_obligation_collateral(
            &self.fixture.contract_client,
            &self.borrower,
            &self.collateral_pool_address,
        )
        .unwrap()
    }

    fn total_supplied(&self) -> i128 {
        get_obligation_j_tokens_as_tokens(
            &self.fixture.e,
            &self.fixture.contract_client,
            &self.borrower,
            &self.fixture.gold_pool_address,
        )
        .unwrap()
    }

    fn initially_borrowed(&self) -> i128 {
        get_obligation_initially_borrowed(
            &self.fixture.contract_client,
            &self.borrower,
            &self.fixture.usdc_pool_address,
        )
        .unwrap()
    }

    fn debt(&self) -> i128 {
        get_obligation_d_tokens_as_tokens(
            &self.fixture.e,
            &self.fixture.contract_client,
            &self.borrower,
            &self.fixture.usdc_pool_address,
        )
        .unwrap()
    }

    fn unpaid_interest(&self) -> i128 {
        get_obligation_unpaid_interest(
            &self.fixture.e,
            &self.fixture.contract_client,
            &self.borrower,
            &self.fixture.usdc_pool_address,
        )
        .unwrap()
    }

    fn liquidation_amount_from_percentage(&self, percentage: i128) -> i128 {
        (self.debt() * percentage) / 100
    }

    fn max_liquidation_amount(&self) -> i128 {
        let borrow_pool = self.fixture.contract_client.get_pool(&self.borrow_pool_address);

        (self.debt() * borrow_pool.config.health_config.liquidation_close_factor_bps) / BPS_FACTOR
    }

    fn max_collateral_amount(&self) -> i128 {
        let collateral_pool = self.fixture.contract_client.get_pool(&self.collateral_pool_address);
        let borrow_pool = self.fixture.contract_client.get_pool(&self.borrow_pool_address);
        let max_liquidation_incentive = borrow_pool
            .config
            .health_config
            .max_liquidation_incentive_bps
            .min(collateral_pool.config.health_config.max_liquidation_incentive_bps);

        (self.debt() * max_liquidation_incentive) / BPS_FACTOR
    }
}

// -- Basic Liquidation Tests --

#[test]
fn test_liquidate_healthy_position_fails() {
    let test = LiquidationTest::new();

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &1,
        &0,
    );

    assert_eq!(result, Err(Ok(MCError::ObligationIsHealthy)));
}

#[test]
fn test_liquidate_zero() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let borrow_pool_before = test.fixture.contract_client.get_pool(&test.borrow_pool_address);
    let collateral_pool_before =
        test.fixture.contract_client.get_pool(&test.collateral_pool_address);
    let debt_before = test.debt();

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &0,
        &0,
    );

    let borrow_pool_after = test.fixture.contract_client.get_pool(&test.borrow_pool_address);
    let collateral_pool_after =
        test.fixture.contract_client.get_pool(&test.collateral_pool_address);
    let debt_after = test.debt();

    assert_eq!(debt_before, debt_after);
    assert_eq!(borrow_pool_before, borrow_pool_after);
    assert_eq!(collateral_pool_before, collateral_pool_after);
}

#[test]
fn test_liquidate_negative_amount() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &-1,
        &0,
    );
    assert_eq!(result, Err(Ok(MCError::NegativeInputAmount)));

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &0,
        &-1,
    );
    assert_eq!(result, Err(Ok(MCError::NegativeInputAmount)));
}

#[test]
fn test_liquidate_self_fails() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let result = test.fixture.contract_client.try_liquidate(
        &test.borrower,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &0,
        &0,
    );

    assert_eq!(result, Err(Ok(MCError::InvalidLiquidationInputs)));
}

#[test]
fn test_liquidate_nonexistent_user_fails() {
    let test = LiquidationTest::new();
    let fake_user = Address::generate(&test.fixture.e);

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &fake_user,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &0,
        &0,
    );

    assert_eq!(result, Err(Ok(MCError::ObligationDoesNotExist)));
}

// -- Close Factor Tests --

#[test]
fn test_liquidate_exceeds_close_factor_fails() {
    let test = LiquidationTest::risky();
    test.wait_n_years(2);

    let max_allowed_liquidation = test.max_liquidation_amount();
    let over_limit_amount = max_allowed_liquidation + 1;

    assert_eq!(
        test.fixture.contract_client.try_liquidate(
            &test.liquidator,
            &test.borrower,
            &None,
            &test.borrow_pool_address,
            &test.collateral_pool_address,
            &over_limit_amount,
            &1,
        ),
        Err(Ok(MCError::InvalidLiquidationInputs))
    );
}

#[test]
fn test_excessive_demanded_collateral_amount() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let max_allowed_liquidation = test.max_liquidation_amount();
    let liquidation_amount = max_allowed_liquidation;
    let collateral_amount = 2 * max_allowed_liquidation;

    assert_eq!(
        test.fixture.contract_client.try_liquidate(
            &test.liquidator,
            &test.borrower,
            &None,
            &test.borrow_pool_address,
            &test.collateral_pool_address,
            &liquidation_amount,
            &collateral_amount,
        ),
        Err(Ok(MCError::LiquidationExcessiveDemandedCollateral))
    );
}

// -- Successful Liquidation Tests --

#[test]
fn test_liquidate_at_exact_close_factor() {
    let test = LiquidationTest::risky();
    test.wait_n_years(2);

    let debt_before = test.debt();
    let collateral_before = test.collateral();
    let ltv_before = test.ltv();

    let liquidation_amount = test.max_liquidation_amount();
    let collateral_seized_amount = 15_000;
    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &collateral_seized_amount,
    );

    let debt_after = test.debt();
    let collateral_after = test.collateral();
    let ltv_after = test.ltv();

    assert_eq!(debt_after, debt_before - liquidation_amount);
    assert!(collateral_after <= collateral_before - collateral_seized_amount);
    assert!(ltv_after < ltv_before);
}

#[test]
fn test_insolvent_liquidation_can_exceed_close_factor() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let debt_before = test.debt();

    let max_allowed_liquidation = test.max_liquidation_amount();
    let liquidation_amount = max_allowed_liquidation + 1;

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &1,
    );

    let debt_after = test.debt();
    assert_eq!(debt_after, debt_before - liquidation_amount);
}

#[test]
fn test_liquidate_deposit_successful() {
    let test = LiquidationTest::risky_with_deposit_as_collateral();
    test.wait_n_years(2);

    let debt_before = test.debt();
    let deposit_before = test.total_supplied();
    let liquidation_amount = test.liquidation_amount_from_percentage(10);
    let collateral_amount = test.liquidation_amount_from_percentage(10);

    assert_eq!(
        test.fixture.contract_client.try_get_user_obligation(&test.liquidator),
        Err(Ok(MCError::ObligationDoesNotExist))
    );

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &collateral_amount,
    );

    let debt_after = test.debt();
    let deposit_after = test.total_supplied();
    let liquidator_j_tokens_tokens = get_obligation_j_tokens_as_tokens(
        &test.fixture.e,
        &test.fixture.contract_client,
        &test.liquidator,
        &test.collateral_pool_address,
    )
    .unwrap();

    assert_eq!(debt_after, debt_before - liquidation_amount);
    assert!(
        deposit_after < deposit_before,
        "Deposit should be reduced from {} to {}",
        deposit_before,
        deposit_after
    );
    assert!(collateral_amount <= liquidator_j_tokens_tokens);
}

#[test]
fn test_liquidating_solvent_debt_reduces_ltv() {
    let test = LiquidationTest::risky_with_deposit_as_collateral();
    test.wait_n_years(2);

    let ltv_before = test.ltv();
    let liquidation_amount = test.max_liquidation_amount();
    let collateral_amount = test.max_collateral_amount();

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &collateral_amount,
    );

    let ltv_after = test.ltv();

    assert!(ltv_after < ltv_before);
}

#[test]
fn test_liquidating_insolvent_debt_increases_ltv() {
    let test = LiquidationTest::risky_with_deposit_as_collateral();
    test.wait_n_years(3);

    let ltv_before = test.ltv();
    let liquidation_amount = test.max_liquidation_amount();
    let collateral_amount = test.max_collateral_amount();

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &collateral_amount,
    );

    let ltv_after = test.ltv();

    assert!(ltv_after > ltv_before);
}

#[test]
fn test_liquidate_both_plain_collateral_and_shares() {
    let test = LiquidationTest::risky_with_both_as_a_collateral();
    test.wait_n_years(3);

    let debt_before = test.debt();
    let deposit_before = test.total_supplied();
    let collateral_before = test.collateral();
    let liquidation_amount = debt_before;
    let collateral_amount = deposit_before + collateral_before;

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &collateral_amount,
    );

    assert_eq!(
        test.fixture.contract_client.try_get_user_obligation(&test.borrower),
        Err(Ok(MCError::ObligationDoesNotExist))
    );
}

#[test]
fn test_min_collateral_seized() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let debt_before = test.debt();
    let deposit_before = test.total_supplied();
    let collateral_before = test.collateral();
    let liquidation_amount = (2 * debt_before) / 3;
    let collateral_amount = ((deposit_before + collateral_before) / 2) + 100;

    test.fixture.contract_client.update_market(&10, &(collateral_amount / 2));

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &collateral_amount,
    );

    assert_eq!(
        get_obligation_collateral(
            &test.fixture.contract_client,
            &test.borrower,
            &test.collateral_pool_address
        ),
        Err(MCError::DepositPositionDoesNotExist)
    );
}

#[test]
fn test_liquidated_all_mixed_collateral() {
    let test = LiquidationTest::risky_with_both_as_a_collateral();
    test.wait_n_years(3);

    let debt_before = test.debt();
    let deposit_before = test.total_supplied();
    let collateral_before = test.collateral();
    let liquidation_amount = debt_before;
    let collateral_amount = deposit_before + collateral_before;

    test.fixture.contract_client.update_market(&10, &(collateral_amount / 2));

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &collateral_amount,
    );

    assert_eq!(
        get_obligation_collateral(
            &test.fixture.contract_client,
            &test.borrower,
            &test.collateral_pool_address
        ),
        Err(MCError::ObligationDoesNotExist)
    );
}
// -- Edge Cases --

#[test]
fn test_liquidate_same_pool_fails() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.borrow_pool_address,
        &1,
        &0,
    );

    assert_eq!(result, Err(Ok(MCError::InvalidLiquidationInputs)));
}

#[test]
fn test_liquidate_multiple_small() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let initial_debt = test.debt();
    let small_amount = test.liquidation_amount_from_percentage(5);

    // Multiple small liquidations
    for i in 1..=3 {
        let result = test.fixture.contract_client.try_liquidate(
            &test.liquidator,
            &test.borrower,
            &None,
            &test.borrow_pool_address,
            &test.collateral_pool_address,
            &small_amount,
            &small_amount,
        );

        match result {
            Ok(_) => {
                let current_debt = test.debt();
                let expected = initial_debt - (small_amount * i);
                assert_eq!(current_debt, expected, "Liquidation {} failed", i);
            }
            Err(Ok(MCError::ObligationIsHealthy)) => {
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
    test.fixture.e.ledger().with_mut(|li| li.timestamp += 40 * 365 * 24 * 60 * 60); // 40 years
    test.fixture.contract_client.refresh_pool(&test.fixture.usdc_pool_address);

    let debt = test.debt();
    let liquidation_amount = test.liquidation_amount_from_percentage(20);

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &1,
    );

    let new_debt = test.debt();

    assert!(new_debt < debt, "Debt should be reduced");
}

#[test]
fn test_liquidate_unpaid_interest_only() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let initially_borrowed_before = test.initially_borrowed();
    let unpaid_interest = test.unpaid_interest();

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &test.borrower,
        &None,
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &unpaid_interest,
        &1,
    );

    let initially_borrowed_after = test.initially_borrowed();
    let new_unpaid_interest = test.unpaid_interest();
    let new_initially_borrowed = test.initially_borrowed();
    let new_debt = test.debt();

    assert_eq!(new_unpaid_interest, 0);
    assert_eq!(new_initially_borrowed, new_debt);
    assert_eq!(initially_borrowed_before, initially_borrowed_after);
}
