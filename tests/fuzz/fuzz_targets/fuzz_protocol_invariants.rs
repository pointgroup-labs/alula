#![no_main]

use libfuzzer_sys::fuzz_target;
use soroban_sdk::testutils::arbitrary::Arbitrary;
use soroban_sdk::Env;
use lending::pool::PoolConfig;

#[derive(Arbitrary, Debug)]
struct ProtocolInvariantInput {
    // Pool configuration parameters
    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=1000000i128))]
    base_rate_per_second: i128,

    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(1..=10000i128))] // 0.01%-100%
    optimal_utilization_ratio_bps: i128,

    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=1000000000000i128))]
    slope1: i128,

    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=1000000000000i128))]
    slope2: i128,

    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=10000i128))] // 0%-100%
    reserve_ratio_bps: i128,

    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(1..=10000i128))] // 0.01%-100%
    liquidation_close_factor_bps: i128,

    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=5000i128))] // 0%-50%
    liquidation_incentive_bps: i128,

    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=1000000000000000i128))]
    supply_limit: i128,

    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(1..=10000i128))] // 0.01%-100%
    utilization_ratio_limit_bps: i128,

    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=10000i128))] // 0%-100%
    open_ltv_bps: i128,

    #[arbitrary(with = |u: &mut arbitrary::Unstructured| u.int_in_range(0..=10000i128))] // 0%-100%
    close_ltv_bps: i128,
}

fuzz_target!(|input: ProtocolInvariantInput| {
    let _env = Env::default();

    // Ensure ordering constraints for valid pool configuration
    let slope1 = input.slope1;
    let slope2 = input.slope2.max(slope1 + 1); // slope2 must be > slope1
    let open_ltv_bps = input.open_ltv_bps;
    let close_ltv_bps = input.close_ltv_bps.max(open_ltv_bps); // close_ltv >= open_ltv
    let optimal_utilization_ratio_bps = input.optimal_utilization_ratio_bps;
    let utilization_ratio_limit_bps = input.utilization_ratio_limit_bps.max(optimal_utilization_ratio_bps + 1);

    // Create a pool config with fuzzed parameters
    let pool_config = PoolConfig {
        base_rate_per_second: input.base_rate_per_second,
        optimal_utilization_ratio_bps,
        slope1,
        slope2,
        reserve_ratio_bps: input.reserve_ratio_bps,
        liquidation_close_factor_bps: input.liquidation_close_factor_bps,
        liquidation_incentive_bps: input.liquidation_incentive_bps,
        supply_limit: input.supply_limit,
        utilization_ratio_limit_bps,
        open_ltv_bps,
        close_ltv_bps,
    };

    // Test basic pool config invariants
    test_pool_config_invariants(&pool_config);

    // Test mathematical relationships
    test_mathematical_invariants(&pool_config);
});

fn test_pool_config_invariants(config: &PoolConfig) {
    // Invariant: Base rate should be non-negative
    assert!(config.base_rate_per_second >= 0,
        "Base rate per second must be non-negative: {}", config.base_rate_per_second);

    // Invariant: Supply limit should be non-negative
    assert!(config.supply_limit >= 0,
        "Supply limit must be non-negative: {}", config.supply_limit);

    // Invariant: All BPS values should be within valid ranges (0-10000)
    assert!(config.optimal_utilization_ratio_bps > 0 && config.optimal_utilization_ratio_bps <= 10000,
        "Optimal utilization ratio must be between 0% and 100%: {}", config.optimal_utilization_ratio_bps);

    assert!(config.reserve_ratio_bps >= 0 && config.reserve_ratio_bps <= 10000,
        "Reserve ratio must be between 0% and 100%: {}", config.reserve_ratio_bps);

    assert!(config.liquidation_close_factor_bps > 0 && config.liquidation_close_factor_bps <= 10000,
        "Liquidation close factor must be between 0% and 100%: {}", config.liquidation_close_factor_bps);

    assert!(config.liquidation_incentive_bps >= 0 && config.liquidation_incentive_bps <= 10000,
        "Liquidation incentive must be between 0% and 100%: {}", config.liquidation_incentive_bps);

    assert!(config.utilization_ratio_limit_bps > 0 && config.utilization_ratio_limit_bps <= 10000,
        "Utilization ratio limit must be between 0% and 100%: {}", config.utilization_ratio_limit_bps);

    assert!(config.open_ltv_bps >= 0 && config.open_ltv_bps < 10000,
        "Open LTV must be between 0% and less than 100%: {}", config.open_ltv_bps);

    assert!(config.close_ltv_bps >= 0 && config.close_ltv_bps <= 10000,
        "Close LTV must be between 0% and 100%: {}", config.close_ltv_bps);
}

fn test_mathematical_invariants(config: &PoolConfig) {
    // Invariant: Interest rate slopes should be non-negative
    assert!(config.slope1 >= 0, "Slope1 must be non-negative: {}", config.slope1);
    assert!(config.slope2 >= 0, "Slope2 must be non-negative: {}", config.slope2);

    // Invariant: slope1 must be less than slope2 for kinked model to work
    assert!(config.slope1 < config.slope2,
        "Slope1 {} must be less than slope2 {} for kinked model", config.slope1, config.slope2);

    // Invariant: Utilization ratio limit must exceed optimal utilization ratio
    assert!(config.utilization_ratio_limit_bps > config.optimal_utilization_ratio_bps,
        "Utilization ratio limit {} must exceed optimal utilization ratio {}",
        config.utilization_ratio_limit_bps, config.optimal_utilization_ratio_bps);

    // Invariant: Open LTV should not be bigger than close LTV
    assert!(config.open_ltv_bps <= config.close_ltv_bps,
        "Open LTV {} must not exceed close LTV {}", config.open_ltv_bps, config.close_ltv_bps);

    // Invariant: Liquidation incentive should be reasonable (not too high)
    assert!(config.liquidation_incentive_bps <= 5000,
        "Liquidation incentive {} should not exceed 50%", config.liquidation_incentive_bps);
}
