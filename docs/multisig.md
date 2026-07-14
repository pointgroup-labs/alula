# Multisig — usage guide

How to create an Alula multisig, propose actions through it, collect
signatures, and submit. Two tools share the work:

- **Webapp UI** (`/multisig/*`) — propose, sign, aggregate. The day-to-day path.
- **`alula` CLI** (`pnpm cli multisig …`) — one-shot account setup and on-chain audit.

That's it. You don't need to learn anything else.

---

## Concepts in 30 seconds

A multisig is a regular Stellar G-address with a few extra rules:

- **N signers**, each weight `1`.
- **Threshold M** — every action needs `M` signatures.
- **Master burned** (weight `0`) — the funder loses solo control after setup.
- **Role tag** (e.g. `upgrade`, `ops`) — used by the webapp to pick the right
  multisig for each action.

Pick `M:N` so you can lose `N − M` signers without bricking the account.
Common shapes: `3-of-5` for ops, `4-of-7` for upgrades, `2-of-3` for
emergency keys.

---

## Part 1 — Create the multisig (one-time, CLI)

You only do this once per account. It needs to be atomic (add signers, set
threshold, and burn the master in one tx), which is exactly what
`alula multisig setup` is for.

### Testnet (auto-funded)

```bash
pnpm cli multisig setup \
  --network testnet \
  --signers GAAA…,GBBB…,GCCC…,GDDD…,GEEE… \
  --threshold 3
```

The CLI generates an ephemeral master keypair, friendbot-funds it,
adds your signers, sets the threshold, burns the master — all in one
transaction. The new G-address is printed at the end.

### Public (bring your own funder)

Friendbot doesn't exist on mainnet, so you supply a funded master:

```bash
# Generate + fund the master from any source (≥ 10 XLM).
stellar keys generate alula-bootstrap --no-fund
stellar keys public-key alula-bootstrap   # send XLM to this G…

# Run setup with that key.
pnpm cli multisig setup \
  --network public \
  --master-secret "$(stellar keys secret alula-bootstrap)" \
  --signers GAAA…,GBBB…,GCCC…,GDDD…,GEEE… \
  --threshold 4

# The master is now weight 0 — useless. Remove it from the keystore.
stellar keys rm alula-bootstrap
```

### Verify what you just created

```bash
pnpm cli multisig verify \
  --network testnet \
  --account G…NEW_MULTISIG \
  --expect-threshold 3 \
  --expect-signers GAAA…,GBBB…,GCCC…,GDDD…,GEEE…
```

`verify` exits non-zero on **any** misconfiguration: master not burned,
non-uniform thresholds, bricked (threshold > reachable weight), zero
threshold, or `--expect-*` mismatch. Wire it into CI so a bad multisig
can't promote.

### Register the address in the webapp

Add the new G-address to `packages/webapp/src/config/multisig.ts` under
its network and role:

```ts
export const KNOWN_MULTISIGS = {
  [Network.Testnet]: { upgrade: 'G…NEW_MULTISIG' },
  [Network.Public]: { upgrade: 'G…OTHER_ADDRESS' },
}
```

Once that's deployed, the Compose page will auto-pick this multisig
whenever someone selects an action with that role.

---

## Part 2 — Day-to-day: propose → sign → submit (UI)

Every authorized action goes through three pages. Each page does one
thing; you move between them by sharing a link or a short string.

### `/multisig/compose` — operator builds the proposal

1. Pick the catalog function (e.g. `queue_market_upgrade`).
2. The page auto-selects the right multisig for that role.
3. Fill in the arguments with the inline form.
4. Click **Build proposal** — the page shows:
   - The unsigned XDR
   - A human-readable diff of what will change
   - A signed-set snapshot (so cosigners verify the same signers you saw)
5. Click **Copy proposal link** and share it with cosigners (Slack, email).

The link looks like `https://app.alula.fi/multisig/sign#p=<base64url>`.
Everything the cosigner needs is in the fragment — there is no backend
session.

### `/multisig/sign` — each cosigner reviews and signs

1. Open the proposal link. The page decodes the fragment and renders:
   - Function name + arguments (structured)
   - Diff against current on-chain state (so you see exactly what changes)
   - Multisig address, network, signer set snapshot, hash
2. Read the diff carefully. The hash at the top is what you're actually signing.
3. Connect your wallet (Freighter, Albedo, Lobstr, hardware via wallet).
4. Click **Sign**. The page:
   - Verifies your wallet's signature locally (catches a wallet that signed the wrong tx)
   - POSTs it to the relay so the operator's Aggregate page picks it up automatically
5. **Done.** No manual hand-off needed in the happy path.

If the relay is unreachable, the page surfaces the error and shows the
sig as a copyable string:

```
alula-sig:v1:<hash>:<your G…>:<base64 sig>
```

Send that one line to the operator (chat, email, anything) and they
paste it into Aggregate manually.

The signer never touches a CLI. The wallet shows the operation type
(`SET_OPTIONS`, `INVOKE_HOST_FUNCTION`, etc.) and the source account
(your multisig) — match those to what `/multisig/sign` displays.

### `/multisig/aggregate` — operator collects signatures and submits

1. Open the same proposal link with `/multisig/aggregate` instead of
   `/multisig/sign` (or click **Aggregate** on Compose after building).
2. The page polls the relay every 5 seconds and lists every sig that's
   been submitted for this proposal hash. Each sig is **re-validated
   client-side** against the snapshot — the relay is untrusted, so a
   compromised or buggy relay can't slip a forged sig past you.
3. If a signer was offline and sent their `alula-sig:v1:…` line by hand,
   paste it into the manual-entry box and it's verified the same way.
4. The page sums verified weights and tells you when threshold is met.
5. Once **quorum met** turns green, click **Submit**.
6. Page polls Horizon for the result hash and shows success / failure.

If you're paranoid (you should be on mainnet), pre-flight from the CLI
before clicking Submit:

```bash
pnpm cli multisig check-quorum \
  --network public \
  --xdr "$(echo '<paste signed XDR from aggregate page>')"
```

> **Relay trust model.** The Cloudflare Worker is convenience-only:
> append-only KV bucket per proposal hash, no auth, no per-sig
> verification on the server side. Sign-time and aggregate-time
> verification both run in the browser. A relay outage degrades to the
> manual chat path; a malicious relay can withhold or replay sigs but
> cannot forge one. The default endpoint is
> `https://multisig-relay.alula.workers.dev`; override with
> `MULTISIG_RELAY_URL` at deploy time.

---

## Part 3 — Audit any multisig (CLI)

Anyone can inspect any multisig — read-only, no keys needed.

```bash
# Snapshot signers, weights, thresholds; warn on bricked / unsafe configs.
pnpm cli multisig verify \
  --network testnet \
  --account G…MULTISIG

# Same, with assertions for runbook automation.
pnpm cli multisig verify \
  --network public \
  --account G…MULTISIG \
  --expect-threshold 4 \
  --expect-signers GAAA…,GBBB…,GCCC…,GDDD…
```

Use this:

- After `setup`, to confirm what you built.
- After every signer rotation, to confirm the change landed.
- In CI, to gate releases on the multisig being in the expected shape.

---

## Adding / removing / rotating signers

The same UI flow as any other action. There is **no special "add signer" UI** —
you compose a `set-options` proposal, cosigners review, you submit.

A signer-set change must be authorized by the **current** threshold. There
is no admin override. Build the proposal with the catalog's
`multisig.set_signers` entry (or, until that ships, the `stellar` CLI as
shown below) and run it through the same compose → sign → aggregate flow.

### Stellar CLI fallback (until catalog entry ships)

```bash
# Add a signer, raising threshold from 3 to 4:
stellar tx new set-options \
  --network testnet \
  --source-account G…MULTISIG \
  --signer G…NEW \
  --signer-weight 1 \
  --low-threshold 4 --med-threshold 4 --high-threshold 4 \
  --build-only > unsigned.xdr

# Hand the XDR to cosigners:
stellar tx sign --network testnet --sign-with-key alice unsigned.xdr > a.xdr
stellar tx sign --network testnet --sign-with-key bob   unsigned.xdr > b.xdr
# … merge signatures (any tool that handles Stellar envelopes)

# Pre-flight, then send:
pnpm cli multisig check-quorum --network testnet --xdr "$(cat merged.xdr)"
stellar tx send --network testnet merged.xdr

# Verify:
pnpm cli multisig verify --network testnet --account G…MULTISIG \
  --expect-threshold 4 --expect-signers GAAA…,GBBB…,GCCC…,GDDD…,GNEW…
```

### Removing a signer

`--signer-weight 0` is "remove". Stellar deletes weight-0 entries from
the signer list.

> **Pre-flight rule the chain won't catch:** confirm `signers - 1 ≥ M`
> after the removal. Removing into a tight quorum (e.g. one of three at
> 3-of-3 without lowering M) bricks the account permanently.

When in doubt, lower M **in the same op** as the removal.

### Rotating a compromised key

Do removal + addition in the same envelope so there is never an
intermediate state with the wrong threshold ratio:

```bash
stellar tx new set-options \
  --source-account G…MULTISIG \
  --signer G…COMPROMISED --signer-weight 0 \
  --build-only > tmp.xdr

stellar tx operation add set-options \
  --signer G…REPLACEMENT --signer-weight 1 \
  tmp.xdr > unsigned.xdr
# … sign with M of the uncompromised signers, check-quorum, send.
```

If the multisig is wired in as a contract admin, the contract needs no
change — it keeps trusting the same G-address; only the keys behind it
move.

---

## Troubleshooting

| Symptom                                                           | Likely cause                                                                 | Fix                                                                                                                       |
| ----------------------------------------------------------------- | ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `verify` says **BRICKED**                                         | `reachable_weight < med_threshold`                                           | Threshold was set too high or signers were removed. The account is unrecoverable; create a new one and migrate authority. |
| `verify` says **master not burned**                               | `setup` was interrupted, or someone ran a half-baked manual ceremony         | Run `setup` again from a clean account, or compose a `set-options` op that burns the master + transfers authority.        |
| `aggregate` rejects a signature                                   | Wrong proposal hash, wrong network passphrase, or signer not in the snapshot | The cosigner signed a different proposal — re-share the link and re-sign.                                                 |
| Sign page shows **relay unreachable**                             | Cloudflare Worker is down or `MULTISIG_RELAY_URL` is misconfigured           | The page falls back to a copyable `alula-sig:v1:…` string — paste it into chat, operator pastes it into Aggregate's manual-entry box. |
| `aggregate` says quorum met but `Submit` fails with `tx_bad_auth` | Stale signer set (someone rotated mid-ceremony)                              | Re-compose against the new signer set; old proposal is dead.                                                              |
| `setup` on public refuses without `--master-secret`               | Friendbot is testnet-only                                                    | Generate + fund a master externally (≥ 10 XLM), then pass it via `--master-secret`.                                       |
| `setup` errors with `signer #N looks like a SECRET key`           | You pasted an `S…` instead of `G…`                                           | Use `stellar keys public-key <name>`, not `secret`.                                                                       |

---

## Quick reference

**Setup:**

```bash
# Testnet — fully automated:
pnpm cli multisig setup --network testnet --signers G…,G…,G… --threshold 2

# Public — BYO master:
pnpm cli multisig setup --network public --master-secret S… \
  --signers G…,G…,G…,G…,G… --threshold 3
```

**Audit:**

```bash
pnpm cli multisig verify --network <net> --account G… \
  [--expect-threshold M] [--expect-signers G…,G…]
```

**Pre-flight a signed envelope:**

```bash
pnpm cli multisig check-quorum --network <net> --xdr "$(cat signed.xdr)"
# or: cat signed.xdr | pnpm cli multisig check-quorum --network <net>
```

**UI pages:**

| Page                  | Who uses it   | What it produces                     |
| --------------------- | ------------- | ------------------------------------ |
| `/multisig/compose`   | Operator      | A proposal link (`#p=…`) to share    |
| `/multisig/sign`      | Each cosigner | A `alula-sig:v1:…` line to send back |
| `/multisig/aggregate` | Operator      | A submitted tx hash                  |

---

## Checklist — creating a new multisig

- [ ] `pnpm cli multisig setup …` succeeds; tx hash recorded.
- [ ] `pnpm cli multisig verify --expect-threshold M --expect-signers …` exits 0.
- [ ] (Public only) Bootstrap secret removed: `stellar keys rm alula-bootstrap`.
- [ ] Address added to `KNOWN_MULTISIGS` in `packages/webapp/src/config/multisig.ts`.
- [ ] First test proposal composed → signed → aggregated → submitted on testnet to confirm the UI loop works end-to-end.
- [ ] Contract admin slots re-pointed at the new G-address via separate proposals.

## Checklist — submitting a proposal

- [ ] Compose page shows the diff you intended.
- [ ] Proposal hash matches what cosigners report seeing.
- [ ] Aggregate page reports **quorum met** (green).
- [ ] (Mainnet) `pnpm cli multisig check-quorum` agrees.
- [ ] Submit succeeded; tx hash recorded.
- [ ] `pnpm cli multisig verify` (if the proposal changed signers/thresholds) confirms the new state.
