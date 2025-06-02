mod common;
use common::{TestFixture, DEFAULT_DEPOSIT_AMOUNT};

use soroban_sdk::Address;

#[test]
#[allow(clippy::mistyped_literal_suffixes)]
#[allow(clippy::zero_prefixed_literal)]
#[allow(clippy::inconsistent_digit_grouping)]
fn test_interest_rates() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user: Address = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();
    // Deposit gold as a collateral to satisfy the health factor threshold
    contract_client.deposit_collateral(&user, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    // Deposit usdc as another user to have a non-empty loan pool
    contract_client.deposit(&user2, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

    // 0% UR
    let rates = contract_client.get_apy(&usdc_pool_address);
    assert_eq!(rates.borrow_rate_bps, 00_31); // 0.31%
    assert_eq!(rates.deposit_rate_bps, 00_00); // 0%

    // Borrow 50% of the deposited value
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 2));

    let rates = contract_client.get_apy(&usdc_pool_address);
    assert_eq!(rates.borrow_rate_bps, 17_46); // 17.46%
    assert_eq!(rates.deposit_rate_bps, 08_73); // 8.73%

    // Borrow 75% of the deposited value
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 4));

    let rates = contract_client.get_apy(&usdc_pool_address);
    assert_eq!(rates.borrow_rate_bps, 27_10); // 27.10%
    assert_eq!(rates.deposit_rate_bps, 20_32); // 20.32%

    // Borrow 80% of the deposited value
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 20));

    let rates = contract_client.get_apy(&usdc_pool_address);
    assert_eq!(rates.borrow_rate_bps, 29_12); // 29.12%
    assert_eq!(rates.deposit_rate_bps, 23_30); // 23.30%

    // Borrow 90% of the deposited value
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));

    let rates = contract_client.get_apy(&usdc_pool_address);
    assert_eq!(rates.borrow_rate_bps, 77_03); // 77.03%
    assert_eq!(rates.deposit_rate_bps, 69_33); // 69.33%

    // Borrow 100% of the deposited value
    contract_client.borrow(&user, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10));
    let rates = contract_client.get_apy(&usdc_pool_address);
    assert_eq!(rates.borrow_rate_bps, 142_72); // 142.72%
    assert_eq!(rates.deposit_rate_bps, 142_72); // 142.72%(same, since UR is 1)
}
