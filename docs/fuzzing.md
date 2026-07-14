# Alula Fuzzing Guide

How to fuzz-test the Alula lending contracts. The quickest path, from the repo
root:

```bash
make test/fuzz
```

which runs the equivalent of:

```bash
RUST_BACKTRACE=1 cargo +nightly fuzz run \
  --fuzz-dir=tests/fuzz --sanitizer=none fuzz_target -- -max_len=1048576
```

The sections below explain that command, how to run variants, and how to
reproduce a crash once the fuzzer finds one.

Stellar's broader guide to fuzzing Soroban contracts:
<https://developers.stellar.org/docs/build/smart-contracts/example-contracts/fuzzing>

## What the fuzzer actually tests

This is **property-based invariant fuzzing**, not blind input crashing. The
target (`tests/fuzz/fuzz_targets/fuzz_target.rs`) decodes each input into a
sequence of protocol commands (deposit, borrow, withdraw, liquidate, price
moves, …), then:

1. Spins up a fresh `TestMarketFixture` (mock BTC/GOLD/USDC tokens, oracle,
   router).
2. Runs each command against the live market.
3. Asserts the protocol's invariants after **every** command
   (`fixture.assert_invariants()`).

A bug is any command sequence that breaks an invariant (e.g. accounting that
doesn't balance, a position that shouldn't be liquidatable but is) or panics.
The `Input` and command types come from the `tests` crate (`tests/src/lib.rs`),
derived via `arbitrary`, so the fuzzer explores realistic multi-step histories
rather than random bytes.

## Prerequisites

`cargo-fuzz` requires the **nightly** toolchain.

```bash
rustup install nightly
cargo install --locked cargo-fuzz
```

## Running

`make test/fuzz` uses `--sanitizer=none`, which is the most portable option and
runs everywhere (including Apple Silicon). To run by hand and pass your own
libFuzzer flags, invoke `cargo fuzz` directly. From the repo root, point it at
the fuzz workspace with `--fuzz-dir`:

```bash
# List available targets
cargo +nightly fuzz list --fuzz-dir=tests/fuzz

# Run indefinitely (Ctrl-C to stop)
cargo +nightly fuzz run --fuzz-dir=tests/fuzz --sanitizer=none fuzz_target

# Time-box a run (e.g. in CI): 5 minutes
cargo +nightly fuzz run --fuzz-dir=tests/fuzz --sanitizer=none fuzz_target \
  -- -max_total_time=300
```

Alternatively, `cd tests/fuzz` first and drop the `--fuzz-dir` flag —
`cargo fuzz` auto-detects the workspace from that directory.

### Sanitizers

Sanitizers add memory/data-race detection at the cost of speed and portability.
They are optional; `--sanitizer=none` is the default here.

- **Linux / Intel macOS** — AddressSanitizer catches memory errors:

  ```bash
  RUST_BACKTRACE=1 ASAN_OPTIONS=abort_on_error=1:symbolize=1 \
    cargo +nightly fuzz run --fuzz-dir=tests/fuzz --sanitizer=address fuzz_target
  ```

- **Apple Silicon** — ASan has known incompatibilities with the Soroban SDK, so
  use ThreadSanitizer (or stay on `--sanitizer=none`):

  ```bash
  RUST_BACKTRACE=1 \
    cargo +nightly fuzz run --fuzz-dir=tests/fuzz --sanitizer=thread fuzz_target
  ```

  Background: <https://github.com/stellar/rs-soroban-sdk/issues/1056>

## Reproducing and triaging a crash

When a run fails, libFuzzer writes the offending input to
`tests/fuzz/artifacts/fuzz_target/crash-<hash>`. Replay it deterministically:

```bash
# Re-run just the crashing input (prints the failing command sequence + backtrace)
RUST_BACKTRACE=1 cargo +nightly fuzz run --fuzz-dir=tests/fuzz fuzz_target \
  tests/fuzz/artifacts/fuzz_target/crash-<hash>

# Shrink it to a minimal reproducer
cargo +nightly fuzz tmin --fuzz-dir=tests/fuzz fuzz_target \
  tests/fuzz/artifacts/fuzz_target/crash-<hash>
```

Commit the minimized artifact and turn it into a regression test in the `tests`
crate so the same command sequence is checked on every `make test`.

The accumulated corpus lives in `tests/fuzz/corpus/fuzz_target/`; keeping it
around lets later runs start from interesting inputs instead of from scratch.
