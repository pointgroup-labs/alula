# @alula/multisig-worker

Cloudflare Worker that acts as a signature relay for Alula's multisig coordination flow. See `docs/superpowers/specs/2026-04-29-alula-multisig-design.md` §7 for the design.

## Endpoints

- `POST /sigs/:proposalHash` — append a `alula-sig:v1:…` payload (structural validation only)
- `GET  /sigs/:proposalHash` — return all current payloads, newline-delimited
- `GET  /health` — liveness check

The Worker stores nothing it doesn't need to and performs no cryptographic validation; signers and aggregators verify locally.

## One-time setup

```bash
cd packages/multisig-worker
pnpm install
wrangler login
wrangler kv namespace create SIGS
# Paste the returned id into wrangler.toml under [[kv_namespaces]].id
wrangler deploy
```

## Local dev

```bash
pnpm dev
# Worker on http://127.0.0.1:8787
curl -X POST http://127.0.0.1:8787/sigs/$(printf 'a%.0s' {1..64}) \
  -H 'content-type: text/plain' \
  --data "alula-sig:v1:$(printf 'a%.0s' {1..64}):GA$(printf 'A%.0s' {1..54}):c2ln"
curl http://127.0.0.1:8787/sigs/$(printf 'a%.0s' {1..64})
```

## Notes

- KV TTL is 30 days. Older proposals are garbage-collected automatically.
- Per-proposal cap of 64 sig payloads to prevent storage abuse.
- The Worker is replaceable; any KV-shaped store with HTTP access can substitute. The webapp's `RelayConfig.baseUrl` is the only coupling point.
