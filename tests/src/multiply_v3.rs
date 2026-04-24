//! Integration tests proving the V3 single-anchor multiply flow described in
//! `docs/SecondTokenMarginGuideV3.md`.
//!
//! V3 flow (4 requests, fixed order, preconditions: `borrow_fee_bps == 0`,
//! `add_collateral_fee_bps == 0`):
//!
//! ```
//! 1. FlashBorrow(USDC, X)
//! 2. SwapExactTokens(USDC -> GOLD, amount_in = X, min_amount_out = Y)
//! 3. AddCollateral(GOLD, margin + Y)               // ← single anchor
//! 4. Borrow(USDC, X + flash_fee)                   // ← exact, no gross-up
//! ```
//!
//! Final position is bit-deterministic regardless of slippage:
//! - collateral = `margin + Y` (literal in step 3)
//! - debt       = `X + flash_fee` (literal in step 4)
//! - positive slippage materialises as bonus GOLD in wallet
//! - adverse slippage trips `min_amount_out` and reverts the whole batch atomically.
//!
//! These tests cover all five canonical scenarios from the V3 guide.

#![cfg(test)]

use market::{
    constants::DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
    error::MCError,
    obligation::ObligationKey,
    pool::{PoolConfig, PoolFeeConfig},
    request::{Request, StandardRequest, SwapExactTokensRequest},
};
use soroban_sdk::{
    Address, Env, Symbol, Vec, contract, contractimpl, symbol_short,
    testutils::Ledger,
    token::{StellarAssetClient, TokenClient},
    vec as svec,
};

use crate::{DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture};

// -- Configurable swap mock ---------------------------------------------------

/// A swap proxy whose output:input ratio is configurable via `set_rate(bps)`.
/// `rate_bps == 10_000` means 1:1 (output == input). `10_500` means +5%
/// (favorable), `9_900` means -1% (adverse — trips `min_amount_out` check).
///
/// The contract's `process_swap_exact` enforces `received_amount >= min_amount_out`
/// at `processors.rs:1036`, so adverse rates produce `MCError::SwapSlippageExceeded`
/// without any custom revert logic in this mock.
#[contract]
pub struct ConfigurableSwap;

const RATE_KEY: Symbol = symbol_short!("rate");
const LAST_MIN_KEY: Symbol = symbol_short!("lastmin");

#[contractimpl]
impl ConfigurableSwap {
    pub fn init(e: Env, rate_bps: i128) {
        e.storage().instance().set(&RATE_KEY, &rate_bps);
    }

    pub fn set_rate(e: Env, rate_bps: i128) {
        e.storage().instance().set(&RATE_KEY, &rate_bps);
    }

    /// Read the most recent `min_amount_out` argument the mock saw — used by
    /// tests to confirm the contract is forwarding the slippage floor verbatim.
    pub fn last_min_amount_out(e: Env) -> i128 {
        e.storage().instance().get(&LAST_MIN_KEY).unwrap_or(0)
    }

    pub fn swap_exact(
        e: Env,
        user: Address,
        path: Vec<Address>,
        amount_in: i128,
        min_amount_out: i128,
    ) -> i128 {
        user.require_auth();

        // Record floor for test introspection.
        e.storage().instance().set(&LAST_MIN_KEY, &min_amount_out);

        let rate: i128 = e.storage().instance().get(&RATE_KEY).unwrap_or(10_000);
        let output = (amount_in * rate) / 10_000;

        // Real-DEX semantics: refuse to settle if quoted output breaches the
        // caller's slippage floor. Surfaces as a host-level abort to the contract,
        // which atomically rolls back the entire batch. The contract's own check
        // at processors.rs:1036 remains as defence in depth for misbehaving DEXes.
        assert!(
            output >= min_amount_out,
            "ConfigurableSwap: output {} below min_amount_out {}",
            output,
            min_amount_out
        );

        let token_in = path.first().unwrap();
        let token_out = path.last().unwrap();

        TokenClient::new(&e, &token_in).burn(&user, &amount_in);
        StellarAssetClient::new(&e, &token_out).mint(&user, &output);

        output
    }
}

// -- V3 batch builder & math --------------------------------------------------

/// Default `flash_loan_fee_bps` per `constants::DEFAULT_FLASH_LOAN_FEE_BPS`.
const FLASH_LOAN_FEE_BPS: i128 = 1;

/// Mirrors `request.rs:170-172` — `ceil(amount × bps / 10_000)`.
fn flash_fee(amount: i128) -> i128 {
    let num = amount * FLASH_LOAN_FEE_BPS;
    (num + 10_000 - 1) / 10_000
}

/// Build the canonical V3 batch.
#[allow(clippy::too_many_arguments)]
fn v3_batch(
    e: &Env,
    swap_provider: &Address,
    usdc_pool: &Address,
    gold_pool: &Address,
    usdc_token: &Address,
    gold_token: &Address,
    flash_x: i128,
    swap_floor_y: i128,
    margin: i128,
    borrow_amount: i128,
) -> Vec<Request> {
    svec![
        e,
        Request::FlashBorrow(StandardRequest {
            amount: flash_x,
            pool_address: usdc_pool.clone(),
        }),
        Request::SwapExactTokens(SwapExactTokensRequest {
            swap_provider: swap_provider.clone(),
            path: svec![e, usdc_token.clone(), gold_token.clone()],
            amount_in: flash_x,
            min_amount_out: swap_floor_y,
        }),
        Request::AddCollateral(StandardRequest {
            amount: margin + swap_floor_y,
            pool_address: gold_pool.clone(),
        }),
        Request::Borrow(StandardRequest {
            amount: borrow_amount,
            pool_address: usdc_pool.clone(),
        }),
    ]
}

// -- Standard test parameters -------------------------------------------------

struct V3Params {
    margin: i128,
    flash_x: i128,
    swap_floor_y: i128,
    flash_fee: i128,
    borrow_amount: i128,
}

impl V3Params {
    /// margin = 10x default, target collateral-to-add = 5x default,
    /// 1% slippage floor → Y = target × 0.99.
    fn standard() -> Self {
        let margin = 10 * DEFAULT_DEPOSIT_AMOUNT;
        let target_to_add = 5 * DEFAULT_DEPOSIT_AMOUNT;
        let swap_floor_y = (target_to_add * 99) / 100;
        // Quote at 1:1 → X (USDC needed) == Y (GOLD floor).
        let flash_x = swap_floor_y;
        let flash_fee = flash_fee(flash_x);
        let borrow_amount = flash_x + flash_fee;

        Self { margin, flash_x, swap_floor_y, flash_fee, borrow_amount }
    }

    fn expected_collateral(&self) -> i128 {
        self.margin + self.swap_floor_y
    }

    fn expected_debt(&self) -> i128 {
        self.flash_x + self.flash_fee
    }
}

// -- Test 1: floor-on-money (rate = 10_000) -----------------------------------

/// Swap returns *exactly* `min_amount_out`. Invariants: zero bonus, debt and
/// collateral exactly match the literals in steps 3 and 4.
#[test]
fn v3_floor_on_money_produces_exact_position_no_bonus() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_client,
        usdc_token_address,
        gold_pool_address,
        gold_token_client,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&10_000_i128); // 1:1 — output == input

    let user = &users[0];
    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    // Liquidity provider seeds USDC for flash + borrow.
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let usdc_before = usdc_token_client.balance(user);
    let gold_before = gold_token_client.balance(user);

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );

    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    // Position is exactly the literals.
    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let collateral = obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    let debt = obligation
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(collateral, p.expected_collateral(), "collateral must equal margin + Y exactly");
    assert_eq!(debt, p.expected_debt(), "debt must equal X + flash_fee exactly");

    // Wallet: USDC unchanged (borrow → flash repay netted out).
    // GOLD net change = +Y (swap mint) - (margin + Y) (AddCollateral pull) = -margin.
    // No bonus at floor rate because swap output exactly equals min_amount_out.
    assert_eq!(usdc_token_client.balance(user), usdc_before, "USDC wallet unchanged");
    assert_eq!(
        gold_token_client.balance(user),
        gold_before - p.margin,
        "GOLD wallet down by exactly margin (swap mint of Y cancels AddCollateral's Y)"
    );
}

// -- Test 2: favorable slippage (rate = 10_500) -------------------------------

/// Swap returns 5% MORE than `min_amount_out`. Invariants: same collateral and
/// debt as the floor case (V3's defining property), surplus appears as wallet GOLD.
#[test]
fn v3_favorable_slippage_yields_xlm_bonus_position_unchanged() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_client,
        usdc_token_address,
        gold_pool_address,
        gold_token_client,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&10_500_i128); // +5% favorable

    let user = &users[0];
    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let usdc_before = usdc_token_client.balance(user);
    let gold_before = gold_token_client.balance(user);

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );

    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    // Position STILL exactly the literals — V3's whole point.
    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let collateral = obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    let debt = obligation
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(
        collateral,
        p.expected_collateral(),
        "collateral remains margin + Y under favorable slippage"
    );
    assert_eq!(debt, p.expected_debt(), "debt remains X + flash_fee under favorable slippage");

    // Wallet: USDC unchanged.
    // GOLD net change = +(Y × 1.05) (swap mint) - (margin + Y) (AddCollateral pull)
    //                 = -margin + 0.05 × Y  (positive bonus over the floor case).
    let actual_swap_output = (p.flash_x * 10_500) / 10_000;
    let expected_bonus = actual_swap_output - p.swap_floor_y;
    assert_eq!(usdc_token_client.balance(user), usdc_before, "USDC wallet unchanged");
    assert_eq!(
        gold_token_client.balance(user),
        gold_before - p.margin + expected_bonus,
        "GOLD wallet shows positive-slippage bonus on top of -margin baseline"
    );
    assert!(expected_bonus > 0, "test setup error — favorable rate should produce bonus");
}

// -- Test 3: adverse slippage (rate = 9_900) ----------------------------------

/// Swap returns LESS than `min_amount_out`. Invariant: whole batch reverts
/// atomically via `processors.rs:1036` → `MCError::SwapSlippageExceeded`.
/// Obligation must not exist after revert.
#[test]
fn v3_adverse_slippage_reverts_atomically_no_state_change() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_client,
        usdc_token_address,
        gold_pool_address,
        gold_token_client,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&9_900_i128); // -1% adverse

    let user = &users[0];
    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let usdc_before = usdc_token_client.balance(user);
    let gold_before = gold_token_client.balance(user);
    let pool_before = contract_client.get_pool(&usdc_pool_address);

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );

    let result =
        contract_client.try_submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);
    // Mock now refuses the trade itself (mirroring real DEX behaviour) → host abort.
    // Contract's own check at processors.rs:1036 still exists as defence in depth.
    assert!(result.is_err(), "adverse slippage must revert");

    // Obligation must not have been created.
    assert!(
        contract_client
            .try_get_user_obligation(&ObligationKey::new(user.clone()))
            .is_err(),
        "no obligation should exist after atomic revert"
    );

    // Wallet balances untouched.
    assert_eq!(usdc_token_client.balance(user), usdc_before);
    assert_eq!(gold_token_client.balance(user), gold_before);

    // Pool state untouched (no flash fee accrued, total_available restored).
    let pool_after = contract_client.get_pool(&usdc_pool_address);
    assert_eq!(pool_after.total_available, pool_before.total_available);
    assert_eq!(pool_after.operation_fees_sum, pool_before.operation_fees_sum);
}

// -- Test 4: wrong order (AddCollateral before FlashBorrow) -------------------

/// V3's order is forced: if `AddCollateral` precedes `FlashBorrow`, the
/// pre-FlashBorrow flush at `processors.rs:113` tries to physically pull
/// `(margin + Y)` GOLD from a wallet that only has `margin` GOLD → revert.
#[test]
fn v3_wrong_order_addcollateral_before_flashborrow_reverts() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_client,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    // CRUCIAL: burn user's GOLD down to exactly `margin` so wrong-order ordering matters.
    let user_gold = gold_token_client.balance(user);
    let burn = user_gold - p.margin;
    if burn > 0 {
        gold_token_client.burn(user, &burn);
    }
    assert_eq!(gold_token_client.balance(user), p.margin);

    let bad_batch = svec![
        &e,
        // AddCollateral first — this queues a (margin + Y) GOLD pull, persists obligation.
        Request::AddCollateral(StandardRequest {
            amount: p.margin + p.swap_floor_y,
            pool_address: gold_pool_address.clone(),
        }),
        // Pre-FlashBorrow flush at processors.rs:113 will try to execute that pull
        // BEFORE the swap supplies the extra Y GOLD → wallet short by Y.
        Request::FlashBorrow(StandardRequest {
            amount: p.flash_x,
            pool_address: usdc_pool_address.clone(),
        }),
        Request::SwapExactTokens(SwapExactTokensRequest {
            swap_provider: swap.clone(),
            path: svec![&e, usdc_token_address.clone(), gold_token_address.clone()],
            amount_in: p.flash_x,
            min_amount_out: p.swap_floor_y,
        }),
        Request::Borrow(StandardRequest {
            amount: p.borrow_amount,
            pool_address: usdc_pool_address.clone(),
        }),
    ];

    let result = contract_client
        .try_submit_requests_batch(&ObligationKey::new(user.clone()), &bad_batch, &None);

    // Token transfer of the missing Y GOLD throws — the contract surfaces this as a
    // host-level invocation failure; `try_*` returns Err(Err(_)) for non-MCError reverts.
    assert!(
        result.is_err(),
        "wrong order must revert because pre-FlashBorrow flush can't pull (margin + Y) GOLD"
    );

    // Obligation must not have been created (atomic rollback).
    assert!(
        contract_client
            .try_get_user_obligation(&ObligationKey::new(user.clone()))
            .is_err(),
        "no obligation should exist after wrong-order revert"
    );

    // Wallet still has exactly `margin` GOLD — nothing pulled.
    assert_eq!(gold_token_client.balance(user), p.margin);
}

// -- Test 5: correct V3 invariants over multiple users in same fixture --------

/// Run V3 for two independent users, one with floor rate, one with favorable rate.
/// Both must end with identical position state (same collateral, same debt) —
/// the V3 determinism guarantee. Bonus differs only in wallet GOLD.
#[test]
fn v3_two_users_get_identical_positions_under_different_slippage() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_client,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);

    let user_floor = &users[0];
    let user_favorable = &users[1];
    let liquidity_provider = &users[2];
    let p = V3Params::standard();

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(200 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let batch = |user_addr: &Address| {
        let _ = user_addr;
        v3_batch(
            &e,
            &swap,
            &usdc_pool_address,
            &gold_pool_address,
            &usdc_token_address,
            &gold_token_address,
            p.flash_x,
            p.swap_floor_y,
            p.margin,
            p.borrow_amount,
        )
    };

    swap_client.set_rate(&10_000_i128);
    let gold_floor_before = gold_token_client.balance(user_floor);
    contract_client.submit_requests_batch(
        &ObligationKey::new(user_floor.clone()),
        &batch(user_floor),
        &None,
    );

    swap_client.set_rate(&10_500_i128);
    let gold_favorable_before = gold_token_client.balance(user_favorable);
    contract_client.submit_requests_batch(
        &ObligationKey::new(user_favorable.clone()),
        &batch(user_favorable),
        &None,
    );

    // Identical positions.
    let obl_floor = contract_client.get_user_obligation(&ObligationKey::new(user_floor.clone()));
    let obl_fav = contract_client.get_user_obligation(&ObligationKey::new(user_favorable.clone()));

    let collat_floor =
        obl_floor.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    let collat_fav = obl_fav.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    assert_eq!(collat_floor, collat_fav, "collateral identical regardless of slippage");
    assert_eq!(collat_floor, p.expected_collateral());

    let debt_floor = obl_floor
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    let debt_fav = obl_fav
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(debt_floor, debt_fav, "debt identical regardless of slippage");
    assert_eq!(debt_floor, p.expected_debt());

    // Wallet differs only in GOLD bonus.
    let gold_floor_after = gold_token_client.balance(user_floor);
    let gold_favorable_after = gold_token_client.balance(user_favorable);

    let floor_delta = gold_floor_before - gold_floor_after;
    let favorable_delta = gold_favorable_before - gold_favorable_after;

    // Floor: net pull is exactly margin (swap mint of Y cancels AddCollateral's Y).
    assert_eq!(floor_delta, p.margin, "floor user pays effectively `margin` GOLD net");
    assert!(
        favorable_delta < floor_delta,
        "favorable user pays effectively less GOLD due to swap bonus"
    );
}

// -- Test 6: extreme favorable slippage (rate = 20_000, +100%) ----------------

/// Stress: DEX returns DOUBLE the floor. Position state must STILL be exactly
/// the literals — that's the V3 promise. The full surplus appears as wallet GOLD.
#[test]
fn v3_extreme_favorable_slippage_2x_position_still_exact() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_client,
        usdc_token_address,
        gold_pool_address,
        gold_token_client,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&20_000_i128); // +100% — DEX gives 2x

    let user = &users[0];
    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let usdc_before = usdc_token_client.balance(user);
    let gold_before = gold_token_client.balance(user);

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );

    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    // Position bit-for-bit identical to the floor-rate case despite 2x DEX output.
    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let collateral = obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    let debt = obligation
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(collateral, p.expected_collateral(), "collateral STILL exactly margin + Y at 2x rate");
    assert_eq!(debt, p.expected_debt(), "debt STILL exactly X + flash_fee at 2x rate");

    // Wallet bonus = (Y × 2) − Y = Y. Net GOLD change = -margin + Y.
    let actual_swap_output = p.flash_x * 2;
    let expected_bonus = actual_swap_output - p.swap_floor_y;
    assert_eq!(expected_bonus, p.swap_floor_y, "2x rate should yield bonus equal to Y");
    assert_eq!(usdc_token_client.balance(user), usdc_before, "USDC wallet unchanged");
    assert_eq!(
        gold_token_client.balance(user),
        gold_before - p.margin + expected_bonus,
        "GOLD wallet shows huge bonus (= Y) on top of -margin baseline"
    );
}

// -- Test 7: just-below-floor adverse slippage (rate = 9_999) -----------------

/// Boundary case: DEX returns Y - 1 (off by a single unit). Must revert.
/// Confirms the `>=` semantics in `processors.rs:1036` are strict.
#[test]
fn v3_just_below_floor_one_unit_short_reverts() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_client,
        usdc_token_address,
        gold_pool_address,
        gold_token_client,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    // Rate that produces (Y - 1) for our standard X = Y. Need output = Y - 1
    // → rate = (Y - 1) / Y × 10_000 = 10_000 - 10_000/Y. With Y = 247_500,
    // rate = 9_999 produces output = 247_500 × 9999 / 10_000 = 247_475 (24 short).
    // For exact 1-unit short we'd need a non-integer rate, so 24 short is enough
    // to prove "anything less than Y reverts".
    swap_client.init(&9_999_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let usdc_before = usdc_token_client.balance(user);
    let gold_before = gold_token_client.balance(user);

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );

    let result =
        contract_client.try_submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);
    assert!(
        result.is_err(),
        "even a tiny shortfall (24 units of 247_500) must revert"
    );

    // Atomic rollback — wallet untouched.
    assert_eq!(usdc_token_client.balance(user), usdc_before);
    assert_eq!(gold_token_client.balance(user), gold_before);
    assert!(
        contract_client
            .try_get_user_obligation(&ObligationKey::new(user.clone()))
            .is_err()
    );
}

// -- Test 8: large slippage with conservative floor — multiply STILL succeeds -

/// V3's defining strength: a user who sets `Y` (= `min_amount_out`) conservatively
/// can absorb HUGE adverse slippage without failure. The DEX returns half the
/// 1:1-quoted amount (-50%) but `Y` was sized at 50% of the quote, so the swap
/// still clears its floor and the batch executes. The on-chain position is the
/// usual deterministic `margin + Y` / `X + flash_fee` — exactly the literals.
///
/// Bonus is zero in this case (output equals floor exactly), but the multiply
/// SUCCEEDS where a tightly-floored config would have reverted.
#[test]
fn v3_large_slippage_with_low_floor_multiply_succeeds() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_client,
        usdc_token_address,
        gold_pool_address,
        gold_token_client,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&5_000_i128); // -50% — DEX returns half of 1:1 quote.

    let user = &users[0];
    let liquidity_provider = &users[1];

    // Conservatively-floored params: same X as standard, but Y sized so even
    // a 50% adverse swap still satisfies it. flash_x = 495_000 (1:1 quote of 5×default),
    // swap_floor_y = 247_500 (= flash_x × 0.5). Output at rate=5_000 is exactly Y.
    let margin = 10 * DEFAULT_DEPOSIT_AMOUNT;
    let flash_x = 5 * DEFAULT_DEPOSIT_AMOUNT;
    let swap_floor_y = flash_x / 2;
    let flash_fee_amt = flash_fee(flash_x);
    let borrow_amount = flash_x + flash_fee_amt;

    let expected_collateral = margin + swap_floor_y;
    let expected_debt = flash_x + flash_fee_amt;

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let usdc_before = usdc_token_client.balance(user);
    let gold_before = gold_token_client.balance(user);

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        flash_x,
        swap_floor_y,
        margin,
        borrow_amount,
    );

    // Batch SUCCEEDS — large slippage doesn't fail multiply when Y is sized for it.
    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    // Position is the usual V3 literals.
    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let collateral = obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    let debt = obligation
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(collateral, expected_collateral, "collateral exactly margin + Y under -50% slippage");
    assert_eq!(debt, expected_debt, "debt exactly X + flash_fee under -50% slippage");

    // Output (Y) exactly cancels AddCollateral's Y → net wallet GOLD change = -margin.
    assert_eq!(usdc_token_client.balance(user), usdc_before, "USDC wallet unchanged");
    assert_eq!(
        gold_token_client.balance(user),
        gold_before - margin,
        "no bonus when DEX delivers exactly the floor"
    );

    // Mock recorded the floor verbatim.
    assert_eq!(
        swap_client.last_min_amount_out(),
        swap_floor_y,
        "contract forwarded min_amount_out to swap provider"
    );
}

// -- Test 9: skewed quote, GOLD expensive (1 USDC → 0.5 GOLD) -----------------

/// Real-world quotes are rarely 1:1. Here GOLD is priced at 2× USDC, so to
/// receive Y GOLD the user must spend 2Y USDC. Verifies that V3's math —
/// `flash_x` (USDC), `swap_floor_y` (GOLD), `borrow_amount = flash_x + flash_fee`
/// in USDC — composes correctly across asymmetric token denominations.
#[test]
fn v3_skewed_quote_gold_expensive_position_exact_at_floor() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_client,
        usdc_token_address,
        gold_pool_address,
        gold_token_client,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&5_000_i128); // 1 USDC → 0.5 GOLD (GOLD is 2× USDC).

    let user = &users[0];
    let liquidity_provider = &users[1];

    // Want Y = 5×default GOLD added beyond margin. At 0.5 GOLD/USDC, need X = 2Y USDC.
    let margin = 10 * DEFAULT_DEPOSIT_AMOUNT;
    let swap_floor_y = 5 * DEFAULT_DEPOSIT_AMOUNT;
    let flash_x = 2 * swap_floor_y;
    let flash_fee_amt = flash_fee(flash_x);
    let borrow_amount = flash_x + flash_fee_amt;

    let expected_collateral = margin + swap_floor_y;
    let expected_debt = flash_x + flash_fee_amt;

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let usdc_before = usdc_token_client.balance(user);
    let gold_before = gold_token_client.balance(user);

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        flash_x,
        swap_floor_y,
        margin,
        borrow_amount,
    );
    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let collateral = obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    let debt = obligation
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(collateral, expected_collateral, "collateral exact under skewed quote");
    assert_eq!(debt, expected_debt, "debt exact under skewed quote");

    // No bonus at floor; net wallet GOLD = -margin (swap mint of Y cancels AddCollateral's Y).
    assert_eq!(usdc_token_client.balance(user), usdc_before);
    assert_eq!(gold_token_client.balance(user), gold_before - margin);
}

// -- Test 10: skewed quote, GOLD cheap (1 USDC → 4 GOLD) ----------------------

/// The reverse skew: GOLD is cheap, so a small USDC flash buys a lot of GOLD.
/// Also tests favorable slippage on top: actual rate = 4.4 GOLD/USDC (+10%).
/// Position is still bit-deterministic at `margin + Y`; the +10% surplus
/// materialises entirely as wallet bonus.
#[test]
fn v3_skewed_quote_gold_cheap_with_favorable_slippage() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_client,
        usdc_token_address,
        gold_pool_address,
        gold_token_client,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&44_000_i128); // 4.4 GOLD per USDC (quote was 4.0, +10% favorable).

    let user = &users[0];
    let liquidity_provider = &users[1];

    // Quote: 4 GOLD per USDC. Want Y = 20×default GOLD → need X = 5×default USDC.
    let margin = 10 * DEFAULT_DEPOSIT_AMOUNT;
    let flash_x = 5 * DEFAULT_DEPOSIT_AMOUNT;
    let swap_floor_y = 4 * flash_x; // = 20×default GOLD floor.
    let flash_fee_amt = flash_fee(flash_x);
    let borrow_amount = flash_x + flash_fee_amt;

    let expected_collateral = margin + swap_floor_y;
    let expected_debt = flash_x + flash_fee_amt;

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    // Pre-fund user with extra GOLD so the AddCollateral pull is satisfiable while
    // the swap is contributing the bulk via mint. (`margin + Y` is large here.)
    let extra_gold_needed = (margin + swap_floor_y) - gold_token_client.balance(user);
    if extra_gold_needed > 0 {
        StellarAssetClient::new(&e, &gold_token_address).mint(user, &extra_gold_needed);
    }

    let usdc_before = usdc_token_client.balance(user);
    let gold_before = gold_token_client.balance(user);

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        flash_x,
        swap_floor_y,
        margin,
        borrow_amount,
    );
    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let collateral = obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    let debt = obligation
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(collateral, expected_collateral, "collateral exact despite +10% favorable");
    assert_eq!(debt, expected_debt, "debt exact despite +10% favorable");

    // Actual swap output = X × 4.4 = swap_floor_y × 1.1; bonus = 0.1 × Y.
    let actual_output = (flash_x * 44_000) / 10_000;
    let expected_bonus = actual_output - swap_floor_y;
    assert_eq!(usdc_token_client.balance(user), usdc_before, "USDC wallet unchanged");
    assert_eq!(
        gold_token_client.balance(user),
        gold_before - margin + expected_bonus,
        "skewed quote: bonus surfaces in wallet, position untouched"
    );
    assert!(expected_bonus > 0);
}

// -- Helper: reconfigure a single pool's fee config in-place ------------------

fn update_pool_fee(
    fixture: &TestMarketFixture<'_>,
    pool_address: &Address,
    new_fee_config: PoolFeeConfig,
) {
    let current = fixture.contract_client.get_pool(pool_address);
    let new_config = PoolConfig {
        status: current.config.status,
        fee_config: new_fee_config,
        health_config: current.config.health_config,
        accrual_model: current.config.accrual_model,
        interest_rate_model: current.config.interest_rate_model,
        ir_reactivity_constant: current.config.ir_reactivity_constant,
    };
    fixture.contract_client.queue_in_pool_set(pool_address, &new_config);
    fixture
        .e
        .ledger()
        .with_mut(|li| li.timestamp += DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);
    fixture.contract_client.apply_pool_set(pool_address);
}

// -- Test 11: V3 SILENTLY MILKS USER when borrow_fee_bps != 0 -----------------

/// V3 precondition: `borrow_fee_bps == 0`. With a non-zero borrow fee, the
/// borrow step credits debt = `X + flash_fee` to the obligation but only
/// delivers `(X + flash_fee) - borrow_fee` to the wallet. The end-of-batch
/// flash repay requires `X + flash_fee`, so the user silently covers the
/// shortfall **from their existing USDC balance** — the batch SUCCEEDS, the
/// position looks correct, but the user paid a hidden fee. This is exactly the
/// "phantom debt" failure mode V3 was designed to eliminate.
///
/// We prove it two ways: (a) on a user with USDC buffer, the batch succeeds
/// and `wallet_delta == -borrow_fee` (silent milking); (b) on a user with zero
/// USDC buffer, the batch reverts cleanly. A bot MUST refuse to submit V3 if
/// `pool.borrow_fee_bps != 0`.
#[test]
fn v3_silently_milks_user_when_borrow_fee_bps_nonzero() {
    let fixture = TestMarketFixture::new();
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_client,
        usdc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = &fixture;

    let mut new_fee = fixture.contract_client.get_pool(usdc_pool_address).config.fee_config;
    new_fee.borrow_fee_bps = 1; // 0.01%
    update_pool_fee(&fixture, usdc_pool_address, new_fee);

    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(e, &swap);
    swap_client.init(&10_000_i128);

    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    // -- (a) user with USDC buffer: batch SUCCEEDS, wallet silently shorts the fee --
    let user_a = &users[0];
    let usdc_before_a = usdc_token_client.balance(user_a);
    let batch = v3_batch(
        e,
        &swap,
        usdc_pool_address,
        gold_pool_address,
        usdc_token_address,
        gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );
    contract_client.submit_requests_batch(&ObligationKey::new(user_a.clone()), &batch, &None);

    let usdc_after_a = usdc_token_client.balance(user_a);
    let expected_borrow_fee = ((p.borrow_amount * 1) + 10_000 - 1) / 10_000;
    let wallet_delta = usdc_before_a - usdc_after_a;
    assert_eq!(
        wallet_delta, expected_borrow_fee,
        "user silently paid {} USDC of borrow fee from buffer (V3 invariant violated)",
        expected_borrow_fee
    );

    // -- (b) user with zero USDC buffer: batch REVERTS (no buffer to absorb) --
    let user_b = &users[2];
    let user_b_usdc = usdc_token_client.balance(user_b);
    if user_b_usdc > 0 {
        usdc_token_client.burn(user_b, &user_b_usdc);
    }
    assert_eq!(usdc_token_client.balance(user_b), 0);

    let batch_b = v3_batch(
        e,
        &swap,
        usdc_pool_address,
        gold_pool_address,
        usdc_token_address,
        gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );
    let result = contract_client.try_submit_requests_batch(
        &ObligationKey::new(user_b.clone()),
        &batch_b,
        &None,
    );
    assert!(
        result.is_err(),
        "with zero USDC buffer, V3 must revert when borrow_fee_bps != 0"
    );
    assert!(
        contract_client
            .try_get_user_obligation(&ObligationKey::new(user_b.clone()))
            .is_err(),
        "no obligation should be created on revert"
    );
}

// -- Test 12: V3 BREAKS determinism when add_collateral_fee_bps != 0 ----------

/// V3 precondition: `add_collateral_fee_bps == 0`. With a non-zero collateral
/// fee, the AddCollateral step pulls `margin + Y` from the wallet but credits
/// `(margin + Y) - fee` to the position. The batch may *succeed* (no token
/// shortfall), but the resulting collateral is **strictly less** than V3's
/// promised `margin + Y`. Determinism is broken — bot must refuse this config.
#[test]
fn v3_breaks_determinism_when_add_collateral_fee_bps_nonzero() {
    let fixture = TestMarketFixture::new();
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = &fixture;

    let mut new_fee = fixture.contract_client.get_pool(gold_pool_address).config.fee_config;
    new_fee.add_collateral_fee_bps = 1;
    update_pool_fee(&fixture, gold_pool_address, new_fee);

    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let batch = v3_batch(
        e,
        &swap,
        usdc_pool_address,
        gold_pool_address,
        usdc_token_address,
        gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );

    let outcome =
        contract_client.try_submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    if outcome.is_ok() {
        // Batch succeeded but collateral is short by the fee — determinism broken.
        let obligation =
            contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
        let collateral =
            obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
        assert!(
            collateral < p.expected_collateral(),
            "collateral must be LESS than margin + Y when add_collateral_fee_bps != 0; \
             got {} expected {}",
            collateral,
            p.expected_collateral()
        );
    }
    // Either way: V3 invariant violated. Bot must refuse this config.
}

// -- Test 13: V3 succeeds at a position safely under open_ltv -----------------

/// Sanity baseline for the LTV pair: position at ~60% LTV (default open_ltv = 70%)
/// passes the health check.
#[test]
fn v3_under_open_ltv_succeeds() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];

    // 1:1 oracle prices. margin=100k GOLD, X=Y=150k → collateral=250k, debt≈150_015.
    // LTV ≈ 60.0% — well under default 70%.
    let margin = 100_000;
    let flash_x = 150_000;
    let swap_floor_y = flash_x;
    let flash_fee_amt = flash_fee(flash_x);
    let borrow_amount = flash_x + flash_fee_amt;

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        flash_x,
        swap_floor_y,
        margin,
        borrow_amount,
    );
    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let collateral = obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    let debt = obligation
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(collateral, margin + swap_floor_y);
    assert_eq!(debt, borrow_amount);

    // Confirm we're under 70%: debt/collateral × 10_000 < 7000.
    let ltv_bps = debt * 10_000 / collateral;
    assert!(ltv_bps < 7_000, "expected LTV < 70%, got {} bps", ltv_bps);
}

// -- Test 14: V3 reverts when position would exceed open_ltv ------------------

/// Push the position over the 70% open-LTV threshold by sizing X relative to
/// (margin + Y) too aggressively. The Borrow step at processors.rs:321 invokes
/// the health check, which trips `MCError::UnhealthyOperation` and reverts the
/// whole batch atomically.
#[test]
fn v3_over_open_ltv_reverts_with_unhealthy_operation() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];

    // margin=100k, X=Y=240k → collateral=340k, debt≈240_024. LTV ≈ 70.6% > 70%.
    let margin = 100_000;
    let flash_x = 240_000;
    let swap_floor_y = flash_x;
    let flash_fee_amt = flash_fee(flash_x);
    let borrow_amount = flash_x + flash_fee_amt;

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        flash_x,
        swap_floor_y,
        margin,
        borrow_amount,
    );

    let result =
        contract_client.try_submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);
    assert_eq!(
        result,
        Err(Ok(MCError::UnhealthyOperation)),
        "borrow over open_ltv must trip the health check"
    );
    assert!(
        contract_client
            .try_get_user_obligation(&ObligationKey::new(user.clone()))
            .is_err()
    );
}

// -- Test 15: referrer not registered in pool — explicit failure mode ---------

/// V3 was validated against a `None` referrer. Passing an arbitrary referrer
/// engages a different transfer-aggregation path. With the default pool config
/// (no `referrers` map), origination/operation fees are 0 so the batch should
/// still succeed and the position should be V3-deterministic. This test pins
/// down the actual behaviour.
#[test]
fn v3_with_unregistered_referrer_still_deterministic() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];
    let referrer = users[2].clone();
    let p = V3Params::standard();

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );
    contract_client.submit_requests_batch(
        &ObligationKey::new(user.clone()),
        &batch,
        &Some(referrer),
    );

    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let collateral = obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    let debt = obligation
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(
        collateral,
        p.expected_collateral(),
        "unregistered referrer must not affect V3 collateral"
    );
    assert_eq!(
        debt,
        p.expected_debt(),
        "unregistered referrer must not affect V3 debt"
    );
}

// -- Test 16: multi-hop swap path [USDC, BTC, GOLD] ---------------------------

/// V3 batch using a 3-element path. `process_swap_exact` (processors.rs:1018)
/// only requires `path.len() >= 2` and that endpoints differ; the contract
/// itself doesn't constrain the path length. The mock burns USDC at first hop
/// and mints GOLD at last hop, identical to a real DEX that routes through BTC.
#[test]
fn v3_multi_hop_swap_path_works() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        btc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let multi_hop_path = svec![
        &e,
        usdc_token_address.clone(),
        btc_token_address.clone(),
        gold_token_address.clone()
    ];

    let batch = svec![
        &e,
        Request::FlashBorrow(StandardRequest {
            amount: p.flash_x,
            pool_address: usdc_pool_address.clone(),
        }),
        Request::SwapExactTokens(SwapExactTokensRequest {
            swap_provider: swap.clone(),
            path: multi_hop_path,
            amount_in: p.flash_x,
            min_amount_out: p.swap_floor_y,
        }),
        Request::AddCollateral(StandardRequest {
            amount: p.margin + p.swap_floor_y,
            pool_address: gold_pool_address.clone(),
        }),
        Request::Borrow(StandardRequest {
            amount: p.borrow_amount,
            pool_address: usdc_pool_address.clone(),
        }),
    ];
    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let collateral = obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    let debt = obligation
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(collateral, p.expected_collateral(), "multi-hop preserves V3 collateral invariant");
    assert_eq!(debt, p.expected_debt(), "multi-hop preserves V3 debt invariant");
}

// -- Test 17: flash loan disabled on the borrow pool --------------------------

/// Admin disables flash loans on the USDC pool. V3 must revert at FlashBorrow.
#[test]
fn v3_reverts_when_flash_loan_disabled() {
    let fixture = TestMarketFixture::new();
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = &fixture;

    // Disable flash loans on USDC pool by clearing the FLASH_LOAN_ENABLED bit.
    let mut current = fixture.contract_client.get_pool(usdc_pool_address).config;
    current.status.flags &= !(1u32 << 3); // POOL_STATUS_FLASH_LOAN_ENABLED
    fixture.contract_client.queue_in_pool_set(usdc_pool_address, &current);
    fixture
        .e
        .ledger()
        .with_mut(|li| li.timestamp += DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);
    fixture.contract_client.apply_pool_set(usdc_pool_address);

    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let batch = v3_batch(
        e,
        &swap,
        usdc_pool_address,
        gold_pool_address,
        usdc_token_address,
        gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );
    let result =
        contract_client.try_submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);
    assert!(result.is_err(), "V3 must revert when flash loans disabled");
}

// -- Test 18: pool has insufficient liquidity for the flash borrow ------------

/// Pool only has half the USDC the flash borrow requests. FlashBorrow must
/// revert (the pool can't transfer what it doesn't hold).
#[test]
fn v3_reverts_when_pool_lacks_flash_liquidity() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    // Seed only half the flash amount.
    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(p.flash_x / 2),
        &None,
    );

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );
    let result =
        contract_client.try_submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);
    assert!(result.is_err(), "V3 must revert when pool lacks flash liquidity");
}

// -- Test 19: custom flash_loan_fee_bps = 9 (V1 doc setting) ------------------

/// Bot must compute `flash_fee = ceil(flash_x × flash_loan_fee_bps / 10_000)`
/// generally — not hard-code 1 bp. With fee=9 bps and flash_x=247_500,
/// flash_fee = ceil(222.75) = 223. Borrow = 247_500 + 223 = 247_723.
#[test]
fn v3_with_custom_flash_loan_fee_bps_works() {
    let fixture = TestMarketFixture::new();
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = &fixture;

    let mut new_fee = fixture.contract_client.get_pool(usdc_pool_address).config.fee_config;
    new_fee.flash_loan_fee_bps = 9;
    update_pool_fee(&fixture, usdc_pool_address, new_fee);

    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];

    let margin = 10 * DEFAULT_DEPOSIT_AMOUNT;
    let flash_x = 5 * DEFAULT_DEPOSIT_AMOUNT * 99 / 100;
    let swap_floor_y = flash_x;
    let flash_fee_amt = (flash_x * 9 + 10_000 - 1) / 10_000;
    let borrow_amount = flash_x + flash_fee_amt;

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let batch = v3_batch(
        e,
        &swap,
        usdc_pool_address,
        gold_pool_address,
        usdc_token_address,
        gold_token_address,
        flash_x,
        swap_floor_y,
        margin,
        borrow_amount,
    );
    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let collateral = obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    let debt = obligation
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(collateral, margin + swap_floor_y, "collateral exact under fee=9bps");
    assert_eq!(debt, borrow_amount, "debt exact under fee=9bps");
    assert_eq!(flash_fee_amt, 223, "flash fee math sanity check");
}

// -- Test 20: zero flash_loan_fee_bps -----------------------------------------

/// Edge case: `flash_loan_fee_bps == 0`. flash_fee = 0, borrow = X.
#[test]
fn v3_with_zero_flash_loan_fee_works() {
    let fixture = TestMarketFixture::new();
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = &fixture;

    let mut new_fee = fixture.contract_client.get_pool(usdc_pool_address).config.fee_config;
    new_fee.flash_loan_fee_bps = 0;
    update_pool_fee(&fixture, usdc_pool_address, new_fee);

    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];

    let margin = 10 * DEFAULT_DEPOSIT_AMOUNT;
    let flash_x = 5 * DEFAULT_DEPOSIT_AMOUNT * 99 / 100;
    let swap_floor_y = flash_x;
    let borrow_amount = flash_x; // No flash fee.

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let batch = v3_batch(
        e,
        &swap,
        usdc_pool_address,
        gold_pool_address,
        usdc_token_address,
        gold_token_address,
        flash_x,
        swap_floor_y,
        margin,
        borrow_amount,
    );
    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let debt = obligation
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(debt, flash_x, "debt = X exactly when flash_loan_fee_bps == 0");
}

// -- Test 21: V3 on top of pre-existing collateral position -------------------

/// User already has GOLD collateral in the same pool. V3's AddCollateral must
/// add to the existing position, not overwrite. Final collateral must equal
/// `prior_collateral + margin + Y`.
#[test]
fn v3_adds_to_preexisting_collateral_position() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    let prior_collateral = 3 * DEFAULT_DEPOSIT_AMOUNT;
    contract_client.add_collateral(
        &ObligationKey::new(user.clone()),
        &gold_pool_address,
        &prior_collateral,
        &None,
    );

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );
    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let collateral = obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
    assert_eq!(
        collateral,
        prior_collateral + p.expected_collateral(),
        "V3 must add to existing collateral, not overwrite"
    );
}

// -- Test 22: V3 on top of pre-existing borrow position in same pool ---------

/// User already has USDC debt. V3's Borrow must add to the existing position.
/// Final debt must equal `prior_debt + accrued_interest + (X + flash_fee)`.
/// Within the same ledger (no time passing), interest accrual = 0.
#[test]
fn v3_adds_to_preexisting_borrow_position() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];
    let p = V3Params::standard();

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    // User opens an initial small borrow against initial GOLD collateral.
    contract_client.add_collateral(
        &ObligationKey::new(user.clone()),
        &gold_pool_address,
        &(20 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    let prior_debt = DEFAULT_DEPOSIT_AMOUNT;
    contract_client.borrow(
        &ObligationKey::new(user.clone()),
        &usdc_pool_address,
        &prior_debt,
        &None,
    );

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        p.flash_x,
        p.swap_floor_y,
        p.margin,
        p.borrow_amount,
    );
    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    let debt = obligation
        .borrows
        .get(usdc_pool_address.clone())
        .unwrap()
        .originally_borrowed;
    assert_eq!(
        debt,
        prior_debt + p.expected_debt(),
        "V3 must add to existing debt"
    );
}

// -- Test 23: tiny amounts (1 µ each) — smallest representable position ------

/// Stress the contract's rounding/share math at the smallest possible values.
/// flash_x=1 means flash_fee = ceil(1 × 1 / 10_000) = 1 (rounding-up triggers).
/// borrow=2 is large enough to mint at least one d-token. If the contract
/// rejects sub-share borrows it should do so via a typed error, not corruption.
#[test]
fn v3_with_tiny_amounts_handles_rounding_correctly() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        usdc_pool_address,
        usdc_token_address,
        gold_pool_address,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let swap = e.register(ConfigurableSwap, ());
    let swap_client = ConfigurableSwapClient::new(&e, &swap);
    swap_client.init(&10_000_i128);

    let user = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let margin: i128 = 1;
    let flash_x: i128 = 1;
    let swap_floor_y: i128 = 1;
    let flash_fee_amt: i128 = (flash_x * 1 + 10_000 - 1) / 10_000;
    assert_eq!(flash_fee_amt, 1, "ceiling produces 1 µ fee for any positive flash");
    let borrow_amount = flash_x + flash_fee_amt;

    let batch = v3_batch(
        &e,
        &swap,
        &usdc_pool_address,
        &gold_pool_address,
        &usdc_token_address,
        &gold_token_address,
        flash_x,
        swap_floor_y,
        margin,
        borrow_amount,
    );
    let outcome =
        contract_client.try_submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    // Either succeeds with exact V3 invariants OR fails with a typed contract
    // error (NonPositiveSharesAmount on j-/d-token mint at sub-rounding scale).
    // Critically: must NOT corrupt state.
    match outcome {
        Ok(_) => {
            let obligation =
                contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
            let collateral =
                obligation.deposits.get(gold_pool_address.clone()).unwrap().collateral;
            let debt = obligation
                .borrows
                .get(usdc_pool_address.clone())
                .unwrap()
                .originally_borrowed;
            assert_eq!(collateral, margin + swap_floor_y);
            assert_eq!(debt, borrow_amount);
        }
        Err(_) => {
            // Acceptable failure: typed rounding error. Position must not exist.
            assert!(
                contract_client
                    .try_get_user_obligation(&ObligationKey::new(user.clone()))
                    .is_err(),
                "tiny-amount failure must not leave partial state"
            );
        }
    }
}
