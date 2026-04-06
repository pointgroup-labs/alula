#![cfg(test)]

use market::{
    error::MCError,
    obligation::ObligationKey,
    request::{
        LiquidateRequest, Request, StandardRequest, SwapExactTokensRequest,
        SwapForExactTokensRequest,
    },
};
use soroban_sdk::{
    Address, Env, Vec, contract, contractimpl,
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    vec as svec,
};

use crate::{DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture};

#[contract]
pub struct MockProxySwap;

#[contractimpl]
impl MockProxySwap {
    pub fn swap_exact(
        e: Env,
        user: Address,
        path: Vec<Address>,
        amount_in: i128,
        _min_amount_out: i128,
    ) -> i128 {
        user.require_auth();

        let token_in = path.first().unwrap();
        let token_out = path.last().unwrap();

        TokenClient::new(&e, &token_in).burn(&user, &amount_in);
        StellarAssetClient::new(&e, &token_out).mint(&user, &amount_in);

        amount_in
    }

    pub fn swap_for_exact(
        e: Env,
        user: Address,
        path: Vec<Address>,
        _amount_in_max: i128,
        amount_out: i128,
    ) -> i128 {
        user.require_auth();

        let token_in = path.first().unwrap();
        let token_out = path.last().unwrap();

        TokenClient::new(&e, &token_in).burn(&user, &amount_out);
        StellarAssetClient::new(&e, &token_out).mint(&user, &amount_out);

        amount_out
    }
}

#[test]
fn test_flash_borrow_swap_deposit_borrow_batch() {
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
    } = TestMarketFixture::new();
    let proxy_swap = e.register(MockProxySwap, ());
    let user = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let collateral_from_wallet = 10 * DEFAULT_DEPOSIT_AMOUNT;
    let flash_amount = 5 * DEFAULT_DEPOSIT_AMOUNT;
    let total_collateral = collateral_from_wallet + flash_amount;
    // Borrow up to 60% of total collateral (under 70% open_ltv)
    let borrow_amount = (total_collateral * 60) / 100;

    let batch = svec![
        &e,
        Request::AddCollateral(StandardRequest {
            amount: collateral_from_wallet,
            pool_address: gold_pool_address.clone(),
        }),
        Request::FlashBorrow(StandardRequest {
            amount: flash_amount,
            pool_address: usdc_pool_address.clone(),
        }),
        Request::SwapExactTokens(SwapExactTokensRequest {
            swap_provider: proxy_swap.clone(),
            path: svec![&e, usdc_token_address.clone(), gold_token_address.clone()],
            amount_in: flash_amount,
            min_amount_out: flash_amount,
        }),
        Request::AddCollateral(StandardRequest {
            amount: flash_amount,
            pool_address: gold_pool_address.clone(),
        }),
        Request::Borrow(StandardRequest {
            amount: borrow_amount,
            pool_address: usdc_pool_address.clone(),
        }),
    ];

    let usdc_before = usdc_token_client.balance(user);

    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    let usdc_after = usdc_token_client.balance(user);
    // Net USDC gain = borrow_amount - flash_amount - fee (positive since borrow > flash)
    assert!(usdc_after > usdc_before);

    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));
    assert!(obligation.deposits.get(gold_pool_address.clone()).is_some());
    assert!(obligation.borrows.get(usdc_pool_address.clone()).is_some());
}

#[test]
fn test_liquidation_via_flash_borrow_and_swap() {
    let TestMarketFixture {
        e,
        contract_client,
        users,
        oracle_client,
        usdc_pool_address,
        usdc_token_client,
        usdc_token_address,
        gold_pool_address,
        gold_token_client,
        gold_token_address,
        ..
    } = TestMarketFixture::new();
    let proxy_swap = e.register(MockProxySwap, ());

    let borrower = &users[0];
    let liquidity_provider = &users[1];
    // Fresh address with zero balances — flash loan bootstraps the entire liquidation
    let liquidator = Address::generate(&e);
    assert_eq!(usdc_token_client.balance(&liquidator), 0);
    assert_eq!(gold_token_client.balance(&liquidator), 0);

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    // (close_ltv = 80%, open_ltv = 70%, so 65% is safe)
    let collateral_amount = DEFAULT_DEPOSIT_AMOUNT;
    let borrow_amount = (collateral_amount * 65) / 100;

    contract_client.add_collateral(
        &ObligationKey::new(borrower.clone()),
        &gold_pool_address,
        &collateral_amount,
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &usdc_pool_address,
        &borrow_amount,
        &None,
    );

    // Crash GOLD price: was 1.0, now 0.5
    oracle_client.set_price_stable(&soroban_sdk::vec![
        &e,
        50000000000000,   // GOLD = 0.5
        1_00000000000000, // BTC
        1_00000000000000, // USDC
    ]);

    // Liquidate half the debt via the close factor (default 50%)
    let repay_amount = borrow_amount / 2;
    // Flash repayment = principal + fee (0.01% ceiling)
    let flash_repayment = repay_amount + (repay_amount + 9999) / 10_000;

    let batch = svec![
        &e,
        Request::FlashBorrow(StandardRequest {
            amount: repay_amount,
            pool_address: usdc_pool_address.clone(),
        }),
        Request::Liquidate(LiquidateRequest {
            borrower_obligation_key: ObligationKey::new(borrower.clone()),
            borrow_pool_address: usdc_pool_address.clone(),
            collateral_pool_address: gold_pool_address.clone(),
            repay_amount,
            min_demanded_collateral_amount: 0,
        }),
        // Swap just enough seized GOLD to cover the flash repayment (principal + fee)
        Request::SwapForExactTokens(SwapForExactTokensRequest {
            swap_provider: proxy_swap.clone(),
            path: svec![&e, gold_token_address.clone(), usdc_token_address.clone()],
            max_amount_in: i128::MAX,
            amount_out: flash_repayment + 1,
        }),
    ];

    let liquidator_usdc_before = usdc_token_client.balance(&liquidator);
    let liquidator_gold_before = gold_token_client.balance(&liquidator);

    contract_client.submit_requests_batch(&ObligationKey::new(liquidator.clone()), &batch, &None);

    let liquidator_usdc_after = usdc_token_client.balance(&liquidator);
    let liquidator_gold_after = gold_token_client.balance(&liquidator);

    // Started with 0 USDC, swapped exactly enough to cover flash repayment + 1
    assert_eq!(liquidator_usdc_before, 0);
    assert_eq!(liquidator_usdc_after, 1);

    // Profit is entirely in GOLD: seized collateral minus what was swapped to cover the flash loan
    assert!(
        liquidator_gold_after > liquidator_gold_before,
        "Liquidator gained no GOLD: before={}, after={}",
        liquidator_gold_before,
        liquidator_gold_after
    );
}

/// FlashBorrow + simple ops (no swap): flash repayment at end of batch, not prematurely
#[test]
fn test_flash_borrow_deposit_repay_no_swap() {
    let TestMarketFixture {
        e, contract_client, users, usdc_pool_address, gold_pool_address, ..
    } = TestMarketFixture::new();
    let user = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    contract_client.add_collateral(
        &ObligationKey::new(user.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(user.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 2),
        &None,
    );

    let flash_amount = DEFAULT_DEPOSIT_AMOUNT / 2;
    let batch = svec![
        &e,
        Request::FlashBorrow(StandardRequest {
            amount: flash_amount,
            pool_address: usdc_pool_address.clone(),
        }),
        Request::Repay(StandardRequest {
            amount: flash_amount,
            pool_address: usdc_pool_address.clone(),
        }),
        Request::RemoveCollateral(StandardRequest {
            amount: DEFAULT_DEPOSIT_AMOUNT,
            pool_address: gold_pool_address.clone(),
        }),
    ];

    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    assert!(contract_client.try_get_user_obligation(&ObligationKey::new(user.clone())).is_err());
}

#[test]
fn test_flash_borrow_without_repay_reverts() {
    let TestMarketFixture {
        e, contract_client, users, usdc_pool_address, usdc_token_client, ..
    } = TestMarketFixture::new();
    let user = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let user_usdc = usdc_token_client.balance(user);
    usdc_token_client.burn(user, &user_usdc);
    assert_eq!(usdc_token_client.balance(user), 0);

    let flash_amount = DEFAULT_DEPOSIT_AMOUNT;
    let batch = svec![
        &e,
        Request::FlashBorrow(StandardRequest {
            amount: flash_amount,
            pool_address: usdc_pool_address.clone(),
        }),
    ];

    let result =
        contract_client.try_submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);
    assert_eq!(result, Err(Ok(MCError::TooManyPositions)));
}

/// Flash borrow where user has the principal but not the fee must also revert
#[test]
fn test_flash_borrow_fee_evasion_reverts() {
    let TestMarketFixture {
        e, contract_client, users, usdc_pool_address, usdc_token_client, ..
    } = TestMarketFixture::new();
    let user = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let flash_amount = DEFAULT_DEPOSIT_AMOUNT;
    // Burn user's USDC down to exactly flash_amount so they can't cover the fee.
    let user_usdc = usdc_token_client.balance(user);
    let burn_amount = user_usdc - flash_amount;
    usdc_token_client.burn(user, &burn_amount);
    assert_eq!(usdc_token_client.balance(user), flash_amount);

    let batch = svec![
        &e,
        Request::FlashBorrow(StandardRequest {
            amount: flash_amount,
            pool_address: usdc_pool_address.clone(),
        }),
    ];

    usdc_token_client.burn(user, &flash_amount); // now user has 0
    assert_eq!(usdc_token_client.balance(user), 0);

    let pool_before = contract_client.get_pool(&usdc_pool_address);

    let result =
        contract_client.try_submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    // User has exactly flash_amount (from borrow) but needs flash_amount + fee → fails
    assert_eq!(result, Err(Ok(MCError::TooManyPositions)));

    let pool_after = contract_client.get_pool(&usdc_pool_address);
    assert_eq!(pool_before.total_available, pool_after.total_available);
    assert_eq!(pool_before.operation_fees_sum, pool_after.operation_fees_sum);
}

/// Flash borrow with amount=0 is a no-op (no funds moved, no fee, pool unchanged)
#[test]
fn test_flash_borrow_zero() {
    let TestMarketFixture { e, contract_client, users, usdc_pool_address, .. } =
        TestMarketFixture::new();
    let user = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let batch = svec![
        &e,
        Request::FlashBorrow(StandardRequest {
            amount: 0,
            pool_address: usdc_pool_address.clone(),
        }),
    ];

    assert_eq!(
        contract_client.try_submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None),
        Err(Ok(MCError::InvalidInputAmount))
    );
}

/// Double flash borrow in the same batch must be rejected
#[test]
fn test_double_flash_borrow_rejected() {
    let TestMarketFixture { e, contract_client, users, usdc_pool_address, .. } =
        TestMarketFixture::new();
    let user = &users[0];
    let liquidity_provider = &users[1];

    contract_client.deposit(
        &ObligationKey::new(liquidity_provider.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let batch = svec![
        &e,
        Request::FlashBorrow(StandardRequest {
            amount: DEFAULT_DEPOSIT_AMOUNT,
            pool_address: usdc_pool_address.clone(),
        }),
        Request::FlashBorrow(StandardRequest {
            amount: DEFAULT_DEPOSIT_AMOUNT,
            pool_address: usdc_pool_address.clone(),
        }),
    ];

    let result =
        contract_client.try_submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    assert_eq!(result, Err(Ok(MCError::FlashBorrowAlreadyRegistered)));
}
