# JLend Fuzzing Guidance

This document contains a comprehensive guide on how to fuzz test the JLend DeFi Protocol Soroban source.

Fuzzing can either be run in the root of the repository via:

```
make test/fuzz
```

or via the step-by-step guidance with possible modifications below.

More comprehensive guide on how to fuzz test Soroban smart contracts can be found here: https://developers.stellar.org/docs/build/smart-contracts/example-contracts/fuzzing

## 1. Install the nightly Rust toolchain. Nightly Rust is required to run cargo-fuzz.

```
rustup install nightly
```

## 2. Install `cargo-fuzz`.

```
cargo install --locked cargo-fuzz
```

## 3. Change directory to `tests/fuzz`

```
cd tests/fuzz
```

## 4. Run fuzz target

### For Linux and Intel-based macOS:

```
RUST_BACKTRACE=1 ASAN_OPTIONS=abort_on_error=1:symbolize=1 cargo +nightly fuzz run --sanitizer=address fuzz_target
```

### For Apple Silicon Macs:

Due to compatibility issues with the address sanitizer on Apple Silicon, use the thread sanitizer instead:

```
RUST_BACKTRACE=1 cargo +nightly fuzz run --sanitizer=thread fuzz_target
```

More information about this issue: https://github.com/stellar/rs-soroban-sdk/issues/1056

## 6. Terminate fuzzing with `CTRL + C`
