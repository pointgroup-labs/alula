# Alula

[![Stellar](https://img.shields.io/badge/Stellar-000000?logo=stellar&logoColor=white&style=for-the-badge)](https://stellar.org)
[![Soroban](https://img.shields.io/badge/Soroban-7B68EE?logo=stellar&logoColor=white&style=for-the-badge)](https://soroban.stellar.org)
[![Rust](https://img.shields.io/badge/Rust-1.90-e64514?logo=rust&logoColor=white&style=for-the-badge)](https://www.rust-lang.org)
[![Tests Status](https://img.shields.io/github/actions/workflow/status/mfactory-lab/alula/ci.yml?logo=githubactions&logoColor=white&style=for-the-badge&label=tests)](./.github/workflows/ci.yml)

Decentralized lending protocol on Stellar. Earn yield, borrow against collateral, execute flash loans.

> ⚠️ Under active development — not production ready.

## Features

- **Lending & Borrowing** — Supply assets to earn yield, borrow against collateral
- **Flash Loans** — Uncollateralized loans settled within a single transaction
- **Leveraged Positions** — Amplify exposure via deposit-with-leverage
- **Isolated Pools** — Each asset has independent risk parameters
- **Dynamic Interest Rates** — Dual-kink model responding to utilization
- **Aggregated Oracles** — Median prices from multiple SEP-40 sources with deviation protection

## Quick Start

```bash
git clone https://github.com/mfactory-lab/alula.git
cd alula
make setup && make build && make test
```

## Architecture

```
contracts/
├── market/                   # Lending, borrowing, liquidations, flash loans
├── market_manager/           # Factory for deploying markets
├── aggregated_oracle/        # Median price aggregation from SEP-40 oracles
└── soroswap_sep_40_adapter/  # AMM price adapter
```

## How It Works

### Pools & Shares

Each asset has an isolated **Pool**. When you deposit, you receive **j-tokens** (supply shares). When you borrow, you owe **d-tokens** (debt shares). Share values grow over time as interest accrues.

### Health Factor

Your **Health Factor** = weighted collateral value / weighted debt value. If it drops below 1.0, your position can be liquidated.

### Interest Rates

Borrow APR scales with pool utilization:

| Utilization |   APR   |
| :---------: | :-----: |
|    0–70%    |  0–30%  |
|   70–80%    | 30–60%  |
|   80–100%   | 60–400% |

### Core Operations

| Function                               | Description                                  |
| -------------------------------------- | -------------------------------------------- |
| `deposit` / `withdraw`                 | Supply assets to earn yield                  |
| `add_collateral` / `remove_collateral` | Manage collateral backing                    |
| `borrow` / `repay`                     | Take and repay loans                         |
| `liquidate`                            | Liquidate unhealthy positions for a bonus    |
| `flash_loan`                           | Borrow without collateral (repay in same tx) |

Full API: [`docs/API.md`](./docs/API.md)

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
- Oracle staleness & deviation checks
- Time-locked governance (24h config, 7d upgrades)
- Emergency pause controls
- Isolated pool risk parameters

## Contributing

1. Fork & branch
2. Make changes
3. `make ci`
4. Open PR

Use [conventional commits](https://www.conventionalcommits.org/).
