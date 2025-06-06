# JLend Development Guidelines

This document provides essential information for developers working on the JLend protocol.
It covers build instructions, testing procedures, and development best practices specific to this project.

## Build and Configuration Instructions

### Prerequisites

- Rust toolchain (specified in `rust-toolchain.toml`)
- Soroban CLI (for contract deployment and interaction)
- Node.js and pnpm (for SDK development)

### Building the Contracts

The project uses a Makefile to simplify the build process:

```bash
# Build all contracts
make build

# Build and optimize contracts for deployment
make build-optimize

# Generate the TypeScript SDK from the contract:
make sdk
```

## Code Structure

The project is organized as follows:

```
jlend/
├── contracts/ # Soroban smart contracts
│ ├── lending/ # Core lending protocol
│ └── reflector-mock-contract/ # Price oracle mock for testing
├── packages/ # TypeScript packages
│ └── sdk/ # JLend Protocol SDK
├── docs/ # Documentation
├── tests/ # Integration tests
├── scripts/ # Build and deployment scripts
└── Makefile # Build automation
```

## Coding Standards

### Rust

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for consistent formatting
- Use `clippy` for linting
- Use custom error types, avoid `panic!` in production code
- Be mindful of contract size limits and gas costs
- Document all public functions, types, and modules

### TypeScript/JavaScript

- Follow the TypeScript best practices
- Use ESLint for linting
- Prefer explicit types, avoid `any`
- Document all public functions, classes, and interfaces

### General Standards

- All public APIs must have JSDoc/rustdoc comments
- Minimize external dependencies, prefer battle-tested libraries
- Use uppercase with underscores for constants
- Avoid magic numbers, use named constants

## Testing Guidelines

### Unit Tests

- Located within the source files in the `src` directory
- Aim for >90% code coverage
- Each test should be independent
- Use descriptive test names following `test_<action>_<expected_outcome>` pattern

### Integration Tests

- Located in the `tests` directory, organized by functionality
- **End-to-End**: Test complete user flows
- **Edge Cases**: Test boundary conditions and error scenarios
- **Performance**: Include performance benchmarks for critical paths

### Running Tests

The project includes comprehensive tests for the smart contracts:

```bash
# Run all tests
make test

# Run tests with coverage reporting
make test-coverage
```

### Creating New Tests

To add a new test:

1. For unit tests, add them to the relevant source file in a `tests` module:

   ```rust
   #[cfg(test)]
   mod tests {
     use super::*;

     #[test]
     fn test_something() {
       // Test code here
     }
   }
   ```

2. For integration tests, create a new file in the `tests` directory:

   ```rust
   // tests/my_new_test.rs
   use {
    lending::contract::{LendingContract, LendingContractClient},
    soroban_sdk::{testutils::Address as _, Address, Env},
   };

   #[test]
   fn test_my_feature() {
   // Set up the environment
   let env = Env::default();

   // Generate a random address for the contract admin
   let contract_admin = Address::generate(&env);

   // Register the contract with the environment
   let contract_id = env.register(
     LendingContract,
     (contract_admin.clone(), Option::<i128>::None),
   );

   // Create a client to interact with the contract
   let contract_client = LendingContractClient::new(&env, &contract_id);

   // Test code here

   // Assertions
   assert!(true, "This test should pass");
   }
   ```

3. For testing expected failures, use the `#[should_panic]` attribute:

   ```rust
   #[test]
   #[should_panic(expected = "Error(Contract, #1)")]
   fn test_expected_failure() {
     // Test code that should panic
   }
   ```

## Commit Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification:

```
<type>(<optional scope>): <subject>

<optional body>

<optional footer>
```

Common types:

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Changes that do not affect the meaning of the code
- `refactor`: Code changes that neither fix a bug nor add a feature
- `test`: Adding or correcting tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements
- `ci`: CI/CD changes

### Examples

```
feat(lending): add collateral ratio validation
fix(sdk): handle network timeout errors
docs(api): update lending pool documentation
```

### Documentation

All public APIs should be documented with doc comments. Documentation is checked in CI with:

```bash
cargo doc --workspace --no-deps
```

### Contract Development

When developing contracts:

1. Follow the existing pattern for error handling using the `LendingContractError` enum in `error.rs`
2. Use the storage patterns defined in `storage.rs` for consistent data management
3. Implement tests for all new functionality
4. Consider gas optimization for frequently called functions

### SDK Development

When working on the TypeScript SDK:

1. Build the SDK with `pnpm run build` in the `packages/sdk` directory
2. Follow TypeScript best practices for type safety
3. Document all public APIs with JSDoc comments

### CI/CD Pipeline

The project uses GitHub Actions for CI/CD:

- Pull requests trigger linting, documentation checks, and tests
- Merges to main branch trigger the same checks
- The pipeline uses `cargo nextest` for efficient test running

## Debugging Tips

1. Use the `--features testing` flag to enable test-specific functionality
2. For contract debugging, use the `soroban contract invoke` command to interact with deployed contracts
3. For SDK debugging, use the TypeScript debugger in your IDE

## Pull Request Process

1. Use `feat/`, `fix/`, or `docs/` prefixes
2. Ensure your code follows the coding standards
3. Update documentation as necessary
4. Add or update tests as necessary
5. Make sure all tests pass
6. Get at least one code review from a maintainer

## Documentation

- Document all public APIs
- Keep README.md up to date
- Update documentation when making significant changes
- Use inline comments for complex logic

## Security Considerations

- Always validate user inputs
- Be mindful of potential overflow/underflow in financial calculations
- Use safe math operations
- Consider edge cases in financial operations
- Follow secure coding practices for smart contracts
- Report security vulnerabilities responsibly
