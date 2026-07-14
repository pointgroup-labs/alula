# Alula Market API Reference

The complete public interface of the **Market** contract
(`contracts/market/src/contract.rs`). Generated clients are available under
`packages/sdk/market` (`make sdk`); this document is the human-readable
companion.

Conventions used throughout:

- **Amounts** are `i128` with **7-decimal** precision (Stellar standard). Oracle
  prices use **14-decimal** precision.
- **Percentages** are basis points (`10000` = 100%).
- Passing `i128::MAX` (or `u64::MAX`) as an `amount` means **"the maximum
  currently allowed"** — full withdraw, full collateral removal, or full-debt
  repay, capped by the position's Open-LTV and available liquidity.
- Every state-changing call requires `require_auth` from the acting party
  (the obligation's `user`, or the `liquidator` / admin as noted).
- Errors are returned as [`MCError`](#error-codes) — a `contracterror` with
  stable numeric codes.

---

## The request-batch model

The Market is built around a single composable primitive:

```rust
fn submit_requests_batch(
    e: Env,
    user: ObligationKey,
    requests: Vec<Request>,
    referrer: Option<Address>,
) -> Result<(), MCError>;
```

A batch is a list of [`Request`](#request-variants) actions executed
**atomically** against one obligation. Health and invariants are checked once,
at the end of the batch — so intermediate states may be transiently unhealthy as
long as the batch settles healthy. This is what makes composite flows possible
in a single transaction:

| Flow | Batch |
| --- | --- |
| Leveraged (multiply) deposit | `FlashBorrow` → `SwapExactTokens` → `AddCollateral` → `Borrow` → (flash repay) |
| Collateral swap | `RemoveCollateral` → `SwapExactTokens` → `AddCollateral` |
| One-shot open | `AddCollateral` → `Borrow` |

The single-action entry points below (`deposit`, `borrow`, `withdraw`, …) are
thin conveniences that wrap one `Request`. Reach for `submit_requests_batch`
whenever you need atomicity across two or more actions.

### Request variants

```rust
enum Request {
    Deposit(StandardRequest),
    Borrow(StandardRequest),
    Withdraw(StandardRequest),
    Repay(StandardRequest),
    AddCollateral(StandardRequest),
    RemoveCollateral(StandardRequest),
    FlashBorrow(StandardRequest),
    SwapExactTokens(SwapExactTokensRequest),
    SwapForExactTokens(SwapForExactTokensRequest),
    Liquidate(LiquidateRequest),
}

struct StandardRequest        { amount: i128, pool_address: Address }
struct SwapExactTokensRequest { swap_provider: Address, path: Vec<Address>, amount_in: i128,  min_amount_out: i128 }
struct SwapForExactTokensRequest { swap_provider: Address, path: Vec<Address>, max_amount_in: i128, amount_out: i128 }
struct LiquidateRequest       { borrower_obligation_key: ObligationKey, borrow_pool_address: Address,
                                collateral_pool_address: Address, repay_amount: i128, min_demanded_collateral_amount: i128 }
```

### ObligationKey

```rust
struct ObligationKey { user: Address, seed: Option<BytesN<32>> }
```

Identifies a position. A single `user` can hold **multiple isolated
obligations** by supplying different `seed` values — useful for segregating
strategies (e.g. a leveraged position kept separate from a plain deposit) so a
liquidation of one cannot touch the other.

---

## User operations

All require the obligation user's authorization.

| Function | Signature (after `e: Env`) | Notes |
| --- | --- | --- |
| `deposit` | `user, pool_address, amount, referrer` | Supply to earn yield; mints j-tokens. |
| `withdraw` | `user, pool_address, amount, referrer` | Actual amount capped to keep the position at its Open-LTV. `i128::MAX` = withdraw all available. |
| `simulate_withdraw` | `user, pool_address, amount, referrer` → `WithdrawResult` | Read-only preview of a withdraw. |
| `add_collateral` | `user, pool_address, amount, referrer` | Collateral-only supply — always healthily withdrawable, earns no interest. |
| `remove_collateral` | `user, pool_address, amount, referrer` | Capped to Open-LTV; `i128::MAX` removes all available. |
| `borrow` | `user, pool_address, amount, referrer` | Draw a loan against collateral; accrues d-tokens. |
| `repay` | `user, pool_address, amount, referrer` | Repay debt; excess is ignored. `i128::MAX` = repay entire debt. |
| `liquidate` | `liquidator, borrower, borrow_pool_address, collateral_pool_address, repay_amount, demanded_collateral_amount` | Requires the liquidator's auth. Repays borrower debt, receives collateral at a bounded discount. `demanded_collateral_amount` is the liquidator's minimum-acceptable collateral (slippage guard). |
| `flash_loan` | `contract, caller, pool_address, amount` | ERC-3156 flash loan. `contract` is the receiver implementing `ModErc3156`; it must validate `caller` as `initiator` and grant a just-in-time `amount + fee` allowance in its `exec_op` callback. |
| `submit_requests_batch` | `user, requests, referrer` | The atomic multi-action primitive above. |

The optional `referrer` is credited an immediate fee share when the pool is
configured for it.

## Bad-debt handling

| Function | Signature | Notes |
| --- | --- | --- |
| `issue_cover_bad_debt` | `user` | Files cover-bad-debt requests to the Insurance Fund for every bad-debt borrow position on the obligation. |
| `claim_cover_bad_debt_results` | `user` | Claims the Insurance Fund's response to previously issued requests. |

## Fees

| Function | Signature | Notes |
| --- | --- | --- |
| `distribute_pool_fees` | `pool_address` | Permissionless. Pays accrued fees to the pool's beneficiaries. |
| `distribute_all_pools_fees` | — | Same, across every pool. |
| `set_take_rate_fees_beneficiaries` | `pool_address, beneficiaries: Map<Address, u32>` | Streaming-fee split. Shares in BPS must sum to 100%. Owner-only. |
| `set_operation_fees_beneficiaries` | `pool_address, beneficiaries: Map<Address, u32>` | Origination-fee split. Shares in BPS must sum to 100%. Owner-only. |

## Views (read-only)

| Function | Returns | Notes |
| --- | --- | --- |
| `get_global_state` | `GlobalState` | Owner, status, market-wide config. |
| `get_user_obligation` | `Obligation` | All deposits, collateral, and borrows for a key. |
| `get_pool` | `Pool` | Raw pool state. |
| `get_pool_data` | `PoolData` | Pool state **plus** computed borrow/supply APYs. Simulation-only. |
| `get_all_pools` | `Vec<Address>` | Every pool address in the market. |
| `get_market_data` | `MarketData` | Aggregated market metrics. Simulation-only. |
| `get_oracle_price_decimals` | `u32` | Oracle price precision (14). |
| `get_pool_asset_oracle_price` | `i128` | Current oracle price for a pool's asset. |
| `refresh_obligation` | `()` | Accrues interest on all pools the obligation touches. |
| `refresh_pool` | `()` | Accrues interest on one pool. |

`get_pool_data` and `get_market_data` are intended for off-chain simulation, not
on-chain composition.

## Governance & administration

Config changes are **timelocked**: queue → wait out the queue period → apply.
An owned market can queue updates; an ungoverned market's config is immutable.

| Function | Signature | Notes |
| --- | --- | --- |
| `queue_in_market_update` | `new_max_positions, new_min_collateral_value_cents, new_bad_debt_lock_d` | Queue a market-config change. |
| `cancel_market_update` | — | Drop the queued market update. |
| `apply_market_update` | — | Apply it once the queue period has elapsed. |
| `get_market_queued_in_update` | → `MarketUpdate` | Inspect the pending market update. |
| `queue_in_pool_set` | `pool_address, pool_config: PoolConfig` | Create a new pool or queue a config change for an existing one. |
| `cancel_pool_set` | `pool_address` | Drop the queued pool set. |
| `apply_pool_set` | `pool_address` | Apply the pool set once applicable. |
| `get_queued_pool_set` | `pool_address` → `QueuedPoolSet` | Inspect the pending pool set. |
| `update_market_status` | `new_status: u32` | Change [market status](#market-status-codes). |
| `fund_update_market_status` | `new_status: u32` | Status change authorized by the Insurance Fund contract (e.g. auto-freeze on shortfall). |
| `upgrade` | `new_wasm_hash: BytesN<32>` | Swap the contract WASM. |

## Farms (delegated staking)

| Function | Signature | Notes |
| --- | --- | --- |
| `set_farms_contract` | `farms_contract: Address` | Enable farm integration. |
| `clear_farms_contract` | — | Disable it. |
| `get_farms_contract` | → `Option<Address>` | Current farms contract, if any. |
| `set_pool_supply_farm` | `pool_address, farm_id: BytesN<32>` | Reward j-token (supply) holders. |
| `set_pool_debt_farm` | `pool_address, farm_id: BytesN<32>` | Reward d-token (debt) holders. |
| `clear_pool_farms` | `pool_address` | Remove a pool's farm config. |
| `refresh_obligation_farms` | `user: Address` | Permissionless. Re-syncs a user's farm stakes with their current positions. |

---

## Market status codes

`update_market_status` / `fund_update_market_status` take a `u32`:

| Code | Status | Effect |
| --- | --- | --- |
| `0` | `Active` | Normal operation. |
| `1` | `BorrowFrozen` | Borrowing disabled. |
| `2` | `BorrowFrozenByAdmin` | Borrowing disabled by admin (sticky). |
| `3` | `DepositFrozen` | Deposits disabled. |
| `4` | `DepositFrozenByAdmin` | Deposits disabled by admin (sticky). |
| `5` | `Frozen` | Deposits **and** borrows disabled. |
| `6` | `FrozenByAdmin` | Fully frozen by admin (sticky). |

`…ByAdmin` states are "hard locks" only the market admin can move into or out
of: the automatic (Insurance-Fund-driven) `fund_update_market_status` path
rejects any transition that either starts from or lands on one of them. The
plain variants (`1`/`3`/`5`) are the ones the Insurance Fund may set or clear
automatically.

## Error codes

`MCError` values are grouped by concern; the numeric code is stable across
releases.

| Code | Variant | Meaning |
| --- | --- | --- |
| 0 | `InternalError` | Unexpected internal failure. |
| 1 | `InvalidInputAmount` | Amount is zero/negative or otherwise invalid. |
| 2 | `DependencyContractError` | A cross-contract call failed. |
| 3 | `MarketIsNotOwned` | Governance action on an ungoverned (immutable) market. |
| 4 | `BorrowForbiddenOnMarket` | Market status forbids borrowing. |
| 5 | `DepositForbiddenOnMarket` | Market status forbids deposits. |
| 6 | `MarketIsFrozen` | Market fully frozen. |
| 7 | `InvalidMarketConfigOrUpdate` | Rejected market config/update. |
| 8 | `IncorrectRequestType` | Request variant not valid in this context. |
| 9 | `OverOrUnderflow` | Checked-arithmetic bound hit. |
| 10 | `TooManyPositions` | Obligation exceeds `max_positions`. |
| 11 | `MinCollateralValueIsNotMet` | Below the market's minimum collateral value. |
| 12 | `NonPositiveSharesAmount` | Share computation produced ≤ 0. |
| 100 | `InvalidInitialization` | Bad init params. |
| 101 | `PoolDoesNotExist` | Unknown pool. |
| 102 | `InvalidLoanPoolConfig` | Rejected pool config. |
| 103 | `NotEnoughPoolFunds` | Insufficient pool liquidity. |
| 104 | `DepositPoolDoesNotExist` | Deposit pool missing. |
| 105 | `BorrowPoolDoesNotExist` | Borrow pool missing. |
| 106 | `CollateralPoolDoesNotExist` | Collateral pool missing. |
| 107 | `PoolAlreadyContainsQueuedPoolSet` | A pool set is already queued. |
| 108 | `PoolDoesNotHaveQueuedPoolSet` | No pool set queued. |
| 109 | `PoolSetIsNotYetApplicable` | Queue period not yet elapsed. |
| 110 | `OperationForbiddenOnPool` | Operation not permitted on this pool. |
| 111 | `MarketAlreadyContainsQueuedInConfigUpdate` | A market update is already queued. |
| 112 | `MarketDoesNotHaveQueuedInConfigUpdate` | No market update queued. |
| 113 | `MarketConfigUpdateIsNotYetApplicable` | Queue period not yet elapsed. |
| 114 | `PoolBadDebtLocked` | Pool locked while bad debt is being processed. |
| 200 | `ObligationDoesNotExist` | Unknown obligation. |
| 201 | `DepositPositionDoesNotExist` | No deposit position for asset. |
| 202 | `BorrowPositionDoesNotExist` | No borrow position for asset. |
| 203 | `WithdrawScarcityOverLimit` | Withdraw exceeds the scarcity limit. |
| 204 | `ScarcityCooldownPeriod` | Within the scarcity cooldown window. |
| 205 | `BorrowPositionForAssetExists` | Asset already borrowed. |
| 206 | `DepositPositionForAssetExists` | Asset already deposited. |
| 207 | `UnhealthyOperation` | Operation would leave the position unhealthy. |
| 400 | `PoolSupplyLimitExceeded` | Pool supply cap reached. |
| 401 | `PoolUtilizationRatioCapExceeded` | Pool utilization cap reached. |
| 500 | `OracleDoesNotKnowAssetPrice` | No price for asset. |
| 501 | `OracleStalePrice` | Price older than the max allowed age. |
| 502 | `NonPositiveOraclePrice` | Oracle returned ≤ 0. |
| 600 | `InvalidLiquidationInputs` | Malformed liquidation request. |
| 601 | `ObligationIsHealthy` | Position not liquidatable. |
| 602 | `ObligationContainsOpenCoverBadDebtRequests` | Pending bad-debt requests block the action. |
| 603 | `BadDebtCoverageCriterionIsNotMet` | Coverage precondition unmet. |
| 604 | `AssetCannotBeUsedAsCollateral` | Asset not eligible as collateral. |
| 605 | `LiquidationExcessiveDemandedCollateral` | Liquidator demanded more collateral than allowed. |
| 701 | `InvalidSwap` | Swap request rejected. |
| 702 | `FlashBorrowAlreadyRegistered` | Duplicate flash borrow in one batch. |
| 703 | `SwapSlippageExceeded` | Swap breached its min/max bound. |

---

See the [top-level README](../README.md) for protocol concepts, and
[`docs/multisig.md`](./multisig.md) for the governance-key workflow that authors
`queue_*` / `apply_*` / `upgrade` transactions.
