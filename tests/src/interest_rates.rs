#![cfg(test)]

use crate::{DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture};

#[test]
#[allow(clippy::mistyped_literal_suffixes)]
#[allow(clippy::zero_prefixed_literal)]
#[allow(clippy::inconsistent_digit_grouping)]
fn test_interest_rates() {
    let TestMarketFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestMarketFixture::new();
    let debtor = &users[0];
    let loan_provider = &users[1];

    contract_client.add_collateral(debtor, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    // 0% UR
    let rates = contract_client.get_apr(&usdc_pool_address);
    assert_eq!(rates.borrow_bps, 00_01); // WARN: calculations for APY yield 0% due to a precision loss
    assert_eq!(rates.supply_bps, 00_00);

    // Borrow 50% of the deposited value
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    let rates = contract_client.get_apy(&usdc_pool_address);
    assert_eq!(rates.borrow_bps, 23_89);
    assert_eq!(rates.supply_bps, 11_30);

    // Borrow 75% of the deposited value
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    let rates = contract_client.get_apy(&usdc_pool_address);
    assert_eq!(rates.borrow_bps, 56_83);
    assert_eq!(rates.supply_bps, 40_14);

    // Borrow 80% of the deposited value
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 20));

    let rates = contract_client.get_apy(&usdc_pool_address);
    assert_eq!(rates.borrow_bps, 82_21);
    assert_eq!(rates.supply_bps, 61_60);

    // Borrow 90% of the deposited value
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

    let rates = contract_client.get_apy(&usdc_pool_address);
    assert_eq!(rates.borrow_bps, 897_41);
    assert_eq!(rates.supply_bps, 692_48);

    // Borrow 100% of the deposited value
    contract_client.borrow(debtor, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

    let rates = contract_client.get_apy(&usdc_pool_address);
    assert_eq!(rates.borrow_bps, 535_981);
    assert_eq!(rates.supply_bps, 535_981);
}
