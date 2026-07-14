# Alula

[![Stellar](https://img.shields.io/badge/Stellar-000000?logo=stellar&logoColor=white&style=for-the-badge)](https://stellar.org)
[![Rust](https://img.shields.io/badge/Rust-1.95-e64514?logo=rust&logoColor=white&style=for-the-badge)](https://www.rust-lang.org)
[![CI](https://img.shields.io/github/actions/workflow/status/pointgroup-labs/alula/ci.yml?logo=githubactions&logoColor=white&style=for-the-badge&label=tests)](./.github/workflows/ci.yml)

Alula is an institutional-grade RWA money-market protocol on Stellar. Configurable, segregated lending pools let originators fund real-world-asset-backed credit on-chain — with cross-pool collateral, flash-loan-powered leverage, and per-pool risk controls — while liquidity providers earn yield in Stellar assets.

## Features

- **Segregated pools** — Per-asset pools with independent LTV limits, interest-rate model, and eligible collateral, plus optional allow-listing.
- **Lending & borrowing** — Supply assets to earn yield; borrow against collateral.
- **Flash loans** — Uncollateralized, EIP-3156-style loans settled within a single transaction.
- **Leveraged positions** — Deposit-with-leverage through an atomic flash-loan + swap flow.
- **Cross-pool collateral** — Collateral in one pool can back borrowing in another, subject to configured rules.
- **Dynamic interest rates** — Dual-kink model that responds to per-pool utilization.
- **Aggregated oracle** — Median of multiple SEP-40 feeds, with a circuit breaker on large short-window moves.
- **Farms integration** — Delegated staking for j-token (supply) and d-token (debt) holders, with automatic stake sync.
- **Insurance fund** — Two-phase bad-debt coverage; losses socialized only after the fund is exhausted.

## How it works

Alula is organized around per-asset pools inside a Market contract. Suppliers receive j-tokens (supply shares) and borrowers accrue d-tokens (debt shares), both of which grow in value as interest accrues. Each user's deposits, collateral, and borrows are tracked as a single obligation, valued with oracle prices.

### Risk management

Risk is enforced per obligation. The protocol tracks a Health Factor (weighted collateral / weighted debt) and a stricter Liquidation Health Factor (LHF) derived from per-asset close-LTV and liability factors. When LHF < 1, the position becomes liquidatable in slices — a liquidator repays debt and receives collateral at a bounded discount. Residual losses from an insolvent position are absorbed first by the pool's insurance fund, and only then socialized across that pool's lenders.

### Interest rates

Borrow APR is a piecewise-linear, dual-kink function of pool utilization, set per pool via `BaseAPR`, `APR_k1`, `APR_k2`, and `APR_max` at the kink points `U_k1` and `U_k2`.

### Fees

Two fee layers run in parallel:

- **Take rate (streaming)** — a cut of borrower interest taken before it reaches lenders, so supply APY is shown net.
- **Origination (atomic)** — charged on operations such as `borrow` and `flash_loan`, with an optional referrer split.

Accrued fees are paid out permissionlessly to configured beneficiaries. Routing a share to the insurance fund builds the buffer that absorbs bad debt before any is socialized to lenders.

### Farms

Pools can attach supply farms (rewarding j-token holders) and debt farms (rewarding d-token holders) through an external Farms contract. The Market syncs stakes automatically as positions change, so incentive programs need no manual staking.

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

```bash
git clone https://github.com/pointgroup-labs/alula.git
cd alula
make setup && make build && make test
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
make ci              # Full CI
```

Run a single test:

```bash
cargo nextest run test_liquidate --workspace --lib
```

## Repository layout

```
contracts/
├── market/                    # Lending, borrowing, liquidations, flash loans
├── market_manager/            # Factory for deploying markets
├── aggregated-oracle/         # Median price aggregation from SEP-40 oracles
├── controlled_insurance_fund/ # Bad-debt coverage
├── farms/                     # Delegated staking / token incentives
├── soroswap_sep_40_adapter/   # Soroswap AMM → SEP-40 price adapter
├── redstone_sep_40_adapter/   # RedStone → SEP-40 price adapter
├── soroswap_swap_provider/    # Swap provider (leverage / collateral swaps)
├── aqua_swap_provider/        # Swap provider (Aqua AMM)
├── soroswap_router_mock/      # Test mock: swap router
└── flash_loan_taker_mock/     # Test mock: flash-loan receiver

libs/
├── farms-interface/           # Farms (delegated staking) interface
├── insurance-fund-interface/  # Insurance fund interface
├── proxy-swap-interface/      # Swap provider interface
└── moderc3156/                # EIP-3156-style flash-loan interface
```

## Security

- Checked arithmetic throughout — no silent overflows.
- `require_auth()` on all user actions and admin/governance changes; maintenance calls (post-timelock applies, fee distribution, state refreshes) are intentionally permissionless.
- Oracle staleness checks with a configurable maximum price age.
- Timelocked config on owned markets; ungoverned markets are immutable.
- Emergency pause controls via market status.
- Segregated pools and isolated markets to contain risk.
- Liquidations execute in health-improving slices.

## Contributing

1. Fork and branch.
2. Make your changes.
3. Run `make ci`.
4. Open a PR, using [conventional commits](https://www.conventionalcommits.org/).
