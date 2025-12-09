#![cfg(test)]

use market::{
    constants::*,
    pool::{PoolConfig, PoolFeeConfig, PoolHealthConfig},
};
use soroban_sdk::testutils::Ledger;

use crate::{DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture};

#[test]
#[allow(clippy::mistyped_literal_suffixes)]
#[allow(clippy::zero_prefixed_literal)]
#[allow(clippy::inconsistent_digit_grouping)]
fn test_interest_rates() {
    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: BPS_FACTOR,
            ..Default::default()
        },
        ..Default::default()
    };
    let TestMarketFixture {
        e, contract_client, usdc_pool_address, gold_pool_address, users, ..
    } = TestMarketFixture::new_with_pool_config(pool_config);
    let debtor = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(debtor, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    // -- Move time --
    e.ledger().with_mut(|li| li.timestamp += 1);
    contract_client.refresh_pool(&usdc_pool_address);

    // 0% UR
    let borrow_bps = contract_client.get_pool(&usdc_pool_address).borrow_apr_bps;
    let supply_bps = contract_client.get_pool(&usdc_pool_address).supply_apr_bps;
    assert_eq!(borrow_bps, 00_01); // WARN: calculations for APY yield 0% due to a precision loss
    assert_eq!(supply_bps, 00_00);

    // Borrow 50% of the deposited value
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    let rates = contract_client.get_pool_data(&usdc_pool_address).apy;
    assert_eq!(rates.borrow_bps, 23_89);
    assert_eq!(rates.supply_bps, 10_10);

    // Borrow 75% of the deposited value
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    let rates = contract_client.get_pool_data(&usdc_pool_address).apy;
    assert_eq!(rates.borrow_bps, 56_83);
    assert_eq!(rates.supply_bps, 35_48);

    // Borrow 80% of the deposited value
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 20));

    let rates = contract_client.get_pool_data(&usdc_pool_address).apy;
    assert_eq!(rates.borrow_bps, 82_21);
    assert_eq!(rates.supply_bps, 54_03);

    // Borrow 90% of the deposited value
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

    let rates = contract_client.get_pool_data(&usdc_pool_address).apy;
    assert_eq!(rates.borrow_bps, 897_41);
    assert_eq!(rates.supply_bps, 544_30);

    // Borrow 100% of the deposited value
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

    let rates = contract_client.get_pool_data(&usdc_pool_address).apy;
    assert_eq!(rates.supply_bps, 355_982);
    assert_eq!(rates.borrow_bps, 535_981);
}

#[test]
#[allow(clippy::inconsistent_digit_grouping)]
fn test_interest_rates_no_take_rate() {
    let pool_config = PoolConfig {
        health_config: PoolHealthConfig {
            utilization_ratio_limit_bps: BPS_FACTOR,
            ..Default::default()
        },
        fee_config: PoolFeeConfig { take_rate_bps: 0, ..Default::default() },
        ..Default::default()
    };
    let TestMarketFixture { contract_client, usdc_pool_address, gold_pool_address, users, .. } =
        TestMarketFixture::new_with_pool_config(pool_config);
    let debtor = &users[0];
    let liquidity_provider = &users[1];

    contract_client.add_collateral(debtor, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(liquidity_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    // Borrow 100% of the deposited value
    contract_client.borrow(debtor, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let rates = contract_client.get_pool_data(&usdc_pool_address).apy;
    assert_eq!(rates.borrow_bps, 535_981);
    assert_eq!(rates.supply_bps, 535_981);
}
