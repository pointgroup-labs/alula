#![cfg(test)]

use market::{
    constants::BPS_FACTOR,
    obligation::{ComputedFees, compute_fees},
    pool::{PoolConfig, PoolFeeConfig},
};
use soroban_fixed_point_math::FixedPoint;

use crate::{
    DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, get_obligation_d_tokens_as_tokens,
    get_pool_accumulated_host_fees, get_pool_accumulated_market_fees, get_pool_fee_config,
};

// -- Default Fees(only for borrow and flash loan(tested in 'flash_loan_taker_mock')) --

#[test]
fn test_borrow_fee() {
    let TestMarketFixture {
        e,
        contract_id,
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        users,
        usdc_token_client,
        ..
    } = TestMarketFixture::new();
    let borrower = &users[0];
    let loan_provider = &users[1];

    contract_client.deposit(borrower, &gold_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT));
    contract_client.deposit(loan_provider, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let market_fees_before = get_pool_accumulated_market_fees(&contract_client, &usdc_pool_address);
    let host_fees_before = get_pool_accumulated_host_fees(&contract_client, &usdc_pool_address);
    assert_eq!(market_fees_before, 0);
    assert_eq!(host_fees_before, 0);

    let pool_usdc_balance_before = usdc_token_client.balance(&contract_id);
    let borrower_usdc_balance_before = usdc_token_client.balance(borrower);

    contract_client.borrow(borrower, &usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT);

    let pool_usdc_balance_after = usdc_token_client.balance(&contract_id);
    let borrower_usdc_balance_after = usdc_token_client.balance(borrower);
    let pool_balance_diff = pool_usdc_balance_before
        .checked_sub(pool_usdc_balance_after)
        .unwrap();
    let borrower_balance_diff = borrower_usdc_balance_after
        .checked_sub(borrower_usdc_balance_before)
        .unwrap();

    let pool_market_fees_after =
        get_pool_accumulated_market_fees(&contract_client, &usdc_pool_address);
    let pool_host_fees_after = get_pool_accumulated_host_fees(&contract_client, &usdc_pool_address);

    let borrower_debt_after =
        get_obligation_d_tokens_as_tokens(&e, &contract_client, borrower, &usdc_pool_address)
            .unwrap();

    let PoolFeeConfig {
        borrow_fee_bps,
        host_fee_bps,
        ..
    } = get_pool_fee_config(&contract_client, &usdc_pool_address);
    let ComputedFees {
        fee_sum,
        market_fee,
        host_fee,
    } = compute_fees(DEFAULT_DEPOSIT_AMOUNT, borrow_fee_bps, host_fee_bps).unwrap();

    let expected_borrower_balance_diff = DEFAULT_DEPOSIT_AMOUNT.checked_sub(fee_sum).unwrap();
    let expected_pool_balance_diff = expected_borrower_balance_diff;
    let expected_market_fees_diff = market_fee;
    let expected_host_fees_diff = host_fee;

    assert_eq!(borrower_balance_diff, expected_borrower_balance_diff);
    assert_eq!(pool_balance_diff, expected_pool_balance_diff);

    assert_eq!(pool_market_fees_after, expected_market_fees_diff);
    assert_eq!(pool_host_fees_after, expected_host_fees_diff);

    assert_eq!(borrower_debt_after, DEFAULT_DEPOSIT_AMOUNT);
}
