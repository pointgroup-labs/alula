# Miscellaneous

## Data Types

### `PoolConfig`

`PoolConfig` groups its parameters into sub-configs. For detailed descriptions of each parameter, see [Configurable Parameters](../tech-docs/deep-dive/configurable-parameters.md).

```rust
pub struct PoolConfig {
    pub health_config: PoolHealthConfig,
    pub fee_config: PoolFeeConfig,
    pub ir_config: KinkedIRConfig,
    pub take_rate_beneficiaries: Vec<(Address, i128)>,
    pub origination_beneficiaries: Vec<(Address, i128)>,
}
```

### `PoolHealthConfig`

```rust
pub struct PoolHealthConfig {
    pub supply_limit: i128,                    // Hard cap on total liquidity (0 = unlimited)
    pub utilization_ratio_limit_bps: i128,     // Utilization threshold for withdrawal throttle
    pub withdraw_scarcity_limit_bps: i128,     // Max % of supply withdrawable per tx under throttle
    pub withdraw_scarcity_cooldown_s: u64,     // Min seconds between withdrawals under throttle
    pub open_ltv_bps: i128,                    // Max LTV at borrow (e.g., 7500 = 75%)
    pub close_ltv_bps: i128,                   // Liquidation threshold (e.g., 8500 = 85%)
    pub liability_factor_bps: i128,            // Risk weight for debt (e.g., 12000 = 1.2×)
    pub liquidation_close_factor_bps: i128,    // Max % of debt repayable per liquidation
    pub max_liquidation_incentive_bps: i128,   // Max collateral discount for liquidators
    pub insolvency_ltv_bps: i128,              // LTV threshold for insolvency handling
}
```

### `PoolFeeConfig`

```rust
pub struct PoolFeeConfig {
    pub borrow_fee_bps: u32,                   // One-time borrow fee
    pub flash_loan_fee_bps: u32,               // Flash loan fee
    pub deposit_fee_bps: u32,                  // Deposit friction fee (usually 0)
    pub withdraw_fee_bps: u32,                 // Withdrawal friction fee (usually 0)
    pub withdraw_scarcity_fee_sc_bps: u32,     // Extra fee scalar during high utilization
    pub add_collateral_fee_bps: u32,           // Add-collateral friction fee (usually 0)
    pub remove_collateral_fee_bps: u32,        // Remove-collateral friction fee (usually 0)
    pub repay_fee_bps: u32,                    // Repayment friction fee (usually 0)
    pub take_rate_bps: u32,                    // % of borrower interest diverted as revenue
}
```

### `KinkedIRConfig`

```rust
pub struct KinkedIRConfig {
    pub base_apr_bps: i128,       // Minimum borrow APR regardless of utilization
    pub kink1_ur_bps: i128,       // First utilization kink (e.g., 8000 = 80%)
    pub kink1_apr_bps: i128,      // APR at kink1
    pub kink2_ur_bps: i128,       // Second utilization kink (e.g., 9000 = 90%)
    pub kink2_apr_bps: i128,      // APR at kink2
    pub max_apr_bps: i128,        // Max APR at 100% utilization
}
```

### `GlobalState`

```rust
pub struct GlobalState {
    pub status: u32,
    pub name: String,
    pub is_owned: bool,
    pub admin: Address,
    pub oracle: Address,
    pub deployer: Address,
    pub max_positions: u32,
    pub insolvency_ltv_bps: i128,
    pub min_collateral_value: i128,
    pub update_in_queue_period: Option<u64>,
}
```

### `Obligation`

```rust
pub struct Obligation {
    pub deposits: Map<Address, DepositPosition>,
    pub borrows: Map<Address, BorrowPosition>,
    pub collaterals: Map<Address, i128>,
}
```

### Types pending documentation

The following types are used in API function signatures but their struct definitions are not yet documented. They will be added in a future update.

| Type | Used by |
| ---- | ------- |
| `DepositPosition` | Field in `Obligation` |
| `BorrowPosition` | Field in `Obligation` |
| `ObligationKey` | `get_all_obligations` |
| `Pool` | `get_pool` |
| `PoolData` | `get_pool_data` |
| `PoolUpdate` | `get_pool_config_queued_in_update` |
| `MarketData` | `get_market_data` |
| `MultiplyPair` | `get_multiply_pair`, `get_all_multiply_pairs` |
| `WithdrawResult` | `simulate_withdraw`, `simulate_earn_obligation_withdraw` |
| `Request` | `submit_requests_batch` |
| `PendingUpgrade` | `get_pending_upgrade` |

***

## Error Codes

See `error.rs` for complete error definitions. Common errors:

| Error                           | Description                             |
| ------------------------------- | --------------------------------------- |
| `ObligationIsHealthy`           | Cannot liquidate healthy position       |
| `ObligationIsNotHealthy`        | Operation would make position unhealthy |
| `LiquidationExceedsCloseFactor` | Exceeds max liquidatable amount         |
| `OracleStalePrice`              | Oracle price too old                    |
| `PoolDoesNotExist`              | Invalid pool address                    |
| `InsufficientLiquidity`         | Not enough liquidity in pool            |
