# @alula/cli

Operator tooling for Alula protocol stewards. **Not for end users** —
end users interact with Alula via the webapp; this CLI is for
ceremonies, deploys, and audits.

## Philosophy

This CLI is the small set of commands that the official `stellar` CLI
does not cover. For anything `stellar` already handles (single-op
SetOptions, signing, submitting, key management), use `stellar`
directly — wrapping it would add maintenance burden without value and
would go stale every time stellar-cli bumps its flag surface.

The full operator runbook lives in
[`docs/multisig-operations.md`](../../docs/multisig.md), which
shows when to reach for `stellar` vs. `alula`.

## Prerequisites

- Node ≥ 22
- pnpm (already pinned in the repo root)
- The `stellar` CLI ≥ 23 ([install](https://github.com/stellar/stellar-cli)) — required for the SetOptions / sign / send steps the runbook expects.
- A funded Stellar account on the target network (testnet via friendbot, public via your funding source).

## Commands

| Command                       | Purpose                                                                                                                                                                                                                                                                                                                        |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `alula multisig setup`        | Build + submit the N+1-op envelope that configures a fresh account: add N signers at weight 1, set M-of-N thresholds, burn master, set `home_domain`. The one ceremony stellar-cli does not handle cleanly. On testnet, `--master-secret` is optional — omit it to auto-generate + friendbot-fund an ephemeral master keypair. |
| `alula multisig verify`       | Inspect signers + thresholds for an account; flag bricked / unsafe configs; optionally assert `--expect-threshold` and `--expect-signers`. Exits non-zero on any warning.                                                                                                                                                      |
| `alula multisig check-quorum` | Decode signatures from a (partially) signed envelope and report whether their on-chain weight meets the source account's `med_threshold`. Run this _before_ `stellar tx send` to avoid wasting a fee on `tx_bad_auth`.                                                                                                         |

Run `pnpm cli <group> <command> --help` to see flags for any command.

## Conventions

- **Network selection:** every command takes `--network testnet` or
  `--network public`. Both honor `HORIZON_URL` and `NETWORK_PASSPHRASE`
  overrides for private networks.
- **Env vars are equivalent to flags.** `--threshold 4` and
  `THRESHOLD=4 alula multisig setup …` do the same thing. CLI flags
  override env vars when both are set.
- **stdout is artifacts, stderr is narration.** `alula multisig verify
--account G… > snapshot.json` writes only the JSON snapshot to the
  file; warnings and progress stay visible in the terminal.
- **No build step.** `bin: tsx src/index.ts` — source is what runs,
  zero version drift.

## Quick reference

```bash
# Create a 3-of-5 multisig on testnet (auto-generates + friendbot-funds master):
pnpm cli multisig setup \
  --network testnet \
  --signers GAAA…,GBBB…,GCCC…,GDDD…,GEEE… \
  --threshold 3

# Same on public (master must already exist + be funded):
pnpm cli multisig setup \
  --network public \
  --master-secret S… \
  --signers GAAA…,GBBB…,GCCC…,GDDD…,GEEE… \
  --threshold 3

# Verify it landed correctly:
pnpm cli multisig verify \
  --network testnet \
  --account GMULTI… \
  --expect-threshold 3 \
  --expect-signers GAAA…,GBBB…,GCCC…,GDDD…,GEEE…

# Pre-flight a signed envelope before `stellar tx send`:
pnpm cli multisig check-quorum \
  --network testnet \
  --xdr "$(cat signed.xdr)"
```
