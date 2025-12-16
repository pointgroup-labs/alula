# Alula Lending Protocol - Soroban Smart Contract Security Audit Report

---

## Executive Summary

| Severity          | Count | Status |
| ----------------- | ----- | ------ |
| **Critical**      | 0     | -      |
| **High**          | 0     | -      |
| **Medium**        | 3     | 🔍     |
| **Low**           | 4     | 🔍     |
| **Informational** | 5     | ✓      |

The Alula lending protocol demonstrates **solid security fundamentals**. Authorization is consistently implemented, arithmetic uses checked operations with proper overflow protection, and storage TTL management is well-implemented. The upgrade mechanism includes proper timelocks.

### Key Security Strengths

1. The contract uses Soroban's `require_auth()` correctly on all state-modifying operations
2. Comprehensive checked arithmetic with `map_over_or_underflow()` prevents silent overflows
3. Good use of TTL extensions on storage access patterns
4. Proper timelock mechanism for upgrades (7 days on mainnet)
5. Atomic initialization via `__constructor` prevents frontrunning

---

## Medium Severity Findings

### M-01: Flash Loan CEI Pattern Violation - Pool Reloaded After Callback

**Location:** `contracts/market/src/processors.rs:432-474`

**Description:** The `process_flash_loan()` function reloads the pool state AFTER the external callback completes, which is good for preserving callback changes.
However, the pool is NOT marked as having an active flash loan before the callback:

```rust
pub fn process_flash_loan(e: &Env, contract: &Address, pool_address: &Address, amount: i128) -> Result<(), MCError> {
    require_positive(amount)?;

    let pool = Pool::try_get(e, pool_address)?;
    pool.require_total_available(amount)?;

    // Tokens transferred out...
    token_client.transfer(&e.current_contract_address(), contract, &amount);

    // External callback - pool state shows stale total_available
    flash_loan_taker_client.exec_op(...);

    // Pool reloaded AFTER callback (good for preserving changes)
    let mut pool = Pool::try_get(e, pool_address)?;
    pool.adjust_accumulated_market_fees(e, fees)?;
    pool.set(e);
}
```

**Impact:** During the flash loan callback, any code reading pool state via `Pool::try_get()` will see `total_available` that includes the flash-loaned amount (since it wasn't decremented). This could:

- Allow borrowing against stale collateral calculations
- Cause incorrect health factor readings during callback
- Enable manipulation of pool utilization ratio calculations

**Severity Justification:** Medium - requires malicious callback contract and specific interaction patterns.

**Recommendation:** Either:

1. Decrement `total_available` before callback and restore after, OR
2. Add a "flash loan in progress" flag that other operations check

---

### M-02: Fee Calculation Rounding Direction Favors Users

**Location:** `contracts/market/src/obligation.rs:1339-1351`

**Description:** The `compute_fees()` function uses `fixed_mul_floor` for fee calculations:

```rust
pub fn compute_fees(original_amount: i128, operation_fee_bps: u32, host_fee_bps: u32) -> Result<ComputedFees, MCError> {
    let fee_sum = original_amount
        .fixed_mul_floor(operation_fee_bps as i128, BPS_FACTOR)  // Floor rounds DOWN
        .map_over_or_underflow()?;
    let host_fee =
        fee_sum.fixed_mul_floor(host_fee_bps as i128, BPS_FACTOR).map_over_or_underflow()?;
    // ...
}
```

**Impact:** Per security best practices, **fee calculations should round UP (ceil) to favor the protocol**.
Using floor rounding means the protocol systematically receives slightly less fees. Over millions of transactions, this compounds.

**Recommendation:** Change to `fixed_mul_ceil`:

```rust
let fee_sum = original_amount
    .fixed_mul_ceil(operation_fee_bps as i128, BPS_FACTOR)
    .map_over_or_underflow()?;
```

---

### M-03: First Depositor Share Inflation - Partial Mitigation

**Location:** `contracts/market/src/pool.rs:400-422` and `constants.rs:134`

**Description:** The protocol uses `INITIAL_SHARES_AMOUNT = 10^5` for the first deposit:

```rust
let shares_amount = if total_shares_amount == 0 {
    INITIAL_SHARES_AMOUNT  // 100,000 shares for first deposit
} else {
    // ... proportional calculation
}
```

**Context:** The previous audit noted this is mitigated because the protocol uses internal accounting (`total_available`) rather than actual token balances.
**This is correct** - direct token donations don't affect share ratios.

**Remaining Risk:** However, the initial share amount still provides limited precision protection. With 7 decimal places standard, an attacker could:

1. First deposit: 1 unit → receives 100,000 shares
2. Second transaction: Deposit a large amount via the protocol
3. The ratio between subsequent deposits and the inflated first deposit could still cause rounding losses

**Severity Justification:** Medium-Low - attack is economically constrained but theoretically possible.

**Recommendation:** Consider:

1. Setting a minimum first deposit amount (e.g., `MIN_FIRST_DEPOSIT = 10^6`)
2. Burning "dead shares" from the first depositor
3. Using virtual shares formula: `shares = (deposit * (total_shares + VIRTUAL)) / (total_supply + VIRTUAL)`

---

## Low Severity Findings

### L-01: Oracle Price Staleness Window (6 Minutes)

**Location:** `contracts/market/src/constants.rs:117`

```rust
pub const MAX_ORACLE_PRICE_AGE_SECONDS: u64 = 6 * SECONDS_PER_MINUTE; // 6 minutes
```

**Impact:** In highly volatile markets, 6 minutes allows significant price movement. Consider 2-3 minutes for tighter protection, or implement a price deviation circuit breaker.

---

### L-02: Zero Oracle Price Accepted

**Location:** `contracts/market/src/oracle.rs:32`

```rust
require_nonnegative(price_data.price)?;  // Allows price == 0
```

**Impact:** A zero price could cause division issues in collateral calculations. Change to `require_positive(price_data.price)?`.

---

### L-03: Pool Config Update TTL Not Extended

**Location:** `contracts/market/src/storage.rs:274-289`

```rust
pub fn queue_in_pool_config_update(...) -> Result<(), MCError> {
    // ...
    e.storage().persistent().set(&key, &pool_update);
    // Missing: extend_shared_storage(e, &key);
    Ok(())
}
```

**Impact:** Queued updates could be archived before execution if TTL expires.

---

### L-04: Upgrade Delay Conditional on Feature Flag

**Location:** `contracts/market/src/constants.rs:139-142`

```rust
#[cfg(feature = "mainnet")]
pub const UPGRADE_DELAY_SECONDS: u64 = 7 * SECONDS_PER_DAY;
#[cfg(not(feature = "mainnet"))]
pub const UPGRADE_DELAY_SECONDS: u64 = 0;
```

**Impact:** If deployed without `mainnet` feature, upgrades are immediate. Consider runtime network ID check as backup.

---

## Informational Findings

### I-01: Consistent Use of Checked Arithmetic ✓

Excellent use of `.checked_add()`, `.checked_sub()`, `.checked_mul()`, `.checked_div()` with `map_over_or_underflow()` throughout the codebase.

### I-02: Proper Authorization Pattern ✓

All state-modifying functions correctly implement `require_auth()` BEFORE state changes.

### I-03: TTL Management Well-Implemented ✓

Three-tier TTL system:

- Instance: 40-41 days
- Shared: 50-51 days
- Individual: 160-180 days

### I-04: Constructor Pattern Prevents Frontrunning ✓

Using `__constructor` instead of separate `initialize()` provides atomic initialization.

### I-05: Upgrade Timelock Pattern ✓

Proper propose → delay → execute pattern with 7-day mainnet delay.

---

## Security Checklist Results

### Critical Priority

- [x] Every state-modifying function has `require_auth()` on correct address
- [x] Authorization checked BEFORE state changes
- [x] No integer overflow/underflow (checked arithmetic throughout)
- [x] No division by zero (checked division with error handling)
- [ ] State updated BEFORE external calls (CEI pattern) - **See M-01**
- [x] TTL extended on all persistent data access
- [x] Upgrade function requires admin auth with timelock
- [x] Atomic initialization via constructor

### High Priority

- [x] Input amounts validated (positive, within bounds)
- [x] Array/collection sizes bounded (`max_positions`, `MAX_RESERVES`)
- [x] Oracle prices checked for staleness and validity
- [x] Contract properly initialized (one-time via constructor)
- [x] All storage keys properly namespaced (DataKey enum)
- [ ] Rounding always in protocol's favor - **See M-02**
- [ ] First depositor attack fully mitigated - **See M-03**

### Medium Priority

- [x] Error handling uses proper types (`MCError` contracterror)
- [x] Events emitted for important state changes
- [x] Decimal precision handled correctly (7 decimal standard)
- [x] Instance storage TTL extended
- [x] State machine transitions validated (MarketStatus)

---

## Recommendations Summary

| Priority   | Finding | Action                                                           |
| ---------- | ------- | ---------------------------------------------------------------- |
| **High**   | M-01    | Mark flash loan in progress or update pool state before callback |
| **High**   | M-02    | Use `fixed_mul_ceil` for fee calculations                        |
| **Medium** | M-03    | Add minimum first deposit or virtual shares                      |
| **Medium** | L-02    | Reject zero oracle prices                                        |
| **Low**    | L-03    | Extend TTL on pool config updates                                |

---

## Positive Security Observations

1. **Rounding Direction for Shares**: Correctly uses `floor` for supply shares (user gets fewer) and `ceil` for debt shares (user owes more)

2. **Liquidation Safeguards**: Two-tier LTV system (open_ltv vs close_ltv), close factor limits, and health checks

3. **Interest Accrual**: Proper per-second compound interest with `SCALED_FIXED_POINT_DENOMINATOR = 10^18` for precision

4. **Pool Config Timelock**: Queue-based config updates with mandatory waiting period

5. **Oracle Integration**: Staleness check, non-negative validation, SEP-40 compliance

---

## Files Reviewed

| File                                    | Purpose                           |
| --------------------------------------- | --------------------------------- |
| `contracts/market/src/contract.rs`      | Main market contract entry points |
| `contracts/market/src/processors.rs`    | Core operation processors         |
| `contracts/market/src/pool.rs`          | Pool data structure and logic     |
| `contracts/market/src/obligation.rs`    | User position tracking            |
| `contracts/market/src/oracle.rs`        | Oracle price integration          |
| `contracts/market/src/storage.rs`       | Storage patterns and TTL          |
| `contracts/market/src/interest_rate.rs` | Interest accrual logic            |
| `contracts/market/src/constants.rs`     | Protocol constants                |
| `contracts/market/src/utils/require.rs` | Validation helpers                |
| `contracts/market/src/utils/math.rs`    | Math utilities                    |
| `contracts/market/src/error.rs`         | Error definitions                 |

---

## Conclusion

The Alula Market Contract demonstrates professional-grade security awareness. The identified medium-severity issues are addressable and do not represent fundamental design flaws. The codebase follows Soroban security best practices with proper authorization, checked arithmetic, and storage management.

**Overall Risk Assessment:** LOW-MEDIUM

The contract is well-architected and suitable for deployment after addressing the identified recommendations.

---

## Disclaimer

This audit report is provided for informational purposes only. It does not guarantee the absence of vulnerabilities. Smart contract security is an ongoing process that requires continuous monitoring, testing, and updates.
