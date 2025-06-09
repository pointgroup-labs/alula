# JLend Fuzzing Guidance

This document contains a comprehensive guide on how to fuzz test the JLend DeFi Protocol Soroban source.

More comprehensive guide on how to fuzz test Soroban smart contracts can be found here: https://developers.stellar.org/docs/build/smart-contracts/example-contracts/fuzzing


## TODO: Add this to Makefile, after fixing `stellar contract build`

# 1. Install the nightly Rust toolchain. Nightly Rust is required to run cargo-fuzz.

```
rustup install nightly
```

# 2. Install `cargo-fuzz`.
 ```
cargo install --locked cargo-fuzz
```

# 3. Change directory to `tests\fuzz`
 ```
cd tests\fuzz
```

# 4. Run fuzz target
 ```
 RUST_BACKTRACE=1 ASAN_OPTIONS=abort_on_error=1:symbolize=1 cargo +nightly fuzz run --sanitizer=address fuzz_target
```

# 5. In case of linking errors, try adding `--sanitizer=thread` to the command.  More: https://github.com/stellar/rs-soroban-sdk/issues/1056
 ```
 RUST_BACKTRACE=1 cargo +nightly fuzz run --sanitizer=thread fuzz_target
```

# 6. Terminate fuzzing with `CTRL + C`