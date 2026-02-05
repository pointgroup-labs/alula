# Miscellaneous

## Data Types

### `PoolConfig`

```rust
pub struct PoolConfig {
    pub open_ltv_bps: i128,              // Max LTV at borrow (default: 7000 = 70%)
    pub close_ltv_bps: i128,             // Liquidation threshold (default: 8000 = 80%)
    pub liability_factor_bps: i128,      // Risk weight for debt (default: 10000 = 100%)
    pub base_rate_bps: i128,             // Base interest rate
    pub slope1_bps: i128,                // Slope before kink1
    pub slope2_bps: i128,                // Slope between kink1 and kink2
    pub slope3_bps: i128,                // Slope after kink2
    pub kink1_utilization_bps: i128,     // First kink point (default: 7000 = 70%)
    pub kink2_utilization_bps: i128,     // Second kink point (default: 8000 = 80%)
    pub reserve_ratio_bps: i128,         // Protocol reserve (default: 1000 = 10%)
    pub borrow_fee_bps: i128,            // One-time borrow fee
    pub flash_loan_fee_bps: i128,        // Flash loan fee
    pub liquidation_bonus_bps: i128,     // Liquidator bonus (default: 1000 = 10%)
    pub liquidation_close_factor_bps: i128, // Max liquidatable (default: 5000 = 50%)
    pub borrow_enabled: bool,
    pub deposit_enabled: bool,
    pub supply_limit: i128,
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
