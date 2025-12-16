[![Stellar Portal](https://img.shields.io/badge/STELLAR-grey?logo=stellar&style=for-the-badge)](https://stellar.org/)
[![Soroban](https://img.shields.io/badge/SOROBAN-blue?logo=stellar&style=for-the-badge)](https://soroban.stellar.org/)
[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg?logo=apache&style=for-the-badge)](./LICENSE)
[![Tests Status](https://img.shields.io/github/actions/workflow/status/mfactory-lab/alula/ci.yml?logo=githubactions&logoColor=white&style=for-the-badge&label=tests)](./.github/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-2024_edition-orange?logo=rust&style=for-the-badge)](https://www.rust-lang.org/)

# Alula Lending Protocol

<div align="center">
  <strong>Decentralized Lending on Stellar/Soroban</strong>
  <br/>
  <em>Earn yield • Borrow against collateral • Leverage positions • Flash loans</em>
</div>

<br/>

Alula is a decentralized lending protocol built on the [Stellar](https://stellar.org/) blockchain using [Soroban](https://soroban.stellar.org/) smart contracts. It enables users to earn yield on deposits and access liquidity through overcollateralized loans with competitive interest rates.

> [!Note]
> This project is a work in progress and is not yet ready for production use.
> We are happy to answer questions if they are raised as issues in this GitHub repo.

## 🌟 Features

- **Lending & borrowing**: Supply assets to earn yield or borrow against collateral.
- **Multiple asset support**: Support for various Stellar assets.
- **Dynamic interest rates**: Dual-kink interest rate model responding to pool utilization
- **Liquidation protection**: Automated liquidation system to maintain protocol solvency.
- **Overcollateralization**: Secure borrowing with configurable collateral ratios.
- **Real-time price feeds**: Integration with Stellar price oracles.
- **Safety mechanisms**: Withdrawal scarcity protection, time-locked governance, and emergency controls.
- **Flash loans**: Uncollateralized loans for arbitrage and refinancing within a single transaction.
- **Leveraged positions**: Amplify exposure through deposit-with-leverage functionality.

## 📋 Table of Contents

- [Overview](#-overview)
- [Architecture](#-architecture)
- [Safety Mechanisms](#-safety-mechanisms)
- [Getting Started](#-getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Building](#building)
  - [Testing](#testing)
  - [Code Quality](#code-quality)
- [Usage](#-usage)
- [Protocol Mechanics](#-protocol-mechanics)
  - [Interest Rate Model](#interest-rate-model)
  - [Health Factor](#health-factor)
  - [Liquidation Process](#liquidation-process)
- [Security](#-security)
- [Contributions](#-contributions)
- [Quick Reference](#-quick-reference)
- [License](#-license)

## 🔍 Overview

Alula is a decentralized lending protocol on Stellar that enables efficient capital markets through:

**For Lenders**:

- Earn passive yield by supplying assets to lending pools
- Choose between Standard (full-featured) or Earn (simplified) obligations
- Withdraw anytime, subject to pool liquidity

**For Borrowers**:

- Access liquidity by borrowing against collateral
- Competitive interest rates based on market utilization
- Flexible collateral management and position monitoring

**For Liquidators**:

- Earn bonuses by liquidating unhealthy positions
- Help maintain protocol solvency

**For Traders**:

- Execute flash loans for arbitrage and complex strategies
- Use leveraged deposits to amplify exposure

The protocol features isolated lending pools per asset, dual-kink interest rates, time-locked governance, and comprehensive safety mechanisms including withdrawal scarcity protection and emergency controls.

## 🏗️ Architecture

The protocol consists of multiple contracts working together:

```
alula/
├── contracts/
│   ├── market/                     # Core lending market contract
│   │   ├── src/
│   │   │   ├── contract.rs         # Entry points: deposit, borrow, liquidate, flash_loan
│   │   │   ├── pool.rs             # Pool state, j-token/d-token accounting
│   │   │   ├── obligation.rs       # User positions (deposits, borrows, collateral)
│   │   │   ├── processors.rs       # Core operation processors
│   │   │   ├── interest_rate.rs    # Dual-kink interest rate model
│   │   │   ├── oracle.rs           # SEP-40 oracle integration
│   │   │   ├── storage.rs          # Storage keys, TTL management
│   │   │   ├── multiply_pair.rs    # Leveraged position support
│   │   │   └── error.rs            # Error definitions
│   │   └── Cargo.toml
│   ├── market_manager/             # Factory: deploys & manages Market instances
│   ├── aggregated_oracle/          # Aggregates prices from multiple SEP-40 oracles
│   └── soroswap_sep_40_adapter/    # Adapts Soroswap AMM prices to SEP-40 interface
├── tests/                          # Comprehensive test suite with fuzzing
├── packages/                       # TypeScript SDK (generated)
└── wasms/                          # Compiled WASM artifacts
```

### Key Components

- **Market Contract**: Core lending operations with isolated pools per asset
- **Market Manager**: Factory contract for deploying and managing Market instances
- **Aggregated Oracle**: Price feed aggregation from multiple SEP-40 sources
- **Pool Module**: Asset pools with j-tokens (supply shares) and d-tokens (debt shares)
- **Obligation Module**: User position tracking with two types:
  - **Standard Obligations**: Full-featured accounts supporting deposits, collateral, and borrowing
  - **Earn Obligations**: Simplified deposit-only accounts for passive yield
- **Interest Rate Module**: Dual-kink (jump-rate) model responding to pool utilization

### Key Concepts

| Concept              | Description                                                                          |
| -------------------- | ------------------------------------------------------------------------------------ |
| **j-tokens**         | Supply shares representing a user's proportion of pool deposits                      |
| **d-tokens**         | Debt shares representing a user's proportion of pool borrowings                      |
| **LTV**              | Loan-to-Value ratio: `open_ltv` (max at borrow), `close_ltv` (liquidation threshold) |
| **Health Factor**    | Position safety ratio; below 1.0 triggers liquidation eligibility                    |
| **Liability Factor** | Risk weight applied to volatile assets (100-200%)                                    |

## 🛡️ Safety Mechanisms

Alula implements multiple layers of protection to ensure protocol stability and user safety:

### Withdrawal Scarcity Protection

When pool utilization exceeds configurable thresholds, the protocol activates scarcity protection:

- **Dynamic Scarcity Fees**: Exponentially scaled fees applied when utilization exceeds withdrawal limits
- **Withdrawal Cooldowns**: Time-based restrictions between scarcity withdrawals (configurable per pool)
- **First-Withdrawal Protection**: Cooldown enforcement applied consistently across all withdrawals

**Why it matters**: Prevents liquidity draining during high utilization periods, protecting borrowers and lenders.

### Time-Locked Governance (Queue/Apply/Cancel)

All pool configuration updates follow a time-delayed pattern:

1. **Queue**: Admin queues a configuration change with timestamp
2. **Wait Period**: Mandatory waiting period (default: 24 hours) before applying changes
3. **Apply or Cancel**: After wait period, admin applies the change or cancels if no longer needed

**Protected Parameters**:

- Interest rate curves (kink points, APR slopes)
- Collateral factors (LTV ratios, liquidation thresholds)
- Fee structures (borrow fees, liquidation spreads)
- Supply limits and utilization caps

**Why it matters**: Gives users time to react to governance decisions, preventing surprise parameter changes.

### Earn Obligations (Isolated Deposit Accounts)

Passive lenders can use Earn obligations for simplified deposit-only functionality:

- **Isolated Storage**: Separate account structure from standard obligations
- **Deposit Only**: Simplified interface for passive yield - no borrowing or collateral management
- **Risk Separation**: Earn users are protected from borrowing-related risks

This reduces complexity for passive lenders who only want to earn yield without managing collateral or health factors.

### Pool and Market Status Controls

Granular control over protocol operations at two levels:

**Pool Level** (per asset):

- `borrow_enabled`: Control borrowing for specific asset
- `deposit_enabled`: Control deposits for specific asset

**Market Level** (global):

- Status 0 (Active): All operations enabled
- Status 1 (Borrow Paused): Deposits/withdrawals allowed, borrowing disabled
- Status 2 (Deposit Paused): Withdrawals/repayments allowed, deposits/borrowing disabled
- Status 3 (Frozen): Only liquidations allowed, emergency mode

**Why it matters**: Enables targeted responses to market conditions or security incidents without full protocol shutdown.

### Negative Interest Protection

Robust handling of edge cases in interest calculations:

- **Rounding Error Tolerance**: Accepts small negative values from fixed-point arithmetic
- **Critical Bug Detection**: Fails immediately on significant negative interest indicating bugs
- **Event Logging**: Emits monitoring events for all negative interest occurrences

This ensures that minor calculation rounding is handled gracefully while critical accounting errors are caught immediately.

### Additional Safety Features

- **Oracle Staleness Checks**: Maximum price age of 6 minutes
- **Liability Factors**: Configurable risk weights for different asset types (100-200%)
- **Position Limits**: Maximum 20 positions per user

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.79+ (nightly for formatting/coverage)
- **Stellar CLI** (`stellar-cli`) for contract deployment
- **Make** for build automation

Optional:

- **Node.js** 18+ for TypeScript SDK
- **pnpm** package manager

### Installation

```bash
# Clone the repository
git clone https://github.com/mfactory-lab/alula.git && cd alula

# Install Rust development tools
make setup
```

### Building

```bash
# Build all contracts (downloads dependencies, builds mocks and main contracts)
make build

# Build for testnet deployment
make build/deploy

# Build for mainnet (with 7-day upgrade timelock)
make build/mainnet

# Build and optimize for production
make build/optimize
```

### Testing

```bash
# Run all tests with cargo-nextest
make test

# Run tests in watch mode (auto-rerun on changes)
make test/watch

# Run a specific test
cargo nextest run <test_name> --workspace --lib

# Run fuzzing suite
make test/fuzz

# Generate coverage report
make cov                     # Terminal report
make cov/html                # HTML report at target/llvm-cov/html/index.html
```

### Code Quality

```bash
make check        # Run cargo check
make lint         # Run clippy with warnings as errors
make fmt          # Format code (cargo sort + nightly fmt)
make ci           # Full CI pipeline: test + lint + fmt/check
```

## 📖 Usage

### Contract Deployment

1. **Deploy the market contract**:

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/market.wasm \
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

Alula uses a dual-kink interest rate model that adjusts rates based on pool utilization to balance supply and demand.

#### How It Works

The model has three zones with progressively steeper rates:

| Utilization Range  | Rate Behavior                                                |
| ------------------ | ------------------------------------------------------------ |
| 0% - 70% (Kink 1)  | Gradual rate increase (Slope 1)                              |
| 70% - 80% (Kink 2) | Moderate rate increase (Slope 2)                             |
| 80% - 100%         | Steep rate increase (Slope 3) to discourage over-utilization |

**Formula**:

```
Rate = Base Rate + Rate Adjustment Based on Utilization Zone
```

**Example**: At 85% utilization, the rate includes:

- Base rate
- Rate increase from 0-70% utilization (Slope 1)
- Rate increase from 70-80% utilization (Slope 2)
- Rate increase from 80-85% utilization (Slope 3)

**Reserve Ratio**: 10% of interest income is retained as protocol reserves.

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

| Audit                    | Status      | Report                          |
| ------------------------ | ----------- | ------------------------------- |
| External Security Audit  | 📋 Planned  | TBD                             |
| Comprehensive Test Suite | ✅ Complete | >90% coverage with fuzz testing |

### Security Features

**Authorization & Access Control**:

- Soroban `require_auth()` on all state-modifying operations
- Admin/deployer role separation
- Time-locked upgrades (7 days on mainnet)
- Queue-based pool configuration updates (24-hour delay)

**Arithmetic Safety**:

- Checked arithmetic (`checked_add`, `checked_mul`, etc.) throughout
- Fixed-point math via `soroban-fixed-point-math` library
- Proper rounding direction (floor for supplies, ceil for debts)

**Risk Management**:

- **Isolated pools**: Each asset has separate risk parameters
- **Overcollateralization**: Configurable LTV ratios per pool
- **Liquidation incentives**: Economic rewards for maintaining solvency
- **Oracle staleness checks**: Maximum 6-minute price age
- **Position limits**: Maximum 20 positions per user

**Storage & State**:

- TTL management prevents data archival
- Instance storage extended on significant operations
- Namespaced storage keys prevent collisions

### Reporting Vulnerabilities

If you discover a security vulnerability:

1. **Do NOT** open a public GitHub issue
2. Email security findings to: [security contact - TBD]
3. Include detailed reproduction steps and impact assessment
4. Allow reasonable time for remediation before public disclosure

We follow responsible disclosure practices and will acknowledge security researchers appropriately.

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

## 📚 Quick Reference

### Make Commands

| Command               | Description                     |
| --------------------- | ------------------------------- |
| `make build`          | Build all contracts             |
| `make build/mainnet`  | Build for mainnet with timelock |
| `make build/optimize` | Build and optimize WASMs        |
| `make test`           | Run all tests                   |
| `make test/watch`     | Run tests in watch mode         |
| `make test/fuzz`      | Run fuzzing suite               |
| `make cov`            | Generate coverage report        |
| `make cov/html`       | Generate HTML coverage          |
| `make lint`           | Run clippy                      |
| `make fmt`            | Format code                     |
| `make ci`             | Full CI pipeline                |
| `make sdk`            | Generate TypeScript SDK         |
| `make clean`          | Clean build artifacts           |

### Protocol Parameters (Defaults)

| Parameter         | Value  | Description                        |
| ----------------- | ------ | ---------------------------------- |
| Open LTV          | 70%    | Maximum LTV at borrow time         |
| Close LTV         | 80%    | LTV threshold for liquidation      |
| Liquidation Bonus | 10%    | Incentive for liquidators          |
| Close Factor      | 50%    | Max debt repayable per liquidation |
| Utilization Limit | 90%    | Max pool utilization for borrowing |
| Oracle Max Age    | 6 min  | Maximum acceptable price staleness |
| Upgrade Delay     | 7 days | Timelock for mainnet upgrades      |

### Decimal Precision

| Type          | Decimals | Example                    |
| ------------- | -------- | -------------------------- |
| Token amounts | 7        | 1 XLM = 10,000,000 stroops |
| Oracle prices | 14       | 1.00 = 10^14               |
| Basis points  | 4        | 100% = 10,000 BPS          |

## 📞 Support

- **Documentation**: [docs/](./docs) [under development]
- **Issues**: [GitHub Issues](https://github.com/mfactory-lab/alula/issues)
- **Discussions**: [GitHub Discussions](https://github.com/mfactory-lab/alula/discussions)
