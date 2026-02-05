# Admin Operations

## Pool Management

### **`initialize_pool`**

Create a new lending pool for an asset.

```rust
fn initialize_pool(
    token_address: Address,
    salt: Option<BytesN<32>>,
    pool_config: Option<PoolConfig>,
) -> Result<Address, MCError>
```

***

### **`initialize_multiply_pair`**

Register a new leveraged pair.

```rust
fn initialize_multiply_pair(
    deposit_pool_address: Address,
    borrow_pool_address: Address,
) -> Result<(), MCError>
```

***

### **`bootstrap_pool`**

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

***

## Pool Configuration (Time-Locked)

### **`queue_in_pool_config_update`**

Queue a pool configuration change (24h delay).

```rust
fn queue_in_pool_config_update(
    pool_address: Address,
    new_pool_config: PoolConfig,
) -> Result<(), MCError>
```

***

### **`apply_pool_config_update`**

Apply a queued configuration after a delay.

```rust
fn apply_pool_config_update(pool_address: Address) -> Result<(), MCError>
```

***

### **`cancel_pool_config_update`**

Cancel a pending configuration update.

```rust
fn cancel_pool_config_update(pool_address: Address) -> Result<(), MCError>
```

***

### **`get_pool_config_queued_in_update`**

Get a pending configuration update for a pool.

```rust
fn get_pool_config_queued_in_update(pool_address: Address) -> Result<PoolUpdate, MCError>
```

***

## Market Management

### **`update_market`**

Update market-level parameters.

```rust
fn update_market(
    new_max_positions: u32,
    new_min_collateral_value: i128,
) -> Result<(), MCError>
```

***

### **`update_market_status`**

Change market status.

```rust
fn update_market_status(new_status: u32) -> Result<(), MCError>
```

| Status        | Value | Description                     |
| ------------- | ----- | ------------------------------- |
| Active        | `0`   | All operations enabled          |
| BorrowFrozen  | `1`   | Borrowing disabled              |
| DepositFrozen | `2`   | Deposits and borrowing disabled |
| Frozen        | `3`   | Only liquidations allowed       |

***

## Upgrades (Time-Locked)

### **`propose_upgrade`**

Propose a contract upgrade (7-day delay on mainnet).

```rust
fn propose_upgrade(new_wasm_hash: BytesN<32>) -> Result<(), MCError>
```

***

### **`execute_upgrade`**

Execute a proposed upgrade after timelock expires.

```rust
fn execute_upgrade() -> Result<(), MCError>
```

***

### **`cancel_upgrade`**

Cancel a pending upgrade.

```rust
fn cancel_upgrade() -> Result<(), MCError>
```

***

### **`get_pending_upgrade`**

Get pending upgrade details.

```rust
fn get_pending_upgrade() -> Option<PendingUpgrade>
```

***

### **`is_upgradable`**

Check if contract upgrades are enabled.

```rust
fn is_upgradable() -> bool
```

***

## Fees

### **`redeem_accumulated_market_fees`**

Withdraw accumulated market fees (admin only).

```rust
fn redeem_accumulated_market_fees(
    user: Address,
    pool_address: Address,
    amount: i128,
) -> Result<(), MCError>
```

***

### **`redeem_accumulated_host_fees`**

Withdraw accumulated host fees (deployer only).

```rust
fn redeem_accumulated_host_fees(
    user: Address,
    pool_address: Address,
    amount: i128,
) -> Result<(), MCError>
```

***

## Bad Debt

### **`cover_obligation_bad_debt`**

Cover bad debt from reserves, socialize remainder across lenders.

```rust
fn cover_obligation_bad_debt(bad_debt_obligation_user: Address) -> Result<(), MCError>
```

***

### **`cover_multiply_pair_bad_debt`**

Cover bad debt for a multiply pair obligation.

```rust
fn cover_multiply_pair_bad_debt(
    bad_debt_obligation_user: Address,
    deposit_pool_address: Address,
    borrow_pool_address: Address,
) -> Result<(), MCError>
```

***

## Miscellaneous

### **`donate_to_reserve`**

Donate tokens to a pool's insurance reserve.

```rust
fn donate_to_reserve(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

***

### **`swap`**

Swap tokens via integrated swap provider.

```rust
fn swap(
    user: Address,
    token_in: Address,
    token_out: Address,
    amount_in: i128,
) -> Result<i128, MCError>
```
