# Alula Lending Protocol - Liquidation Mechanics Security Audit

---

## Executive Summary

This is a focused security audit of the liquidation mechanics in the Alula lending protocol's Market contract. The analysis covers the complete liquidation flow from health factor calculation to collateral seizure and bad debt socialization.

### Liquidation Architecture Overview

1. **Two-phase health check**: Uses `close_ltv` (liquidation threshold) separately from `open_ltv` (borrow threshold)
2. **Dual scenario handling**: Distinguishes between solvent (LTV-improving) and insolvent liquidations
3. **Collateral layering**: Supports both plain collateral and j-tokens (deposit shares) as liquidatable assets

### Key Files Analyzed

| File                                 | Description                                                    |
| ------------------------------------ | -------------------------------------------------------------- |
| `contracts/market/src/obligation.rs` | Core liquidation logic (`fn liquidate`, lines 859-1124)        |
| `contracts/market/src/processors.rs` | Liquidation orchestration (`process_liquidate`, lines 846-925) |
| `contracts/market/src/contract.rs`   | Entry point (`fn liquidate`, lines 914-944)                    |
| `contracts/market/src/pool.rs`       | Share calculations and pool adjustments                        |
| `contracts/market/src/oracle.rs`     | Price feed integration                                         |
| `contracts/market/src/constants.rs`  | Protocol parameters                                            |

---

## Findings Summary

| ID   | Severity | Title                                         | Status     |
| ---- | -------- | --------------------------------------------- | ---------- |
| M-01 | Medium   | 99% Safety Margin Rounding Amplification      | Open       |
| M-02 | Medium   | Division Before Multiplication Precision Loss | Open       |
| L-01 | Low      | Close Factor Not Enforced in Insolvency       | Open       |
| L-02 | Low      | Dust Cleanup Exploitation Vector              | Acceptable |
| L-03 | Low      | Uncertain J-Token Rounding Direction          | Open       |
| L-04 | Low      | Multiple Oracle Price Fetches                 | Open       |

---

## Detailed Findings

### MEDIUM SEVERITY

---

#### M-01: 99% Safety Margin in LTV-Improving Calculation May Allow Slight Position Worsening

**Location**: `contracts/market/src/obligation.rs:945-951`

**Description**: In the LTV-improving (solvent) liquidation scenario, the code applies a 99% safety margin to the maximum collateral that can be seized:

```rust
let max_ltv_improving_collateral_seized = numerator
    .checked_div(denominator)
    .map_over_or_underflow()?
    .checked_mul(99)
    .map_over_or_underflow()?
    .checked_div(100)
    .map_over_or_underflow()?;
```

This 99% factor is applied _after_ the division, meaning rounding errors from the division are multiplied. For very small liquidations or positions with specific value ratios, this can result in collateral seizure amounts that don't strictly guarantee LTV improvement.

**Impact**: In edge cases, a liquidator could potentially seize slightly more collateral than mathematically required for strict LTV improvement. This is bounded by the 1% margin but could result in minor value extraction from borrowers.

**Recommendation**: Consider applying the 99% factor to the numerator _before_ the division to prevent rounding error amplification:

```rust
let safe_numerator = obligation_collateral_value
    .checked_mul(liquidated_value)
    .map_over_or_underflow()?
    .checked_mul(99)
    .map_over_or_underflow()?
    .checked_div(100)
    .map_over_or_underflow()?;
let max_ltv_improving_collateral_seized = safe_numerator
    .checked_div(denominator)
    .map_over_or_underflow()?;
```

---

#### M-02: Division Before Multiplication in Collateral Amount Calculation

**Location**: `contracts/market/src/obligation.rs:961-964`

**Description**: When calculating the redeemed collateral amount with incentive:

```rust
let redeemed_collateral_amount_with_max_incentive =
    collateral_value_to_redeem_with_max_incentive
        .checked_div(collateral_asset_price)
        .map_over_or_underflow()?;
```

The division happens after multiplying by the incentive factor. However, for collateral with high oracle decimals (14 decimals as noted in README.md), this division can cause significant precision loss when the collateral price is large relative to the debt value.

**Impact**: Liquidators may receive slightly less collateral than they should, reducing liquidation incentives marginally. In extreme price ratio cases, this could make small liquidations unprofitable.

**Recommendation**: Ensure the oracle price decimals are accounted for consistently and consider whether the calculation order could be optimized for precision.

---

### LOW SEVERITY

---

#### L-01: Close Factor Not Enforced in Insolvency Scenario

**Location**: `contracts/market/src/obligation.rs:920-929` vs `obligation.rs:972-996`

**Description**: The close factor check (preventing liquidation of more than 50% of debt) is only enforced in the solvent (LTV-improving) scenario:

```rust
let (mut collateral_to_sell_to_liquidator, liquidated_amount) = if is_solvent {
    // -- LTV-improving scenario --
    // Check if the liquidation doesn't exceed the close factor
    let liquidated_borrow_bps = liquidated_amount
        .fixed_div_ceil(position_debt, BPS_FACTOR)
        .map_over_or_underflow()?;
    if liquidated_borrow_bps > liquidation_close_factor_bps {
        return Err(MCError::LiquidationExceedsCloseFactor);
    }
    // ...
} else {
    // -- Insolvency scenario --
    // NO close factor check here
}
```

In insolvency, any amount up to the full debt can be liquidated. While this is likely intentional (to enable faster clearing of underwater positions), there's no documentation explaining this design choice.

**Impact**: Underwater positions can be fully liquidated in a single transaction, which may not always be desirable from a fairness perspective. A race condition exists where multiple liquidators may attempt to liquidate the same insolvent position.

**Recommendation**: Document this behavior explicitly. Consider whether a staged liquidation approach for insolvent positions would provide better outcomes.

---

#### L-02: Minimum Collateral Value Dust Cleanup Could Be Exploited

**Location**: `contracts/market/src/obligation.rs:1001-1010`

**Description**: The dust cleanup mechanism gives the liquidator additional collateral when remaining value is below `min_collateral_value`:

```rust
let is_all_collateral_drained = if collateral_value_left < min_collateral_value {
    // If collateral(both plain collateral and supply shares) that's left is worth
    // less than the configured `min_collateral_value` on the market, the liquidator
    // additionally receives all of the collateral that's left
    collateral_to_sell_to_liquidator += collateral_left;
    true
} else {
    false
};
```

A sophisticated liquidator could calculate the exact repay amount to trigger this condition, receiving a "bonus" of `collateral_left` tokens beyond the normal incentive.

**Impact**: Small additional value extraction. With default `min_collateral_value` of 10^5 (0.01) and 14-decimal oracle prices, this represents at most 0.01 \* price worth of "bonus" collateral per liquidation.

**Recommendation**: This is likely acceptable given the small magnitude. Consider whether the dust cleanup bonus should be capped or split between liquidator and protocol reserves.

---

#### L-03: J-Tokens Seized Calculation Has TODO Comment Indicating Uncertainty

**Location**: `contracts/market/src/obligation.rs:1035-1039`

**Description**: There's a TODO comment indicating developer uncertainty about ceiling/flooring behavior:

```rust
// TODO: Does this always work with ceiling/flooring?
collateral_pool
    .compute_j_tokens_from_tokens_ceil(tokens_from_j_tokens_seized)?
    .min(deposit_position.j_tokens)
```

Using `ceil` for j-token calculation means the liquidator receives slightly more j-tokens than strictly necessary to cover `tokens_from_j_tokens_seized`. The `.min(deposit_position.j_tokens)` prevents over-seizure but the rounding direction benefits the liquidator at the borrower's expense.

**Impact**: Borrowers may lose marginally more j-tokens than strictly necessary in liquidations involving deposit shares. The difference is typically 1 unit of j-tokens per liquidation.

**Recommendation**: Resolve the TODO - use `floor` rounding when calculating j-tokens to seize (favor borrower) or document why `ceil` is preferred.

---

#### L-04: Oracle Price Fetched Multiple Times Per Liquidation

**Location**: `contracts/market/src/obligation.rs:896-899` and various `compute_*_value` functions

**Description**: During a single liquidation, oracle prices are fetched multiple times:

1. In `compute_debt_value_scaled_w_liability_factors`
2. In `compute_collateral_value_scaled_w_close_ltvs`
3. In `compute_debt_value` and `compute_collateral_value`
4. Directly in the liquidation function for borrowed and collateral asset prices

While each call has staleness checks, if a Soroban transaction spans multiple ledger closes (unlikely but possible in edge cases), or if the oracle is updated mid-transaction by another contract, values could become inconsistent.

**Impact**: Theoretical inconsistency risk. In practice, Soroban transactions are atomic within a ledger, so this is low-risk.

**Recommendation**: Cache oracle prices at the start of liquidation and reuse them throughout the calculation.

---

## INFORMATIONAL

---

#### I-01: Well-Designed Two-LTV System

The protocol correctly separates `open_ltv` (max LTV when borrowing) from `close_ltv` (liquidation threshold), providing a buffer zone that protects borrowers from immediate liquidation after borrowing at maximum capacity.

---

#### I-02: Proper Interest Accrual Before Liquidation

The `process_liquidate` function correctly calls `obligation.accrue_interest(e)?` before performing health checks, ensuring debt calculations reflect current accrued interest.

---

#### I-03: Self-Liquidation Prevention

The check `if liquidator == &borrower_obligation_key.user` at `processors.rs:858-860` correctly prevents self-liquidation, which could otherwise be used to circumvent fees or manipulate positions.

---

#### I-04: Proper Handling of Plain Collateral vs J-Tokens

The liquidation logic correctly prioritizes seizing plain collateral before j-tokens, and properly transfers j-tokens to the liquidator's obligation when plain collateral is insufficient.

---

#### I-05: Insolvency LTV Threshold Design

The `insolvency_ltv_bps` parameter (default 98.5%) correctly distinguishes between:

- **Solvent liquidations** (LTV < 98.5%): Must improve position health
- **Insolvent liquidations** (LTV >= 98.5%): Full debt clearance mode to minimize bad debt

This dual-mode system is well-designed for managing different risk scenarios.

---

#### I-06: Liability Factor Application

The `liability_factor_bps` (100-200%) is correctly applied to volatile assets' debt values when calculating health, providing additional safety margins for risky collateral.

---

## Rounding Direction Analysis

| Operation                      | Direction  | Favors     | Correct?     |
| ------------------------------ | ---------- | ---------- | ------------ |
| Debt -> d_tokens (burn)        | Floor      | Borrower   | Yes          |
| Collateral -> j_tokens (seize) | Ceil       | Liquidator | Questionable |
| j_tokens -> tokens (value)     | Floor      | Protocol   | Yes          |
| d_tokens -> tokens (debt)      | Ceil       | Protocol   | Yes          |
| LTV calculation                | Ceil       | Protocol   | Yes          |
| Max collateral seized          | 99% margin | Borrower   | Yes          |

---

## Positive Security Observations

1. **Checked Arithmetic Throughout**: All liquidation calculations use `checked_*` operations with proper overflow handling via `map_over_or_underflow()?`

2. **Require Auth on Liquidator**: The `liquidator.require_auth()` at `contract.rs:925` ensures only authorized parties can initiate liquidations

3. **Market Frozen Check**: `require_not_frozen(&e)?` prevents liquidations during emergency freeze, but allows them during partial pauses (correct behavior)

4. **Collateral Seizability Check**: `collateral_pool.require_collateral_is_seizable()?` at `processors.rs:871` ensures assets with `close_ltv_bps == 0` cannot be seized

5. **Event Emission**: Comprehensive liquidation events are emitted for off-chain monitoring

6. **Bad Debt Socialization**: The `cover_bad_debt` mechanism correctly handles positions with no remaining liquidatable collateral by:
   - First using accumulated reserve fees
   - Then socializing remaining bad debt across lenders

---

## Liquidation Flow Diagram

```
liquidate() [contract.rs:914]
    |
    v
require_auth(liquidator) + require_not_frozen()
    |
    v
process_liquidate() [processors.rs:846]
    |
    +-- require_positive(repay_amount)
    +-- require_nonnegative(min_demanded_collateral_amount)
    +-- Validate: borrow_pool != collateral_pool
    +-- Validate: liquidator != borrower
    |
    v
obligation.accrue_interest()
    |
    v
obligation.liquidate() [obligation.rs:859]
    |
    +-- Compute debt_value_w_liability_factors
    +-- Compute collateral_value_w_close_ltvs
    +-- Check: debt > collateral (unhealthy)
    |
    +-- Determine: is_solvent (LTV < insolvency_ltv)?
    |
    +-- IF SOLVENT:
    |   +-- Enforce close_factor (max 50% debt)
    |   +-- Calculate max_ltv_improving_collateral
    |   +-- Calculate collateral_with_max_incentive
    |   +-- Take minimum of constraints
    |
    +-- IF INSOLVENT:
    |   +-- No close_factor limit
    |   +-- Calculate collateral_with_max_incentive
    |   +-- Take what's available
    |
    +-- Apply dust cleanup (min_collateral_value)
    +-- Check: collateral >= demanded_amount
    +-- Distribute between plain_collateral and j_tokens
    +-- Update borrower positions
    |
    v
Transfer j_tokens to liquidator obligation (if needed)
    |
    v
pool.liquidation_repay_debt()
pool.liquidation_redeem_collateral()
    |
    v
Emit liquidation event
    |
    v
Execute token transfers
```

---

## Recommendations Summary

| Priority | Action                                                         |
| -------- | -------------------------------------------------------------- |
| High     | Review M-01: Consider reordering 99% safety margin calculation |
| High     | Review M-02: Analyze precision loss in collateral calculations |
| Medium   | Document L-01: Explain close factor bypass in insolvency       |
| Medium   | Resolve L-03: Fix TODO comment about j-token rounding          |
| Low      | Consider L-04: Cache oracle prices for consistency             |
| Low      | Accept L-02: Dust cleanup is bounded and acceptable            |

---

## Appendix: Key Constants

| Constant                            | Value         | Description                        |
| ----------------------------------- | ------------- | ---------------------------------- |
| `DEFAULT_CLOSE_FACTOR_BPS`          | 5,000 (50%)   | Max debt repayable per liquidation |
| `DEFAULT_LIQUIDATION_INCENTIVE_BPS` | 1,000 (10%)   | Bonus for liquidators              |
| `DEFAULT_OPEN_LTV_BPS`              | 7,000 (70%)   | Max LTV at borrow time             |
| `DEFAULT_CLOSE_LTV_BPS`             | 8,000 (80%)   | Liquidation threshold              |
| `DEFAULT_INSOLVENCY_LTV_BPS`        | 9,850 (98.5%) | Insolvency threshold               |
| `DEFAULT_MIN_COLLATERAL_VALUE`      | 10^5          | Dust threshold (0.01)              |
| `MAX_ORACLE_PRICE_AGE_SECONDS`      | 360 (6 min)   | Oracle staleness limit             |

---

_This audit focuses specifically on liquidation mechanics. A comprehensive protocol audit should cover all contract functionality._
