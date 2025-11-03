[![Stellar Portal](https://img.shields.io/badge/STELLAR-grey?logo=stellar&style=for-the-badge)](https://stellar.org/)
[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg?logo=apache&style=for-the-badge)](./LICENSE)
[![Tests Status](https://img.shields.io/github/actions/workflow/status/mfactory-lab/alula/ci.yml?logo=githubactions&logoColor=white&style=for-the-badge&label=tests)](./.github/workflows/ci.yml)

# Alula DeFi Protocol

Alula is a decentralized lending protocol built on the [Stellar](https://stellar.org/) blockchain using [Soroban](https://soroban.stellar.org/) smart contracts. It enables users to earn yield on deposits and access liquidity through overcollateralized loans with competitive interest rates.

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

Alula creates efficient lending markets on Stellar where users can:

- **Supply assets** to earn passive yield from borrower interest payments via standard or Earn obligations
- **Borrow assets** against deposited collateral with competitive rates based on utilization
- **Liquidate positions**: Participate in liquidations of unhealthy positions to earn liquidation bonuses
- **Manage risk**: Monitor health factors and adjust positions to avoid liquidation
- **Execute flash loans**: Utilize uncollateralized loans for arbitrage, refinancing, or complex DeFi strategies

The protocol uses a pool-based model where each supported asset has its own lending pool with isolated risk parameters. Recent safety mechanism additions (PR #89) include withdrawal scarcity protection, time-locked governance, Earn obligations for passive lenders, and granular market controls.

## 🏗️ Architecture

The protocol consists of a single unified market contract with modular components:

```
├── contracts/market/           # Main market contract
│   ├── lib.rs                  # Entry points: deposit, borrow, liquidate, etc.
│   ├── pool.rs                 # Pool state management and accounting
│   ├── obligation.rs           # User positions (deposits, borrows, collateral)
│   ├── interest_rate.rs        # Dual-kink interest rate model
│   ├── liquidation.rs          # Liquidation logic and health calculations
│   ├── flash_loan.rs           # Flash loan implementation
│   ├── deposit_with_leverage.rs # Leveraged deposit functionality
│   ├── storage.rs              # Storage keys and state management
│   ├── helpers.rs              # Authorization and validation
│   ├── multiply_pair.rs        # Oracle integration via Keccak seed derivation
│   ├── constants.rs            # Protocol-wide constants
│   ├── events.rs               # Event emission system
│   └── error.rs                # Error definitions
├── tests/                      # Comprehensive test suite
└── packages/                   # TypeScript SDK and utilities (planned)
```

### Key Components

- **Market Contract**: Unified contract handling all protocol operations with modular internal structure
- **Pool Module**: Asset pools with isolated risk parameters, utilization tracking, and interest accrual
- **Obligation Module**: User position tracking with two types:
  - **Standard Obligations**: Full-featured accounts supporting deposits, collateral, and borrowing
  - **Earn Obligations**: Isolated deposit-only accounts for passive lenders (introduced in PR #89)
- **Interest Rate Module**: Dual-kink (jump-rate) model responding to pool utilization
- **Liquidation Module**: Health factor monitoring and liquidation execution
- **Oracle Integration**: Price feed via aggregated oracle contract using deterministic seed derivation

## 🛡️ Safety Mechanisms

Alula implements multiple layers of protection to ensure protocol stability and user safety:

### Withdrawal Scarcity Protection

When pool utilization exceeds configurable thresholds, the protocol activates scarcity protection:

- **Dynamic Scarcity Fees**: Exponentially scaled fees applied when utilization > withdrawal scarcity limit (default: 100%)
- **Withdrawal Cooldowns**: Time-based restrictions between scarcity withdrawals (configurable per pool)
- **First-Withdrawal Bypass Protection**: Cooldown enforcement only after initial scarcity withdrawal (fixed in PR #89)

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

Passive lenders can use Earn obligations for deposit-only functionality:

- **Seed Isolation**: Separate storage seed from standard obligations
- **Operation Restrictions**: Only deposit and withdraw allowed (no borrowing or collateral)
- **Risk Separation**: Earn users are isolated from borrowing risks
- **Simplified UX**: Simpler interface for passive yield seekers

**Why it matters**: Reduces complexity for passive lenders and separates them from active borrowers.

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

- **Rounding Error Tolerance**: Accepts small negative values (≤100 stroops) due to fixed-point math
- **Critical Bug Detection**: Fails loudly on significant negative interest (>100 stroops absolute)
- **Event Logging**: Emits `computed_interest_is_negative` events for monitoring

**Why it matters**: Distinguishes harmless rounding errors from critical accounting bugs.

### Additional Safety Features

- **Invariant Monitoring**: Pool state invariants tracked (see security audit for recommendations)
- **Oracle Staleness Checks**: Maximum price age of 6 minutes (based on Reflector 5-minute resolution)
- **Liability Factors**: Configurable risk weights for different asset types (100-200%)
- **Position Limits**: Maximum 20 positions per user to prevent gas exhaustion

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.84.1 or later
- **Stellar CLI** for contract deployment
- **Node.js** 18+ for TypeScript utilities
- **pnpm** package manager

### Installation

```bash
# Clone the repository
git clone https://github.com/mfactory-lab/alula.git && cd alula

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

Alula uses a dual-kink (jump-rate) interest rate model to determine borrowing and supply rates dynamically based on pool utilization.

#### Model Parameters

- **Base rate**: Floor rate applied when utilization is zero
- **Kink 1 (70%)**: First utilization threshold where rate slope increases
- **Kink 2 (80%)**: Second utilization threshold where rate slope increases sharply
- **Slope 1**: Incremental rate for utilization below Kink 1
- **Slope 2**: Incremental rate between Kink 1 and Kink 2
- **Slope 3**: Steep incremental rate above Kink 2 to discourage over-utilization
- **Reserve ratio**: Fraction of interest retained as protocol reserves (default: 10%)

```
if Utilization < Kink1 (70%):
  Interest Rate = Base Rate + Utilization × Slope 1

if Kink1 ≤ Utilization < Kink2 (80%):
  Interest Rate = Base Rate + Kink1 × Slope 1 + (Utilization - Kink1) × Slope 2

if Utilization ≥ Kink2:
  Interest Rate = Base Rate + Kink1 × Slope 1 + (Kink2 - Kink1) × Slope 2 + (Utilization - Kink2) × Slope 3
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

- [ ] **External Security Audit** - Planned by established audit firm
- [x] **Comprehensive Test Suite** - >90% code coverage with fuzz testing
- [x] **Formal Verification** - Critical functions verified for correctness

### Security Features

**Cryptographic Primitives**:

- Keccak-256 for deterministic seed derivation
- Ed25519 signatures (Stellar native, quantum-vulnerable in 10-15 years)
- Secure randomness for flash loan callbacks

**Risk Management**:

- **Isolated pools**: Each asset has separate risk parameters
- **Overcollateralization**: Minimum 125% collateral ratio (configurable per pool)
- **Liquidation incentives**: Economic rewards for liquidators (5-10% bonus)
- **Health factor monitoring**: Real-time position safety tracking
- **Liability factors**: Risk-adjusted debt weights (100-200% based on asset volatility)

**Smart Contract Security**:

- Use of `checked_*` arithmetic operations to prevent overflows
- Comprehensive input validation on all external functions
- Access control for administrative functions (admin/deployer separation)
- Reentrancy protection via Soroban execution model
- Time-lock governance for sensitive parameter changes (24-hour default delay)
- Event logging for all critical operations

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

## 📞 Support

- **Documentation**: [docs/](./docs) [under development]
- **Issues**: [GitHub Issues](https://github.com/mfactory-lab/alula/issues)
- **Discussions**: [GitHub Discussions](https://github.com/mfactory-lab/alula/discussions)
