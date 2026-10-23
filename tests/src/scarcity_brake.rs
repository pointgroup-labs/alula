//! Why the withdrawal-scarcity allowance stays disabled by default: its cooldown clock lives on
//! `DepositPosition`, one per `ObligationKey { user, seed }`, so a single address walks past the
//! allowance in one ledger by moving to a second seed.
#![cfg(test)]

use market::{
    error::MCError,
    obligation::ObligationKey,
    pool::{PoolConfig, PoolFeeConfig, PoolHealthConfig},
};
use soroban_sdk::{
    BytesN,
    testutils::{BytesN as _, Ledger},
};

use crate::{DEFAULT_COLLATERAL_AMOUNT, DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture};

/// Arms both halves of the brake on the USDC pool and drives it to its utilization limit.
fn armed_and_stressed<'a>(
    second_seed: bool,
) -> (TestMarketFixture<'a>, ObligationKey, Option<ObligationKey>) {
    let f = TestMarketFixture::new();
    let armed = PoolConfig {
        health_config: PoolHealthConfig {
            withdraw_scarcity_limit_bps: 5_000, // half the remaining liquidity per withdrawal
            withdraw_scarcity_cooldown_s: 3_600,
            ..Default::default()
        },
        fee_config: PoolFeeConfig { withdraw_max_scarcity_fee_bps: 500, ..Default::default() },
        ..Default::default()
    };
    f.contract_client.queue_in_pool_set(&f.usdc_pool_address, &armed);
    let period = f.contract_client.get_global_state().update_in_queue_period;
    f.e.ledger().with_mut(|li| li.timestamp += period);
    f.contract_client.apply_pool_set(&f.usdc_pool_address);

    let lender = ObligationKey::new(f.users[0].clone());
    let borrower = ObligationKey::new(f.users[1].clone());
    f.contract_client.deposit(
        &lender,
        &f.usdc_pool_address,
        &(100 * DEFAULT_DEPOSIT_AMOUNT),
        &None,
    );
    // Both obligations are funded before the pool is stressed: depositing later would relieve the
    // stress and the brake would simply not engage.
    let seeded = second_seed.then(|| {
        let k = ObligationKey::new_with_seed(f.users[0].clone(), BytesN::random(&f.e));
        f.contract_client.deposit(&k, &f.usdc_pool_address, &(20 * DEFAULT_DEPOSIT_AMOUNT), &None);
        k
    });
    f.contract_client.add_collateral(
        &borrower,
        &f.gold_pool_address,
        &(600 * DEFAULT_COLLATERAL_AMOUNT),
        &None,
    );
    // Borrow up to the utilization limit so any further withdrawal dips into the allowance.
    let supply = crate::get_pool_total_supply(&f.contract_client, &f.usdc_pool_address).unwrap();
    f.contract_client.borrow(&borrower, &f.usdc_pool_address, &(supply * 9 / 10), &None);
    (f, lender, seeded)
}

/// The brake does bind one position: a second dip inside the cooldown is refused.
#[test]
fn test_the_cooldown_binds_a_single_position() {
    let (f, lender, _) = armed_and_stressed(false);
    let dip = 3 * DEFAULT_DEPOSIT_AMOUNT;

    assert!(
        f.contract_client.try_withdraw(&lender, &f.usdc_pool_address, &dip, &None).is_ok(),
        "the first dip into the allowance was refused"
    );
    assert_eq!(
        f.contract_client.try_withdraw(&lender, &f.usdc_pool_address, &dip, &None),
        Err(Ok(MCError::ScarcityCooldownPeriod)),
        "a second dip inside the cooldown was admitted"
    );
}

/// **And one address walks past it in the same ledger by using a second seed.** The clock is per
/// position, so a fresh seed starts at zero. This is why enabling the allowance by default buys little:
/// it binds a lender who holds one position and not one who splits.
#[test]
fn test_a_second_seed_bypasses_the_cooldown_in_the_same_ledger() {
    let (f, lender, seeded) = armed_and_stressed(true);
    let seeded = seeded.unwrap();
    let dip = 3 * DEFAULT_DEPOSIT_AMOUNT;

    assert!(f.contract_client.try_withdraw(&lender, &f.usdc_pool_address, &dip, &None).is_ok());
    assert_eq!(
        f.contract_client.try_withdraw(&lender, &f.usdc_pool_address, &dip, &None),
        Err(Ok(MCError::ScarcityCooldownPeriod)),
        "the cooldown did not bind the first position"
    );
    assert!(
        f.contract_client.try_withdraw(&seeded, &f.usdc_pool_address, &dip, &None).is_ok(),
        "the second seed was also held back, so the clock is not per position after all"
    );
}
