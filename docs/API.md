# Market Contract API

Complete API reference for the Alula Market smart contract.

## User Operations

### Deposits & Withdrawals

#### `deposit`

Supply assets to a lending pool and receive j-tokens (supply shares).

```rust
fn deposit(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

| Parameter      | Type      | Description                    |
| -------------- | --------- | ------------------------------ |
| `user`         | `Address` | User making the deposit        |
| `pool_address` | `Address` | Target pool address            |
| `amount`       | `i128`    | Amount to deposit (7 decimals) |

---

#### `deposit_earn`

Deposit into an isolated "Earn" obligation — deposit-only, no borrowing allowed.

```rust
fn deposit_earn(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

---

#### `withdraw`

Redeem j-tokens for underlying assets. Amount is capped to maintain healthy LTV.

```rust
fn withdraw(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

| Parameter | Type   | Description                                                               |
| --------- | ------ | ------------------------------------------------------------------------- |
| `amount`  | `i128` | Desired withdrawal amount. Use `i128::MAX` to withdraw maximum available. |

---

#### `withdraw_earn`

Withdraw from an Earn obligation.

```rust
fn withdraw_earn(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

---

#### `simulate_withdraw`

Simulate a withdrawal to preview fees and actual amount. Read-only, no state changes.

```rust
fn simulate_withdraw(user: Address, pool_address: Address, amount: i128) -> Result<WithdrawResult, MCError>
```

---

#### `simulate_earn_withdraw`

Simulate withdrawal from Earn obligation.

```rust
fn simulate_earn_withdraw(user: Address, pool_address: Address, amount: i128) -> Result<WithdrawResult, MCError>
```

---

### Collateral Management

#### `add_collateral`

Lock assets as collateral. Collateral doesn't earn interest but is always available for healthy withdrawals.

```rust
fn add_collateral(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

---

#### `remove_collateral`

Unlock and withdraw collateral. Amount is capped to maintain healthy LTV.

```rust
fn remove_collateral(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

---

### Borrowing

#### `borrow`

Borrow assets against your collateral.

```rust
fn borrow(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

---

#### `repay`

Repay borrowed assets. Use `i128::MAX` to repay entire debt.

```rust
fn repay(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

---

### Liquidation

#### `liquidate`

Liquidate an unhealthy position (health factor < 1.0). Liquidator repays debt and receives collateral at a discount.

```rust
fn liquidate(
    liquidator: Address,
    borrower: Address,
    borrower_obligation_seed: Option<BytesN<32>>,
    borrow_pool_address: Address,
    collateral_pool_address: Address,
    repay_amount: i128,
    demanded_collateral_amount: i128,
) -> Result<(), MCError>
```

| Parameter                    | Type                 | Description                        |
| ---------------------------- | -------------------- | ---------------------------------- |
| `liquidator`                 | `Address`            | Address performing liquidation     |
| `borrower`                   | `Address`            | Address being liquidated           |
| `borrower_obligation_seed`   | `Option<BytesN<32>>` | Seed for multiply pair obligations |
| `borrow_pool_address`        | `Address`            | Pool of debt being repaid          |
| `collateral_pool_address`    | `Address`            | Pool of collateral being seized    |
| `repay_amount`               | `i128`               | Amount of debt to repay            |
| `demanded_collateral_amount` | `i128`               | Minimum collateral expected        |

---

### Flash Loans

#### `flash_loan`

Borrow without collateral — must repay within the same transaction. Follows ERC-3156 standard.

```rust
fn flash_loan(
    contract: Address,
    caller: Address,
    pool_address: Address,
    amount: i128,
) -> Result<(), MCError>
```

| Parameter      | Type      | Description                               |
| -------------- | --------- | ----------------------------------------- |
| `contract`     | `Address` | Contract implementing `FlashLoanReceiver` |
| `caller`       | `Address` | Original caller (for auth)                |
| `pool_address` | `Address` | Pool to borrow from                       |
| `amount`       | `i128`    | Amount to borrow                          |

---

### Leveraged Positions

#### `deposit_with_leverage`

Create a leveraged position using flash loans and swaps.

```rust
fn deposit_with_leverage(
    user: Address,
    deposit_pool_address: Address,
    borrow_pool_address: Address,
    deposit_as_margin: bool,
    amount: i128,
    leverage_multiplier: u32,
) -> Result<(), MCError>
```

| Parameter             | Type   | Description                                                    |
| --------------------- | ------ | -------------------------------------------------------------- |
| `deposit_as_margin`   | `bool` | If true, margin is in deposit asset; otherwise in borrow asset |
| `leverage_multiplier` | `u32`  | Multiplier × 100 (e.g., 300 = 3x, 550 = 5.5x)                  |

---

#### `withdraw_from_leveraged`

Close or reduce a leveraged position.

```rust
fn withdraw_from_leveraged(
    user: Address,
    deposit_pool_address: Address,
    borrow_pool_address: Address,
    amount: i128,
) -> Result<(), MCError>
```

---

### Batch Operations

#### `submit_requests_batch`

Submit multiple operations in a single transaction.

```rust
fn submit_requests_batch(user: Address, requests: Vec<Request>) -> Result<(), MCError>
```

---

## Query Functions

### Obligations

#### `get_user_obligation`

Get user's standard obligation (deposits, collateral, borrows).

```rust
fn get_user_obligation(user: Address) -> Result<Obligation, MCError>
```

---

#### `get_earn_user_obligation`

Get user's Earn obligation (deposit-only).

```rust
fn get_earn_user_obligation(user: Address) -> Result<Obligation, MCError>
```

---

#### `get_multiply_pair_obligation`

Get user's obligation for a specific leveraged pair.

```rust
fn get_multiply_pair_obligation(
    user: Address,
    deposit_pool_address: Address,
    borrow_pool_address: Address,
) -> Result<Obligation, MCError>
```

---

#### `get_all_obligations`

Get all obligation keys in the market.

```rust
fn get_all_obligations() -> Vec<ObligationKey>
```

---

### Pools

#### `get_pool`

Get pool state and configuration.

```rust
fn get_pool(pool_address: Address) -> Result<Pool, MCError>
```

---

#### `get_pool_data`

Get pool data with computed APYs. For simulations only.

```rust
fn get_pool_data(pool_address: Address) -> Result<PoolData, MCError>
```

---

#### `get_all_pools`

Get all pool addresses.

```rust
fn get_all_pools() -> Vec<Address>
```

---

#### `get_pool_asset_oracle_price`

Get current oracle price for a pool's asset.

```rust
fn get_pool_asset_oracle_price(pool_address: Address) -> Result<i128, MCError>
```

---

### Multiply Pairs

#### `get_multiply_pair`

Get a specific multiply pair configuration.

```rust
fn get_multiply_pair(
    deposit_pool_address: Address,
    borrow_pool_address: Address,
) -> Result<MultiplyPair, MCError>
```

---

#### `get_all_multiply_pairs`

Get all registered multiply pairs.

```rust
fn get_all_multiply_pairs() -> Vec<MultiplyPair>
```

---

### Market

#### `get_global_state`

Get market's global configuration.

```rust
fn get_global_state() -> GlobalState
```

---

#### `get_market_data`

Get comprehensive market data including all pools. For simulations only.

```rust
fn get_market_data() -> Result<MarketData, MCError>
```

---

#### `get_asset_decimals`

Returns `7` (Stellar standard).

```rust
fn get_asset_decimals() -> u32
```

---

#### `get_oracle_price_decimals`

Get oracle price decimals (typically `14`).

```rust
fn get_oracle_price_decimals() -> u32
```

---

### Refresh (Interest Accrual)

#### `refresh_pool`

Manually accrue interest on a pool.

```rust
fn refresh_pool(pool_address: Address) -> Result<(), MCError>
```

---

#### `refresh_obligation`

Accrue interest on all pools in a user's obligation.

```rust
fn refresh_obligation(user: Address) -> Result<(), MCError>
```

---

#### `refresh_earn_obligation`

Accrue interest for Earn obligation.

```rust
fn refresh_earn_obligation(user: Address) -> Result<(), MCError>
```

---

#### `refresh_multiply_pair_obligation`

Accrue interest for multiply pair obligation.

```rust
fn refresh_multiply_pair_obligation(
    user: Address,
    deposit_pool_address: Address,
    borrow_pool_address: Address,
) -> Result<(), MCError>
```

---

## Admin Functions

### Pool Management

#### `initialize_pool`

Create a new lending pool for an asset.

```rust
fn initialize_pool(
    token_address: Address,
    salt: Option<BytesN<32>>,
    pool_config: Option<PoolConfig>,
) -> Result<Address, MCError>
```

---

#### `initialize_multiply_pair`

Register a new leveraged pair.

```rust
fn initialize_multiply_pair(
    deposit_pool_address: Address,
    borrow_pool_address: Address,
) -> Result<(), MCError>
```

---

#### `bootstrap_pool`

Add incentives to bootstrap a pool's supply.

```rust
fn bootstrap_pool(
    pool_address: Address,
    sponsor: Address,
    amount: i128,
    start_period: u64,
    end_period: u64,
) -> Result<(), MCError>
```

---

### Pool Configuration (Time-Locked)

#### `queue_in_pool_config_update`

Queue a pool configuration change (24h delay).

```rust
fn queue_in_pool_config_update(
    pool_address: Address,
    new_pool_config: PoolConfig,
) -> Result<(), MCError>
```

---

#### `apply_pool_config_update`

Apply a queued configuration after the delay period.

```rust
fn apply_pool_config_update(pool_address: Address) -> Result<(), MCError>
```

---

#### `cancel_pool_config_update`

Cancel a pending configuration update.

```rust
fn cancel_pool_config_update(pool_address: Address) -> Result<(), MCError>
```

---

#### `get_pool_config_queued_in_update`

Get pending configuration update for a pool.

```rust
fn get_pool_config_queued_in_update(pool_address: Address) -> Result<PoolUpdate, MCError>
```

---

### Market Management

#### `update_market`

Update market-level parameters.

```rust
fn update_market(
    new_max_positions: u32,
    new_min_collateral_value: i128,
) -> Result<(), MCError>
```

---

#### `update_market_status`

Change market status (Active, BorrowFrozen, DepositFrozen, Frozen).

```rust
fn update_market_status(new_status: u32) -> Result<(), MCError>
```

| Status        | Value | Description                     |
| ------------- | ----- | ------------------------------- |
| Active        | `0`   | All operations enabled          |
| BorrowFrozen  | `1`   | Borrowing disabled              |
| DepositFrozen | `2`   | Deposits and borrowing disabled |
| Frozen        | `3`   | Only liquidations allowed       |

---

### Upgrades

#### `upgrade`

Upgrade the contract to a new WASM binary (admin only).

```rust
fn upgrade(new_wasm_hash: BytesN<32>)
```

---

### Fees

#### `distribute_pool_fees`

Distribute accumulated fees for a specific pool to beneficiaries.

```rust
fn distribute_pool_fees(pool_address: Address) -> Result<(), MCError>
```

---

#### `distribute_all_pools_fees`

Distribute accumulated fees for all pools to beneficiaries.

```rust
fn distribute_all_pools_fees() -> Result<(), MCError>
```

---

#### `set_take_rate_fees_beneficiaries`

Configure beneficiaries for take rate fees.

```rust
fn set_take_rate_fees_beneficiaries(
    pool_address: Address,
    beneficiaries: Vec<FeeBeneficiary>,
) -> Result<(), MCError>
```

---

#### `set_operation_fees_beneficiaries`

Configure beneficiaries for operation fees (borrow fees, flash loan fees).

```rust
fn set_operation_fees_beneficiaries(beneficiaries: Vec<FeeBeneficiary>) -> Result<(), MCError>
```

---

### Bad Debt

#### `issue_cover_bad_debt`

Request coverage for bad debt from the insurance fund.

```rust
fn issue_cover_bad_debt(user: Address) -> Result<(), MCError>
```

---

#### `issue_cover_bad_debt_pair`

Request coverage for bad debt on a multiply pair obligation.

```rust
fn issue_cover_bad_debt_pair(
    user: Address,
    deposit_pool_address: Address,
    borrow_pool_address: Address,
) -> Result<(), MCError>
```

---

#### `claim_cover_bad_debt_results`

Claim coverage results after insurance fund processes the request.

```rust
fn claim_cover_bad_debt_results(user: Address) -> Result<(), MCError>
```

---

#### `claim_cover_bad_debt_result_pair`

Claim coverage results for a multiply pair obligation.

```rust
fn claim_cover_bad_debt_result_pair(
    user: Address,
    deposit_pool_address: Address,
    borrow_pool_address: Address,
) -> Result<(), MCError>
```

---

### Miscellaneous

#### `donate`

Donate tokens to a pool's reserve.

```rust
fn donate(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

---

#### `swap`

Swap tokens via integrated swap provider.

```rust
fn swap(
    user: Address,
    token_in: Address,
    token_out: Address,
    amount_in: i128,
) -> Result<i128, MCError>
```

---

### Farms Integration

#### `set_farms_contract`

Set the farms contract address for delegated staking.

```rust
fn set_farms_contract(farms_contract: Address) -> Result<(), MCError>
```

---

#### `clear_farms_contract`

Remove farms contract integration.

```rust
fn clear_farms_contract() -> Result<(), MCError>
```

---

#### `get_farms_contract`

Get the configured farms contract address.

```rust
fn get_farms_contract() -> Option<Address>
```

---

#### `set_pool_supply_farm`

Configure a supply (j-token) farm for a pool.

```rust
fn set_pool_supply_farm(pool_address: Address, farm_id: BytesN<32>) -> Result<(), MCError>
```

---

#### `set_pool_debt_farm`

Configure a debt (d-token) farm for a pool.

```rust
fn set_pool_debt_farm(pool_address: Address, farm_id: BytesN<32>) -> Result<(), MCError>
```

---

#### `clear_pool_farms`

Clear all farm configuration for a pool.

```rust
fn clear_pool_farms(pool_address: Address) -> Result<(), MCError>
```

---

#### `refresh_obligation_farms`

Sync all farm stakes for a user's standard obligation.

```rust
fn refresh_obligation_farms(user: Address) -> Result<(), MCError>
```

---

#### `refresh_earn_obligation_farms`

Sync all farm stakes for a user's Earn obligation.

```rust
fn refresh_earn_obligation_farms(user: Address) -> Result<(), MCError>
```

---

#### `refresh_multiply_pair_farms`

Sync all farm stakes for a user's multiply pair obligation.

```rust
fn refresh_multiply_pair_farms(
    user: Address,
    deposit_pool_address: Address,
    borrow_pool_address: Address,
) -> Result<(), MCError>
```

---

### Admin Transfer

#### `propose_new_admin`

Propose a new admin address. The new admin must accept.

```rust
fn propose_new_admin(new_admin: Address) -> Result<(), MCError>
```

---

#### `accept_proposed_admin`

Accept the admin role (called by the proposed new admin).

```rust
fn accept_proposed_admin() -> Result<(), MCError>
```

---

### Pool Status

#### `update_pool_status`

Update a pool's operational status.

```rust
fn update_pool_status(pool_address: Address, new_status: u32) -> Result<(), MCError>
```

---

#### `fund_update_market_status`

Allow the insurance fund to update market status (emergency freeze).

```rust
fn fund_update_market_status(new_status: u32) -> Result<(), MCError>
```

---

## Data Types

### PoolConfig

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

### GlobalState

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

### Obligation

```rust
pub struct Obligation {
    pub deposits: Map<Address, DepositPosition>,
    pub borrows: Map<Address, BorrowPosition>,
    pub collaterals: Map<Address, i128>,
}
```

---

## Error Codes

See [`error.rs`](../contracts/market/src/error.rs) for complete error definitions.

Common errors:

| Error                           | Description                             |
| ------------------------------- | --------------------------------------- |
| `ObligationIsHealthy`           | Cannot liquidate healthy position       |
| `ObligationIsNotHealthy`        | Operation would make position unhealthy |
| `LiquidationExceedsCloseFactor` | Exceeds max liquidatable amount         |
| `OracleStalePrice`              | Oracle price too old                    |
| `PoolDoesNotExist`              | Invalid pool address                    |
| `InsufficientLiquidity`         | Not enough liquidity in pool            |
