# User Operations

## Deposits & Withdrawals

### **`deposit`**

Supply assets to a pool and receive jTokens (supply shares).

```rust
fn deposit(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

| Parameter      | Type      | Description                    |
| -------------- | --------- | ------------------------------ |
| `user`         | `Address` | User making the deposit        |
| `pool_address` | `Address` | Target pool address            |
| `amount`       | `i128`    | Amount to deposit (7 decimals) |

***

### **`deposit_into_earn_obligation`**

Deposit into an isolated _Earn_ obligation. Deposit-only, no borrowing allowed.

```rust
fn deposit_into_earn_obligation(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

***

### **`withdraw`**

Redeem jTokens for underlying assets. Amount is capped to maintain healthy LTV.

```rust
fn withdraw(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

| Parameter | Type   | Description                                                               |
| --------- | ------ | ------------------------------------------------------------------------- |
| `amount`  | `i128` | Desired withdrawal amount. Use `i128::MAX` to withdraw maximum available. |

***

### **`withdraw_from_earn_obligation`**

Withdraw from an _Earn_ obligation.

```rust
fn withdraw_from_earn_obligation(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

***

### **`simulate_withdraw`**

Simulate a withdrawal to preview fees and actual amount. Read-only, no state changes.

```rust
fn simulate_withdraw(user: Address, pool_address: Address, amount: i128) -> Result<WithdrawResult, MCError>
```

***

### **`simulate_earn_obligation_withdraw`**

Simulate withdrawal from an _Earn_ obligation.

```rust
fn simulate_earn_obligation_withdraw(user: Address, pool_address: Address, amount: i128) -> Result<WithdrawResult, MCError>
```

***

## Collateral Management

### **`add_collateral`**

Lock assets as collateral. Collateral doesn't earn interest but is always available for healthy withdrawals.

```rust
fn add_collateral(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

***

### **`remove_collateral`**

Unlock and withdraw collateral. Amount is capped to maintain healthy LTV.

```rust
fn remove_collateral(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

***

## Borrows

### **`borrow`**

Borrow assets against collateral.

```rust
fn borrow(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

***

### **`repay`**

Repay borrowed assets. Use `i128::MAX` to repay entire debt.

```rust
fn repay(user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>
```

***

## Liquidation

### **`liquidate`**

Liquidate an unhealthy position (liquidation health factor < 1.0). Liquidator repays debt and receives collateral at a discount.

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

***

## Flash Loans

### **`flash_loan`**

Borrow without collateral. The loan must be repaid within the same transaction. Follows ERC-3156 standard.

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

***

## Leveraged Positions

### **`deposit_with_leverage`**

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

***

### **`withdraw_from_leveraged`**

Close or reduce a leveraged position.

```rust
fn withdraw_from_leveraged(
    user: Address,
    deposit_pool_address: Address,
    borrow_pool_address: Address,
    amount: i128,
) -> Result<(), MCError>
```

***

## Batch Operations

### **`submit_requests_batch`**

Submit multiple operations in a single transaction.

```rust
fn submit_requests_batch(user: Address, requests: Vec<Request>) -> Result<(), MCError>
```
