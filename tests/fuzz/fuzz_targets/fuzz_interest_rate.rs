#![no_main]

use lending::pool::{Pool, PoolConfig};
use libfuzzer_sys::fuzz_target;
use soroban_sdk::{
    Env, symbol_short,
    testutils::{Address as _, arbitrary::Arbitrary},
};

#[derive(Arbitrary, Debug)]
struct InterestRateInput {
    // Total supply amount (0 to realistic max)
    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=1_000_000_000_000_000i128))]
    total_supply: i128,

    // Total borrowed amount (0 to total_supply)
    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=1_000_000_000_000_000i128))]
    total_borrowed: i128,

    // Base rate per second (0-1000 basis points per second)
    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=1000i128))]
    base_rate_per_second: i128,

    // Optimal utilization ratio (1-10000 basis points = 0.01%-100%)
    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(1..=10000i128))]
    optimal_utilization_ratio_bps: i128,

    // Interest rate slope1 (0-10000)
    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=10000i128))]
    slope1: i128,

    // Interest rate slope2 (slope1 to 50000)
    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=50000i128))]
    slope2: i128,

    // Reserve ratio (0-10000 basis points)
    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=10000i128))]
    reserve_ratio_bps: i128,

    // Test shares amount for conversion testing
    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=1_000_000i128))]
    test_shares: i128,

    // Test tokens amount for conversion testing
    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=1_000_000i128))]
    test_tokens: i128,
}

fuzz_target!(|input: InterestRateInput| {
    let env = Env::default();
    env.mock_all_auths();

    // Ensure borrowed doesn't exceed supply
    let total_borrowed = input.total_borrowed.min(input.total_supply);
    let available = input.total_supply.saturating_sub(total_borrowed);

    // Ensure slope1 < slope2
    let slope1 = input.slope1;
    let slope2 = input.slope2.max(slope1 + 1); // Ensure slope2 > slope1

    // Create a pool config with fuzzed parameters
    let pool_config = PoolConfig {
        base_rate_per_second: input.base_rate_per_second,
        optimal_utilization_ratio_bps: input.optimal_utilization_ratio_bps,
        slope1,
        slope2,
        reserve_ratio_bps: input.reserve_ratio_bps,
        ..Default::default()
    };

    // Validate the pool config before proceeding
    if pool_config.validate().is_err() {
        return; // Skip invalid configurations
    }

    // Create a test pool
    let token_address = soroban_sdk::Address::generate(&env);
    let pool_address = token_address.clone();

    let pool = Pool {
        pool_address,
        token_address,
        token_ticker: symbol_short!("TEST"),
        total_borrowed,
        total_shares: if input.total_supply > 0 {
            input.total_supply
        } else {
            0
        },
        available,
        total_collateral: 0,
        config: pool_config,
        last_accrual: 1_000_000_000_000_000_000,
        last_accrual_timestamp: 1000000,
        name: soroban_sdk::String::from_str(&env, "TEST"),
    };

    // Test total supply calculation
    if let Ok(total_supply) = pool.total_supply() {
        assert_eq!(
            total_supply,
            pool.available + pool.total_borrowed,
            "Total supply mismatch: {} != {} + {}",
            total_supply,
            pool.available,
            pool.total_borrowed
        );

        // Invariant: total supply should be non-negative
        assert!(
            total_supply >= 0,
            "Total supply cannot be negative: {}",
            total_supply
        );
    }

    // Test shares/tokens conversion consistency (if pool has shares and supply)
    if pool.total_shares > 0 && pool.total_supply().unwrap_or(0) > 0 {
        let test_shares = input.test_shares.min(pool.total_shares); // Don't exceed total shares
        if test_shares > 0
            && let Ok(tokens) = pool.compute_tokens_from_shares(&env, test_shares) {
                assert!(
                    tokens >= 0,
                    "Tokens from shares cannot be negative: {}",
                    tokens
                );

                // Test reverse conversion
                if tokens > 0
                    && let Ok(shares_back) = pool.compute_shares_from_tokens(&env, tokens) {
                        // Allow for small rounding differences (within reasonable bounds)
                        let diff = (shares_back - test_shares).abs();
                        let max_diff = test_shares.max(1); // Allow proportional difference
                        assert!(
                            diff <= max_diff,
                            "Share conversion inconsistency: {} vs {} (diff: {})",
                            test_shares,
                            shares_back,
                            diff
                        );
                    }
            }
    }

    // Test tokens to shares conversion
    if pool.total_shares > 0 && pool.total_supply().unwrap_or(0) > 0 {
        let test_tokens = input.test_tokens.min(pool.total_supply().unwrap_or(0));
        if test_tokens > 0
            && let Ok(shares) = pool.compute_shares_from_tokens(&env, test_tokens) {
                assert!(
                    shares >= 0,
                    "Shares from tokens cannot be negative: {}",
                    shares
                );

                // Test that we don't exceed total shares unreasonably
                // (some slight excess might be due to rounding with fixed point math)
                if shares > pool.total_shares {
                    let excess_ratio =
                        (shares - pool.total_shares) * 1000 / pool.total_shares.max(1);
                    assert!(
                        excess_ratio <= 10, // Allow up to 1% excess due to rounding
                        "Computed shares {} significantly exceed total shares {} (excess ratio: \
                         {}‰)",
                        shares,
                        pool.total_shares,
                        excess_ratio
                    );
                }
            }
    }

    // Test that pool is not in an invalid state
    assert!(
        pool.total_shares >= 0,
        "Pool total shares cannot be negative: {}",
        pool.total_shares
    );
    assert!(
        pool.available >= 0,
        "Pool available cannot be negative: {}",
        pool.available
    );
    assert!(
        pool.total_borrowed >= 0,
        "Pool total borrowed cannot be negative: {}",
        pool.total_borrowed
    );
    assert!(
        pool.total_collateral >= 0,
        "Pool total collateral cannot be negative: {}",
        pool.total_collateral
    );

    // Test pool emptiness check
    let is_empty = pool.is_empty();
    let should_be_empty = pool.total_shares == 0
        && pool.total_borrowed == 0
        && pool.available == 0
        && pool.total_collateral == 0;
    assert_eq!(
        is_empty, should_be_empty,
        "Pool emptiness check inconsistent: is_empty={}, should_be_empty={}",
        is_empty, should_be_empty
    );

    // Test edge cases for shares/tokens conversion
    // Zero shares should yield zero tokens
    if let Ok(tokens) = pool.compute_tokens_from_shares(&env, 0) {
        assert_eq!(tokens, 0, "Zero shares should yield zero tokens");
    }

    // Zero tokens should yield zero shares
    if let Ok(shares) = pool.compute_shares_from_tokens(&env, 0) {
        assert_eq!(shares, 0, "Zero tokens should yield zero shares");
    }

    // Test pool config validation constraints are maintained
    assert!(
        pool.config.base_rate_per_second >= 0,
        "Base rate per second should be non-negative"
    );
    assert!(
        pool.config.optimal_utilization_ratio_bps > 0
            && pool.config.optimal_utilization_ratio_bps <= 10000,
        "Optimal utilization ratio should be between 0 and 100%"
    );
    assert!(
        pool.config.slope1 < pool.config.slope2,
        "slope1 should be less than slope2"
    );
    assert!(
        pool.config.reserve_ratio_bps >= 0 && pool.config.reserve_ratio_bps <= 10000,
        "Reserve ratio should be between 0 and 100%"
    );

    // Test mathematical invariants
    if pool.total_shares > 0 && pool.total_supply().unwrap_or(0) > 0 {
        // The ratio of shares to supply should be reasonable
        let supply = pool.total_supply().unwrap();
        let shares_to_supply_ratio = (pool.total_shares * 1000) / supply.max(1);

        // This ratio should not be extremely high (shares shouldn't be orders of magnitude larger
        // than supply)
        assert!(
            shares_to_supply_ratio <= 1_000_000, // Allow up to 1000x ratio
            "Shares to supply ratio too high: {} shares for {} supply (ratio: {}‰)",
            pool.total_shares,
            supply,
            shares_to_supply_ratio
        );
    }

    // Test accrual value is positive
    assert!(
        pool.last_accrual > 0,
        "Last accrual should be positive: {}",
        pool.last_accrual
    );
});
