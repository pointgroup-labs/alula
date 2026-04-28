# Alula Multisig Management — Design

**Status:** Draft for review
**Date:** 2026-04-29
**Scope:** v1-C — all three multisigs (Upgrade, Program, Ops), end-to-end

## 1. Summary

Alula's privileged operations (WASM upgrades, pool/market configuration, treasury distribution, insurance-fund withdrawals) are currently gated by single-`Address` `require_auth()` checks in the contracts. This design replaces those single addresses with **Stellar classic-account multisig accounts** — three of them, each tuned to its purpose — and adds a single section to the existing `webapp` package that lets a signer click a link, connect their wallet, review a human-readable diff, and sign.

**Zero contract changes.** All security comes from Stellar Core's network-enforced signer thresholds and `timeBounds`, plus the existing on-contract queue/apply timelock pattern.

The three multisigs are:

| Multisig | M-of-N | Role |
|---|---|---|
| Upgrade | 4-of-7 | WASM-hash changes, contract upgrades, admin rotation |
| Program | 6-of-10 | Pool configs, market config updates, market status, farms wiring |
| Ops | 3-of-7 | Fee-beneficiary configuration, treasury withdrawals, insurance-fund operations |

The webapp UI is sufficient for a non-technical Stellar user: one click on a shared link, one wallet popup, one click to sign. Signatures are coordinated through a 30-line Cloudflare Worker backed by KV (with a manual paste fallback baked in for resilience).

## 2. Goals and non-goals

**Goals**
- Replace single-admin custody with M-of-N multisig for all three role classes
- Provide a click-link-and-sign UX usable by signers who are not engineers
- Enforce a 48-hour timelock on Upgrade-class actions
- Cover every privileged function across `market`, `market_manager`, and `controlled_insurance_fund` in v1
- Add no on-chain dependencies (no timelock contract, no governance contract)
- Add no operational dependencies that the team cannot run with no maintenance budget

**Non-goals**
- On-chain governance (token-weighted voting, delegation, proposals open to outside parties) — out of scope, may be a future protocol direction
- Per-operation thresholds (e.g. "this Ops action needs 5/7") — uniform-weight signers per multisig are sufficient
- Replacing or wrapping the on-contract queue/apply pattern with a separate timelock contract — the existing contract code already handles this
- Hardware-wallet-specific UX beyond what `@creit.tech/stellar-wallets-kit` already supports
- A formal audit log database — git history of completed proposals (kept by operators in a private repo) is the audit trail

## 3. Constraints and locked decisions

These are settled and not revisited below:

1. **No contract changes.** Existing `Address::require_auth()` slots are replaced with classic G-account addresses configured for native multisig.
2. **Pattern X auth.** Whenever a Soroban call requires the multisig's auth, the multisig account is the **transaction source account**. Signers sign the tx envelope with their own keys; Stellar Core verifies signatures against the account's signer set at medium threshold; `require_auth()` succeeds via `SorobanCredentials::SourceAccount`. This avoids the auth-entry signing flow that has uneven wallet support.
3. **Timelock = Stellar pre-auth `minTime`.** The `apply_*` transaction is constructed with `timeBounds.minTime = queueLandedAt + DELAY` and `maxTime = minTime + 7d`. Stellar Core enforces both. No on-chain timelock contract.
4. **UI lives inside `packages/webapp`.** Lazy-loaded routes under `/multisig/*`. Not linked from the main user navigation. The only new sibling package is `packages/multisig-worker` (the Cloudflare Worker), which is small and self-contained.
5. **Coordination via URL fragment + KV relay, with manual-paste fallback.** No D1, no schema, no Cloudflare Access, no operator login required.
6. **Parallel signing.** Signatures over a Stellar tx envelope are independent; signers do not chain.
7. **Function catalog as the scope knob.** Adding a new privileged function later means adding one file to `webapp/src/lib/multisig/catalog/`; pages and lib do not change.

## 4. Architecture overview

```
                       ┌─────────────────────────────────────────┐
                       │            packages/webapp              │
                       │                                         │
   pages/multisig/     │  compose.vue ──▶ build XDR              │
                       │  sign.vue    ──▶ decode + sign + relay  │
                       │  aggregate.vue ─▶ collect + submit      │
                       │                                         │
   src/lib/multisig/   │  build.ts, decode.ts, sign.ts,          │
                       │  submit.ts, url.ts, wasm.ts,            │
                       │  catalog/{upgrade,program,ops}/*.ts     │
                       └────────────────────┬────────────────────┘
                                            │
                                  ┌─────────┴─────────┐
                                  ▼                   ▼
                  ┌──────────────────────┐   ┌─────────────────────┐
                  │   Stellar RPC/Horizon│   │  Cloudflare Worker  │
                  │   (submission, state)│   │  + KV (sig relay)   │
                  └──────────────────────┘   └─────────────────────┘
```

### 4.1 Components and responsibilities

**`packages/webapp/src/lib/multisig/`** — pure TypeScript, framework-free.

- `build.ts` — given a catalog entry and args, produce an unsigned tx XDR with the multisig account as source. Handles fee estimation (simulated cost × 1.5), seqnum fetch, network passphrase, timeBounds.
- `decode.ts` — given a tx XDR, identify which catalog entry it represents, decode args, render a human-readable summary.
- `sign.ts` — given a tx XDR and a wallet adapter, produce an envelope signature.
- `submit.ts` — given a fully signed envelope, broadcast via RPC, return tx hash.
- `url.ts` — pack/unpack proposal payloads to/from URL fragments and `.alulasig` files.
- `wasm.ts` — compute SHA-256 of WASM bytes client-side; verify against on-chain hash; produce diff between current `market_wasm_hash` and proposed.
- `relay.ts` — POST/GET signature payloads to/from the KV relay; gracefully degrade to manual paste on error.
- `catalog/` — one file per privileged function (see §6).

**`packages/webapp/src/pages/multisig/`** — Vue/Nuxt routes.

- `compose.vue` — operator's proposal builder. Pick role → pick function → form auto-renders from `ArgSchema` → preview shows summary + before/after diff → "Generate signing link".
- `sign.vue` — signer's view. Reads URL fragment, decodes, shows summary + diff + WASM hash check + timelock notice. Connect wallet → sign → auto-POST sig to relay → "Send confirmation" copy button as fallback.
- `aggregate.vue` — proposer/aggregator dashboard. Shows proposal + live signer status from relay polling + paste box for manual sigs + Submit button when threshold met.

In practice `compose.vue` and `aggregate.vue` are likely the same route in two states; treating them separately in this doc for clarity.

**`multisig-worker/`** — sibling Cloudflare Worker (`packages/multisig-worker/`).

- `src/index.ts` — Hono app with two routes:
  - `POST /sigs/:proposalHash` — append a sig payload to the KV value (deduped, capped at 64 entries to prevent abuse, 30-day TTL).
  - `GET /sigs/:proposalHash` — return the current list.
- `wrangler.toml` — KV namespace binding, route `/api/multisig/*`.
- No DB, no schema, no migrations, no auth.

### 4.2 Why these boundaries

The lib is pure TS so the entire signing/decoding/submitting flow can be exercised from vitest without DOM or Vue. The Worker is small enough that it can be re-implemented on any other backend (Lambda, Deno Deploy, a tiny VPS) in an hour. The Vue layer is the only place that depends on Nuxt — if the multisig surface ever needs to move out of `webapp`, only the page files have to change.

## 5. Proposal lifecycle

### 5.1 Single-stage flow (Program and Ops, mostly)

```
[Operator]                                    [Signers, in parallel]              [Stellar]
    │                                              │
    ├─ /multisig/compose                           │
    ├─ pick role + function + args                 │
    ├─ build unsigned XDR                          │
    │   sourceAccount = multisig G…                │
    │   fee, seqnum, timeBounds set                │
    ├─ generate signing URL                        │
    │   https://app.alula.fi/multisig/sign#p=…     │
    ├─ post URL once to operator/signer channel ──▶│
    │                                              │
    │                                              ├─ open URL
    │                                              ├─ page decodes, shows diff
    │                                              ├─ connect wallet
    │                                              ├─ sign envelope
    │                                              ├─ page POSTs sig to relay
    │                                              └─ shows fallback copy if relay fails
    │                                              (each signer independently)
    ├─ /multisig/aggregate                         │
    ├─ poll relay; sigs appear live                │
    ├─ when total weight ≥ threshold:              │
    └─ click Submit ─────────────────────────────────────────────▶ tx broadcast
                                                                   tx hash returned
```

### 5.2 Two-stage flow (Upgrade-class actions: queue + apply)

The on-contract queue/apply pattern (`queue_in_market_upgrade` → `apply_in_market_upgrade`, similarly for `queue_pool_set`/`apply_pool_set` and `queue_market_update`/`apply_market_update`) becomes **two linked proposals** in the UI:

1. **Stage 1 — queue.** Standard single-stage flow above. After submission, the on-contract `QueuedInMarketUpgrade` (or equivalent) is set, recording the change at a known ledger time `T_queue`.

2. **Stage 2 — apply.** Operator opens `compose.vue`, selects "Apply queued upgrade" for the previous proposal. Webapp reads the on-chain queued state (via RPC) to confirm the queue landed and to extract its details. Webapp builds the apply XDR with `timeBounds.minTime = T_queue + 48h` and `maxTime = minTime + 7d`. Signers sign as soon as it is ready; the network rejects submission until `minTime` passes. After `minTime`, anyone with the proposal can submit.

Cancellation (`cancel_*`) is a third single-stage proposal targeting the same queued-state slot.

### 5.3 Key invariants

1. **Server is never on the critical path.** If the relay is unavailable, every step still works via manual paste of a `~150-byte` payload (`alula-sig:v1:<proposalHash>:<signerPubkey>:<base64sig>`).
2. **Server-supplied data is always re-verified client-side.** The signing page recomputes the `proposalHash` from the local unsigned XDR and rejects sig payloads whose embedded hash does not match. The relay can withhold sigs but cannot forge them.
3. **Sig payloads are validated before they affect the dashboard.** Each pasted/relayed sig is decoded, its proposal hash compared, its signer pubkey checked against the multisig's on-chain signer set (snapshotted at compose), and its ed25519 signature verified locally. Invalid payloads are silently rejected with a UI warning.
4. **Threshold awareness is display-only.** The UI's "5/6 collected" is a hint; Stellar Core remains the only authority that decides whether a tx is sufficiently signed.
5. **WASM-hash verification is mandatory.** When a queue-upgrade proposal is composed, the operator uploads the `.wasm` bytes; the page computes SHA-256 client-side and shows the result alongside the on-chain `market_wasm_hash`. Each signer's view also re-computes the SHA-256 if they re-upload the artifact, and shows the proposed hash prominently. A clear "Old → New" diff is the centerpiece of the sign-page UI.
6. **Signer-set rotation is detected.** At compose time, the on-chain signers + thresholds for the multisig account are snapshotted into the proposal payload. At sign time, the page re-fetches and warns loudly if the live signer set differs from the snapshot.
7. **Sequence-number staleness is visible.** Because tx envelope signing covers the seqnum, if the multisig account's seqnum advances between compose and submission (by an unrelated tx), the proposal becomes unsubmittable. The aggregate page polls seqnum and shows "stale — rebuild required" when this happens.
8. **Apply XDRs are composed only after queue lands.** The seqnum for the apply tx must reflect the post-queue state, so apply is composed against the live chain after queue has landed. The 48h timelock clock starts from `T_queue` (queue-landing ledger time), not from when apply was composed.

## 6. Function catalog

The catalog is the registry of privileged functions the UI knows how to compose proposals for. Each entry exports a `FunctionDef`:

```ts
interface FunctionDef {
  multisig: 'upgrade' | 'program' | 'ops'
  contract: 'market' | 'market_manager' | 'controlled_insurance_fund'
  function: string
  argSchema: ArgSchema           // typed Soroban arg shape, drives the form
  fetchBeforeSnapshot?: (env: ChainEnv, args: any) => Promise<unknown>
  renderSummary: (args: any, snapshot: unknown) => HumanDiff
  isTimelocked: boolean
  pairWith?: { queue: string, apply: string, cancel?: string }
}
```

### 6.1 Upgrade multisig (4-of-7)

Account becomes the admin of `MarketManager`. Owns WASM-hash changes and admin rotation.

| Function | Contract | Timelocked | Pair |
|---|---|---|---|
| `queue_in_market_upgrade` | `market_manager` | yes | apply, cancel |
| `apply_in_market_upgrade` | `market_manager` | (gated by parent's queue) | — |
| `cancel_in_market_upgrade` | `market_manager` | no | — |
| `queue_in_manager_upgrade` | `market_manager` | yes | apply, cancel |
| `apply_in_manager_upgrade` | `market_manager` | (gated by parent's queue) | — |
| `cancel_in_manager_upgrade` | `market_manager` | no | — |
| Admin rotation (propose) | `market_manager` | no | — |
| Admin rotation (accept) | `market_manager` | no | — |

Admin rotation for `market` and `controlled_insurance_fund` is also Upgrade-class (changes who governs the protocol) and lives in this multisig's catalog even though the target contract is not `market_manager`. The pattern uses each contract's own `propose_new_admin` / `accept_proposed_admin` two-step flow.

### 6.2 Program multisig (6-of-10)

Account becomes the admin of `Market`. Owns risk parameters and pool configuration.

| Function | Contract | Timelocked | Pair |
|---|---|---|---|
| `queue_pool_set` | `market` | yes | apply, cancel |
| `apply_pool_set` | `market` | (gated) | — |
| `cancel_pool_set` | `market` | no | — |
| `queue_market_update` | `market` | yes | apply, cancel |
| `apply_market_update` | `market` | (gated) | — |
| `cancel_market_update` | `market` | no | — |
| `update_market_status` | `market` | no | — |
| `set_farms_contract` | `market` | no | — |

`update_market_status` only covers transitions that are not `*ByAdmin`-protected, per the existing contract logic in `MarketStatus::is_admin_protected`.

### 6.3 Ops multisig (3-of-7)

Account becomes the admin of `ControlledInsuranceFund` and is added as a fee beneficiary administrator. Owns treasury distribution, fee beneficiary configuration, and insurance-fund day-to-day operations.

| Function | Contract | Timelocked | Pair |
|---|---|---|---|
| `set_take_rate_fees_beneficiaries` | `market` | no | — |
| `set_operation_fees_beneficiaries` | `market` | no | — |
| `withdraw` (treasury) | `controlled_insurance_fund` | no | — |
| `mark_ready` | `controlled_insurance_fund` | no | — |
| `update_market_status` (descriptive) | `controlled_insurance_fund` | no | — |
| `set_market` | `controlled_insurance_fund` | no | — |

Note that `distribute_pool_fees` and `distribute_all_pools_fees` exist on `Market` but are confirmed permissionless (anyone can call them); they are not catalog entries. If during implementation we discover an additional Ops-class function, it is added as one new file in `catalog/ops/`.

### 6.4 Snapshot fetchers

For each function whose semantic effect is "change a current value", `fetchBeforeSnapshot` reads the current value from chain so the UI can render `Before → After` rather than only `After`. Examples:

- `queue_pool_set` → snapshot = current `PoolConfig` for that pool
- `update_market_status` → snapshot = current `MarketStatus`
- `set_farms_contract` → snapshot = current farms contract address (or `None`)
- `set_take_rate_fees_beneficiaries` → snapshot = current beneficiary list and weights
- `withdraw` → snapshot = fund's current balance and locked amount for the token

## 7. Coordination relay

### 7.1 Worker behavior

```ts
// pseudocode
app.post('/sigs/:hash', async (c) => {
  const payload = (await c.req.text()).trim()
  if (!isWellFormedSigPayload(payload)) return c.text('bad', 400)
  const key = `sigs:${c.req.param('hash')}`
  const cur = (await KV.get(key))?.split('\n').filter(Boolean) ?? []
  if (cur.length >= 64) return c.text('full', 429)
  if (!cur.includes(payload)) cur.push(payload)
  await KV.put(key, cur.join('\n'), { expirationTtl: 30 * 24 * 3600 })
  return c.text('ok')
})

app.get('/sigs/:hash', async (c) =>
  c.text((await KV.get(`sigs:${c.req.param('hash')}`)) ?? ''))
```

The Worker performs only structural validation (regex check on `alula-sig:v1:<hex64>:<G…>:<base64>`); cryptographic validation happens client-side in every signer/aggregator browser. This keeps the Worker stateless and replaceable.

### 7.2 Why KV and not D1

The Worker's only data shape is `proposalHash → list of sig payloads`, with TTL. KV is exactly this shape and requires no schema/migrations. D1 would add complexity (migrations, query layer, ORM) for no functional gain — there are no joins, no indexes beyond the primary key, and no queries beyond "give me all sigs for hash X".

### 7.3 Why a relay at all

Without it, every signer must paste a sig string back into the chat channel and the operator must paste each into the dashboard. With ~7-10 signers per upgrade, that's 7-10 manual paste operations per round. The Worker eliminates this with negligible code (~30 lines) and zero ongoing cost. The paste fallback remains in the UI permanently so the system never has a hard dependency on Cloudflare being up.

## 8. Trust and security model

### 8.1 What is enforced where

| Property | Enforced by | Notes |
|---|---|---|
| M-of-N signature threshold | Stellar Core | Native classic-account multisig at medium threshold |
| Soroban `require_auth` for the multisig | Stellar Core (Soroban host) | Via `SorobanCredentials::SourceAccount` |
| 48h timelock on apply | Stellar Core | Via tx envelope `timeBounds.minTime` |
| Apply seqnum + queued-state coherence | The on-contract `QueuedIn*` storage | Exists today, unchanged |
| Sig payload authenticity | Webapp client-side ed25519 verify | Defense in depth before submission; Core re-verifies |
| Proposal hash binding | Webapp client-side | Sig payloads carry the hash; mismatched hashes are rejected |
| WASM-hash → bytes integrity | Webapp client-side SHA-256 | Operator uploads bytes; signer can re-upload to re-verify |

### 8.2 What a compromised component can and cannot do

- **Compromised Worker / KV:** can withhold sigs, serve junk, return stale data. Cannot forge sigs (client-side ed25519 verify), cannot make a different XDR look like the original (client-side hash check), cannot bypass threshold (network-enforced). Worst case: signers fall back to manual paste.
- **Compromised webapp build:** can present a misleading summary. The wallet popup, however, shows the *raw* envelope hash being signed; signers should be trained to verify the operation type and target contract in their wallet's review screen, not just trust the in-page summary. This is a real residual risk and the only one not mitigated by client-side cryptography alone.
- **Compromised single signer key:** loses one weight unit. Threshold remains uncrossed. Standard multisig assumption.
- **Compromised operator workstation:** an attacker can broadcast malicious proposals, but signers are the gate. Operator's keys are not custodial.

### 8.3 Cloudflare Access is intentionally not used

Earlier drafts considered putting Access in front of `/multisig/*` to gate viewing. We rejected this because:

- It would require operator-class accounts for every signer, contradicting the "non-technical Stellar user" requirement.
- It is UX-quality security only; it does not change what the cryptographic multisig can or cannot accept.
- It complicates the URL-share flow (Access redirects break embedded fragments).

The pages are publicly viewable. Anyone with the URL can see the proposal — which is fine, because privileged actions are gated on signatures, not on viewing.

## 9. Failure modes

| Failure | Behavior |
|---|---|
| Worker / KV down | Signer page shows "Sig saved locally — copy this string and send manually". Aggregator paste box accepts it. |
| Multisig account seqnum advances mid-flight | Aggregate page shows "Sequence number stale, rebuild required". Operator rebuilds with current seqnum; signers re-sign. |
| Signer set rotated mid-flight | Sign page shows "Signer set changed since this proposal was created" and refuses to sign. Operator rebuilds. |
| Insufficient resource fee at submission | Submission fails with a Soroban resource error. Aggregate page offers "fee-bump" — wraps the tx in a `FeeBumpTransaction` paid by a designated hot account, single-sig, no re-collection of multisig sigs. |
| Apply tx submitted before `minTime` | Network rejects with `tx_too_early`. Aggregate page shows wait-time. |
| Apply tx submitted after `maxTime` | Network rejects with `tx_too_late`. Operator rebuilds apply with new `timeBounds`; signers re-sign. |
| Wrong WASM bytes uploaded vs intended | Caught at compose time (operator sees mismatched SHA-256), and again at sign time (signer can re-upload artifact and verify). The proposal cannot proceed without a hash match. |
| URL fragment too long for chat platform | Operator downloads `.alulasig` file via a button on the compose page; signers drag-drop it into the sign page. Same payload, different transport. |
| Relay returns junk for a proposal hash | Client-side validation rejects every entry. Aggregate shows "0/N collected" despite Worker traffic; operator switches to manual paste. |

## 10. Build phases

A single engineer, ~4 weeks, in this order:

1. **Week 1 — lib + Worker + Upgrade catalog.** `build.ts`/`decode.ts`/`sign.ts`/`submit.ts`/`url.ts`/`wasm.ts`/`relay.ts`. Cloudflare Worker with KV. Catalog entries for the queue/apply/cancel triples on `in_market_upgrade` and `in_manager_upgrade` (six entries). Admin-rotation entries are deferred to week 4. Vitest unit tests for the lib. End-to-end test against testnet with a 4-of-7 synthetic multisig: queue → 4 sigs via relay → submit; apply with `minTime` → wait → submit.

2. **Week 2 — pages.** `compose.vue`, `sign.vue`, `aggregate.vue`. Generic form rendering from `ArgSchema`. Wallet integration via existing `@creit.tech/stellar-wallets-kit`. `.alulasig` file fallback. Cypress flow against testnet.

3. **Week 3 — Program catalog + diff snapshots.** Eight entries (`queue/apply/cancel_pool_set`, `queue/apply/cancel_market_update`, `update_market_status`, `set_farms_contract`). `fetchBeforeSnapshot` for each, with proper Soroban type decoding for `PoolConfig` (the most complex shape). Half-day spike to enumerate any Program-class admin functions on the `farms` contract that should be added. Operator-runbook documentation.

4. **Week 4 — Ops catalog + admin-rotation flows + polish.** Ops entries (six). Admin-rotation flows for all three contracts. Pre-flight `simulateTransaction` integration. Signer-name display from a local `signers.json` config. Optional `MEMO_HASH` of proposal hash on the submission tx for on-chain audit. Final testnet rehearsal of all three multisigs.

Out of v1: any function not enumerated in §6, on-chain governance, custom thresholds.

## 11. URL fragment and payload formats

### 11.1 Proposal payload (`#p=…`)

Base64url-encoded JSON:

```json
{
  "v": 1,
  "kind": "proposal",
  "network_passphrase": "Public Global Stellar Network ; September 2015",
  "multisig": "upgrade",
  "function_id": "market_manager.queue_in_market_upgrade",
  "args": { "new_wasm_hash": "..." },
  "snapshot": { "current_market_wasm_hash": "..." },
  "unsigned_xdr": "AAAA...",
  "proposal_hash": "<sha256 of canonicalized fields>",
  "created_by": "GABC...",
  "created_at": 1745899200,
  "signer_set_snapshot": [
    { "key": "GXYZ...", "weight": 1 },
    ...
  ],
  "thresholds_snapshot": { "low": 0, "med": 4, "high": 4 },
  "parent_proposal_hash": null
}
```

`proposal_hash` is the SHA-256 of the canonical JSON encoding of `{network_passphrase, function_id, args, unsigned_xdr, created_at}`. Other fields are advisory and are validated locally against the unsigned XDR.

### 11.2 Sig payload (relay or paste)

Plain text, ~150 bytes:

```
alula-sig:v1:<proposal_hash_hex>:<signer_pubkey_G…>:<base64_signature>
```

Strict format. Anything else is rejected.

### 11.3 `.alulasig` file fallback

A file containing the same base64url-encoded JSON as the URL fragment, optionally with a `signatures` array appended for partially-signed snapshots. Same validation as the URL form.

## 12. Open questions

These are not blockers for v1 but should be confirmed during implementation:

1. **Fee-bump source account for Ops/Program.** The fee-bump pattern needs a designated hot account that pays inclusion fees. Where does its XLM come from, who tops it up, and is its key custodied by an operator or a small hot multisig?
2. **Are there any privileged functions on `farms` itself?** The `libs/farms-interface/` crate only shows `set_stake_delegated`. The actual farms contract may have its own admin functions (creating farms, configuring rewards) that should be cataloged. Needs a half-day during week 3.
3. **`treasury` is currently defined as InsuranceFund withdrawal + fee-beneficiary distribution.** Confirm with the team that no other notion of "treasury" exists (e.g. a separate vault contract).
4. **Per-signer name display.** A `webapp/src/lib/multisig/signers.json` mapping `G… → "Alice"` is convenient. Does the team want this checked into the public repo, or kept in a private location and loaded via env?
5. **Mainnet rollout sequencing.** When the multisigs are activated (admin-rotation txs from current single admins to the new G-account multisigs), it should happen in a defined order with rehearsal on testnet first. The runbook in week 4 covers this; the team should confirm timing.

## 13. Glossary

- **Pattern X.** Auth pattern where the multisig account is the tx source account; envelope signing satisfies `require_auth` via `SorobanCredentials::SourceAccount`.
- **Proposal hash.** SHA-256 over canonicalized proposal fields, used to bind sig payloads to a specific proposal.
- **Sig payload.** A `~150-byte` string `alula-sig:v1:<hash>:<G…>:<sig>` that one signer produces and the aggregator collects.
- **Aggregator.** Whoever runs the `aggregate.vue` page in this round. Not a privileged role; can be any signer or the operator.
- **Operator.** Whoever composes the proposal in `compose.vue`. Not a privileged role; their keys are not custodied by the protocol.
- **Multisig account.** A Stellar classic G-account configured with multiple signers and a medium threshold matching the M of M-of-N.
