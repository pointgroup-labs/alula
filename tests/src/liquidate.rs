#![cfg(test)]

use market::{
    constants::{
        BPS_FACTOR, DEFAULT_BAD_DEBT_LOCK_D, DEFAULT_INSOLVENCY_LTV_BPS, DEFAULT_MAX_POSITIONS,
    },
    error::MCError,
    obligation::ObligationKey,
    request::{LiquidateRequest, Request},
};
use soroban_sdk::{
    Address,
    testutils::{Address as _, IssuerFlags, Ledger},
    token::TokenClient,
};

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture,
    bad_debt::{ONE, gold_cents, lend_btc_and_usdc, lock_of, set_5_usd_dust_threshold},
    compute_unparameterized_ltv_bps, get_obligation_collateral, get_obligation_d_tokens,
    get_obligation_d_tokens_as_tokens, get_obligation_initially_borrowed, get_obligation_j_tokens,
    get_obligation_j_tokens_as_tokens, get_obligation_unpaid_interest, get_pool_total_available,
    get_pool_total_borrowed,
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
            &ObligationKey::new(liquidity_provider.clone()),
            &borrow_pool_address,
            &(2 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );
        fixture.contract_client.add_collateral(
            &ObligationKey::new(borrower.clone()),
            &collateral_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &None,
        );

        fixture.contract_client.borrow(
            &ObligationKey::new(borrower.clone()),
            &borrow_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 3), // Conservative 33% borrow ratio
            &None,
        );

        Self { fixture, borrower, liquidator, borrow_pool_address, collateral_pool_address }
    }

    /// Creates a risky position closer to liquidation threshold
    fn risky() -> Self {
        let fixture = TestMarketFixture::new();
        let (borrow_pool_address, collateral_pool_address) =
            (fixture.usdc_pool_address.clone(), fixture.gold_pool_address.clone());

        let borrower = fixture.users[0].clone();
        let liquidity_provider = fixture.users[1].clone();
        let liquidator = fixture.users[2].clone();

        fixture.contract_client.deposit(
            &ObligationKey::new(liquidity_provider.clone()),
            &borrow_pool_address,
            &((3 * DEFAULT_DEPOSIT_AMOUNT) / 2),
            &None,
        );

        let collateral = DEFAULT_DEPOSIT_AMOUNT;
        let borrow_amount = (DEFAULT_DEPOSIT_AMOUNT * 65) / 100; // 65% borrow ratio(default open LTV is 70%)

        fixture.contract_client.add_collateral(
            &ObligationKey::new(borrower.clone()),
            &collateral_pool_address,
            &collateral,
            &None,
        );
        fixture.contract_client.borrow(
            &ObligationKey::new(borrower.clone()),
            &borrow_pool_address,
            &borrow_amount,
            &None,
        );

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
            &ObligationKey::new(liquidity_provider.clone()),
            &borrow_pool_address,
            &(2 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );

        fixture.contract_client.deposit(
            &ObligationKey::new(borrower.clone()),
            &collateral_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &None,
        );
        fixture.contract_client.borrow(
            &ObligationKey::new(borrower.clone()),
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
            &ObligationKey::new(liquidity_provider.clone()),
            &borrow_pool_address,
            &(2 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );

        fixture.contract_client.add_collateral(
            &ObligationKey::new(borrower.clone()),
            &collateral_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 2),
            &None,
        );
        fixture.contract_client.deposit(
            &ObligationKey::new(borrower.clone()),
            &collateral_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 2),
            &None,
        );
        fixture.contract_client.borrow(
            &ObligationKey::new(borrower.clone()),
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

    fn set_min_collateral_value_cents(&self, cents: i128) {
        let update_in_queue_period =
            self.fixture.contract_client.get_global_state().update_in_queue_period;

        self.fixture.contract_client.queue_in_market_update(
            &DEFAULT_MAX_POSITIONS,
            &cents,
            &DEFAULT_BAD_DEBT_LOCK_D,
        );
        self.fixture.e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);
        self.fixture.contract_client.refresh_pool(&self.borrow_pool_address);
        self.fixture.contract_client.refresh_pool(&self.collateral_pool_address);
        self.fixture.contract_client.apply_market_update();
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
        &ObligationKey::new(test.borrower.clone()),
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

    assert_eq!(
        test.fixture.contract_client.try_liquidate(
            &test.liquidator,
            &ObligationKey::new(test.borrower.clone()),
            &test.borrow_pool_address,
            &test.collateral_pool_address,
            &0,
            &0,
        ),
        Err(Ok(MCError::InvalidInputAmount))
    );
}

#[test]
fn test_liquidate_negative_amount() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &-1,
        &0,
    );
    assert_eq!(result, Err(Ok(MCError::InvalidInputAmount)));

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &0,
        &-1,
    );
    assert_eq!(result, Err(Ok(MCError::InvalidInputAmount)));
}

#[test]
fn test_liquidate_self_fails() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let result = test.fixture.contract_client.try_liquidate(
        &test.borrower,
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &1,
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
        &ObligationKey::new(fake_user),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &1,
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
            &ObligationKey::new(test.borrower.clone()),
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
            &ObligationKey::new(test.borrower.clone()),
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
        &ObligationKey::new(test.borrower.clone()),
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
        &ObligationKey::new(test.borrower.clone()),
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
    let min_demanded_collateral = test.collateral();

    assert_eq!(
        test.fixture
            .contract_client
            .try_get_user_obligation(&ObligationKey::new(test.liquidator.clone())),
        Err(Ok(MCError::ObligationDoesNotExist))
    );

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &min_demanded_collateral,
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
    assert!(min_demanded_collateral <= liquidator_j_tokens_tokens);
}

#[test]
fn test_liquidate_deposit_reverts_for_deauthorized_liquidator() {
    let test = LiquidationTest::risky_with_deposit_as_collateral();
    test.wait_n_years(2);

    // Deposit-share seizure moves no tokens, so a transfer-gated collateral token can only
    // reject the liquidator through the zero-amount collateral transfer the market still makes.
    assert_eq!(test.collateral(), 0);

    test.fixture.gold_issuer.set_flag(IssuerFlags::RevocableFlag);
    test.fixture.gold_sac.set_authorized(&test.liquidator, &false);

    let debt_before = test.debt();
    let deposit_before = test.total_supplied();
    let liquidation_amount = test.liquidation_amount_from_percentage(10);

    // The SAC's BalanceDeauthorizedError is contract error 11, which the client decodes as
    // MCError's code 11. Patching a zero-amount skip into the transfer loop makes this succeed.
    assert_eq!(
        test.fixture.contract_client.try_liquidate(
            &test.liquidator,
            &ObligationKey::new(test.borrower.clone()),
            &test.borrow_pool_address,
            &test.collateral_pool_address,
            &liquidation_amount,
            &0,
        ),
        Err(Ok(MCError::MinCollateralValueIsNotMet))
    );
    assert_eq!(test.debt(), debt_before);
    assert_eq!(test.total_supplied(), deposit_before);
    assert_eq!(
        test.fixture
            .contract_client
            .try_get_user_obligation(&ObligationKey::new(test.liquidator.clone())),
        Err(Ok(MCError::ObligationDoesNotExist))
    );

    test.fixture.gold_sac.set_authorized(&test.liquidator, &true);
    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &0,
    );
    assert_eq!(test.debt(), debt_before - liquidation_amount);
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
        &ObligationKey::new(test.borrower.clone()),
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
        &ObligationKey::new(test.borrower.clone()),
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
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &collateral_amount,
    );

    assert_eq!(
        test.fixture
            .contract_client
            .try_get_user_obligation(&ObligationKey::new(test.borrower.clone())),
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

    let update_in_queue_period =
        test.fixture.contract_client.get_global_state().update_in_queue_period;

    test.fixture.contract_client.queue_in_market_update(&10, &1, &DEFAULT_BAD_DEBT_LOCK_D);
    test.fixture.e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);
    test.fixture.contract_client.refresh_pool(&test.borrow_pool_address);
    test.fixture.contract_client.refresh_pool(&test.collateral_pool_address);
    test.fixture.contract_client.apply_market_update();

    // Two thirds of the debt would leave collateral under the floor beside the rest of the debt.
    assert_eq!(
        test.fixture.contract_client.try_liquidate(
            &test.liquidator,
            &ObligationKey::new(test.borrower.clone()),
            &test.borrow_pool_address,
            &test.collateral_pool_address,
            &liquidation_amount,
            &collateral_amount,
        ),
        Err(Ok(MCError::LiquidationLeavesDust))
    );
    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &debt_before,
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

    let update_in_queue_period =
        test.fixture.contract_client.get_global_state().update_in_queue_period;

    test.wait_n_years(3);

    test.fixture.contract_client.queue_in_market_update(&10, &1, &DEFAULT_BAD_DEBT_LOCK_D);
    test.fixture.e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);
    test.fixture.contract_client.refresh_pool(&test.borrow_pool_address);
    test.fixture.contract_client.refresh_pool(&test.collateral_pool_address);
    test.fixture.contract_client.apply_market_update();

    let debt_before = test.debt();
    let deposit_before = test.total_supplied();
    let collateral_before = test.collateral();
    let liquidation_amount = debt_before;
    let collateral_amount = deposit_before + collateral_before;

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &collateral_amount,
    );

    // The collateral is sold for its value, so the debt it did not cover is left for the cover to write off.
    assert_eq!(
        get_obligation_collateral(
            &test.fixture.contract_client,
            &test.borrower,
            &test.collateral_pool_address
        ),
        Err(MCError::DepositPositionDoesNotExist)
    );
    test.fixture.contract_client.issue_cover_bad_debt(&ObligationKey::new(test.borrower.clone()));
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
        &ObligationKey::new(test.borrower.clone()),
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
            &ObligationKey::new(test.borrower.clone()),
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
        &ObligationKey::new(test.borrower.clone()),
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
        &ObligationKey::new(test.borrower.clone()),
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

#[test]
fn test_liquidation_increases_borrow_pool_total_available() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let pool_available_before =
        get_pool_total_available(&test.fixture.contract_client, &test.borrow_pool_address);
    let pool_borrowed_before =
        get_pool_total_borrowed(&test.fixture.contract_client, &test.borrow_pool_address);

    let liquidation_amount = test.max_liquidation_amount();

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &liquidation_amount,
        &1,
    );

    let pool_available_after =
        get_pool_total_available(&test.fixture.contract_client, &test.borrow_pool_address);
    let pool_borrowed_after =
        get_pool_total_borrowed(&test.fixture.contract_client, &test.borrow_pool_address);

    assert_eq!(
        pool_available_after,
        pool_available_before + liquidation_amount,
        "Repaid debt must flow into total_available"
    );
    assert_eq!(
        pool_borrowed_after,
        pool_borrowed_before - liquidation_amount,
        "Repaid debt must reduce total_borrowed"
    );
}

#[test]
fn test_liquidate_with_excess_repay_amount_refunds_difference() {
    let test = LiquidationTest::risky();
    test.wait_n_years(3);

    let liquidator_borrow_token_before = test.fixture.usdc_token_client.balance(&test.liquidator);
    let liquidator_collateral_token_before =
        test.fixture.gold_token_client.balance(&test.liquidator);
    let contract_borrow_token_before =
        test.fixture.usdc_token_client.balance(&test.fixture.contract_id);

    let actual_debt = test.debt();

    let overshoot = 500;
    let repay_amount = actual_debt + overshoot;

    test.fixture.contract_client.liquidate(
        &test.liquidator,
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &repay_amount,
        &0,
    );

    let liquidator_borrow_token_after = test.fixture.usdc_token_client.balance(&test.liquidator);
    let contract_borrow_token_after =
        test.fixture.usdc_token_client.balance(&test.fixture.contract_id);

    let liquidator_net_spend = liquidator_borrow_token_before - liquidator_borrow_token_after;
    assert_eq!(
        liquidator_net_spend, actual_debt,
        "Liquidator should only pay the actual debt ({actual_debt}), not the full repay_amount \
         ({repay_amount}). Net spend was {liquidator_net_spend}"
    );

    let contract_borrow_token_increase = contract_borrow_token_after - contract_borrow_token_before;
    assert_eq!(
        contract_borrow_token_increase, actual_debt,
        "Contract borrow-token balance should increase by actual debt"
    );

    let liquidator_collateral_token_after =
        test.fixture.gold_token_client.balance(&test.liquidator);
    assert!(
        liquidator_collateral_token_after > liquidator_collateral_token_before,
        "Liquidator should receive collateral"
    );

    assert_eq!(
        get_obligation_d_tokens_as_tokens(
            &test.fixture.e,
            &test.fixture.contract_client,
            &test.borrower,
            &test.borrow_pool_address,
        ),
        Err(MCError::ObligationDoesNotExist)
    );
}

#[test]
fn test_liquidate_below_min_collateral_while_close_ltv_solvent() {
    let test = LiquidationTest::new();
    test.set_min_collateral_value_cents(100);

    // Half the debt would leave both positions under the floor; the whole debt is past the close factor, which
    // a position this small does not have.
    assert_eq!(
        test.fixture.contract_client.try_liquidate(
            &test.liquidator,
            &ObligationKey::new(test.borrower.clone()),
            &test.borrow_pool_address,
            &test.collateral_pool_address,
            &test.max_liquidation_amount(),
            &0,
        ),
        Err(Ok(MCError::LiquidationLeavesDust))
    );
    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &test.debt(),
        &0,
    );

    assert!(
        result.is_ok(),
        "Position below min_collateral_value_cents must be liquidatable even when close-LTV \
         solvent, got: {result:?}"
    );
    assert_eq!(
        get_obligation_d_tokens(
            &test.fixture.contract_client,
            &test.borrower,
            &test.borrow_pool_address
        ),
        Err(MCError::BorrowPositionDoesNotExist),
        "Debt should be cleared by the liquidation"
    );
}

#[test]
fn test_below_min_collateral_liquidation_not_rejected_as_healthy() {
    let test = LiquidationTest::new();
    test.set_min_collateral_value_cents(100);

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &1,
        &0,
    );

    assert_ne!(
        result,
        Err(Ok(MCError::ObligationIsHealthy)),
        "Sub-minimum collateral positions must not be treated as healthy"
    );
}

#[test]
fn test_healthy_position_above_min_collateral_still_healthy() {
    let test = LiquidationTest::new();

    let result = test.fixture.contract_client.try_liquidate(
        &test.liquidator,
        &ObligationKey::new(test.borrower.clone()),
        &test.borrow_pool_address,
        &test.collateral_pool_address,
        &1,
        &0,
    );

    assert_eq!(result, Err(Ok(MCError::ObligationIsHealthy)));
}

#[test]
fn test_min_collateral_deduction_scales_with_position_count() {
    let fixture = TestMarketFixture::new();

    let liquidity_provider = fixture.users[0].clone();
    let liquidator = fixture.users[1].clone();

    let single_pos_borrower = fixture.users[2].clone();
    let split_pos_borrower = fixture.users[3].clone();

    let borrow_pool = fixture.usdc_pool_address.clone();
    let gold_pool = fixture.gold_pool_address.clone();
    let btc_pool = fixture.btc_pool_address.clone();

    let total_collateral: i128 = 5_000_000;
    let half_collateral: i128 = total_collateral / 2;
    let debt: i128 = 2_500_000;

    // Deep borrow-pool liquidity so both borrows are serviceable.
    fixture.contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &borrow_pool,
        &(10 * total_collateral),
        &None,
    );

    // Borrower A: all collateral in a single pool (N = 1).
    fixture.contract_client.add_collateral(
        &ObligationKey::new(single_pos_borrower.clone()),
        &gold_pool,
        &total_collateral,
        &None,
    );
    fixture.contract_client.borrow(
        &ObligationKey::new(single_pos_borrower.clone()),
        &borrow_pool,
        &debt,
        &None,
    );

    // Borrower B: same total collateral value & debt, split across two pools (N = 2).
    fixture.contract_client.add_collateral(
        &ObligationKey::new(split_pos_borrower.clone()),
        &gold_pool,
        &half_collateral,
        &None,
    );
    fixture.contract_client.add_collateral(
        &ObligationKey::new(split_pos_borrower.clone()),
        &btc_pool,
        &half_collateral,
        &None,
    );
    fixture.contract_client.borrow(
        &ObligationKey::new(split_pos_borrower.clone()),
        &borrow_pool,
        &debt,
        &None,
    );

    // Raise min_collateral_value_cents to 10 cents -> threshold 1e13.
    let update_in_queue_period = fixture.contract_client.get_global_state().update_in_queue_period;
    fixture.contract_client.queue_in_market_update(
        &DEFAULT_MAX_POSITIONS,
        &10,
        &DEFAULT_BAD_DEBT_LOCK_D,
    );
    fixture.e.ledger().with_mut(|li| li.timestamp += update_in_queue_period);
    fixture.contract_client.refresh_pool(&borrow_pool);
    fixture.contract_client.refresh_pool(&gold_pool);
    fixture.contract_client.refresh_pool(&btc_pool);
    fixture.contract_client.apply_market_update();

    // N = 1: deduction is a single threshold -> still healthy.
    let single_pos_result = fixture.contract_client.try_liquidate(
        &liquidator,
        &ObligationKey::new(single_pos_borrower.clone()),
        &borrow_pool,
        &gold_pool,
        &1,
        &0,
    );
    assert_eq!(
        single_pos_result,
        Err(Ok(MCError::ObligationIsHealthy)),
        "Single-position borrower must stay healthy: only 1 * threshold is reserved"
    );

    // N = 2: deduction is two thresholds -> now liquidatable.
    // Position is solvent, so the close factor caps repayment at 50% of debt;
    // repay a meaningful slice so shares are actually burned.
    let split_pos_result = fixture.contract_client.try_liquidate(
        &liquidator,
        &ObligationKey::new(split_pos_borrower.clone()),
        &borrow_pool,
        &gold_pool,
        &(debt / 4),
        &0,
    );
    assert!(
        split_pos_result.is_ok(),
        "Two-position borrower must be liquidatable: 2 * threshold is reserved, pushing the \
         close-LTV collateral below the debt. Got: {split_pos_result:?}"
    );
}

/// A liquidator is not seeded into a pool it borrows from: a deposit and a borrow in one pool could be
/// neither liquidated, since a same-pool liquidation is refused, nor covered as bad debt.
#[test]
fn test_a_liquidator_is_not_seeded_into_a_pool_it_borrows_from() {
    let f = TestMarketFixture::new();
    let [lp, borrower, liquidator] = [0, 1, 2].map(|i| ObligationKey::new(f.users[i].clone()));
    f.contract_client.deposit(&lp, &f.gold_pool_address, &1_000_000_000, &None);
    f.contract_client.deposit(&lp, &f.usdc_pool_address, &1_000_000_000, &None);

    // The borrower backs a GOLD loan with USDC supply shares, which a liquidation seizes as shares.
    f.contract_client.deposit(&borrower, &f.usdc_pool_address, &100_000_000, &None);
    f.contract_client.borrow(&borrower, &f.gold_pool_address, &i128::MAX, &None);
    f.contract_client.add_collateral(&liquidator, &f.btc_pool_address, &100_000_000, &None);
    f.contract_client.borrow(&liquidator, &f.usdc_pool_address, &1_000_000, &None);

    f.oracle_client.set_price_stable(&soroban_sdk::vec![
        &f.e,
        2_00000000000000,
        1_00000000000000,
        1_00000000000000
    ]);
    f.e.ledger().with_mut(|li| li.timestamp += 1);

    assert_eq!(
        f.contract_client.try_liquidate(
            &liquidator.user,
            &borrower,
            &f.gold_pool_address,
            &f.usdc_pool_address,
            &1_000_000,
            &0
        ),
        Err(Ok(MCError::BorrowPositionForAssetExists))
    );
}

// Feed order is GOLD, BTC, USDC. Prices near 1e7 make one unit of value about one token, so each rounding
// step in the seizure moves the result by a whole token. The USDC price leaves a remainder on every
// value it produces.
const GOLD_START_PRICE: i128 = 10_000_000;
const USDC_PRICE: i128 = 10_000_003;
const TOKEN_SCALE: i128 = 10_000_000;
const PRICED_COLLATERAL: i128 = 1_000_000;
const PRICED_BORROW: i128 = 650_000;

/// A borrower holding `PRICED_COLLATERAL` GOLD (plain, or as supply shares) against a `PRICED_BORROW` USDC loan,
/// with each pool's `max_liquidation_incentive_bps` set and GOLD repriced so the unweighted LTV is about
/// `ltv_bps`. Returns the fixture, the borrower, the liquidator and the GOLD price.
fn priced_position(
    ltv_bps: i128,
    gold_incentive_bps: i128,
    usdc_incentive_bps: i128,
    as_supply: bool,
) -> (TestMarketFixture<'static>, Address, Address, i128) {
    let f = TestMarketFixture::new();
    for (pool, incentive) in
        [(&f.gold_pool_address, gold_incentive_bps), (&f.usdc_pool_address, usdc_incentive_bps)]
    {
        let mut config = f.contract_client.get_pool(pool).config;
        config.health_config.max_liquidation_incentive_bps = incentive;
        f.contract_client.queue_in_pool_set(pool, &config);
    }
    let period = f.contract_client.get_global_state().update_in_queue_period;
    f.e.ledger().with_mut(|li| li.timestamp += period);
    f.contract_client.apply_pool_set(&f.gold_pool_address);
    f.contract_client.apply_pool_set(&f.usdc_pool_address);

    let set_gold_price = |gold: i128| {
        f.oracle_client.set_price_stable(&soroban_sdk::vec![
            &f.e,
            gold,
            GOLD_START_PRICE,
            USDC_PRICE
        ]);
    };
    set_gold_price(GOLD_START_PRICE);

    let (lender, borrower, liquidator) = (&f.users[0], &f.users[1], &f.users[2]);
    let borrower_key = ObligationKey::new(borrower.clone());
    f.contract_client.deposit(
        &ObligationKey::new(lender.clone()),
        &f.usdc_pool_address,
        &(10 * PRICED_COLLATERAL),
        &None,
    );
    if as_supply {
        f.contract_client.deposit(&borrower_key, &f.gold_pool_address, &PRICED_COLLATERAL, &None);
    } else {
        f.contract_client.add_collateral(
            &borrower_key,
            &f.gold_pool_address,
            &PRICED_COLLATERAL,
            &None,
        );
    }
    f.contract_client.borrow(&borrower_key, &f.usdc_pool_address, &PRICED_BORROW, &None);

    let gold_price = PRICED_BORROW * USDC_PRICE * BPS_FACTOR / (ltv_bps * PRICED_COLLATERAL);
    set_gold_price(gold_price);

    let (borrower, liquidator) = (borrower.clone(), liquidator.clone());
    (f, borrower, liquidator, gold_price)
}

fn seized_with_incentive(repay: i128, gold_price: i128, incentive_bps: i128) -> i128 {
    let repaid_value = repay * USDC_PRICE / TOKEN_SCALE;
    let collateral_amount = repaid_value * TOKEN_SCALE / gold_price;

    collateral_amount * (BPS_FACTOR + incentive_bps) / BPS_FACTOR
}

/// Where the incentive bounds the seizure, a liquidation hands over exactly
/// `floor(floor(floor(repay * p_debt / 10^7) * 10^7 / p_collateral) * (1 + min incentive))` collateral, refuses a
/// demand one unit above it, and moves supply shares (not cash) when the collateral is held as a deposit.
#[test]
fn test_a_liquidation_seizes_the_repaid_value_plus_the_smaller_incentive() {
    // (LTV, GOLD incentive, USDC incentive, repay, collateral held as supply shares)
    let cases = [
        (8_500, 500, 1_000, 123_457, false),
        (8_500, 1_000, 500, 98_765, true),
        (8_500, 300, 300, 10_007, false),
        (9_950, 1_000, 2_000, 123_457, true),
        (9_950, 2_000, 1_000, 200_003, false),
    ];
    for (ltv_bps, gold_incentive, usdc_incentive, repay, as_supply) in cases {
        let (f, borrower, liquidator, gold_price) =
            priced_position(ltv_bps, gold_incentive, usdc_incentive, as_supply);
        let borrower_key = ObligationKey::new(borrower.clone());
        let expected = seized_with_incentive(repay, gold_price, gold_incentive.min(usdc_incentive));

        let debt = get_obligation_d_tokens_as_tokens(
            &f.e,
            &f.contract_client,
            &borrower,
            &f.usdc_pool_address,
        )
        .unwrap();
        let debt_value = (debt * USDC_PRICE + TOKEN_SCALE - 1) / TOKEN_SCALE;
        let collateral_value = PRICED_COLLATERAL * gold_price / TOKEN_SCALE;
        if ltv_bps < DEFAULT_INSOLVENCY_LTV_BPS {
            // The LTV-improving cap, `0.999 * repaid value * collateral / debt`, is above the incentive bound.
            let repaid_value = repay * USDC_PRICE / TOKEN_SCALE;
            assert!(
                expected * gold_price * 1_000 * debt_value
                    < 999 * repaid_value * collateral_value * TOKEN_SCALE
            );
        }

        assert_eq!(
            f.contract_client.try_liquidate(
                &liquidator,
                &borrower_key,
                &f.usdc_pool_address,
                &f.gold_pool_address,
                &repay,
                &(expected + 1),
            ),
            Err(Ok(MCError::LiquidationExcessiveDemandedCollateral))
        );

        let gold_before = f.gold_token_client.balance(&liquidator);
        let usdc_before = f.usdc_token_client.balance(&liquidator);
        f.contract_client.liquidate(
            &liquidator,
            &borrower_key,
            &f.usdc_pool_address,
            &f.gold_pool_address,
            &repay,
            &expected,
        );

        assert_eq!(usdc_before - f.usdc_token_client.balance(&liquidator), repay);
        assert_eq!(
            get_obligation_d_tokens_as_tokens(
                &f.e,
                &f.contract_client,
                &borrower,
                &f.usdc_pool_address
            ),
            Ok(debt - repay)
        );
        let gold_received = f.gold_token_client.balance(&liquidator) - gold_before;
        if as_supply {
            // No GOLD is borrowed, so one supply share is one token.
            assert_eq!(gold_received, 0);
            assert_eq!(
                get_obligation_j_tokens(&f.contract_client, &liquidator, &f.gold_pool_address),
                Ok(expected)
            );
            assert_eq!(
                get_obligation_j_tokens(&f.contract_client, &borrower, &f.gold_pool_address),
                Ok(PRICED_COLLATERAL - expected)
            );
        } else {
            assert_eq!(gold_received, expected);
            assert_eq!(
                get_obligation_collateral(&f.contract_client, &borrower, &f.gold_pool_address),
                Ok(PRICED_COLLATERAL - expected)
            );
        }
    }
}

/// Where the LTV-improving cap bounds a solvent liquidation, the collateral value taken stays at or below
/// 99.9 % of `repaid value * collateral value / debt value` (and within 0.2 % of it), so each step lowers the LTV.
#[test]
fn test_a_solvent_liquidation_keeps_a_margin_below_the_ltv_neutral_seizure() {
    let (f, borrower, liquidator, gold_price) = priced_position(8_500, 3_000, 3_000, false);
    let borrower_key = ObligationKey::new(borrower.clone());

    for repay in [50_000_i128, 123_457, 150_001] {
        let debt = get_obligation_d_tokens_as_tokens(
            &f.e,
            &f.contract_client,
            &borrower,
            &f.usdc_pool_address,
        )
        .unwrap();
        let collateral =
            get_obligation_collateral(&f.contract_client, &borrower, &f.gold_pool_address).unwrap();
        let debt_value = (debt * USDC_PRICE + TOKEN_SCALE - 1) / TOKEN_SCALE;
        let collateral_value = collateral * gold_price / TOKEN_SCALE;
        let repaid_value = repay * USDC_PRICE / TOKEN_SCALE;
        let ltv_before =
            compute_unparameterized_ltv_bps(&f.e, &f.contract_client, &borrower).unwrap();

        let gold_before = f.gold_token_client.balance(&liquidator);
        f.contract_client.liquidate(
            &liquidator,
            &borrower_key,
            &f.usdc_pool_address,
            &f.gold_pool_address,
            &repay,
            &0,
        );
        let seized = f.gold_token_client.balance(&liquidator) - gold_before;

        // seized * p / 10^7 <= 0.999 * repaid_value * collateral_value / debt_value, cross-multiplied
        let seized_scaled = seized * gold_price * 1_000 * debt_value;
        let neutral_scaled = repaid_value * collateral_value * TOKEN_SCALE;
        assert!(seized_scaled <= 999 * neutral_scaled, "seized {seized} for {repay}");
        assert!(seized_scaled >= 998 * neutral_scaled, "the LTV cap did not bind: {seized}");
        assert!(
            compute_unparameterized_ltv_bps(&f.e, &f.contract_client, &borrower).unwrap()
                <= ltv_before
        );
    }
}

/// A partial liquidation at a debt-share rate below one burns `floor(repaid * total_d_tokens / total_borrowed)`
/// debt shares: rounding favours the remaining debt, not the borrower.
#[test]
fn test_a_partial_liquidation_burns_the_floored_debt_shares() {
    let test = LiquidationTest::risky();
    test.wait_n_years(2);
    let borrower_key = ObligationKey::new(test.borrower.clone());

    for repay in [1_001_i128, 7_777, 3_333] {
        let pool = test.fixture.contract_client.get_pool(&test.borrow_pool_address);
        assert!(pool.total_d_tokens < pool.total_borrowed, "the share rate must not be one");
        let d_before = get_obligation_d_tokens(
            &test.fixture.contract_client,
            &test.borrower,
            &test.borrow_pool_address,
        )
        .unwrap();

        test.fixture.contract_client.liquidate(
            &test.liquidator,
            &borrower_key,
            &test.borrow_pool_address,
            &test.collateral_pool_address,
            &repay,
            &0,
        );

        let d_after = get_obligation_d_tokens(
            &test.fixture.contract_client,
            &test.borrower,
            &test.borrow_pool_address,
        )
        .unwrap();
        assert_eq!(d_before - d_after, repay * pool.total_d_tokens / pool.total_borrowed);
    }
}

/// One dollar in the oracle's 14 decimals.
const USD: i128 = 1_00000000000000;

/// Under the $5 floor, `users[0]` borrows `debt` USDC from deep lenders against plain collateral at $1.
fn dust_borrower(
    f: &TestMarketFixture,
    collateral: &[(&Address, i128)],
    debt: i128,
) -> ObligationKey {
    set_5_usd_dust_threshold(f);
    lend_btc_and_usdc(f);
    let borrower = ObligationKey::new(f.users[0].clone());
    for (pool, amount) in collateral {
        f.contract_client.add_collateral(&borrower, pool, amount, &None);
    }
    f.contract_client.borrow(&borrower, &f.usdc_pool_address, &debt, &None);
    borrower
}

/// GOLD and BTC prices with USDC at $1, in the oracle's feed order. The clock stays put, so no interest accrues
/// between the borrow and the liquidation.
fn set_collateral_prices(f: &TestMarketFixture, gold: i128, btc: i128) {
    f.oracle_client.set_price_stable(&soroban_sdk::vec![&f.e, gold, btc, USD]);
}

/// The borrower's USDC debt as the next liquidation sees it.
fn usdc_debt(f: &TestMarketFixture, borrower: &ObligationKey) -> i128 {
    f.contract_client.refresh_pool(&f.usdc_pool_address);
    get_obligation_d_tokens_as_tokens(
        &f.e,
        &f.contract_client,
        &borrower.user,
        &f.usdc_pool_address,
    )
    .unwrap()
}

/// `users[2]` liquidates the borrower's USDC debt against `collateral_pool`. Returns the USDC spent and the plain
/// collateral received.
fn liquidate_usdc(
    f: &TestMarketFixture,
    borrower: &ObligationKey,
    collateral_pool: &Address,
    repay: i128,
) -> Result<(i128, i128), MCError> {
    let (liquidator, collateral) = (&f.users[2], TokenClient::new(&f.e, collateral_pool));
    let (usdc_before, collateral_before) =
        (f.usdc_token_client.balance(liquidator), collateral.balance(liquidator));
    f.contract_client
        .try_liquidate(liquidator, borrower, &f.usdc_pool_address, collateral_pool, &repay, &0)
        .map_err(Result::unwrap)?
        .unwrap();
    Ok((
        usdc_before - f.usdc_token_client.balance(liquidator),
        collateral.balance(liquidator) - collateral_before,
    ))
}

/// Collateral that `repay` USDC buys at the default 10 % incentive: `floor(floor(repay * $1 / price) * 1.1)`.
fn seized_for(repay: i128, collateral_price: i128) -> i128 {
    repay * USD / collateral_price * 11 / 10
}

/// USDC that buys a whole collateral position worth `value` at the 10 % incentive, rounded up at each step.
fn whole_position_repay(value: i128) -> i128 {
    let discounted = (10 * value + 10) / 11;
    (discounted * TOKEN_SCALE + USD - 1) / USD
}

fn value_ceil(amount: i128, price: i128) -> i128 {
    (amount * price + TOKEN_SCALE - 1) / TOKEN_SCALE
}

/// A repay that would leave both positions with dust is refused, so a stroop repay takes nothing. Repaying the
/// whole $1 debt against $4.50 of GOLD buys `debt * 1.1` of it and the borrower keeps the other $3.40.
#[test]
fn test_a_liquidation_that_would_leave_dust_is_refused() {
    let f = TestMarketFixture::new();
    let borrower = dust_borrower(&f, &[(&f.gold_pool_address, 10 * ONE)], ONE);
    set_collateral_prices(&f, gold_cents(45), USD);
    let debt = usdc_debt(&f, &borrower);

    assert_eq!(
        liquidate_usdc(&f, &borrower, &f.gold_pool_address, 10),
        Err(MCError::LiquidationLeavesDust)
    );
    let seized = seized_for(debt, gold_cents(45));
    assert_eq!(liquidate_usdc(&f, &borrower, &f.gold_pool_address, 2 * debt), Ok((debt, seized)));
    assert_eq!(
        get_obligation_collateral(&f.contract_client, &f.users[0], &f.gold_pool_address),
        Ok(10 * ONE - seized)
    );
    assert_eq!(
        get_obligation_d_tokens(&f.contract_client, &f.users[0], &f.usdc_pool_address),
        Err(MCError::BorrowPositionDoesNotExist)
    );
}

/// A position worth less than twice the floor has no close factor: half of a $3 debt against $6.20 of GOLD would
/// leave both under the floor and is refused, the whole debt is accepted and the borrower keeps $2.90.
#[test]
fn test_a_small_position_is_liquidated_past_the_close_factor() {
    let f = TestMarketFixture::new();
    let borrower = dust_borrower(&f, &[(&f.gold_pool_address, 20 * ONE)], 3 * ONE);
    set_collateral_prices(&f, gold_cents(31), USD);
    let debt = usdc_debt(&f, &borrower);

    assert_eq!(
        liquidate_usdc(&f, &borrower, &f.gold_pool_address, debt / 2),
        Err(MCError::LiquidationLeavesDust)
    );
    let seized = seized_for(debt, gold_cents(31));
    assert_eq!(liquidate_usdc(&f, &borrower, &f.gold_pool_address, debt), Ok((debt, seized)));
    assert_eq!(
        get_obligation_collateral(&f.contract_client, &f.users[0], &f.gold_pool_address),
        Ok(20 * ONE - seized)
    );
}

/// Each collateral position under the floor is sold whole or not at all. Against $4 of GOLD and $4 of BTC owing $6,
/// the GOLD goes whole for its discounted value with the excess repay refunded; the BTC then covers the remaining
/// debt and the borrower keeps $1.40 of it.
#[test]
fn test_small_collateral_positions_are_each_sold_whole_or_not_at_all() {
    let f = TestMarketFixture::new();
    let (gold, btc) = (&f.gold_pool_address, &f.btc_pool_address);
    let borrower = dust_borrower(&f, &[(gold, 12 * ONE), (btc, 12 * ONE)], 6 * ONE);
    let third = USD / 3;
    set_collateral_prices(&f, third, third);
    let debt = usdc_debt(&f, &borrower);

    assert_eq!(liquidate_usdc(&f, &borrower, gold, debt / 2), Err(MCError::LiquidationLeavesDust));
    let gold_repay = whole_position_repay(value_ceil(12 * ONE, third));
    assert_eq!(liquidate_usdc(&f, &borrower, gold, 2 * debt), Ok((gold_repay, 12 * ONE)));

    let debt_left = usdc_debt(&f, &borrower);
    assert_eq!(debt_left, debt - gold_repay);
    assert_eq!(
        liquidate_usdc(&f, &borrower, btc, debt_left / 2),
        Err(MCError::LiquidationLeavesDust)
    );
    let seized = seized_for(debt_left, third);
    assert_eq!(liquidate_usdc(&f, &borrower, btc, 2 * debt_left), Ok((debt_left, seized)));
    let obligation = f.contract_client.get_user_obligation(&borrower);
    assert_eq!((obligation.deposits.len(), obligation.borrows.len()), (1, 0));
    assert_eq!(
        get_obligation_collateral(&f.contract_client, &f.users[0], btc),
        Ok(12 * ONE - seized)
    );
}

/// An insolvent position worth less than twice the floor is sold whole for its discounted value: the liquidator
/// pays `ceil(value / 1.1)`, not its offer, and the debt left behind empty collateral goes to a cover.
fn sell_insolvent_position_whole(
    gold: i128,
    debt: i128,
    gold_price: i128,
) -> TestMarketFixture<'static> {
    let f = TestMarketFixture::new();
    let borrower = dust_borrower(&f, &[(&f.gold_pool_address, gold)], debt);
    set_collateral_prices(&f, gold_price, USD);
    let debt = usdc_debt(&f, &borrower);

    assert_eq!(
        liquidate_usdc(&f, &borrower, &f.gold_pool_address, 10),
        Err(MCError::LiquidationLeavesDust)
    );
    let repay = whole_position_repay(value_ceil(gold, gold_price));
    assert_eq!(liquidate_usdc(&f, &borrower, &f.gold_pool_address, 2 * debt), Ok((repay, gold)));
    assert_eq!(usdc_debt(&f, &borrower), debt - repay);

    f.contract_client.issue_cover_bad_debt(&borrower);
    f
}

/// $2.40 of GOLD against $3: the GOLD goes for $2.18 and the $0.82 left is written off without a lock.
#[test]
fn test_an_insolvent_dust_position_is_sold_whole_and_the_rest_written_off() {
    let f = sell_insolvent_position_whole(30 * ONE, 3 * ONE, gold_cents(8));

    assert_eq!(lock_of(&f, &f.usdc_pool_address), (0, 0));
    assert_eq!(f.contract_client.get_pool(&f.usdc_pool_address).total_borrowed, 0);
    assert_eq!(
        f.contract_client.try_get_user_obligation(&ObligationKey::new(f.users[0].clone())),
        Err(Ok(MCError::ObligationDoesNotExist))
    );
}

/// $4 of GOLD against $20: the GOLD goes for $3.64 and the $16.36 left, above the floor, goes to the fund.
#[test]
fn test_an_insolvent_position_sold_whole_leaves_its_debt_to_the_fund() {
    let f = sell_insolvent_position_whole(100 * ONE, 20 * ONE, gold_cents(4));

    assert_eq!(lock_of(&f, &f.usdc_pool_address).0, 1);
}

/// $144 of GOLD against $130 is above twice the floor in both positions, so the close factor still caps a
/// solvent liquidation at half the debt.
#[test]
fn test_a_position_worth_twice_the_floor_keeps_the_close_factor() {
    let f = TestMarketFixture::new();
    let borrower = dust_borrower(&f, &[(&f.gold_pool_address, 200 * ONE)], 130 * ONE);
    set_collateral_prices(&f, gold_cents(72), USD);
    let debt = usdc_debt(&f, &borrower);

    assert_eq!(
        liquidate_usdc(&f, &borrower, &f.gold_pool_address, debt),
        Err(MCError::InvalidLiquidationInputs)
    );
    let half = debt / 2;
    assert_eq!(
        liquidate_usdc(&f, &borrower, &f.gold_pool_address, half),
        Ok((half, seized_for(half, gold_cents(72))))
    );
}

/// A large position cannot be left with dust collateral either: against $90 of GOLD owing $95, repaying $78 would
/// leave $4.20 of GOLD and is refused; `818_181_819` USDC is the smallest repay whose `repay * 1.1` takes it all.
#[test]
fn test_a_large_position_cannot_be_left_with_dust_collateral() {
    let f = TestMarketFixture::new();
    let borrower = dust_borrower(&f, &[(&f.gold_pool_address, 200 * ONE)], 95 * ONE);
    set_collateral_prices(&f, gold_cents(45), USD);
    let whole = 818_181_819;
    assert_eq!(seized_for(whole - 1, gold_cents(45)), 200 * ONE - 2);

    for repay in [78 * ONE, whole - 1] {
        assert_eq!(
            liquidate_usdc(&f, &borrower, &f.gold_pool_address, repay),
            Err(MCError::LiquidationLeavesDust)
        );
    }
    assert_eq!(liquidate_usdc(&f, &borrower, &f.gold_pool_address, whole), Ok((whole, 200 * ONE)));
}

/// A liquidation may leave either position worth exactly the floor. With $9 of GOLD, a repay leaving $5.00 of an $8
/// debt is accepted and one stroop more is refused; against $8.80, `36_363_636` USDC leaves $5.0000004 of GOLD and
/// one stroop more would leave $4.9999999.
#[test]
fn test_a_liquidation_may_leave_exactly_the_floor() {
    let floor = 5 * ONE;

    let f = TestMarketFixture::new();
    let borrower = dust_borrower(&f, &[(&f.gold_pool_address, 100 * ONE)], 8 * ONE);
    set_collateral_prices(&f, gold_cents(9), USD);
    let repay = usdc_debt(&f, &borrower) - floor;
    assert_eq!(
        liquidate_usdc(&f, &borrower, &f.gold_pool_address, repay + 1),
        Err(MCError::LiquidationLeavesDust)
    );
    liquidate_usdc(&f, &borrower, &f.gold_pool_address, repay).unwrap();
    assert_eq!(usdc_debt(&f, &borrower), floor);

    let f = TestMarketFixture::new();
    let borrower = dust_borrower(&f, &[(&f.gold_pool_address, 100 * ONE)], 88 * ONE / 10);
    set_collateral_prices(&f, gold_cents(9), USD);
    let repay = 36_363_636;
    assert_eq!(
        liquidate_usdc(&f, &borrower, &f.gold_pool_address, repay + 1),
        Err(MCError::LiquidationLeavesDust)
    );
    liquidate_usdc(&f, &borrower, &f.gold_pool_address, repay).unwrap();
    let gold_left =
        get_obligation_collateral(&f.contract_client, &f.users[0], &f.gold_pool_address).unwrap();
    assert_eq!(gold_left, 100 * ONE - seized_for(repay, gold_cents(9)));
    assert_eq!(gold_left * gold_cents(9) / TOKEN_SCALE, 500_000_004_000_000);
}

/// The close factor lifts only below twice the floor. Beside $100 of BTC, $10.00 of GOLD keeps it and one price
/// unit less sells the GOLD whole; a $10.00 debt against $11.50 of GOLD keeps it and a $9.999999 one clears.
#[test]
fn test_the_close_factor_lifts_just_under_twice_the_floor() {
    let f = TestMarketFixture::new();
    let (gold, btc) = (&f.gold_pool_address, &f.btc_pool_address);
    let borrower = dust_borrower(&f, &[(gold, 10 * ONE), (btc, 1_000 * ONE)], 95 * ONE);
    set_collateral_prices(&f, USD, USD / 10);
    let debt = usdc_debt(&f, &borrower);
    assert_eq!(
        liquidate_usdc(&f, &borrower, gold, 2 * debt),
        Err(MCError::InvalidLiquidationInputs)
    );
    set_collateral_prices(&f, USD - 1, USD / 10);
    let repay = whole_position_repay(value_ceil(10 * ONE, USD - 1));
    assert_eq!(liquidate_usdc(&f, &borrower, gold, 2 * debt), Ok((repay, 10 * ONE)));

    let gold_price = USD * 115 / 1_000;
    for (borrowed, outcome) in [(10 * ONE, false), (10 * ONE - 10, true)] {
        let f = TestMarketFixture::new();
        let borrower = dust_borrower(&f, &[(&f.gold_pool_address, 100 * ONE)], borrowed);
        set_collateral_prices(&f, gold_price, USD);
        let debt = usdc_debt(&f, &borrower);
        assert_eq!(debt, borrowed);

        let expected = if outcome {
            Ok((debt, seized_for(debt, gold_price)))
        } else {
            Err(MCError::InvalidLiquidationInputs)
        };
        assert_eq!(liquidate_usdc(&f, &borrower, &f.gold_pool_address, 2 * debt), expected);
    }
}

/// Supply-share collateral follows the same rule: a stroop repay is refused and the whole debt moves `debt * 1.1`
/// of shares. An insolvent position held half in plain GOLD, half in shares, goes whole: every token and share.
#[test]
fn test_supply_share_collateral_follows_the_dust_rule() {
    let f = TestMarketFixture::new();
    set_5_usd_dust_threshold(&f);
    lend_btc_and_usdc(&f);
    let borrower = ObligationKey::new(f.users[0].clone());
    f.contract_client.deposit(&borrower, &f.gold_pool_address, &(10 * ONE), &None);
    f.contract_client.borrow(&borrower, &f.usdc_pool_address, &ONE, &None);
    set_collateral_prices(&f, gold_cents(45), USD);
    let debt = usdc_debt(&f, &borrower);

    assert_eq!(
        liquidate_usdc(&f, &borrower, &f.gold_pool_address, 10),
        Err(MCError::LiquidationLeavesDust)
    );
    assert_eq!(liquidate_usdc(&f, &borrower, &f.gold_pool_address, 2 * debt), Ok((debt, 0)));
    // No GOLD is borrowed, so one supply share is one token.
    let seized = seized_for(debt, gold_cents(45));
    assert_eq!(
        get_obligation_j_tokens(&f.contract_client, &f.users[2], &f.gold_pool_address),
        Ok(seized)
    );
    assert_eq!(
        get_obligation_j_tokens(&f.contract_client, &f.users[0], &f.gold_pool_address),
        Ok(10 * ONE - seized)
    );

    let f = TestMarketFixture::new();
    let borrower = dust_borrower(&f, &[(&f.gold_pool_address, 15 * ONE)], 3 * ONE);
    f.contract_client.deposit(&borrower, &f.gold_pool_address, &(15 * ONE), &None);
    set_collateral_prices(&f, gold_cents(8), USD);
    let debt = usdc_debt(&f, &borrower);
    let repay = whole_position_repay(value_ceil(30 * ONE, gold_cents(8)));

    assert_eq!(
        liquidate_usdc(&f, &borrower, &f.gold_pool_address, 2 * debt),
        Ok((repay, 15 * ONE))
    );
    assert_eq!(
        get_obligation_j_tokens(&f.contract_client, &f.users[2], &f.gold_pool_address),
        Ok(15 * ONE)
    );
    assert_eq!(
        get_obligation_collateral(&f.contract_client, &f.users[0], &f.gold_pool_address),
        Err(MCError::DepositPositionDoesNotExist)
    );
}

/// A batched liquidation follows the same rule as a direct one.
#[test]
fn test_a_batched_liquidation_follows_the_dust_rule() {
    let f = TestMarketFixture::new();
    let borrower = dust_borrower(&f, &[(&f.gold_pool_address, 10 * ONE)], ONE);
    set_collateral_prices(&f, gold_cents(45), USD);
    let debt = usdc_debt(&f, &borrower);
    let liquidator = ObligationKey::new(f.users[2].clone());
    let batch = |repay_amount| {
        soroban_sdk::vec![
            &f.e,
            Request::Liquidate(LiquidateRequest {
                borrower_obligation_key: borrower.clone(),
                borrow_pool_address: f.usdc_pool_address.clone(),
                collateral_pool_address: f.gold_pool_address.clone(),
                repay_amount,
                min_demanded_collateral_amount: 0,
            }),
        ]
    };

    assert_eq!(
        f.contract_client.try_submit_requests_batch(&liquidator, &batch(10), &None),
        Err(Ok(MCError::LiquidationLeavesDust))
    );
    let (usdc_before, gold_before) =
        (f.usdc_token_client.balance(&f.users[2]), f.gold_token_client.balance(&f.users[2]));
    f.contract_client.submit_requests_batch(&liquidator, &batch(2 * debt), &None);
    assert_eq!(usdc_before - f.usdc_token_client.balance(&f.users[2]), debt);
    assert_eq!(
        f.gold_token_client.balance(&f.users[2]) - gold_before,
        seized_for(debt, gold_cents(45))
    );
}

/// With a zero floor nothing is dust. Selling a whole supply-share position at a share rate below one leaves the
/// borrower the remainder that `ceil(floor(shares * rate) / rate)` does not reach.
#[test]
fn test_at_a_zero_floor_a_sold_out_share_position_keeps_its_remainder() {
    let f = TestMarketFixture::new();
    let c = &f.contract_client;
    let [borrower, usdc_lender, liquidator, gold_lender, gold_borrower] =
        [0, 1, 2, 3, 4].map(|i| ObligationKey::new(f.users[i].clone()));

    // An uncovered GOLD loss leaves each GOLD share worth about a fifth of a token.
    c.deposit(&gold_lender, &f.gold_pool_address, &(10 * ONE), &None);
    c.add_collateral(&gold_borrower, &f.btc_pool_address, &(100 * ONE), &None);
    c.borrow(&gold_borrower, &f.gold_pool_address, &(8 * ONE), &None);
    set_collateral_prices(&f, USD, 1);
    c.liquidate(
        &liquidator.user,
        &gold_borrower,
        &f.gold_pool_address,
        &f.btc_pool_address,
        &10,
        &0,
    );
    c.issue_cover_bad_debt(&gold_borrower);
    f.controlled_insurance_fund_client.mark_ready(&0, &0);
    c.claim_cover_bad_debt_results(&gold_borrower);

    c.deposit(&usdc_lender, &f.usdc_pool_address, &(100 * ONE), &None);
    c.deposit(&borrower, &f.gold_pool_address, &(3 * ONE), &None);
    c.borrow(&borrower, &f.usdc_pool_address, &ONE, &None);
    set_collateral_prices(&f, gold_cents(1), USD);

    let pool = c.get_pool(&f.gold_pool_address);
    let shares = get_obligation_j_tokens(c, &f.users[0], &f.gold_pool_address).unwrap();
    let tokens = pool.compute_tokens_from_j_tokens_floor(&f.e, shares).unwrap();
    let seized = pool.compute_j_tokens_from_tokens_ceil(tokens).unwrap().min(shares);
    assert!(seized < shares, "the share rate leaves no remainder: {shares} shares");

    c.liquidate(&liquidator.user, &borrower, &f.usdc_pool_address, &f.gold_pool_address, &ONE, &0);
    assert_eq!(get_obligation_j_tokens(c, &f.users[2], &f.gold_pool_address), Ok(seized));
    assert_eq!(get_obligation_j_tokens(c, &f.users[0], &f.gold_pool_address), Ok(shares - seized));
}
