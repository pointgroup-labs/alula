[![Stellar Portal](https://img.shields.io/badge/STELLAR-grey?logo=stellar&style=for-the-badge)](https://stellar.org/)
[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg?logo=apache&style=for-the-badge)](LICENSE)
[![Tests Status](https://img.shields.io/github/actions/workflow/status/mfactory-lab/jlend/ci.yml?logo=githubactions&logoColor=white&style=for-the-badge&label=tests)](./.github/workflows/ci.yml)

# JLend DeFi Protocol

JLend is a decentralized lending protocol built on the [Stellar](https://stellar.org/) blockchain using [Soroban](https://soroban.stellar.org/) smart contracts. It enables users to earn yield on deposits and access liquidity through overcollateralized loans with competitive interest rates.

> [!Note]
> This project is a work in progress and is not yet ready for production use.
> We are happy to answer questions if they are raised as issues in this GitHub repo.

## 🌟 Features

- **Lending & borrowing**: Supply assets to earn yield or borrow against collateral.
- **Multiple asset support**: Support for various Stellar assets.
- **Dynamic interest rates**: Kinked interest rate model based on pool utilization.
- **Liquidation protection**: Automated liquidation system to maintain protocol solvency.
- **Overcollateralization**: Secure borrowing with configurable collateral ratios.
- **Real-time price feeds**: Integration with Stellar price oracles.

## 📋 Table of Contents

- [Overview](#-overview)
- [Architecture](#️-architecture)
- [Getting Started](#-getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Building](#building)
  - [Testing](#testing)
- [Usage](#-usage)
  - [Contract Deployment](#contract-deployment)
  - [Pool Management](#pool-management)
  - [Lending Operations](#lending-operations)
- [Protocol Mechanics](#-protocol-mechanics)
  - [Interest Rate Model](#interest-rate-model)
  - [Health Factor](#health-factor)
  - [Liquidation Process](#liquidation-process)
- [Security](#-security)
- [Contributions](#-contributions)
- [License](#-license)

## 🔍 Overview

JLend creates efficient lending markets on Stellar where users can:

- **Supply assets** to earn passive yield from borrower interest payments.
- **Borrow assets** against deposited collateral with competitive rates.
- **Liquidate positions**: Participate in liquidations of unhealthy positions of other users to earn liquidation bonuses.
- **Manage risk**: Monitor health factors and adjust positions to avoid liquidation.

The protocol uses a pool-based model where each supported asset has its own lending pool with isolated risk parameters.

## 🏗️ Architecture

The protocol consists of several key components:

```
├── contracts/
│ ├── lending/ # Main lending contract
│ ├── pool/ # Lending pools
│ ├── obligation/ # Obligations
│ └── oracle/ # Oracle integration
├── tests/ # Comprehensive test suite
└── packages/ # TypeScript SDK and utilities
```

- **Lending contract**: Main protocol logic handling deposits, borrows, and liquidations.
- **Pool**: Individual asset pools with isolated risk parameters.
- **Obligation**: User position tracking (deposits, borrows, collateral).
- **Oracle integration**: Price feed integration for asset valuation.

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.84.1 or later
- **Stellar CLI** for contract deployment
- **Node.js** 18+ for TypeScript utilities
- **pnpm** package manager

### Installation

```bash
# Clone the repository
git clone https://github.com/mfactory-lab/jlend.git && cd jlend

# Install dependencies
pnpm install
```

### Building

Build all contracts:

```bash
make build
```

Or build specific components:

```bash
# Build only the lending contract
cargo build --release --target wasm32-unknown-unknown -p lending

# Build with optimizations
make build-optimize
```

### Testing

Run the comprehensive test suite:

```bash
# Run all tests
make test

# Run specific test categories
cargo test -p tests deposit
cargo test -p tests liquidate
cargo test -p tests interest_rates

# Run fuzz tests
cargo test -p tests fuzz
```

## 📖 Usage

### Contract Deployment

1. **Deploy the lending contract**:

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/lending.wasm \
  --source YOUR_ACCOUNT \
  --network testnet
```

2. **Initialize the contract**:

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source YOUR_ACCOUNT \
  --network testnet \
  -- __constructor \
  --admin YOUR_ACCOUNT \
  --liquidation_threshold_percent 80
```

### Pool Management

**Initialize a new lending pool**:

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source YOUR_ACCOUNT \
  --network testnet \
  -- initialize_pool \
  --token_address USDC_TOKEN_ADDRESS \
  --token_ticker USDC
```

**Get pool information**:

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --network testnet \
  -- get_pool \
  --pool_address POOL_ADDRESS
```

### Lending Operations

**Make a deposit**:

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source YOUR_ACCOUNT \
  --network testnet \
  -- deposit \
  --user YOUR_ACCOUNT \
  --pool_address POOL_ADDRESS \
  --amount 1000000000  # 100 USDC (7 decimals)
```

**Borrow against collateral**:

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source YOUR_ACCOUNT \
  --network testnet \
  -- borrow \
  --user YOUR_ACCOUNT \
  --pool_address POOL_ADDRESS \
  --amount 500000000   # 50 USDC
```

**Check user position**:

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --network testnet \
  -- get_user_obligation \
  --user USER_ACCOUNT
```

## ⚙️ Protocol Mechanics

### Interest Rate Model

JLend uses a kinked (jump-rate) interest rate model to determine borrowing and supply rates dynamically based on pool utilization.

#### Model Parameters
- **Base rate**: Floor rate applied when utilization is zero.
- **Optimal utilization**`: The utilization point ("kink") at which the slope changes (default: 80%).
- **Slope 1**: Incremental rate added for each basis point of utilization below the optimal ratio.
- **Slope 2**: Incremental rate applied for each basis point of utilization above the optimal ratio; always greater than `slope1`.
- **Reserve ratio**: Fraction of interest retained as protocol reserves (default: 10 %).

```
Interest Rate = Base Rate + Current Utilization × Slope 1 (if Current Utilization < Optimal Utilization)
Interest Rate = Base Rate + Optimal Utilization × Slope 1 + (Current Utilization - Optimal Utilization) × Slope 2 (if Current Utilization > Optimal Utilization)
```

### Health Factor

The health factor determines the safety of a borrowing position:

```
Health Factor = (Collateral Value × Liquidation Threshold) / Total Borrowed Value
```

- **Healthy**: Health Factor > 1.0
- **Can be liquidated**: Health Factor < 1.0

### Liquidation Process

When a position becomes unhealthy (Health Factor < 1.0):

1. **Liquidators** can repay up to the close factor (default: 50%) of the debt.
2. **Liquidation bonus**: Liquidators receive collateral at a discount (default: 5%).

## 🔒 Security

### Audit Status

- [ ] External security audit (planned).
- [x] Comprehensive test suite with >90% coverage.
- [x] Fuzz testing for edge cases.
- [x] Formal verification of critical functions.

### Risk Management

- **Isolated pools**: Each asset has separate risk parameters.
- **Overcollateralization**: Minimum 125% collateral ratio.
- **Liquidation bonus**: Economic incentives for liquidators.

### Best Practices

- Use of `checked_*` arithmetic operations to prevent overflows.
- Comprehensive input validation.
- Access control for administrative functions.
- Immutable contract deployment.

## 🤝 Contributions

We welcome contributions!

### Development Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/amazing-feature`
3. Make your changes and add tests
4. Run the test suite: `make test`
5. Submit a pull request

### Code Standards

- Follow Rust best practices and idioms.
- Maintain test coverage above 90%.
- Document all public APIs.
- Use conventional commit messages.

## 📄 License

This project is licensed under the [Apache License 2.0](https://opensource.org/licenses/Apache-2.0).

## 📞 Support

- **Documentation**: [docs/](./docs/) [under development]
- **Issues**: [GitHub Issues](https://github.com/mfactory-lab/jlend/issues)
- **Discussions**: [GitHub Discussions](https://github.com/mfactory-lab/jlend/discussions)
