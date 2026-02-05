# Query Operations

## Obligations

### **`get_user_obligation`**

Get user's standard obligation (deposits, collateral, borrows).

```rust
fn get_user_obligation(user: Address) -> Result<Obligation, MCError>
```

***

### **`get_earn_user_obligation`**

Get user's _Earn_ obligation (deposit-only).

```rust
fn get_earn_user_obligation(user: Address) -> Result<Obligation, MCError>
```

***

### **`get_multiply_pair_obligation`**

Get user's obligation for a specific leveraged pair.

```rust
fn get_multiply_pair_obligation(
    user: Address,
    deposit_pool_address: Address,
    borrow_pool_address: Address,
) -> Result<Obligation, MCError>
```

***

### **`get_all_obligations`**

Get all obligation keys in the market.

```rust
fn get_all_obligations() -> Vec<ObligationKey>
```

***

## Pools

### **`get_pool`**

Get pool state and configuration.

```rust
fn get_pool(pool_address: Address) -> Result<Pool, MCError>
```

***

### **`get_pool_data`**

Get pool data with computed APYs. For simulations only.

```rust
fn get_pool_data(pool_address: Address) -> Result<PoolData, MCError>
```

***

### **`get_all_pools`**

Get all pool addresses.

```rust
fn get_all_pools() -> Vec<Address>
```

***

### **`get_pool_asset_oracle_price`**

Get current oracle price for a pool's asset.

```rust
fn get_pool_asset_oracle_price(pool_address: Address) -> Result<i128, MCError>
```

***

## Multiply Pairs

### **`get_multiply_pair`**

Get a specific multiply pair configuration.

```rust
fn get_multiply_pair(
    deposit_pool_address: Address,
    borrow_pool_address: Address,
) -> Result<MultiplyPair, MCError>
```

***

### **`get_all_multiply_pairs`**

Get all registered multiply pairs.

```rust
fn get_all_multiply_pairs() -> Vec<MultiplyPair>
```

***

## Market

### **`get_global_state`**

Get market's global configuration.

```rust
fn get_global_state() -> GlobalState
```

***

### **`get_market_data`**

Get comprehensive market data including all pools. For simulations only.

```rust
fn get_market_data() -> Result<MarketData, MCError>
```

***

### **`get_asset_decimals`**

Returns `7` (Stellar standard).

```rust
fn get_asset_decimals() -> u32
```

***

### **`get_oracle_price_decimals`**

Get oracle price decimals (typically `14`).

```rust
fn get_oracle_price_decimals() -> u32
```

***

## Refresh (Interest Accrual)

### **`refresh_pool`**

Manually accrue interest on a pool.

```rust
fn refresh_pool(pool_address: Address) -> Result<(), MCError>
```

***

### **`refresh_obligation`**

Accrue interest on all pools in a user's obligation.

```rust
fn refresh_obligation(user: Address) -> Result<(), MCError>
```

***

### **`refresh_earn_obligation`**

Accrue interest for an _Earn_ obligation.

```rust
fn refresh_earn_obligation(user: Address) -> Result<(), MCError>
```

***

### **`refresh_multiply_pair_obligation`**

Accrue interest for multiply pair obligation.

```rust
fn refresh_multiply_pair_obligation(
    user: Address,
    deposit_pool_address: Address,
    borrow_pool_address: Address,
) -> Result<(), MCError>
```
