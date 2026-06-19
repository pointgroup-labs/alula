# Incentivix — Incentives Platform for Stellar

An open-source incentives platform for Stellar protocols and fintechs. Projects define what actions earn rewards (liquidity provisioning, lending, staking, referrals, trading, and more) and Incentivix handles the reward logic, distribution, and tracking automatically. Powered by a composable, audited Soroban smart contract that any project can deploy and integrate without custom development.

## Why Stellar Needs This

Every Stellar protocol needs a way to run incentive programs — whether it's incentivizing liquidity, rewarding borrowers, distributing governance tokens, or sharing protocol revenue. Today, each project must build this infrastructure from scratch, leading to fragmented, unaudited, and inconsistent implementations.

Incentivix solves this by providing a single, audited, composable primitive that any protocol can plug into. Instead of every team spending months building and securing their own reward logic, they launch an incentive program in minutes and focus on what makes their protocol unique.

This is the kind of shared infrastructure that accelerates ecosystem growth — one platform, used by many protocols, creating a consistent experience for users across the Stellar network.

## What Builders Can Do With It

### Liquidity Mining (DEX / AMM)

Incentivize liquidity provision with configurable lockups:

1. Deploy a program with the LP token as the staked asset
2. Configure locking (e.g., 30-day lock with 50% early withdrawal penalty)
3. Fund with governance tokens on a 6-month emission schedule
4. Users stake LP tokens and earn rewards proportionally

### Lending Protocol Rewards

Reward depositors and borrowers without building reward logic into the lending contract:

1. Deploy a **supply program** per pool — reward users for providing liquidity
2. Deploy a **debt program** per pool (optional) — incentivize borrowing in specific markets
3. On every deposit, withdraw, borrow, repay, or liquidation, the lending contract syncs the user's updated position to the program
4. Users claim rewards directly from the program

### Revenue Sharing / Real Yield

Distribute protocol revenue to token holders:

1. Deploy a program with the protocol's governance or utility token
2. Initialize a stablecoin (e.g., USDC) as the reward token
3. Periodically fund the reward pool from protocol revenue
4. Set an emission schedule that distributes funded amounts over the desired period
5. Stakers earn real yield proportional to their share

### Governance Staking & Vesting

Incentivize long-term token holding with a natural vesting curve:

1. Deploy a program with the governance token
2. Configure a 1-year lock with 100% early withdrawal penalty (effective cliff)
3. Distribute additional governance tokens as rewards
4. The linear penalty decay creates a smooth vesting schedule — early exit forfeits proportionally

### Multi-Protocol Partnership Rewards

Two or more protocols can co-incentivize a shared user base:

1. Deploy a single program with a shared staking token
2. Each protocol registers their own reward token on the same program
3. Each protocol independently funds and schedules their emissions
4. Users stake once and earn rewards from all participating protocols simultaneously

### Airdrop / Points Distribution

Distribute tokens to qualifying holders with precision:

1. Deploy a program with direct reward credits enabled
2. Users stake a qualifying token
3. Admin credits specific amounts to eligible users
4. Users harvest at their convenience

### Launchpad / Token Sale Allocation

Allocate new token launches to committed stakers:

1. Deploy a program with a qualifying token (e.g., the platform's native token)
2. Configure locking with a deadline matching the launch date
3. Fund and schedule emissions to distribute allocation proportionally
4. Stakers earn launch tokens while locked — early exit forfeits allocation via penalty

### Real World Assets (RWA)

Distribute yield from tokenized real-world assets — bonds, real estate, commodities, or treasury bills:

1. Deploy a delegated program with the RWA token as the staked asset
2. The issuing platform acts as delegate authority, syncing holder balances and enforcing compliance (KYC/AML) before allowing positions
3. Initialize a stablecoin as the reward token to represent yield (e.g., bond coupon payments)
4. Fund the reward pool from real-world revenue and schedule emissions to match payment periods (monthly, quarterly)
5. Use **Constant** distribution mode for fixed-income products where every holder earns equal yield, or **Proportional** for equity-like instruments
6. Configure deposit caps via SEP-40 oracle to enforce regulatory limits on total asset value
7. Seeded staking identities allow the same holder to have separate positions across different tranches or maturities

### Payment & Remittance Incentives

Reward users for transaction activity on payment corridors:

1. Deploy a delegated program — the payment platform acts as delegate authority
2. The platform syncs a user's cumulative transaction volume (or rolling balance) as their program stake
3. Initialize reward tokens to incentivize corridor usage (e.g., platform tokens or cashback in stablecoins)
4. Higher-volume users earn proportionally more rewards, driving liquidity to target corridors

## How It Works

Each deployed contract instance manages a single incentive program. Deploy multiple instances for multiple programs.

### Two Staking Modes

**Non-Delegated** — users directly deposit and withdraw tokens from the program contract. Supports the full lifecycle: deposit warmup, withdrawal cooldown, and locking with early withdrawal penalties. Best for standalone staking pools, LP incentive programs, and governance locking.

**Delegated** — an external contract (the "delegate authority") manages user stakes. The delegate syncs stake amounts with positions tracked in the integrating protocol. No warmup, cooldown, or penalties apply — the integrating protocol handles position lifecycle. Supports two authorities per program for multi-contract architectures. Best for lending protocols, AMMs, vaults, or any system where positions are managed by another contract.

### Multi-Reward System

Each program supports up to 10 independent reward tokens, each with its own funding pool, emission schedule, and accumulator. A single program can distribute governance tokens, stablecoins, and partner tokens simultaneously.

Rewards are distributed via a **rewards-per-share accumulator** — a constant-time algorithm that fairly distributes emissions proportional to each user's stake, regardless of the number of participants. O(1) per operation — no loops, no snapshots, no scaling bottlenecks.

**Distribution modes:**

- **Proportional** — rewards scale with stake amount. 2× the stake earns 2× the rewards.
- **Constant** — every staker earns an equal share regardless of stake size.

### Configurable Emission Curves

Rewards follow piecewise-constant schedules with up to 20 time segments. Each segment defines a rate (tokens per second) that applies until the next segment begins:

- Month 1–3: 100 tokens/second (bootstrapping phase)
- Month 4–6: 50 tokens/second (gradual reduction)
- Month 7+: 10 tokens/second (long-term maintenance)

Emissions automatically stop when the funded reward pool is depleted.

### Flexible Lockups

Three locking modes support different use cases:

- **None** — no locking, free withdrawal at any time
- **Continuous** — lock window starts from each user's most recent stake. Every user has an independent lock period. Staking again resets the timer.
- **With Expiry** — lock window starts from a global admin-configured timestamp. All users share the same deadline. After expiry, all users unstake without penalty.

When locking is active, early withdrawal applies a **linear-decay penalty** — full penalty at lock start, decreasing to zero at lock end. Slashed amounts are held by the contract for admin recovery.

### Staking Identity

Users are identified by an owner address and an optional seed. Simple mode (owner only) supports one position per user. Seeded mode (owner + seed) allows multiple positions per user — useful for protocols that track separate obligations, vaults, or collateral types per user.

### Oracle Integration

Programs optionally integrate with SEP-40 price oracles to enforce deposit caps in USD terms rather than token terms. Oracle prices are validated for staleness and zero/negative values. This prevents deposit caps from becoming meaningless during price volatility.

### Treasury Fees

A configurable percentage (0–20%) is deducted from harvested rewards. Fees accumulate per reward token and can be withdrawn by the admin at any time.

### Direct Reward Credits

When enabled, the admin can credit rewards directly to specific users, bypassing proportional distribution. Useful for airdrops, bonus rewards, or retroactive corrections.

## Admin Operations

### Program Setup

1. Deploy the program contract with configuration
2. Register reward token(s) with distribution mode
3. Fund the reward pool(s)
4. Set emission schedule(s)
5. Unfreeze the program to start accepting stakes

### Configuration

The admin can update program parameters at any time: deposit cap, minimum stake amount, harvest delay, treasury fee, oracle, harvest permissions, and direct reward credits.

For non-delegated programs, locking parameters can only be changed when there are no active stakes. For delegated programs, the delegate authority can be changed at any time.

### Lifecycle

- **Freeze/Unfreeze** — frozen programs reject new stakes but allow unstaking, harvesting, and withdrawals. Programs start frozen at deployment to allow setup before going live.
- **Upgrade** — admin can upgrade the contract to a new version.
- **Two-step admin transfer** — safe admin key rotation via propose/accept pattern prevents accidental key loss.

### Reward Management

- Register new reward tokens (up to 10 per program)
- Anyone can fund an initialized reward pool
- Admin sets and updates the emission schedule
- Admin recovers unissued reward tokens, accumulated treasury fees, or early withdrawal penalty amounts

## Queries

Users and frontends can inspect state without submitting transactions:

- **Program state** — total staked, number of users, configuration, reward token list, frozen status
- **Position state** — active stake, pending deposits, pending withdrawals, reward tallies
- **Pending rewards** — simulates reward accrual and returns claimable amounts per reward token (after treasury fee deduction) without modifying state

## Limits

| Parameter                  | Max Value |
| -------------------------- | --------- |
| Reward tokens per program  | 10        |
| Emission schedule segments | 20        |
| Treasury fee               | 20%       |
| Harvest delay              | 1 day     |
| Deposit warmup period      | 1 day     |
| Withdrawal cooldown period | 1 day     |
| Locking duration           | 365 days  |

## Security

- All state-modifying operations require appropriate authorization
- Atomic initialization via constructor prevents frontrunning
- Checked arithmetic throughout — no overflow/underflow risk
- Rounding always favors the protocol to prevent value extraction
- Fee-on-transfer tokens are detected and rejected
- Oracle prices are validated for staleness, zero values, and negative values
- Two-step admin transfer prevents accidental admin key loss
- Programs start frozen — no user interaction until admin explicitly unfreezes

## Open Source

Incentivix is released as a public good for the Stellar ecosystem. Any project can deploy, integrate, and build on top of it. A lightweight interface crate is provided for cross-contract integration without importing the full implementation, keeping integrating contracts lean.

The contract is designed to be a shared standard — the more protocols that adopt it, the more consistent the user experience across the Stellar network, and the less duplicated security-critical code in the ecosystem.
