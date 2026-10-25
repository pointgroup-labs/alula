# Alula

[![Stellar](https://img.shields.io/badge/Stellar-grey?logo=stellar&logoColor=white&style=for-the-badge)](https://stellar.org)
[![Rust](https://img.shields.io/badge/Rust-1.95-blue?logo=rust&logoColor=white&style=for-the-badge)](https://www.rust-lang.org)
[![CI](https://img.shields.io/github/actions/workflow/status/pointgroup-labs/alula/ci.yml?logo=githubactions&logoColor=white&style=for-the-badge&label=CI)](./.github/workflows/ci.yml)

**Lending for crypto and real-world assets on Stellar.**

Lenders earn yield, borrowers draw credit against collateral spread across pools, and leveraged positions open in a single transaction. Permissioned markets bring tokenized funds on-chain: an access gate decides who may deposit, borrow, or liquidate, and a two-key oracle prices fund shares at their published net asset value. Open markets run the same contracts without a gate.

## Features

- **Isolated risk per asset** — every pool has its own LTVs, rate curve, fees, supply cap, and operation switches.
- **Cross-pool collateral** — collateral in one pool backs debt in another, across up to ten pools per position.
- **Leverage in one transaction** — EIP-3156-style flash loans and `submit_requests_batch` build leveraged positions atomically.
- **Real-world assets** — permissioned markets with pluggable access gates and NAV pricing for tokenized funds.
- **Resilient pricing** — a per-market router takes the median of up to three SEP-40 sources and prices only when they agree.
- **Exits stay open** — freezes and access gates never block repaying, withdrawing, or removing collateral.
- **Insurance fund** — bad debt above dust is claimed from the market's fund before it reaches lenders.
- **Guardian** — a dedicated role that can freeze new risk and briefly pause liquidations.
- **Adaptive rates** — a dual-kink curve per pool, with an optional controller that steers toward a target utilization.
- **Farms** — optional supply and debt incentives, staked automatically as positions change.

## How it works

Suppliers receive j-tokens (shares of a pool's supply) and borrowers d-tokens (shares of its debt); accruing interest raises the value of both, and j-tokens are valued net of protocol fees. A user's deposits, collateral, and debt in a market form an obligation; one account can hold several, isolated by seed.

### Lending and liquidation

- **Borrowing** — borrows and withdrawals value collateral at its pool's `open_ltv`, against debt weighted by each borrow pool's liability factor.
- **Liquidation** — once weighted debt exceeds collateral valued at `close_ltv`, a liquidator repays debt — up to the pool's close factor while the LTV is below `insolvency_ltv_bps` — and receives collateral plus a bounded bonus.
- **Bad debt** — when every collateral position is dust, anyone can call `issue_cover_bad_debt`. On an insolvent position, the market writes off debts up to `min_collateral_value_cents` and claims the rest from the insurance fund. While a claim is open, the pool pauses deposits, withdrawals, borrowing, and flash loans. Anyone can settle the claim once the fund pays or its deadline passes, and without calling the fund 24 hours after the deadline. The pool's lenders bear only what the fund leaves uncovered.

### Pricing

Each market reads one SEP-40 oracle, fixed at deployment, and refuses prices that are not positive, dated in the future, or dated more than six minutes ago.

- **Oracle router** — per asset, the median of up to three sources, such as Reflector, RedStone, Chainlink, or a cross rate. The router drops answers older than their source's `max_age_s` and serves the median only when at least `min_sources` of them lie within `max_deviation_bps` of it. Because it stamps its answer with the current time, each source's `max_age_s` sets freshness. Configuration changes wait out a timelock set at deployment.
- **NAV oracle** — a NAV within the growth and drawdown band goes live on the publisher's word; one outside the band waits for a second key, the confirmer. A pending rise keeps the previous NAV; a pending drop or a stale NAV stops pricing at once.
- **Aggregated oracle** — a median of SEP-40 sources with a circuit breaker on sharp moves. It dates its answer by its oldest periodic source (Reflector) or, when none answers, by the current time, so a push source such as RedStone or Chainlink counts until its adapter's resolution runs out: 13 h and 25 h on mainnet.

### Permissioned markets

An access gate answers `is_permitted(market, account, action)` for deposits, borrows, and liquidations; only an explicit `true` permits the action. Repayments, withdrawals, collateral moves, and flash loans never consult it. A new gate waits out the market's timelock; removing one takes effect at once and freezes deposits and borrowing until the admin reopens the market. Two adapters ship with the protocol: an allow or block list run by the gate's admin, and a gate that follows an Asterizm token's own allowlist and freezes.

### Emergency controls

- **Freeze** — the admin of an owned market, or its guardian, can stop deposits and borrowing at once; exits stay open.
- **Liquidation pause** — the admin, or the guardian as part of a freeze, can pause liquidations for 24 hours, after which borrowers get a 6-hour grace to repay. The admin can end the pause early; the grace still follows. The next pause can start 54 hours after the last one began; the market ignores earlier requests.
- **Token changes** — if a pool's token changes its decimals or its approved code, anyone can call `sync_token_metadata` to put the pool on hold: it takes no new deposits or collateral, and the token has no price, which halts anything that needs one. Another sync lifts the hold once the token is restored or the timelock approves its new code.

### Interest rates

Borrow APR is piecewise linear in utilization: `base_apr_bps` at zero, `kink1_apr_bps` at `kink1_ur_bps`, `kink2_apr_bps` at `kink2_ur_bps`, and `max_apr_bps` at full utilization. An optional controller scales it between 0.1× and 10× toward a target utilization.

### Fees

- **Take rate** — a share of borrower interest, taken before it reaches lenders; supply APY is quoted net of it.
- **Operation fees** — on deposits, withdrawals, borrows, repayments, collateral moves, and flash loans; all but flash-loan fees can share a cut with a referrer.
- **Scarcity fee** — a withdrawal that leaves utilization above the pool's limit pays a capped surcharge and may take only a set share of the remaining liquidity, optionally once per cooldown.

Anyone can call `distribute_pool_fees` to pay accrued fees to the configured beneficiaries.

### Core operations

| Function                               | Description                                              |
| -------------------------------------- | -------------------------------------------------------- |
| `deposit` / `withdraw`                 | Supply assets to earn yield                              |
| `add_collateral` / `remove_collateral` | Manage collateral backing                                |
| `borrow` / `repay`                     | Take and repay loans                                     |
| `liquidate`                            | Liquidate unhealthy positions for a bonus                |
| `flash_loan`                           | Borrow without collateral, repaid in the same tx         |
| `distribute_pool_fees`                 | Distribute accrued fees to beneficiaries                 |
| `submit_requests_batch`                | Run multiple actions atomically (e.g. leveraged deposit) |

## Getting started

Requires [Rust](https://rustup.rs) and the [Stellar CLI](https://developers.stellar.org/docs/tools/cli). The pinned toolchain installs itself; formatting, coverage, and fuzzing also need nightly Rust, and fuzzing needs `cargo-fuzz`.

```bash
git clone https://github.com/pointgroup-labs/alula.git
cd alula
make setup   # install cargo tools
make test    # build the contracts and run the tests
```

Common tasks:

```bash
make build           # Build contracts
make build/optimize  # Optimize WASM
make test            # Run tests
make test/fuzz       # Fuzz testing
make lint            # Clippy
make fmt             # Format
make cov             # Coverage
make sdk             # Generate TypeScript SDK
make ci              # Tests, clippy and format check
```

Run the tests matching a name:

```bash
cargo nextest run test_liquidate --workspace --lib
```

## Repository layout

```
contracts/
├── market/                    # Lending, borrowing, liquidations, flash loans
├── market_manager/            # Deploys markets and queues their upgrades
├── oracle_router/             # Per-market median of SEP-40 sources with a quorum
├── nav_oracle/                # Two-key NAV feed for tokenized funds
├── aggregated-oracle/         # Median of SEP-40 sources with a circuit breaker
├── redstone_sep_40_adapter/   # RedStone → SEP-40
├── chainlink_sep_40_adapter/  # Chainlink Data Feeds → SEP-40
├── cross_rate_sep_40_adapter/ # Cross rate (A/B × B/USD) → SEP-40
├── soroswap_sep_40_adapter/   # Soroswap spot price → SEP-40
├── allowlist_access_gate/     # Allow or block list access gate
├── asterizm_access_gate/      # Access gate on an Asterizm token's allowlist
├── controlled_insurance_fund/ # Bad-debt coverage
├── farms/                     # Delegated staking incentives
├── soroswap_swap_provider/    # Swaps through Soroswap
├── aqua_swap_provider/        # Swaps through Aqua
└── *_mock/                    # Test mocks

libs/
├── access-gate-interface/     # Access gate interface
├── farms-interface/           # Farms (delegated staking) interface
├── insurance-fund-interface/  # Insurance fund interface
├── proxy-swap-interface/      # Swap provider interface
├── sep-40-oracle/             # SEP-40 price feed interface
└── moderc3156/                # EIP-3156-style flash-loan interface

tests/                         # Integration, fuzz and resource-limit tests
packages/                      # TypeScript SDK and CLI
scripts/                       # Deployment and governance checks
docs/                          # API, parameters, multisig and fuzzing guides
audits/                        # Audit reports
```

## Security

- **Audits** — Halborn and Highland Security; reports are in [`audits/`](audits/).
- **Formal verification** — Certora Prover proofs for the Market contract's entry points are being finalized.
- **Governance** — on owned markets, economic parameters, gate installs, and token-code approvals wait out `update_in_queue_period`; status, pool switches, the guardian, fee beneficiaries, and farms change at once. The insurance fund can also freeze or reopen a market, except where the admin has locked the freeze. Code upgrades need both the Market Manager admin and the market admin. A market upgrade waits out the market's upgrade period and a manager upgrade waits 12 hours; users therefore get at least the shorter of the two as notice. Oracle-router changes and NAV role grants have their own timelocks, set at deployment; insurance-fund upgrades wait 72 hours.
- **Exits** — freezes and access gates never block repaying, withdrawing, or removing collateral. Withdrawals wait while the pool has an open bad-debt claim, are rationed past its utilization limit, and need live prices while the account has debt.
- **Oracle** — the market refuses stale, future, and non-positive prices; the router serves a price only when enough of its sources agree.
- **Arithmetic** — release builds keep overflow checks, so unchecked arithmetic traps rather than wraps.
- **Verify** — `CHECK_SOURCE=<funded account> scripts/check-market.sh <network> <market>` reads a market's governance from chain: its timelocks against production floors, pending admins, and queued upgrades and parameter changes.
- **Report a vulnerability** — email [support@alula.finance](mailto:support@alula.finance) rather than opening a public issue.

## License

[AGPL-3.0](LICENSE)

## Contributing

1. Fork and branch.
2. Make your changes.
3. Run `make ci`.
4. Open a PR, using [conventional commits](https://www.conventionalcommits.org/).
