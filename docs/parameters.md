# Alula Market & Pool Parameters

This is the full list of parameters that affect you if you supply liquidity to
an Alula market: what each one does, its default, how far it can be moved, and
who can move it.

Sections 1-8 cover the Market contract. Sections 9-11 cover farms, the price
oracle and the insurance fund. Those are separate contracts, but their settings
change your yield and, in one case, your ability to withdraw, and they are not
all controlled by the same key as the market.

## Conventions

|                       |                                           |
|-----------------------|-------------------------------------------|
| Percentages           | basis points (`10000` = 100%, `500` = 5%) |
| Token amounts         | `i128`, 7 decimals (Stellar standard)     |
| Oracle prices         | 14 decimals                               |
| Durations             | seconds                                   |
| Collateral thresholds | USD cents                                 |

You don't need a transaction to read any of this. `get_market_data()` returns
every pool's config along with current APY, share rates and oracle price,
`get_pool_data(pool)` does the same for a single pool, and `get_global_state()`
returns the market-level settings.

---

## 1. What determines your yield

A pool's borrow rate depends on how much of it is currently lent out. Lenders
get that rate, scaled by utilization, less the protocol's cut:

```text
U            = total_borrowed / total_supply
borrow_APR   = kinked_curve(U) × interest_rate_modifier
supply_APR   = borrow_APR × U × (1 − take_rate)
APY          = (1 + APR / 31_556_926) ^ 31_556_926 − 1     // compounded per second
```

At stock defaults, with the pool 70% utilized:

```text
borrow_APR = 30%                    (U sits exactly on kink 1)
supply_APR = 30% × 0.70 × 0.90 = 18.9%
supply_APY ≈ 20.8%
```

`total_supply` here means available liquidity plus outstanding borrows.
Protocol fees that have accrued but not yet been distributed are excluded from
available liquidity, so they aren't part of what lenders can withdraw against.

Collateral doesn't earn. Assets you add with `add_collateral` back borrowing
and sit in a separate balance from the lendable pool. Only `deposit` mints
j-tokens, which are the yield-bearing supply shares.

---

## 2. Interest rate model

Each pool has its own two-kink curve. `borrow_APR` runs linearly between four
points: `(0, base)`, `(U_k1, APR_k1)`, `(U_k2, APR_k2)` and `(100%, APR_max)`.

| Parameter       | Meaning                  | Default        | Bounds                            |
|-----------------|--------------------------|----------------|-----------------------------------|
| `base_apr_bps`  | Rate at 0% utilization   | `1` (0.01%)    | `0 ≤ base ≤ kink1_apr`            |
| `kink1_ur_bps`  | First kink utilization   | `7000` (70%)   | `0 < k1 < k2 < 100%`              |
| `kink1_apr_bps` | Rate at kink 1           | `3000` (30%)   | `≥ base`                          |
| `kink2_ur_bps`  | Second kink utilization  | `8000` (80%)   | `> k1`, `< 100%`                  |
| `kink2_apr_bps` | Rate at kink 2           | `6000` (60%)   | `≥ kink1_apr`                     |
| `max_apr_bps`   | Rate at 100% utilization | `40000` (400%) | `≥ kink2_apr`, `≤ 200000` (2000%) |
| `accrual_model` | Compounding method       | `Compounded`   | only value defined                |

The last segment is steep on purpose. At defaults, a pool that is 90% utilized
charges 230% APR, and that is what pulls liquidity back into a pool that has
been drained.

### Rate controller (optional)

An optional modifier scales the whole curve up or down. It is recalculated on
every interest accrual, starts at `1.0×`, and is clamped between `0.1×` and
`10×`.

| Parameter                      | Meaning                                  | Default      | Bounds      |
|--------------------------------|------------------------------------------|--------------|-------------|
| `ir_reactivity_constant`       | Drift speed; `0` disables the controller | `0`          | `0..=100`   |
| `target_utilization_ratio_bps` | Utilization the controller steers around | `6500` (65%) | `0..=10000` |

When utilization sits above the target the modifier drifts down; below the
target it drifts up. Each step is proportional to
`elapsed_seconds × |U − target| × reactivity`, so at maximum reactivity it
reaches its clamp within a few hours.

The default is `0`, which pins the modifier at `1.0×` and leaves the kinked
curve in sole control of the rate.

---

## 3. Fees

There are two kinds. The take rate is continuous: a share of borrower interest
skimmed before it reaches lenders, which is why the supply APY you see is
already net of it. Operation fees are one-off charges on individual actions.

| Parameter                       | Charged on                               | Default      | Ceiling                               |
|---------------------------------|------------------------------------------|--------------|---------------------------------------|
| `take_rate_bps`                 | Borrower interest, continuously          | `1000` (10%) | `< 10000`                             |
| `deposit_fee_bps`               | Supplying                                | `0`          | `< 10000`                             |
| `borrow_fee_bps`                | Borrowing                                | `0`          | `< 10000`                             |
| `add_collateral_fee_bps`        | Adding collateral                        | `0`          | `< 10000`                             |
| `flash_loan_fee_bps`            | Flash loan principal                     | `1` (0.01%)  | `< 10000`                             |
| `withdraw_fee_bps`              | Withdrawing supply                       | `0`          | `1000` (10%)                          |
| `repay_fee_bps`                 | Repaying debt                            | `0`          | `1000` (10%)                          |
| `remove_collateral_fee_bps`     | Removing collateral                      | `0`          | `1000` (10%)                          |
| `withdraw_max_scarcity_fee_bps` | Surcharge on scarce withdrawals (see §5) | `500` (5%)   | `withdraw_fee + scarcity_fee < 10000` |

Three exit fees are capped in the contract. `withdraw`, `repay` and
`remove_collateral` are treated as constrained fees: they can never go above
10%, and no single governance update can raise any of them by more than 3
percentage points.

That cap does not cover every exit-side charge.
`withdraw_max_scarcity_fee_bps` is validated only against overflow — it must
stay below 100%, and its sum with `withdraw_fee_bps` must too — so a single
governance update can take it from 5% to just under 100%. Withdrawing under
scarcity is therefore the one exit governance *can* make expensive after the
fact, and the constrained-fee rule does not prevent it. Entry-side fees
(`deposit`, `borrow`, `add_collateral`, `flash_loan`) have no equivalent limit
either.

`take_rate_beneficiaries`, `operation_fee_beneficiaries` and `referrers` are
address-to-share maps that decide where fees go. The first two have to sum to
exactly 100%, referrer shares to at most 100%. Sending a share to the insurance
fund is how the buffer in §6 gets funded.

---

## 4. Risk parameters

These apply to borrowers, which is indirectly how they apply to you: they
decide whether lenders get repaid.

| Parameter                       | Meaning                                           | Default        | Bounds                     |
|---------------------------------|---------------------------------------------------|----------------|----------------------------|
| `open_ltv_bps`                  | Max LTV at which a borrow may be opened           | `7000` (70%)   | `0..=10000`, `< close_ltv` |
| `close_ltv_bps`                 | LTV at which the position becomes liquidatable    | `8000` (80%)   | `0..=10000`, `≥ open_ltv`  |
| `liability_factor_bps`          | Debt multiplier applied to volatile borrow assets | `10000` (1.0×) | `10000..=20000`            |
| `liquidation_close_factor_bps`  | Max share of a debt closable in one liquidation   | `5000` (50%)   | `0..=10000`                |
| `max_liquidation_incentive_bps` | Max discount handed to the liquidator             | `1000` (10%)   | `50..=10000`               |
| `utilization_ratio_limit_bps`   | Utilization ceiling reachable by borrowing        | `9000` (90%)   | `0..=10000`                |
| `supply_limit`                  | Cap on lendable supply; `0` = unlimited           | `0`            | `≥ 0`                      |

The gap between `open_ltv` and `close_ltv` is the borrower's cushion before
liquidation. A wider gap makes it less likely that a price move turns into bad
debt. If a liquidation touches two pools, the incentive actually paid is the
lower of the two pools' `max_liquidation_incentive_bps`.

`utilization_ratio_limit_bps` only gates borrowing. Utilization can still climb
past it through accrued interest or withdrawals, and once it does, the scarcity
rules in §5 apply.

`supply_limit` is measured against available plus borrowed, so accrued interest
slowly eats into the room for new deposits. Collateral isn't counted. With
7-decimal precision, a 10M-token cap is `100_000_000_000_000`.

---

## 5. Liquidity and withdrawal mechanics

These come into play when a withdrawal would push the pool's utilization above
`utilization_ratio_limit_bps`.

| Parameter                       | Meaning                                                                    | Default        | Bounds           |
|---------------------------------|----------------------------------------------------------------------------|----------------|------------------|
| `withdraw_scarcity_limit_bps`   | Max single withdrawal, as a share of total supply, once the pool is scarce | `10000` (100%) | `0..=10000`      |
| `withdraw_scarcity_cooldown_s`  | Required gap between a lender's scarcity withdrawals                       | `0`            | `< 86400` (24 h) |
| `withdraw_max_scarcity_fee_bps` | Surcharge at 100% utilization                                              | `500` (5%)     | see §3           |

The surcharge starts at zero when the pool is at its utilization limit and
rises linearly to the full `withdraw_max_scarcity_fee_bps` at 100% utilization.
It is added on top of `withdraw_fee_bps` and calculated on the utilization the
pool would have *after* the withdrawal, so it prices the liquidity you are
removing.

Going over `withdraw_scarcity_limit_bps`, or withdrawing before the cooldown
expires, fails outright with `WithdrawScarcityOverLimit` or
`ScarcityCooldownPeriod`. Both are inert at stock defaults (the limit is 100%,
the cooldown is zero), so unless governance has tightened them you'll only see
the surcharge.

`simulate_withdraw()` tells you the exact amount and fee before you sign.

---

## 6. Operation switches

### Pool status flags

Four independent bits per pool, all on by default, toggled by governance with
immediate effect: `deposit_enabled`, `borrow_enabled`,
`add_collateral_enabled`, `flash_loan_enabled`.

There is no flag for withdrawal or repayment, but that is not the same as
supply always being retrievable. The scarcity controls of §5 are pool-level and
both accept zero: set `utilization_ratio_limit_bps` and
`withdraw_scarcity_limit_bps` to `0` and the permitted withdrawal computes to
zero, so every withdrawal from a pool carrying any debt reverts with
`WithdrawScarcityOverLimit`. Governance cannot switch withdrawals off with a
flag, but it can starve them to nothing through those two parameters — under
the usual 12-hour pool-config timelock.

A pool with a farm attached (§9) has a second such path. Withdrawing refreshes
the farm stake, and the farms contract refuses that call while the farm is
frozen, so the withdrawal reverts with it. The farm admin is therefore also a
party that can hold up exits from the pool.

### Market status

The market has one status: `Active`, `BorrowFrozen`, `DepositFrozen` or
`Frozen`, plus `…ByAdmin` variants of the last three.

| Status          | New borrows | New deposits | Withdraw / repay / liquidate |
|-----------------|-------------|--------------|------------------------------|
| `Active`        | yes         | yes          | yes                          |
| `BorrowFrozen`  | no          | yes          | yes                          |
| `DepositFrozen` | no          | no           | yes                          |
| `Frozen`        | no          | no           | yes                          |

None of these block exits. Withdrawal, repayment, collateral removal and
liquidation work at every freeze level, and the test suite asserts this
directly (`tests/src/update.rs`). Freezing a market stops it taking on new
risk; it doesn't hold your funds.

Two things outside the status system can pause withdrawals: the bad-debt lock
below, and a frozen incentive farm attached to the pool (§9).

The insurance fund can freeze the market on its own authority, as a circuit
breaker. It can't set or clear the `…ByAdmin` variants, which only the market
admin can move.

### Bad-debt lock

When a liquidation leaves a shortfall that isn't covered, a claim opens against
the insurance fund and the affected pool locks for `bad_debt_lock_d` seconds.
While it's locked, deposit, withdraw, borrow and flash loan are blocked on that
pool. Repay, collateral changes and liquidation stay open. The point is to stop
lenders racing for the exit ahead of a loss that is about to be shared out.

The lock only lifts once every outstanding claim on the pool has settled;
waiting out the duration isn't enough on its own. A claim that sits pending
past its deadline can be cancelled by anyone, so an unanswered claim can't hold
the lock open indefinitely.

Losses are absorbed in order: liquidation proceeds first, then the pool's
insurance fund, and only what's left is socialized across that pool's lenders.
Whether that last step can happen, and how much it costs, is the main tail risk
in everything above.

---

## 7. Market-level parameters

| Parameter                    | Meaning                                                        | Default        | Bounds                |
|------------------------------|----------------------------------------------------------------|----------------|-----------------------|
| `max_positions`              | Max simultaneous positions per obligation                      | `20`           | `2..=25`              |
| `min_collateral_value_cents` | Dust floor each borrow-backing collateral position must retain | `500` ($5)     | `0..=10000` ($100)    |
| `bad_debt_lock_d`            | Lock duration set on a pool when a bad-debt claim opens        | `43200` (12 h) | `0..=432000` (5 days) |

### Fixed at deployment

These are written once in the constructor. Nothing can change them afterwards,
including the admin.

| Parameter                | Meaning                                               | Bounds                         |
|--------------------------|-------------------------------------------------------|--------------------------------|
| `oracle`                 | SEP-40 price feed address                             | any                            |
| `insurance_fund`         | Insurance fund contract address                       | any                            |
| `name`                   | Market name                                           | any                            |
| `insolvency_ltv_bps`     | LTV above which a liquidation is treated as insolvent | `9500..=10000`, default `9850` |
| `update_in_queue_period` | Governance timelock for all config changes            | not range-checked              |
| `is_owned`               | Whether the market has governance at all              | —                              |

`is_owned = false` makes the market immutable. Every path that changes
configuration (market update, pool config, pool status, market status) checks
this flag first. Where it's false, the existing pools stay exactly as
configured, permanently, and the admin can only add new pools.

The oracle can't be swapped either. Prices older than 6 minutes are rejected,
so a stalled feed stops price-dependent operations instead of letting them run
on old data.

`update_in_queue_period` is worth a look before you supply. It's the delay on
every economic change, it's chosen freely at deployment, and the contract
doesn't enforce a minimum. The test suite uses 24 hours throughout, but that's
a convention rather than a rule. Read the deployed value from
`get_global_state()`.

---

## 8. Who can change what, and how fast

| Change                                                                          | Authority                                   | Delay                    |
|---------------------------------------------------------------------------------|---------------------------------------------|--------------------------|
| Pool config: rates, fees, LTVs, limits                                          | Admin queues, anyone applies                | `update_in_queue_period` |
| Market config: `max_positions`, `min_collateral_value_cents`, `bad_debt_lock_d` | Admin queues, anyone applies                | `update_in_queue_period` |
| Pool status flags                                                               | Admin                                       | immediate                |
| Market status (freeze)                                                          | Admin                                       | immediate                |
| Emergency market freeze                                                         | Insurance fund                              | immediate                |
| Fee beneficiaries, farms                                                        | Admin                                       | immediate                |
| Contract upgrade                                                                | Market Manager admin **and** market admin queue, anyone applies | per-market, `43200` (12 h) on `main` |
| Admin transfer                                                                  | Two-step propose / accept                   | immediate on accept      |

Queued changes are visible before they take effect.
`get_queued_pool_set(pool)` and `get_market_queued_in_update()` show what's
pending and when it was queued, so you can see an unwelcome change coming and
leave before it lands, assuming the timelock is long enough to be useful.

Once the delay has passed, anyone can apply a queued change. Governance can't
sit on an approved change and spring it later.

The per-market upgrade delay is fixed when the market is deployed and has no
setter: `60 s` is only the floor `MarketManager::deploy` validates against, not
what any market runs. The live `main` market is set to `43200` — read it from the
manager's `DeployedMarket(<market>)` entry rather than from this table.

Freezes are immediate while config changes wait, on the reasoning that making
the protocol safer shouldn't be delayed and changing its economics should be.

---

## 9. Incentive farms (optional, per pool)

Farms live in a separate contract. A pool can have a supply farm, which rewards
j-token holders, and a debt farm, which rewards d-token holders. The Market
keeps stakes in sync for you, so there's nothing to stake by hand. Attaching or
clearing a farm is an admin action, with no timelock on it.

Farm rewards come on top of the supply APY in §1. They are not included in the
numbers `get_apy()` returns.

| Parameter                   | Meaning                                                      | Bounds                     |
|-----------------------------|--------------------------------------------------------------|----------------------------|
| `reward_token`              | Token paid out; up to 10 per farm                            | `≤ 10` rewards             |
| `reward_type`               | `Proportional` (split by stake) or `Constant`                | —                          |
| `reward_schedule_curve`     | Emission schedule: `(ts_start, reward_per_time_unit)` points | `1..=20` points, ascending |
| `treasury_fee_bps`          | Cut of rewards taken by the farm treasury                    | `0..=2000` (20%)           |
| `deposit_cap`               | Cap on total staked                                          | `≥ 0`                      |
| `min_stake_amount`          | Minimum stake to participate                                 | `≥ 0`                      |
| `min_harvest_delay`         | Minimum gap between harvests                                 | `≤ 86400` (24 h)           |
| `is_harvest_permissionless` | Whether anyone can trigger a harvest                         | —                          |
| `is_reward_once_enabled`    | One-shot reward mode                                         | —                          |

Emissions aren't committed capital. The farm admin can reschedule the curve
with `set_reward_schedule` and pull unfunded rewards back out with
`withdraw_unused_rewards`, at any time. An advertised farm APR is a current
rate, not a promise.

The farm admin is a different key from the market admin, and farms can be
frozen. The Market syncs stake with no fallback if that call fails, so a frozen
farm makes every stake-touching operation on the pool revert, withdrawal
included. This is the one way someone other than the market admin can block
exits, so it's worth finding out who holds the farm admin key before supplying
to a pool that has incentives on it.

The farm contract also has locking parameters: `locking_duration` (up to 365
days), `deposit_warmup_period`, `withdrawal_cooldown_period` and
`early_withdrawal_penalty_bps`. These only apply to non-delegated farms. Farms
attached to a market are always delegated, so none of them affect lending LPs.

---

## 10. Price oracle

The market's `oracle` address is fixed at deployment (§7). Any price older than
6 minutes is rejected, as is any price timestamped in the future, so a stalled
feed halts price-dependent operations rather than letting them run on stale
data.

If the market points at the Aggregated Oracle, that contract takes a median
across several SEP-40 feeds and adds a deviation check on top.

| Parameter                        | Meaning                                                               | Bounds                      | Changeable            |
|----------------------------------|-----------------------------------------------------------------------|-----------------------------|-----------------------|
| `oracles`                        | Source feeds aggregated into a median                                 | `1..=10`                    | fixed at construction |
| `base_asset`, `decimals`         | Quote asset and price precision                                       | —                           | fixed at construction |
| `periodic_oracles_price_max_age` | Staleness ceiling for cadence-based feeds                             | `60..=43200` (1 min – 12 h) | fixed at construction |
| `max_dev_bps`                    | Max move between two consecutive medians before the price is rejected | per asset                   | fixed at registration |
| `max_dev_consecutive_diff_secs`  | Window over which that deviation check applies                        | per asset                   | fixed at registration |

Both deviation parameters are written once, by `add_asset`, and there is no way
to change them afterwards: re-registering an asset fails with
`AssetAlreadyRegistered`, and the oracle exposes neither a setter nor a removal
call. Correcting either value means deploying a new oracle and repointing the
market at it. Get them right before the first `add_asset`.

The deviation check cuts both ways. If the median moves more than `max_dev_bps`
inside `max_dev_consecutive_diff_secs`, the oracle returns no price at all.
That stops a manipulated feed from driving liquidations, and it also stops
liquidations during a real, fast crash, which is exactly when bad debt forms.
Worth checking on any specific deployment, along with the source count: at
`oracles = 1` there is no median protection at all.

---

## 11. Insurance fund

The fund sits between a bad liquidation and losses being shared out among
lenders. It has no risk parameters, just an admin and a market it's bound to.

Coverage is discretionary rather than formulaic. When a shortfall opens a
claim, the fund admin calls `mark_ready(request_id, covered_amount)` and picks
the amount, anywhere from nothing up to the full claim. The admin can also
withdraw any unlocked balance whenever they want. There's no minimum reserve,
no bonding, and no on-chain commitment to cover any particular share of a loss.

So the protection the fund offers is an operational commitment from whoever
runs it, not something the protocol enforces. What you can verify on-chain is
its current token balance and the share of pool fees routed to it through
`take_rate_beneficiaries` and `operation_fee_beneficiaries` (§3). Both are
worth checking.

The fund can also freeze the market as a circuit breaker, though it can't touch
the `…ByAdmin` statuses (§6).

---

## 12. Due diligence checklist

Before supplying, read these on-chain and check they match what you were told:

1. `update_in_queue_period` and `is_owned` from `get_global_state()`: how much
   notice you get before economics change, and whether they can change at all.
2. `admin`, the key that controls queueing, and whether it is a multisig.
3. `take_rate_bps`, your cut of the borrow rate.
4. The kinked curve, and where the rate goes if utilization spikes.
5. `open_ltv` / `close_ltv` / `liability_factor` on every pool that can borrow
   against your asset. This is your exposure to their collateral quality.
6. `max_liquidation_incentive_bps`, and whether liquidators are paid enough to
   act before positions turn insolvent.
7. `withdraw_scarcity_limit_bps` and `withdraw_scarcity_cooldown_s`, in case a
   scarce pool caps your exit size.
8. The `insurance_fund` balance, its share of the fee routing, and who controls
   it.
9. The number of oracle sources and `max_dev_bps`: whether a single feed can
   move your market, and how easily the circuit breaker halts liquidations.
10. Whether the pool has a farm attached, and who holds the farm admin key. A
    frozen farm blocks withdrawals on that pool.
11. Anything pending via `get_queued_pool_set()` /
    `get_market_queued_in_update()`.

Items 1, 2 and 8 through 10 are the ones you can't undo by withdrawing: they
decide who can act on your position and how much warning you get. The rest is
economics you can reassess whenever you like.
