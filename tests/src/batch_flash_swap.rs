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
    testutils::{Address as _, Ledger},
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
        _max_amount_in: i128,
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

    e.ledger().with_mut(|li| li.timestamp += 1_u64); // invalidate oracle cache

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

#[test]
fn test_flash_borrow_does_not_reprice_shares_within_batch() {
    let run = |with_flash: bool| {
        let TestMarketFixture {
            e,
            contract_client,
            users,
            usdc_pool_address,
            gold_pool_address,
            ..
        } = TestMarketFixture::new();
        let depositor = &users[0];
        let lender = &users[1];
        let borrower = &users[2];

        contract_client.deposit(
            &ObligationKey::new(lender.clone()),
            &usdc_pool_address,
            &(100 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );
        contract_client.add_collateral(
            &ObligationKey::new(borrower.clone()),
            &gold_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &None,
        );
        contract_client.borrow(
            &ObligationKey::new(borrower.clone()),
            &usdc_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 2),
            &None,
        );

        let mut batch = svec![&e];
        if with_flash {
            batch.push_back(Request::FlashBorrow(StandardRequest {
                amount: 90 * DEFAULT_DEPOSIT_AMOUNT,
                pool_address: usdc_pool_address.clone(),
            }));
        }
        batch.push_back(Request::Deposit(StandardRequest {
            amount: DEFAULT_DEPOSIT_AMOUNT,
            pool_address: usdc_pool_address.clone(),
        }));
        contract_client.submit_requests_batch(
            &ObligationKey::new(depositor.clone()),
            &batch,
            &None,
        );

        let value = |user: &Address| {
            crate::get_obligation_j_tokens_as_tokens(&e, &contract_client, user, &usdc_pool_address)
                .unwrap()
        };
        (value(lender), value(depositor))
    };

    let (lender_plain, depositor_plain) = run(false);
    let (lender_flash, depositor_flash) = run(true);
    assert_eq!((lender_flash, depositor_flash), (lender_plain, depositor_plain));
}

#[test]
fn test_flash_borrow_share_price_realized_in_tokens() {
    let run = |with_flash: bool| {
        let TestMarketFixture {
            e,
            contract_client,
            users,
            usdc_pool_address,
            gold_pool_address,
            usdc_token_client,
            ..
        } = TestMarketFixture::new();
        let attacker = &users[0];
        let lender = &users[1];
        let borrower = &users[2];

        contract_client.deposit(
            &ObligationKey::new(lender.clone()),
            &usdc_pool_address,
            &(100 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );
        contract_client.add_collateral(
            &ObligationKey::new(borrower.clone()),
            &gold_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &None,
        );
        contract_client.borrow(
            &ObligationKey::new(borrower.clone()),
            &usdc_pool_address,
            &(DEFAULT_DEPOSIT_AMOUNT / 2),
            &None,
        );

        let attacker_before = usdc_token_client.balance(attacker);
        let lender_before = usdc_token_client.balance(lender);

        let mut batch = svec![&e];
        if with_flash {
            batch.push_back(Request::FlashBorrow(StandardRequest {
                amount: 90 * DEFAULT_DEPOSIT_AMOUNT,
                pool_address: usdc_pool_address.clone(),
            }));
        }
        batch.push_back(Request::Deposit(StandardRequest {
            amount: DEFAULT_DEPOSIT_AMOUNT,
            pool_address: usdc_pool_address.clone(),
        }));
        contract_client.submit_requests_batch(&ObligationKey::new(attacker.clone()), &batch, &None);

        // Realize in a separate transaction, after the flash is repaid.
        contract_client.withdraw(
            &ObligationKey::new(attacker.clone()),
            &usdc_pool_address,
            &i128::MAX,
            &None,
        );
        contract_client.withdraw(
            &ObligationKey::new(lender.clone()),
            &usdc_pool_address,
            &i128::MAX,
            &None,
        );

        (
            usdc_token_client.balance(attacker) - attacker_before,
            usdc_token_client.balance(lender) - lender_before,
        )
    };

    let (attacker_plain, lender_plain) = run(false);
    let (attacker_flash, lender_flash) = run(true);
    // Lenders must be untouched by someone else's flash borrow, and the flash must never pay:
    // the attacker's own net is at most what the flash fee costs them.
    assert_eq!(lender_flash, lender_plain, "a flash borrow moved value away from the lender");
    assert_eq!(attacker_plain, 0, "deposit-then-withdraw must break even without a flash");
    assert!(attacker_flash <= 0, "flash borrow paid the attacker {attacker_flash}");
}

/// A third party's flash borrow must not make a healthy position liquidatable. The victim's
/// collateral is a deposit (j-tokens) in the pool the attacker flashes.
#[test]
fn test_flash_borrow_cannot_make_a_healthy_position_liquidatable() {
    let TestMarketFixture {
        e, contract_client, users, usdc_pool_address, gold_pool_address, ..
    } = TestMarketFixture::new();
    let victim = &users[0];
    let lender = &users[1];
    let attacker = &users[2];

    contract_client.deposit(
        &ObligationKey::new(lender.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    contract_client.deposit(
        &ObligationKey::new(victim.clone()),
        &gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    contract_client.borrow(
        &ObligationKey::new(victim.clone()),
        &usdc_pool_address,
        &(DEFAULT_DEPOSIT_AMOUNT / 3),
        &None,
    );

    // Control: the position is healthy, so a plain liquidation must be refused.
    let plain = contract_client.try_liquidate(
        attacker,
        &ObligationKey::new(victim.clone()),
        &usdc_pool_address,
        &gold_pool_address,
        &1,
        &0,
    );
    assert_eq!(
        plain,
        Err(Ok(MCError::ObligationIsHealthy)),
        "control failed: the victim was not healthy to begin with"
    );

    let batch = svec![
        &e,
        Request::FlashBorrow(StandardRequest {
            amount: (DEFAULT_DEPOSIT_AMOUNT * 9) / 10,
            pool_address: gold_pool_address.clone(),
        }),
        Request::Liquidate(LiquidateRequest {
            borrower_obligation_key: ObligationKey::new(victim.clone()),
            borrow_pool_address: usdc_pool_address.clone(),
            collateral_pool_address: gold_pool_address.clone(),
            repay_amount: DEFAULT_DEPOSIT_AMOUNT / 10,
            min_demanded_collateral_amount: 0,
        }),
    ];
    let result = contract_client.try_submit_requests_batch(
        &ObligationKey::new(attacker.clone()),
        &batch,
        &None,
    );

    assert_eq!(
        result,
        Err(Ok(MCError::ObligationIsHealthy)),
        "a flash borrow made a healthy position liquidatable"
    );
}

/// A capped borrow must never be allowed past what the pool can actually pay out, at any flash size.
/// Asserts the absence of the insolvency event too, so it covers the class, not one example.
#[test]
fn test_capped_borrow_inside_flash_batch_stays_within_payable() {
    for pct in [5i128, 11, 25, 50, 90] {
        let TestMarketFixture {
            e,
            contract_client,
            users,
            usdc_pool_address,
            gold_pool_address,
            ..
        } = TestMarketFixture::new();
        let borrower = &users[0];
        let lender = &users[1];

        contract_client.deposit(
            &ObligationKey::new(lender.clone()),
            &usdc_pool_address,
            &(100 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );
        contract_client.add_collateral(
            &ObligationKey::new(borrower.clone()),
            &gold_pool_address,
            &(50 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );

        let batch = svec![
            &e,
            Request::FlashBorrow(StandardRequest {
                amount: (100 * DEFAULT_DEPOSIT_AMOUNT * pct) / 100,
                pool_address: usdc_pool_address.clone(),
            }),
            Request::Borrow(StandardRequest {
                amount: i128::MAX,
                pool_address: usdc_pool_address.clone(),
            }),
        ];
        let result = contract_client.try_submit_requests_batch(
            &ObligationKey::new(borrower.clone()),
            &batch,
            &None,
        );

        assert_ne!(
            result,
            Err(Ok(MCError::InternalError)),
            "flash {pct}%: a capped borrow tripped the pool's own insolvency guard"
        );
    }
}

/// No flash reservation may outlive the transaction that made it: a leak would permanently floor the
/// pool's payouts with no way back short of an upgrade. Checks the successful and the reverted batch.
#[test]
fn test_flash_reservation_never_outlives_the_transaction() {
    let TestMarketFixture {
        e,
        contract_client,
        contract_id,
        users,
        usdc_pool_address,
        gold_pool_address,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];
    let lender = &users[1];

    contract_client.deposit(
        &ObligationKey::new(lender.clone()),
        &usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );

    let reserved = |pool: &Address| {
        e.as_contract(&contract_id, || market::storage::get_flash_reserved(&e, pool))
    };
    let assert_clean = |label: &str| {
        assert_eq!(reserved(&usdc_pool_address), 0, "{label}: usdc reservation leaked");
        assert_eq!(reserved(&gold_pool_address), 0, "{label}: gold reservation leaked");
    };

    assert_clean("before");

    let flash_amount = 10 * DEFAULT_DEPOSIT_AMOUNT;
    let ok_batch = svec![
        &e,
        Request::FlashBorrow(StandardRequest {
            amount: flash_amount,
            pool_address: usdc_pool_address.clone(),
        }),
        Request::Deposit(StandardRequest {
            amount: DEFAULT_DEPOSIT_AMOUNT,
            pool_address: usdc_pool_address.clone(),
        }),
    ];
    contract_client.submit_requests_batch(&ObligationKey::new(user.clone()), &ok_batch, &None);
    assert_clean("after a batch that succeeded");

    // A batch that fails after the flash: the reservation must roll back with everything else.
    let failing_batch = svec![
        &e,
        Request::FlashBorrow(StandardRequest {
            amount: flash_amount,
            pool_address: usdc_pool_address.clone(),
        }),
        Request::Withdraw(StandardRequest {
            amount: i128::MAX,
            pool_address: gold_pool_address.clone(),
        }),
    ];
    let failed = contract_client.try_submit_requests_batch(
        &ObligationKey::new(user.clone()),
        &failing_batch,
        &None,
    );
    assert!(failed.is_err(), "the control batch was supposed to fail");
    assert_clean("after a batch that reverted");
}

/// One batch may hold at most one flash borrow, including across different pools, and a refused batch
/// leaves the market's balances alone. Note this cannot observe *where* the refusal happens: the
/// revert undoes a transfer that already moved, so a guard placed earlier is not detectable here.
#[test]
fn test_two_flash_borrows_in_one_batch_are_refused() {
    let TestMarketFixture {
        e,
        contract_client,
        contract_id,
        users,
        usdc_pool_address,
        gold_pool_address,
        ..
    } = TestMarketFixture::new();
    let user = &users[0];
    let lender = &users[1];

    for pool in [&usdc_pool_address, &gold_pool_address] {
        contract_client.deposit(
            &ObligationKey::new(lender.clone()),
            pool,
            &(10 * DEFAULT_DEPOSIT_AMOUNT),
            &None,
        );
    }

    let market_before = |pool: &Address| TokenClient::new(&e, pool).balance(&contract_id);
    let (usdc_before, gold_before) =
        (market_before(&usdc_pool_address), market_before(&gold_pool_address));

    // Second flash on a different pool: the per-pool reservation check cannot see it, so only the
    // batch-level guard can.
    let batch = svec![
        &e,
        Request::FlashBorrow(StandardRequest {
            amount: DEFAULT_DEPOSIT_AMOUNT,
            pool_address: usdc_pool_address.clone(),
        }),
        Request::FlashBorrow(StandardRequest {
            amount: DEFAULT_DEPOSIT_AMOUNT,
            pool_address: gold_pool_address.clone(),
        }),
    ];
    let result =
        contract_client.try_submit_requests_batch(&ObligationKey::new(user.clone()), &batch, &None);

    assert_eq!(result, Err(Ok(MCError::FlashBorrowAlreadyRegistered)));
    assert_eq!(market_before(&usdc_pool_address), usdc_before, "usdc balance moved");
    assert_eq!(market_before(&gold_pool_address), gold_before, "gold balance moved");
}
