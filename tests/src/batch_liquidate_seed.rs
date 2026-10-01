//! A batched `Liquidate` credits the obligation the batch was submitted under, seed included, so a
//! later request in the same batch can spend the seized shares.
#![cfg(test)]

use market::{
    obligation::ObligationKey,
    request::{LiquidateRequest, Request},
};
use soroban_sdk::{BytesN, testutils::Ledger, vec as svec};

use crate::{DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, get_deposit_position};

/// A borrower whose collateral is held as supply shares, gone unhealthy on three years of interest.
fn liquidatable(f: &TestMarketFixture) -> (soroban_sdk::Address, soroban_sdk::Address) {
    let (borrower, lender) = (f.users[0].clone(), f.users[1].clone());
    f.contract_client.deposit(
        &ObligationKey::new(lender),
        &f.usdc_pool_address,
        &(2 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    f.contract_client.deposit(
        &ObligationKey::new(borrower.clone()),
        &f.gold_pool_address,
        &DEFAULT_DEPOSIT_AMOUNT,
        &None,
    );
    f.contract_client.borrow(
        &ObligationKey::new(borrower.clone()),
        &f.usdc_pool_address,
        &((DEFAULT_DEPOSIT_AMOUNT * 65) / 100),
        &None,
    );
    f.pass_time(3 * 365 * 24 * 60 * 60);
    f.contract_client.refresh_pool(&f.usdc_pool_address);
    f.contract_client.refresh_pool(&f.gold_pool_address);
    (borrower, f.users[2].clone())
}

#[test]
fn test_a_batched_liquidation_credits_the_batchs_own_obligation() {
    let f = TestMarketFixture::new();
    let (borrower, liquidator) = liquidatable(&f);
    let seed = BytesN::from_array(&f.e, &[7u8; 32]);
    let seeded = ObligationKey::new_with_seed(liquidator.clone(), seed);

    f.e.ledger().with_mut(|li| li.timestamp += 1); // invalidate the oracle cache

    let batch = svec![
        &f.e,
        Request::Liquidate(LiquidateRequest {
            borrower_obligation_key: ObligationKey::new(borrower.clone()),
            borrow_pool_address: f.usdc_pool_address.clone(),
            collateral_pool_address: f.gold_pool_address.clone(),
            repay_amount: 10_000,
            min_demanded_collateral_amount: 0,
        }),
    ];
    f.contract_client.submit_requests_batch(&seeded, &batch, &None);

    let into_seeded = f
        .contract_client
        .try_get_user_obligation(&seeded)
        .ok()
        .and_then(|r| r.ok())
        .and_then(|o| o.deposits.get(f.gold_pool_address.clone()))
        .map_or(0, |d| d.j_tokens);
    let into_plain = get_deposit_position(&f.contract_client, &liquidator, &f.gold_pool_address)
        .map_or(0, |d| d.j_tokens);

    assert!(
        into_seeded > 0,
        "the seized shares went to the plain obligation ({into_plain}) instead of the seeded one \
         the batch was submitted under"
    );
    assert_eq!(into_plain, 0, "the seized shares landed in the plain obligation as well");
}
