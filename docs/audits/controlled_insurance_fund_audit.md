# Security Audit Report: `controlled_insurance_fund`

**Audit Date:** 2025-12-28
**Auditor:** Claude (Soroban Security Expert)
**Contract Version:** Current `feat/new_insurance_fund` branch
**Severity Scale:** CRITICAL | HIGH | MEDIUM | LOW | INFO

---

## Executive Summary

The `controlled_insurance_fund` contract implements a human-in-the-loop insurance fund for covering bad debt in the Alula lending protocol. The design is sound and appropriate for its use case, with proper authorization on critical functions and defensive programming patterns.

**Overall Assessment:** The contract is well-implemented with **no critical vulnerabilities**. Two issues require attention before production deployment.

| Category             | Rating               |
| -------------------- | -------------------- |
| Security             | **Good**             |
| Code Quality         | **Good**             |
| Completeness         | **Needs Minor Work** |
| Production Readiness | **Almost Ready**     |

---

## Contract Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CONTROLLED INSURANCE FUND                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ADMIN FUNCTIONS                      MARKET (InsuranceFund trait)          │
│   ════════════════                     ═════════════════════════════         │
│   • set_market(market)                 • add_reserves(token, amount)         │
│   • withdraw(token, to, amount)        • request_coverage(token, amount)     │
│   • mark_ready(request_id, amount)     • get_status(request_id)              │
│   • update_market_status(status)       • claim_coverage(request_id)          │
│   • get_request(request_id)                                                  │
│                                                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   REQUEST STATE MACHINE                                                      │
│   ═════════════════════                                                      │
│                                                                              │
│   Market calls              Admin approves           Market claims           │
│   request_coverage()        mark_ready()             claim_coverage()        │
│         │                        │                        │                  │
│         ▼                        ▼                        ▼                  │
│   ┌──────────┐            ┌──────────────┐          ┌──────────────┐        │
│   │ PENDING  │ ─────────► │    READY     │ ───────► │   DELETED    │        │
│   │          │            │ (amt locked) │          │  (claimed)   │        │
│   └──────────┘            └──────────────┘          └──────────────┘        │
│                                                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   STORAGE                                                                    │
│   ═══════                                                                    │
│   Instance:   Admin, Market, RequestsCounter                                 │
│   Persistent: Request(id), LockedAmount(token)                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Files Reviewed

| File                                                 | Lines | Description                    |
| ---------------------------------------------------- | ----- | ------------------------------ |
| `contracts/controlled_insurance_fund/src/lib.rs`     | 195   | Main contract logic            |
| `contracts/controlled_insurance_fund/src/storage.rs` | 148   | Storage types and helpers      |
| `contracts/controlled_insurance_fund/src/error.rs`   | 12    | Error definitions              |
| `contracts/controlled_insurance_fund/src/market.rs`  | 7     | Market client interface        |
| `insurance_fund_trait/src/lib.rs`                    | 63    | InsuranceFund trait definition |

**Related Market Integration:**

- `contracts/market/src/processors.rs` — `process_issue_cover_bad_debt`, `process_claim_cover_bad_debt_results`
- `contracts/market/src/contract.rs` — `fund_update_market_status`
- `contracts/market/src/obligation.rs` — `insurance_fund_requests_ids`

---

## Findings

### CIF-01: `update_market_status` Missing Authorization

| Attribute    | Value           |
| ------------ | --------------- |
| **Severity** | HIGH            |
| **Status**   | Open            |
| **Location** | `lib.rs:98-102` |
| **Type**     | Authorization   |

**Description:**

The `update_market_status` function allows anyone to call it:

```rust
pub fn update_market_status(e: Env, new_status: u32) {
    // NO require_admin() CHECK
    let market = storage::get_market(&e);
    let market_client = market::MarketPartialClient::new(&e, &market);
    market_client.fund_update_market_status(&new_status);
}
```

The Market's `fund_update_market_status` verifies the caller is the insurance fund contract address (which it is), so the call succeeds.

**Impact:**

Anyone can toggle the market status between non-admin-protected states:

- `Active` (0)
- `BorrowFrozen` (1)
- `DepositFrozen` (3)
- `Frozen` (5)

The `*ByAdmin` statuses (2, 4, 6) are protected by the Market and cannot be set this way.

**Proof of Concept:**

```rust
// Attacker calls directly
insurance_fund_client.update_market_status(5); // Sets market to Frozen
// Market operations now blocked until someone calls again with 0
```

**Recommendation:**

Add authorization check:

```rust
pub fn update_market_status(e: Env, new_status: u32) {
    require_admin(&e);  // ADD THIS LINE
    let market = storage::get_market(&e);
    let market_client = market::MarketPartialClient::new(&e, &market);
    market_client.fund_update_market_status(&new_status);
}
```

---

### CIF-02: `set_market` Can Be Called Multiple Times

| Attribute    | Value                    |
| ------------ | ------------------------ |
| **Severity** | HIGH                     |
| **Status**   | Open (TODO acknowledged) |
| **Location** | `lib.rs:20-26`           |
| **Type**     | Access Control           |

**Description:**

The contract includes a TODO comment acknowledging this issue:

```rust
pub fn set_market(e: Env, market: Address) {
    // TODO: This must be a one-time lock or something like that
    require_admin(&e);
    storage::extend_instance_storage(&e);
    storage::set_market(&e, market);
}
```

**Impact:**

1. **Orphaned Requests:** If market is changed while requests are pending, the old market cannot claim them
2. **Locked Funds:** LockedAmount remains for orphaned requests with no way to unlock
3. **Trust Violation:** Changing market breaks the expected 1:1 relationship

Note: This does NOT allow direct fund theft because:

- The admin already has `withdraw()` for unlocked funds
- Claim transfers to the stored market, not the caller
- New market doesn't know old request IDs

**Recommendation:**

Make `set_market` one-time only:

```rust
pub fn set_market(e: Env, market: Address) {
    require_admin(&e);

    if e.storage().instance().has(&DataKey::Market) {
        panic_with_error!(&e, ContractError::MarketAlreadySet);
    }

    storage::extend_instance_storage(&e);
    storage::set_market(&e, market);
}
```

---

### CIF-03: Request Counter Overflow (Theoretical)

| Attribute    | Value              |
| ------------ | ------------------ |
| **Severity** | LOW                |
| **Status**   | Open               |
| **Location** | `storage.rs:64-71` |
| **Type**     | Arithmetic         |

**Description:**

The request counter uses unchecked addition:

```rust
fn next_requests_counter(e: &Env) -> u64 {
    let counter: u64 = e.storage().instance()
        .get(&DataKey::RequestsCounter)
        .expect("RequestsCounter must be set");

    e.storage().instance().set(&DataKey::RequestsCounter, &(counter + 1));  // No checked_add

    counter
}
```

**Impact:**

Theoretical only. Would require 2^64 requests (18.4 quintillion) to overflow.

**Recommendation:**

Use checked arithmetic for defense in depth:

```rust
let next = counter.checked_add(1).expect("Counter overflow");
e.storage().instance().set(&DataKey::RequestsCounter, &next);
```

---

### CIF-04: Missing Input Validation on Amounts

| Attribute    | Value                |
| ------------ | -------------------- |
| **Severity** | LOW                  |
| **Status**   | Open                 |
| **Location** | `lib.rs:40, 59, 112` |
| **Type**     | Input Validation     |

**Description:**

Functions accept amounts without validating they are positive:

```rust
pub fn withdraw(e: Env, token: Address, to: Address, amount: i128) { ... }
pub fn mark_ready(e: Env, request_id: u64, covered_amount: i128) { ... }
fn request_coverage(e: Env, token: Address, amount: i128) -> IssueRequestResult { ... }
```

**Impact:**

- Zero amounts waste gas
- Negative amounts could cause unexpected behavior (though `min()` in `mark_ready` provides some protection)

**Recommendation:**

Add validation:

```rust
if amount <= 0 {
    panic_with_error!(&e, ContractError::InvalidAmount);
}
```

---

### CIF-05: No Admin Transfer Function

| Attribute    | Value              |
| ------------ | ------------------ |
| **Severity** | LOW                |
| **Status**   | Open               |
| **Location** | `storage.rs:50-56` |
| **Type**     | Access Control     |

**Description:**

Admin is set in constructor with no way to transfer:

```rust
pub fn __constructor(e: Env, admin: Address) {
    storage::set_admin(&e, admin);
    storage::init_requests_counter(&e);
}
```

**Impact:**

If admin key is lost or compromised, no recovery possible.

**Recommendation:**

Add admin transfer function:

```rust
pub fn set_admin(e: Env, new_admin: Address) {
    require_admin(&e);
    storage::set_admin(&e, new_admin);
}
```

---

### CIF-06: Short TTL Periods

| Attribute    | Value              |
| ------------ | ------------------ |
| **Severity** | INFO               |
| **Status**   | Open               |
| **Location** | `storage.rs:10-14` |
| **Type**     | Configuration      |

**Description:**

TTL periods are 40-41 days:

```rust
pub const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const INSTANCE_BUMP: u32 = 41 * LEDGERS_PER_DAY;
pub const PERSISTENT_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
pub const PERSISTENT_BUMP: u32 = 41 * LEDGERS_PER_DAY;
```

**Impact:**

Bad debt resolution might take longer than 40 days in complex scenarios. If requests expire before being claimed, locked amounts become orphaned.

**Recommendation:**

Consider increasing to 90-180 days for persistent data.

---

### CIF-07: `add_reserves` Is No-Op (By Design)

| Attribute    | Value            |
| ------------ | ---------------- |
| **Severity** | INFO             |
| **Status**   | Acknowledged     |
| **Location** | `lib.rs:107-110` |
| **Type**     | Design           |

**Description:**

```rust
fn add_reserves(e: Env, _token: Address, _amount: i128) {
    require_market(&e); // NB: Only validation for this token-balance driven implementation
    storage::extend_instance_storage(&e);
}
```

This is intentional — the contract uses actual token balances rather than internal accounting. The comment documents this design choice.

**Status:** No action needed.

---

### CIF-08: Missing Instance TTL Extension in Some Functions

| Attribute    | Value                         |
| ------------ | ----------------------------- |
| **Severity** | INFO                          |
| **Status**   | Open                          |
| **Location** | `lib.rs:40-53, 59-93, 98-102` |
| **Type**     | TTL Management                |

**Description:**

Some functions don't extend instance TTL:

- `withdraw()`
- `mark_ready()`
- `update_market_status()`

**Impact:**

If only these functions are called frequently, instance storage TTL won't be extended. Low risk since other functions extend TTL.

**Recommendation:**

Add `storage::extend_instance_storage(&e);` to all public functions for consistency.

---

## Security Checklist

### Authorization

| Check                                         | Status                            |
| --------------------------------------------- | --------------------------------- |
| `require_auth()` on state-modifying functions | ✅ Pass                           |
| Authorization before state changes            | ✅ Pass                           |
| Correct authorization subject                 | ✅ Pass                           |
| All admin functions protected                 | ⚠️ `update_market_status` missing |

### Access Control

| Check                     | Status                    |
| ------------------------- | ------------------------- |
| Initialization protection | ✅ Constructor pattern    |
| One-time market lock      | ⚠️ Not implemented (TODO) |
| Admin transfer capability | ⚠️ Not implemented        |

### Arithmetic

| Check                                | Status                             |
| ------------------------------------ | ---------------------------------- |
| Checked arithmetic on critical paths | ✅ `mark_ready` uses `checked_add` |
| Division by zero                     | ✅ N/A                             |
| Rounding direction                   | ✅ Uses `min()` conservatively     |

### State Management

| Check                   | Status                                 |
| ----------------------- | -------------------------------------- |
| CEI pattern             | ✅ Safe (trusted market, no callbacks) |
| Storage key namespacing | ✅ Uses typed `DataKey` enum           |
| TTL extension on access | ⚠️ Inconsistent                        |

### Input Validation

| Check               | Status         |
| ------------------- | -------------- |
| Positive amounts    | ⚠️ Not checked |
| Request ID validity | ✅ Checked     |

---

## Design Assessment

### Strengths

1. **Clean Architecture:** Well-separated concerns across files
2. **Constructor Pattern:** Atomic initialization, no frontrunning
3. **Defensive Checks:** Validates invariants (e.g., `current_balance >= total_locked`)
4. **Error Events:** Emits events for error conditions
5. **Human-in-the-Loop:** Admin approval prevents automated drainage
6. **Locked Liquidity:** Protects claimed amounts from withdrawal

### Design Trade-offs

| Choice                    | Pros                           | Cons                     |
| ------------------------- | ------------------------------ | ------------------------ |
| Admin-controlled coverage | Security, oversight            | Centralization, delays   |
| Balance-based reserves    | Simple, no internal accounting | Relies on token contract |
| Single market             | Simple                         | Not multi-market capable |
| No request cancellation   | Prevents manipulation          | Stuck requests possible  |

### Flexibility Assessment

| Aspect               | Score | Notes                   |
| -------------------- | ----- | ----------------------- |
| Multi-market support | 2/10  | Single market only      |
| Token flexibility    | 10/10 | Any SEP-41 token        |
| Governance           | 3/10  | Single admin            |
| Upgradability        | 0/10  | No upgrade mechanism    |
| Request management   | 4/10  | No cancel/reject/expire |

---

## Recommendations Summary

### Must Fix (Before Production)

| Priority | Finding                                         | Effort |
| -------- | ----------------------------------------------- | ------ |
| 1        | Add `require_admin()` to `update_market_status` | Low    |
| 2        | Make `set_market` one-time only                 | Low    |

### Should Fix

| Priority | Finding                            | Effort |
| -------- | ---------------------------------- | ------ |
| 3        | Add admin transfer function        | Low    |
| 4        | Add input validation for amounts   | Low    |
| 5        | Use checked arithmetic on counter  | Low    |
| 6        | Extend TTL in all public functions | Low    |

### Consider for Future

| Priority | Finding                             | Effort |
| -------- | ----------------------------------- | ------ |
| 7        | Increase TTL periods to 90-180 days | Low    |
| 8        | Add request cancellation/rejection  | Medium |
| 9        | Add success events                  | Low    |

---

## Conclusion

The `controlled_insurance_fund` contract is a **well-designed implementation** of a human-controlled insurance mechanism. The code is clean, readable, and follows Soroban best practices.

**Two issues require immediate attention:**

1. **CIF-01 (HIGH):** `update_market_status` is callable by anyone — add `require_admin(&e)`
2. **CIF-02 (HIGH):** `set_market` should be one-time only — the TODO is acknowledged

After these fixes, the contract is suitable for production deployment.

---

## Appendix: Test Coverage Observations

From `tests/src/bad_debt.rs`:

- ✅ `test_obligation_does_not_have_bad_debt_by_default`
- ✅ `test_partially_socialize_full_bad_debt_loss`
- ✅ `test_completely_cover_bad_debt`
- ✅ `test_donate`

**Missing Test Coverage:**

- Authorization failure cases
- Edge cases (zero amounts, duplicate requests)
- TTL expiration scenarios
- Market change scenarios

---

_Report generated by Claude (Soroban Security Expert)_
_Audit methodology based on Stellar/Soroban security best practices_
