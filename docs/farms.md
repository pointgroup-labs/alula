# Farms Contract

A general-purpose staking and yield farming contract for the Stellar/Soroban ecosystem. Deploy standalone farms for any Stellar token with configurable reward emissions, lockups, and delegation support.

## Overview

The Farms contract is a protocol-agnostic building block for the Soroban ecosystem. Any project can deploy farm instances to incentivize user behavior — staking LP tokens, rewarding depositors in a lending protocol, distributing governance tokens, running airdrops, or any scenario where tokens should be distributed to participants over time.

Each deployed contract instance manages a single farm. Deploy multiple instances for multiple farms.

## Key Features

- **Multi-reward** — up to 10 reward tokens per farm, each with independent emission schedules
- **Two staking modes** — direct user staking or delegated staking for protocol integrations
- **Flexible lockups** — optional warmup periods, withdrawal cooldowns, and linear-decay early withdrawal penalties
- **Configurable emission curves** — piecewise-constant schedules with up to 20 segments
- **Oracle-based deposit caps** — optional SEP-40 oracle integration for USD-denominated caps
- **Treasury fees** — configurable protocol fee on harvested rewards (0–20%)
- **Permissionless harvesting** — optional mode allowing anyone to trigger reward claims for any user
- **Direct reward credits** — admin can airdrop rewards to specific users
- **Two-step admin transfer** — safe admin key rotation via propose/accept pattern
- **Fee-on-transfer protection** — rejects tokens that don't deliver the exact transfer amount
- **Upgradeable** — admin-gated WASM upgrade support

## Staking Modes

### Non-Delegated

Users directly deposit and withdraw tokens from the farm contract. This mode supports the full lifecycle of staking mechanics:

- **Deposit warmup** — new stakes can be held in a pending state for a configurable period before they start earning rewards. Users can cancel pending deposits at any time to recover their tokens.
- **Withdrawal cooldown** — after initiating an unstake, users must wait a configurable period before they can withdraw their tokens.
- **Locking with penalties** — stakes can be locked for a configurable duration. Early withdrawal applies a linear-decay penalty that starts at the configured maximum and decreases to zero at lock end. Slashed amounts are held by the contract for admin recovery.

Best for: standalone staking pools, LP farming, governance token locking.

### Delegated

An external contract (the "delegate authority") manages user stakes. Users don't move tokens directly — the delegate syncs stake amounts with positions tracked in the integrating protocol. No warmup, cooldown, or penalties apply — the delegate authority handles position lifecycle.

Supports two authorities per farm (primary and secondary) for flexibility in multi-contract architectures.

Best for: lending protocols, AMMs, vaults, or any system where user positions are managed by another contract.

## Staking Identity

Users are identified by an owner address and an optional seed. The simple mode (owner only) supports one position per user. The seeded mode (owner + 32-byte seed) allows multiple positions per user — useful for protocols that track separate obligations, vaults, or collateral types per user.

## Reward System

### Multiple Reward Tokens

Each farm supports up to 10 independent reward tokens. Each reward token has its own funding pool, emission schedule, and accumulator. This allows a single farm to distribute multiple incentive tokens simultaneously (e.g., governance token + stablecoin yield + partner tokens).

### Reward Distribution

Rewards are distributed via a **rewards-per-share accumulator** — a constant-time algorithm that fairly distributes emissions proportional to each user's stake, regardless of the number of participants. No loops over users, no snapshots — O(1) per operation.

### Distribution Modes

- **Proportional** — rewards scale with stake amount. A user with 2× the stake earns 2× the rewards.
- **Constant** — every staker earns an equal share regardless of stake size. Each active user counts equally.

### Emission Schedules

Rewards are emitted according to a piecewise-constant schedule defined by up to 20 time segments. Each segment specifies a start timestamp and an emission rate (tokens per second). The last segment's rate continues indefinitely until the funded pool is exhausted.

Example schedule:
- Month 1–3: 100 tokens/second (bootstrapping phase)
- Month 4–6: 50 tokens/second (gradual reduction)
- Month 7+: 10 tokens/second (long-term maintenance)

Emissions automatically stop when the funded reward pool is depleted, regardless of the schedule.

### Treasury Fees

A configurable percentage (0–20%) is deducted from harvested rewards. These fees accumulate per reward token and can be withdrawn by the admin at any time.

### Direct Reward Credits

When enabled, the admin can credit rewards directly to specific users, bypassing the normal proportional distribution. This is useful for airdrops, bonus rewards, or retroactive corrections.

## Locking Modes

### None
No locking, no penalties. Users can unstake freely at any time.

### Continuous
The lock window starts from each user's most recent stake timestamp. Every user has their own independent lock period. Staking again resets the lock timer.

### With Expiry
The lock window starts from a global admin-configured timestamp. All users share the same lock deadline. After the expiry, all users can unstake without penalty.

### Penalty Calculation

When locking is active, early withdrawal applies a linear-decay penalty:

```
effective_penalty = max_penalty × time_remaining / total_lock_duration
```

- At lock start → full penalty applies
- Halfway through → half penalty
- At lock end → zero penalty

Penalty amounts are slashed from the user's withdrawal and held by the contract. The admin can recover slashed tokens at any time.

## Oracle Integration

Farms optionally integrate with SEP-40 price oracles to enforce deposit caps in USD terms rather than token terms. When an oracle is configured:

- The deposit cap is evaluated against the USD value of total staked tokens
- Oracle prices are checked for staleness and validity (zero/negative prices are rejected)
- This prevents deposit caps from becoming meaningless during price volatility

## Admin Operations

### Farm Configuration

The admin can update farm parameters at any time:

- **Deposit cap** — maximum total staked value (0 = unlimited)
- **Minimum stake amount** — per-operation minimum
- **Minimum harvest delay** — prevents harvest spam
- **Treasury fee** — protocol fee on harvested rewards
- **Oracle** — set or clear the price oracle for USD-denominated caps
- **Harvest permissions** — toggle between owner-only and permissionless harvesting
- **Direct reward credits** — enable or disable `reward_once`

For non-delegated farms, locking parameters (lock duration, penalty, cooldown, warmup) can only be changed when there are no active stakes.

For delegated farms, the delegate authority can be changed at any time.

### Farm Lifecycle

- **Freeze/Unfreeze** — frozen farms reject new stakes but allow unstaking, harvesting, and withdrawals. Farms start frozen at deployment to allow setup before going live.
- **Upgrade** — admin can upgrade the contract WASM to a new version.

### Reward Management

- **Initialize** — register a new reward token (up to 10 per farm)
- **Fund** — anyone can add tokens to an initialized reward pool
- **Schedule** — admin sets the emission curve
- **Withdraw unused** — admin recovers unissued reward tokens
- **Withdraw treasury fees** — admin claims accumulated protocol fees
- **Withdraw slashed** — admin recovers early withdrawal penalty amounts

## User Flows

### Non-Delegated: Stake → Earn → Withdraw

```
1. Stake tokens
   ├── [if warmup configured] → tokens held in pending state
   │   ├── After warmup: activate via refresh_farming_position
   │   └── Cancel anytime: cancel_pending_deposit → full refund
   └── [no warmup] → stake activates immediately, starts earning

2. Rewards accrue over time based on stake share

3. Harvest rewards (single token or all at once)

4. Unstake
   ├── [if within lock period] → penalty applied, net amount enters cooldown
   └── [if lock expired or no lock] → full amount enters cooldown

5. Withdraw tokens after cooldown period
```

### Delegated: Protocol Integration

```
1. User interacts with integrating protocol
   → Protocol calls set_stake_delegated with user's updated position

2. Rewards accrue proportionally based on stake

3. User claims rewards directly from the farm contract

4. Position changes in protocol
   → Protocol calls set_stake_delegated with new balance
```

### Admin: Setting Up a Farm

```
1. Deploy farm contract with configuration
2. Register reward token(s) with distribution mode
3. Fund the reward pool(s)
4. Set emission schedule(s)
5. Unfreeze the farm to start accepting stakes
```

## Integration Guide

The `farms_interface` crate provides a lightweight client for cross-contract calls without importing the full farm implementation. This keeps integrating contracts lean.

### Lending Protocol

Reward depositors and borrowers by deploying delegated farms:

1. Deploy a **supply farm** per pool — reward users for providing liquidity
2. Deploy a **debt farm** per pool (optional) — incentivize borrowing in specific markets
3. On every deposit, withdraw, borrow, repay, or liquidation, sync the user's updated position to the farm
4. Users claim rewards directly from the farm

### DEX / AMM

Incentivize liquidity provision:

1. Deploy a non-delegated farm with the LP token as the staked asset
2. Configure locking (e.g., 30-day lock with 50% early withdrawal penalty)
3. Fund with governance tokens on a 6-month emission schedule
4. Users stake LP tokens and earn rewards proportionally

### Airdrop / Points Distribution

Distribute tokens to qualifying holders:

1. Deploy a farm with direct reward credits enabled
2. Users stake a qualifying token
3. Admin credits specific amounts to eligible users
4. Users harvest at their convenience

### Governance Staking

Incentivize long-term token holding:

1. Deploy a non-delegated farm with the governance token
2. Configure a 1-year lock with 100% early withdrawal penalty (effective cliff)
3. Distribute voting power or additional governance tokens as rewards
4. The linear penalty decay creates a natural vesting curve

### Real Yield / Revenue Sharing

Distribute protocol revenue to token holders:

1. Deploy a non-delegated farm with the protocol's governance or utility token
2. Initialize a stablecoin (e.g., USDC) as the reward token
3. Periodically fund the reward pool from protocol revenue
4. Set an emission schedule that distributes funded amounts over the desired period
5. Stakers earn real yield proportional to their share

### Launchpad / Token Sale Allocation

Allocate new token launches to committed stakers:

1. Deploy a non-delegated farm with a qualifying token (e.g., the platform's native token)
2. Configure locking with an expiry matching the launch date
3. Initialize the launched token as the reward
4. Fund and schedule emissions to distribute allocation proportionally
5. Stakers earn launch tokens while locked — early exit forfeits allocation via penalty

### Multi-Protocol Partnership Rewards

Two or more protocols can co-incentivize a shared user base:

1. Deploy a single farm with a shared staking token
2. Each protocol initializes their own reward token on the same farm
3. Each protocol independently funds and schedules their emissions
4. Users stake once and earn rewards from all participating protocols simultaneously

### Real World Assets (RWA)

Distribute yield from tokenized real-world assets — bonds, real estate, commodities, or treasury bills:

1. Deploy a delegated farm with the RWA token as the staked asset
2. The issuing platform acts as delegate authority, syncing holder balances and enforcing compliance (KYC/AML) before allowing positions
3. Initialize a stablecoin as the reward token to represent yield (e.g., bond coupon payments)
4. Fund the reward pool from real-world revenue and schedule emissions to match payment periods (monthly, quarterly)
5. Use the **Constant** distribution mode for fixed-income products where every holder earns equal yield regardless of position size, or **Proportional** for equity-like instruments
6. Configure deposit caps via SEP-40 oracle to enforce regulatory limits on total asset value
7. Seeded staking identities allow the same holder to have separate positions across different RWA tranches or maturities

### Payment & Remittance Incentives

Reward users for transaction activity on payment corridors:

1. Deploy a delegated farm — the payment platform acts as delegate authority
2. The platform syncs a user's cumulative transaction volume (or rolling balance) as their farm stake
3. Initialize reward tokens to incentivize corridor usage (e.g., platform tokens or cashback in stablecoins)
4. Higher-volume users earn proportionally more rewards, driving liquidity to target corridors

## Queries

Users and frontends can inspect farm and position state without submitting transactions:

- **Farm state** — total staked, number of users, configuration, reward token list, frozen status
- **Position state** — active stake, pending deposits, pending withdrawals, reward tallies
- **Pending rewards** — simulates reward accrual and returns claimable amounts per reward token (after treasury fee deduction) without modifying state

## Limits

| Parameter                    | Max Value      |
| ---------------------------- | -------------- |
| Reward tokens per farm       | 10             |
| Emission schedule segments   | 20             |
| Treasury fee                 | 20%            |
| Harvest delay                | 1 day          |
| Deposit warmup period        | 1 day          |
| Withdrawal cooldown period   | 1 day          |
| Locking duration             | 365 days       |

## Security

- All state-modifying operations require appropriate authorization
- Atomic initialization via constructor prevents frontrunning
- Checked arithmetic throughout — no overflow/underflow risk
- Rounding always favors the protocol to prevent value extraction
- Fee-on-transfer tokens are detected and rejected
- Oracle prices are validated for staleness, zero values, and negative values
- Two-step admin transfer prevents accidental admin key loss
- Farms start frozen — no user interaction until admin explicitly unfreezes
