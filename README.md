# Alula

[![Stellar](https://img.shields.io/badge/Stellar-000000?logo=stellar&logoColor=white&style=for-the-badge)](https://stellar.org)
[![Soroban](https://img.shields.io/badge/Soroban-7B68EE?logo=stellar&logoColor=white&style=for-the-badge)](https://soroban.stellar.org)
[![Rust](https://img.shields.io/badge/Rust-1.90-e64514?logo=rust&logoColor=white&style=for-the-badge)](https://www.rust-lang.org)
[![Tests Status](https://img.shields.io/github/actions/workflow/status/pointgroup-labs/alula/ci.yml?logo=githubactions&logoColor=white&style=for-the-badge&label=tests)](./.github/workflows/ci.yml)

Alula is an institutional-grade RWA money-market protocol on Stellar designed to bring real-world credit on-chain in a policy-aligned way. It combines configurable, segregated lending pools with a yield-optimization layer that keeps liquidity productive even when some pools are underutilized. Built on Stellar’s compliance and settlement stack, Alula enables institutions to originate and fund RWA-backed credit on-chain, while giving liquidity providers transparent, risk-aligned yield in Stellar assets.

> ⚠️ Under active development — not production ready.

## Features

- **Configurable, segregated pools**: Per-asset pools with per-pool parameters (LTV limits, interest-rate model, eligible collateral) and optional permissioning / allow-lists
- **Lending & borrowing**: Supply assets to earn yield, borrow against collateral
- **Flash loans**: Uncollateralized loans settled within a single transaction
- **Leveraged positions**: Amplify exposure via deposit-with-leverage (flash-loan + swap flow)
- **Cross-pool collateral evaluation**: Collateral in one pool can support borrowing in another, subject to configured rules
- **Dynamic interest rates**: Dual-kink model responding to utilization (parameterized per pool)
- **Aggregated oracle**: Median prices from multiple SEP-40 sources, with optional circuit-breaker behavior (returning no price on large, short-window price moves)
- **Farms integration**: Delegated staking for j-token (supply) and d-token (debt) holders with automatic stake sync
- **Insurance fund**: Bad debt coverage with two-phase claim flow; shortfalls socialized only after fund exhaustion

## Quick Start

```bash
git clone https://github.com/pointgroup-labs/alula.git
cd alula
make setup && make build && make test
```

## Directories

```
contracts/
├── market/                    # Lending, borrowing, liquidations, flash loans
├── market_manager/            # Factory for deploying markets
├── aggregated-oracle/         # Median price aggregation from SEP-40 oracles
├── controlled_insurance_fund/ # Insurance fund for bad debt coverage
├── soroswap_sep_40_adapter/   # AMM price adapter
├── soroswap_router_mock/      # Test mock for swap router
└── flash_loan_taker_mock/     # Test mock for flash loans

libs/
├── farms-interface/           # Interface for farms (delegated staking)
├── insurance-fund-interface/  # Interface for insurance fund
└── moderc3156/                # ERC-3156 flash loan interface
```

## How It Works

Alula is a money-market protocol built around per-asset pools inside a Market contract. Users supply assets into a pool to earn yield and receive jTokens (supply shares); borrowers take loans and accrue dTokens (debt shares). Share values grow as interest accrues, and all positions are tracked as obligations that summarize a user’s deposits, collateral, and borrows across assets.

### Risk Management

Risk is enforced at the obligation level using oracle-priced valuations. The protocol monitors a position’s Health Factor (weighted collateral value / weighted debt value) and a risk-adjusted Liquidation Health Factor (LHF) derived from per-asset close-LTV and liability-factor parameters.

When LHF < 1, the obligation becomes eligible for liquidation in slices: a liquidator repays debt and receives collateral at a bounded discount (liquidation bonus). If a position becomes insolvent, the protocol can enter insolvency handling to reduce bad debt; any residual losses after liquidations are covered first by the pool’s insurance fund, and only then (if needed) socialized across lenders in that pool.

### Interest Rates

Borrow APR is a configurable dual-kink function of utilization (piecewise-linear). Pools define parameters such as:

- `BaseAPR`, `APR_k1`, `APR_k2`, `APR_max`
- `U_k1`, `U_k2`

### Fees & Insurance Fund

Markets support a dual-layer fee model:

- **Take rate (streaming)**: a portion of borrower interest is diverted before reaching lenders; supply APY is shown net of take rate
- **Origination fee (atomic)**: charged on certain operations (e.g., `borrow`, `flash_loan`), with optional referrer split

Accrued fees are distributed via the permissionless distribute method according to configured beneficiaries. Each pool can fund an insurance fund via fee routing to absorb residual losses after liquidations before any shortfall is socialized to lenders.

### Farms Integration

Alula supports delegated staking through an external Farms contract. Pools can be configured with supply farms (rewarding j-token holders) and debt farms (rewarding d-token holders). The Market contract automatically syncs user stakes when positions change, enabling token incentive programs without requiring users to manually stake.

### Core Operations

| Function                               | Description                                  |
| -------------------------------------- | -------------------------------------------- |
| `deposit` / `withdraw`                 | Supply assets to earn yield                  |
| `add_collateral` / `remove_collateral` | Manage collateral backing                    |
| `borrow` / `repay`                     | Take and repay loans                         |
| `liquidate`                            | Liquidate unhealthy positions for a bonus    |
| `flash_loan`                           | Borrow without collateral (repay in same tx) |
| `distribute_pool_fees`                 | Distribute accrued fees to beneficiaries     |

See [docs/API.md](./docs/API.md) for the complete API reference.

## Development

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

Run specific test:

```bash
cargo nextest run test_liquidate --workspace --lib
```

## Security

- Checked arithmetic — no overflows
- `require_auth()` on all mutations
- Oracle staleness checks with configurable maximum price age
- Owned markets support queued config updates; ungoverned markets have immutable configuration
- Emergency pause controls (market status)
- Segregated pools / isolated markets to contain risk
- Liquidations execute in slices (close-factor in health-improving mode)

## Contributions

1. Fork & branch
2. Make changes
3. `make ci`
4. Open PR

Use [conventional commits](https://www.conventionalcommits.org/).
