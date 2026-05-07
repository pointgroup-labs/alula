<script lang="ts" setup>
/**
 * Compose page — operator selects a catalog function, fills args, and
 * builds an unsigned proposal envelope. Output is a `#p=…` URL that the
 * operator distributes to signers.
 *
 * Phase 1 only renders args of kind `wasm-hash`; other kinds throw at
 * encode time so this UI fails loudly rather than producing a malformed
 * proposal.
 */

import type { FunctionDef, MultisigRole, ProposalPayload, SimulateResult } from '~/utils/multisig'
import { CONTRACT_ID } from '@alula/client-sdk'
import { Networks } from '@stellar/stellar-sdk'
import { KNOWN_MULTISIGS } from '~/config'
import {
  buildProposal,
  encodeProposalToFragment,
  listAllFunctions,
  simulateProposalEnvelope,
} from '~/utils/multisig'
import AddressPicker from './AddressPicker.vue'
import WasmHashVerifier from './WasmHashVerifier.vue'

// Picker fallback strings live inside <AddressPicker>; this file no
// longer needs the MANUAL sentinel directly.

const wallet = useWallet()
const rpcStore = useRpcStore()
const marketsStore = useMarketsStore()
const toast = useToast()

const functionId = ref<string | null>(null)
const args = ref<Record<string, string>>({})

// Effective addresses owned at this level. <AddressPicker> manages its
// own dropdown vs manual-input split internally; we only see the final
// string. Empty string means "not set" — the disabled-reason guard
// surfaces that as "address is empty".
const multisigAccountAddress = ref<string>('')
const marketAddress = ref<string>('')

// Market-manager picker: defaults to the SDK constant for the current
// network and only opens an override input when the operator asks.
// Distinct shape from the other two pickers (default-or-override, not
// dropdown-or-manual), so it stays bespoke.
const marketManagerOverride = ref(false)
const marketManagerManual = ref<string>('')

// Per-arg state for the wasm-hash verifier lives inside <WasmHashVerifier>
// itself — it's per-instance and never read from the parent.

const building = ref(false)
const buildError = ref<string | null>(null)
const builtProposal = ref<ProposalPayload | null>(null)
// Result of the post-build simulate pass. `null` until build runs;
// reset to `null` whenever the form changes so the success card can't
// claim "passed simulation" against a stale envelope.
const simulateResult = ref<SimulateResult | null>(null)
// Wall-clock time the current `simulateResult` was produced. Drives the
// "Xs ago" freshness badge in the simulation banner — without it, the
// Re-simulate button is a no-feedback action (the green box looks the
// same before and after the click). `null` whenever simulateResult is.
const simulatedAt = ref<number | null>(null)
// Live "ago" tick — re-evaluated by `now()` to keep the freshness label
// honest after the page sits idle.
const now = ref(Date.now())
// Re-simulate spinner state. Distinct from `building` because the
// envelope is unchanged — only the on-chain check is refreshed.
const resimulating = ref(false)
// Monotonic request token. Bumped by every form-edit watcher and read
// by `build()`/`resimulate()` before they await; on resolve they only
// publish results if the token they captured is still the live value.
// Without this, an in-flight RPC call that resolves AFTER the user has
// already started editing the form would resurrect a stale proposal or
// stamp a stale "simulation passed" against a form that's since moved.
const buildToken = ref(0)
function invalidateBuild() {
  buildToken.value += 1
  builtProposal.value = null
  simulateResult.value = null
  simulatedAt.value = null
}

// All catalog functions, flattened across roles, sorted for the dropdown:
// role (alpha) → stage (queue → apply → cancel → other) → displayName.
// Operators pick the action they want; `role` falls out of that choice.
const STAGE_ORDER: Record<string, number> = { queue: 0, apply: 1, cancel: 2 }
function stageRank(fn: FunctionDef<any, any>): number {
  const prefix = fn.function.split('_')[0]!
  return STAGE_ORDER[prefix] ?? 99
}
const allFunctions = computed<FunctionDef<any, any>[]>(() =>
  listAllFunctions().toSorted((a, b) =>
    a.multisig.localeCompare(b.multisig)
    || stageRank(a) - stageRank(b)
    || a.displayName.localeCompare(b.displayName),
  ),
)

const functionOptions = computed<string[]>(() => allFunctions.value.map(f => f.id))

const selectedFn = computed<FunctionDef<any, any> | undefined>(() =>
  allFunctions.value.find(f => f.id === functionId.value),
)

// Role is derived from the selected function. Kept as a computed so the
// existing `watch(role)` that snaps the multisig still fires when the
// operator changes function across roles.
const role = computed<MultisigRole | undefined>(() => selectedFn.value?.multisig)

// The Market picker is only meaningful for catalog entries that target a
// specific market contract. Every Phase 1 entry targets `market_manager`,
// so the field stays hidden by default. Future Program/Ops entries that
// invoke a single market directly will flip this true.
const requiresMarket = computed(() => selectedFn.value?.contract === 'market')
const requiresMarketManager = computed(() => selectedFn.value?.contract === 'market_manager')

// `affectsAllMarkets` flags actions whose on-chain effect propagates to
// every market spawned by the manager — surfaces as a blast-radius warning
// so the operator can't silently broadcast a market WASM swap.
const affectsAllMarkets = computed(() => Boolean(selectedFn.value?.affectsAllMarkets))

const networkPassphrase = computed(() =>
  rpcStore.network === 'public' ? Networks.PUBLIC : Networks.TESTNET,
)

const networkLabel = computed(() =>
  rpcStore.network === 'public' ? 'Mainnet' : 'Testnet',
)

// `CONTRACT_ID` is the SDK-shipped market_manager address per network.
// Only `testnet` is populated today; `public` returns undefined and the
// override field is forced open in that case.
const defaultMarketManager = computed(() => CONTRACT_ID[rpcStore.network] ?? '')

const marketManagerAddress = computed(() =>
  marketManagerOverride.value || !defaultMarketManager.value
    ? marketManagerManual.value
    : defaultMarketManager.value,
)

// Effective addresses are plain refs above; no computed reconciliation
// needed because <AddressPicker> hands us the final string directly.

function truncateAddress(addr: string, head = 6, tail = 6): string {
  if (!addr) { return '' }
  if (addr.length <= head + tail + 1) { return addr }
  return `${addr.slice(0, head)}…${addr.slice(-tail)}`
}

function capitalize(s: string | undefined): string {
  if (!s) { return '' }
  return s.charAt(0).toUpperCase() + s.slice(1)
}

// Format-validation helpers. Stellar uses Crockford base32 (A-Z2-7) for
// strkeys; G… for accounts (56 chars), C… for contract IDs (56 chars).
// Soroban wasm hashes are 32-byte SHA-256 → 64 lowercase hex characters
// (we canonicalise on input so signers see the exact same bytes the
// composer pasted).
const STELLAR_ACCOUNT_RE = /^G[A-Z2-7]{55}$/
const SOROBAN_CONTRACT_RE = /^C[A-Z2-7]{55}$/
const WASM_HASH_RE = /^[0-9a-f]{64}$/

function isStellarAccount(s: string): boolean { return STELLAR_ACCOUNT_RE.test(s) }
function isSorobanContract(s: string): boolean { return SOROBAN_CONTRACT_RE.test(s) }
function isWasmHash(s: string): boolean { return WASM_HASH_RE.test(s) }

// Per-arg canonicaliser. wasm-hash is normalised aggressively (trim +
// lowercase) because the field is meant to round-trip a fixed 64-hex
// string and any whitespace or case difference would break determinism
// of the proposal hash. Other kinds are left as-is on input — trimming
// mid-edit would prevent the user from typing a trailing space in a
// future free-text field. The build path runs a final canonical pass
// for all kinds, so untrimmed values still get cleaned before hashing.
function canonicalizeArg(value: string, kind: string): string {
  if (kind === 'wasm-hash') { return value.trim().toLowerCase() }
  return value
}

// `canonicalizeArg` for the build path — trims everything since by then
// the user has clicked Build and we want a clean payload regardless of
// kind. Separate from the on-input version so live editing isn't fighty.
function canonicalizeArgForBuild(value: string, kind: string): string {
  if (kind === 'wasm-hash') { return value.trim().toLowerCase() }
  return value.trim()
}

// Canonicalise on every keystroke so the visible field, the length
// counter, and the validator agree at all times. Without this an
// uppercase paste would show "64/64" green next to "must be 64 hex
// characters" red, because the regex only matches lowercase hex.
// Mutate in place — Vue's deep watcher on `args` picks up nested writes
// and reallocating the whole object on every keystroke would force
// every arg field to re-bind, which gets expensive once functions have
// more than one input.
function onArgInput(name: string, value: string, kind: string) {
  args.value[name] = canonicalizeArg(value, kind)
}

// Per-arg client-side validator. Returns null if valid (or empty —
// emptiness is reported separately as "incomplete" vs "invalid").
function validateArg(value: string, kind: string): string | null {
  if (!value) { return null }
  switch (kind) {
    case 'wasm-hash':
      return isWasmHash(value) ? null : 'must be 64 hex characters (SHA-256)'
    default:
      return null
  }
}

// Human-readable labels indexed by their value. The dropdowns are bound to
// plain string arrays (so JSelect writes a string into v-model — see the
// note on its `selectHandler` in JSelect.vue). The `#option` slot reads
// from these maps to render rich row text.
//
// `functionDefsById` is the structured form used by the option slot to
// render two-line rows (displayName + mono id). `functionLabels` is the
// fallback flat string used in the closed-state default slot.
const functionDefsById = computed<Record<string, FunctionDef<any, any>>>(() => {
  const out: Record<string, FunctionDef<any, any>> = {}
  for (const fn of allFunctions.value) { out[fn.id] = fn }
  return out
})
const functionLabels = computed<Record<string, string>>(() => {
  const out: Record<string, string> = {}
  for (const fn of allFunctions.value) { out[fn.id] = fn.displayName }
  return out
})

// Known-options + label maps consumed by <AddressPicker>. The picker
// injects its own MANUAL entry, so these only describe real addresses.
const marketLabels = computed<Record<string, string>>(() => {
  if (!requiresMarket.value) { return {} }
  const out: Record<string, string> = {}
  for (const m of Object.values(marketsStore.state.markets)) {
    if (m) {
      out[m.address] = `${capitalize(m.marketName) || 'Unnamed'} · ${truncateAddress(m.address)}`
    }
  }
  return out
})

const marketKnownOptions = computed<string[]>(() => {
  if (!requiresMarket.value) { return [] }
  return Object.values(marketsStore.state.markets)
    .filter(Boolean)
    .map(m => m.address)
})

const knownMultisigForRole = computed(() => {
  if (!role.value) { return }
  return KNOWN_MULTISIGS[rpcStore.network]?.[role.value]
})

const multisigLabels = computed<Record<string, string>>(() => {
  const known = knownMultisigForRole.value
  if (!known) { return {} }
  return { [known]: `${capitalize(role.value)} multisig · ${truncateAddress(known)}` }
})

const multisigKnownOptions = computed<string[]>(() => {
  const known = knownMultisigForRole.value
  return known ? [known] : []
})

const marketsLoading = computed(() => marketsStore.state.loading)

// On role change (driven by the function picker), pre-select the known
// multisig for that role on this network (if any). Saves the operator a
// click in the common path. `flush: 'post'` keeps the trace tidy when
// other watches also react to the same function change.
watch(role, () => {
  multisigAccountAddress.value = knownMultisigForRole.value ?? ''
}, { immediate: true, flush: 'post' })

// Reset every piece of state that becomes meaningless when the catalog
// function or the network changes — args (schema-bound) and any in-flight
// or already-built proposal. Address pickers are reset by their own
// callers because the rules differ (function change keeps addresses;
// network change scrubs them). The wasm-verifier widget owns its own
// per-instance state and is unmounted by the v-for key change anyway.
function resetFormState() {
  args.value = {}
  buildError.value = null
  invalidateBuild()
}

// On network change, scrub everything that's network-coupled. The SDK
// default for market_manager flips, pasted args (wasm hashes, addresses)
// may have been built for the old network, and any already-built proposal
// has the OLD network passphrase baked into its hash + URL. Letting the
// success card linger across a network switch is how operators ship a
// testnet URL thinking it's mainnet.
watch(() => rpcStore.network, (next, prev) => {
  resetNetworkCoupledFields()
  // Surface the wipe explicitly so an operator who switched mid-paste
  // doesn't think the page crashed. The proposal hash bakes the
  // network passphrase, so keeping form state across networks would
  // produce a hash that no longer matches what the operator typed.
  if (prev) {
    toast.create({
      title: `Switched to ${next === 'public' ? 'Mainnet' : 'Testnet'}`,
      body: 'Form cleared — network is baked into the proposal hash.',
      modelValue: 3500,
    })
  }
})

// Function changes invalidate the form: args belong to the previous
// schema, wasm-verifier state is keyed by previous arg names, and any
// built proposal hashes the old function id.
watch(functionId, resetFormState)

// Any other form edit invalidates the previously-built proposal so the
// operator can never copy a URL that doesn't match the current form.
// `invalidateBuild()` also bumps the build token, which causes any
// in-flight build/simulate RPC call to discard its result on resolve
// rather than overwrite the cleared state.
// The three address sources are computeds-of-strings, so a shallow watch
// is enough; only `args` needs `deep`.
watch(
  [multisigAccountAddress, marketManagerAddress, marketAddress],
  invalidateBuild,
)
watch(args, invalidateBuild, { deep: true })

const fragmentUrl = computed(() => {
  if (!builtProposal.value) { return '' }
  const fragment = encodeProposalToFragment(builtProposal.value)
  if (!import.meta.client) { return `#p=${fragment}` }
  const base = `${globalThis.location.origin}/multisig/sign`
  return `${base}#p=${fragment}`
})

// Per-address validity. Empty fields are intentionally NOT errors here —
// they'll surface as "Multisig address is empty" via `disabledReason`.
// Errors here are *malformed* values, which deserve inline hints.
const addressErrors = computed<Record<'multisig' | 'market' | 'marketManager', string | null>>(() => ({
  multisig: multisigAccountAddress.value && !isStellarAccount(multisigAccountAddress.value)
    ? 'expected G… (56 chars, base32)'
    : null,
  market: requiresMarket.value && marketAddress.value && !isSorobanContract(marketAddress.value)
    ? 'expected C… (56 chars, base32)'
    : null,
  marketManager: requiresMarketManager.value && marketManagerAddress.value && !isSorobanContract(marketManagerAddress.value)
    ? 'expected C… (56 chars, base32)'
    : null,
}))

// Per-argument validity, keyed by argName.
const argErrors = computed<Record<string, string | null>>(() => {
  const fn = selectedFn.value
  if (!fn) { return {} }
  const out: Record<string, string | null> = {}
  for (const [name, field] of Object.entries(fn.argSchema)) {
    out[name] = validateArg(args.value[name] ?? '', (field as { kind: string }).kind)
  }
  return out
})

// Single source of truth for "why can't I build?". Returns the first
// blocking condition or null. The Build button reads this for its tooltip
// and inline status; `canBuild` is a derived boolean.
const disabledReason = computed<string | null>(() => {
  if (!wallet.publicKey) { return 'Connect a wallet to compose' }
  const fn = selectedFn.value
  if (!fn) { return 'Pick a function' }
  if (!multisigAccountAddress.value) { return 'Multisig account address is empty' }
  if (addressErrors.value.multisig) { return `Multisig account: ${addressErrors.value.multisig}` }
  if (requiresMarketManager.value) {
    if (!marketManagerAddress.value) { return 'market_manager address is empty' }
    if (addressErrors.value.marketManager) { return `market_manager: ${addressErrors.value.marketManager}` }
  }
  if (requiresMarket.value) {
    if (!marketAddress.value) { return 'Market address is empty' }
    if (addressErrors.value.market) { return `Market: ${addressErrors.value.market}` }
  }
  for (const argName of Object.keys(fn.argSchema)) {
    if (!args.value[argName]) { return `Argument "${argName}" is empty` }
    const err = argErrors.value[argName]
    if (err) { return `Argument "${argName}": ${err}` }
  }
  return null
})

const canBuild = computed(() => disabledReason.value === null)

// Pre-build review rows. Renders the action + arg values in a single
// scannable card so the operator can sanity-check before broadcasting.
// Intentionally NOT calling `selectedFn.renderSummary` — that needs an
// on-chain snapshot for the before/after diff, which we don't have until
// `buildProposal` runs. This is a simpler "here's what you're about to
// commit to" view.
type PreviewRow = { label: string, value: string, kind?: string, error?: string | null }
const previewRows = computed<PreviewRow[]>(() => {
  const fn = selectedFn.value
  if (!fn) { return [] }
  const rows: PreviewRow[] = [
    // Surface the network at the top of the review — the proposal hash
    // bakes the network passphrase, so a wrong network here ships as a
    // wrong-network proposal. Cheap insurance against the mainnet/
    // testnet substitution class of bug.
    { label: 'Network', value: networkLabel.value },
    { label: 'Action', value: fn.displayName },
    { label: 'Method', value: `${fn.contract}.${fn.function}` },
    { label: 'Multisig role', value: capitalize(fn.multisig) },
    { label: 'Multisig account', value: multisigAccountAddress.value || '—', error: addressErrors.value.multisig },
  ]
  if (requiresMarketManager.value) {
    rows.push({ label: 'market_manager', value: marketManagerAddress.value || '—', error: addressErrors.value.marketManager })
  }
  if (requiresMarket.value) {
    rows.push({ label: 'Market', value: marketAddress.value || '—', error: addressErrors.value.market })
  }
  if (affectsAllMarkets.value) {
    rows.push({ label: 'Scope', value: 'All markets governed by this market_manager' })
  }
  for (const [name, field] of Object.entries(fn.argSchema)) {
    rows.push({
      label: name,
      value: args.value[name] || '—',
      kind: (field as { kind: string }).kind,
      error: argErrors.value[name],
    })
  }
  return rows
})

async function copyText(text: string, body: string) {
  if (!text) { return }
  try {
    await navigator.clipboard.writeText(text)
    toast.create({ title: 'Copied', body, modelValue: 2000 })
  } catch (error) {
    toast.create({
      title: 'Copy failed',
      body: String((error as Error)?.message ?? error),
      variant: 'danger',
      modelValue: 4000,
    })
  }
}

// Programmatic open instead of `<a href target="_blank">` so the button
// renders as a real <button> in the actions row — keeps it baseline-aligned
// with its sibling <button>s under flexbox. `noopener,noreferrer` denies
// the new tab access to `window.opener`.
function openInNewTab() {
  if (!fragmentUrl.value) { return }
  globalThis.open(fragmentUrl.value, '_blank', 'noopener,noreferrer')
}

async function build() {
  buildError.value = null
  invalidateBuild()
  const fn = selectedFn.value
  const composer = wallet.publicKey
  if (!fn || !composer) { buildError.value = 'Form is incomplete'; return }

  building.value = true
  // Snapshot the token AFTER `invalidateBuild()` so any subsequent
  // form-edit watcher will produce a different value and we can detect
  // that our resolution is stale.
  const myToken = buildToken.value
  const isStale = () => buildToken.value !== myToken
  try {
    const rpcUrl = rpcStore.sorobanRPCUrl
    if (!rpcUrl) { buildError.value = `No Soroban RPC configured for ${networkLabel.value}`; return }

    // Canonicalise args (lowercase hex hashes, trim whitespace) before
    // hashing so the proposal hash is deterministic regardless of how the
    // composer formatted their paste. Mirror this back into the form so
    // the success card shows what was actually committed.
    // NB: writing back to `args.value` would normally bump the build
    // token via the deep watcher and abort our own build. Suppress that
    // by re-snapshotting the token after the mirror-back.
    const canonical: Record<string, string> = {}
    for (const [name, field] of Object.entries(fn.argSchema)) {
      canonical[name] = canonicalizeArgForBuild(args.value[name] ?? '', (field as { kind: string }).kind)
    }
    args.value = canonical
    await nextTick()
    const tokenAfterMirror = buildToken.value

    const payload = await buildProposal({
      fn,
      args: { ...canonical },
      multisigAccountAddress: multisigAccountAddress.value,
      env: {
        rpcUrl,
        networkPassphrase: networkPassphrase.value,
        addresses: {
          market_manager: marketManagerAddress.value,
          market: marketAddress.value,
        },
      },
      composerAddress: composer,
    })
    if (buildToken.value !== tokenAfterMirror) { return }
    builtProposal.value = payload

    // Simulate the freshly-built envelope. Failures here usually mean
    // the multisig isn't the manager's admin (auth check), the contract
    // is in an incompatible state (e.g. another upgrade already queued),
    // or args decode to something the contract rejects. Surfacing these
    // at compose time saves a full sign/aggregate round trip.
    const sim = await runSimulate(payload.unsigned_xdr)
    if (buildToken.value !== tokenAfterMirror) { return }
    simulateResult.value = sim
    simulatedAt.value = Date.now()
  } catch (error) {
    if (isStale()) { return }
    buildError.value = (error as Error).message
  } finally {
    building.value = false
  }
}

// Single-purpose simulate runner shared by `build()` and the success
// card's "Re-simulate" button. Owns the rpcUrl lookup and the
// missing-RPC error path so neither caller has to special-case it. The
// envelope itself is not rebuilt — that would stamp a fresh
// `created_at` and break the share URL signers may have already seen.
async function runSimulate(unsignedXdr: string): Promise<SimulateResult> {
  const rpcUrl = rpcStore.sorobanRPCUrl
  if (!rpcUrl) {
    return { ok: false, error: `No Soroban RPC configured for ${networkLabel.value}` }
  }
  return simulateProposalEnvelope(rpcUrl, networkPassphrase.value, unsignedXdr)
}

async function resimulate() {
  if (!builtProposal.value) { return }
  resimulating.value = true
  // Capture the token of the envelope we're about to simulate. If the
  // user edits the form mid-flight (which clears builtProposal and bumps
  // the token), discard our result rather than stamp it onto whatever
  // envelope the form holds now.
  const myToken = buildToken.value
  try {
    const sim = await runSimulate(builtProposal.value.unsigned_xdr)
    if (buildToken.value !== myToken) { return }
    simulateResult.value = sim
    simulatedAt.value = Date.now()
  } finally {
    resimulating.value = false
  }
}

// Reset everything that's network-coupled or operator-private:
// override toggles, manual buffers, the chosen multisig (snapped back
// to the role default if known), and the form/build state. Shared
// between the network watcher and the explicit "Start over" button so
// the two paths can't diverge.
function resetNetworkCoupledFields() {
  marketManagerOverride.value = false
  marketManagerManual.value = ''
  marketAddress.value = ''
  multisigAccountAddress.value = knownMultisigForRole.value ?? ''
  resetFormState()
}

// Human-readable "Xs ago" for the simulation result.
const simulatedAgo = computed(() => {
  if (!simulatedAt.value) { return '' }
  const seconds = Math.max(0, Math.floor((now.value - simulatedAt.value) / 1000))
  if (seconds < 5) { return 'just now' }
  if (seconds < 60) { return `${seconds}s ago` }
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) { return `${minutes}m ago` }
  const hours = Math.floor(minutes / 60)
  return `${hours}h ago`
})

// 1 Hz ticker only runs client-side and only stays mounted with the
// page. Cheap enough not to bother gating on `simulatedAt` — the
// computed bails out instantly when nothing's stamped.
let nowTicker: ReturnType<typeof setInterval> | null = null
onMounted(() => {
  nowTicker = setInterval(() => { now.value = Date.now() }, 1000)
})
onUnmounted(() => {
  if (nowTicker) { clearInterval(nowTicker) }
})
</script>

<template>
  <main
    class="multisig-compose-page container"
  >
    <header class="multisig-compose-page__hero">
      <span class="multisig-compose-page__eyebrow">Multisig</span>
      <h1 class="multisig-compose-page__title">
        Compose proposal
      </h1>
      <p class="multisig-compose-page__lead">
        Build an unsigned multisig proposal. Share the resulting URL with co-signers; they sign in
        their browser, signatures are aggregated, and the operator submits when the threshold is met.
      </p>
    </header>

    <section class="multisig-section">
      <header class="multisig-section__header">
        <h2 class="multisig-section__title">
          Target
        </h2>
        <p class="multisig-section__subtitle">
          Pick the catalog function this proposal should authorize. The required multisig role
          is inferred from your selection.
        </p>
      </header>

      <div class="multisig-card">
        <div class="multisig-field">
          <label class="multisig-field__label">
            Action
            <span
              v-if="role"
              class="multisig-field__role-chip"
              :class="`multisig-field__role-chip--${role}`"
              :title="`Signers from the ${capitalize(role)} role must approve this proposal`"
            >
              Role: {{ capitalize(role) }}
            </span>
          </label>
          <j-select
            :model-value="functionId ?? undefined"
            :options="functionOptions"
            :unselected="false"
            label="— select an action —"
            @update:model-value="(v) => functionId = (v as string) ?? null"
          >
            <template #default>
              {{ functionId ? (functionLabels[functionId] ?? '— select an action —') : '— select an action —' }}
            </template>
            <template #option="{ option }">
              <span
                v-if="functionDefsById[option as string]"
                class="fn-option"
              >
                <span class="fn-option__name">{{ functionDefsById[option as string]?.displayName }}</span>
                <span class="fn-option__id">{{ functionDefsById[option as string]?.id }}</span>
              </span>
              <template v-else>
                {{ option }}
              </template>
            </template>
          </j-select>
          <p
            v-if="selectedFn"
            class="multisig-field__hint multisig-field__hint--lead"
          >
            {{ selectedFn.description }}
          </p>
        </div>
      </div>
    </section>

    <section class="multisig-section">
      <header class="multisig-section__header">
        <h2 class="multisig-section__title">
          Addresses
        </h2>
        <p class="multisig-section__subtitle">
          The signing account is the multisig itself. The other addresses depend on which contract
          the action targets — most upgrade actions go straight to the <code>market_manager</code>
          and never reference an individual market.
        </p>
      </header>

      <div class="multisig-card">
        <div class="multisig-field">
          <label class="multisig-field__label">Multisig account</label>
          <address-picker
            v-model="multisigAccountAddress"
            :known-options="multisigKnownOptions"
            :labels="multisigLabels"
            select-label="Pick a multisig account"
            manual-placeholder="GABC… (56-char Stellar account)"
            :error="addressErrors.multisig"
          />
          <p
            v-if="!knownMultisigForRole && !multisigAccountAddress"
            class="multisig-field__hint"
          >
            No known multisig for <strong>{{ capitalize(role) }}</strong> on {{ networkLabel }} —
            paste the account G… above.
          </p>
        </div>

        <div
          v-if="requiresMarketManager"
          class="multisig-field multisig-field--inline"
        >
          <span class="multisig-field__label-row">
            <span class="multisig-field__label">market_manager</span>
            <button
              type="button"
              class="multisig-field__toggle"
              @click="marketManagerOverride = !marketManagerOverride"
            >
              {{ marketManagerOverride ? 'Use default' : 'Override' }}
            </button>
          </span>
          <button
            v-if="!marketManagerOverride && defaultMarketManager"
            type="button"
            class="multisig-field__readonly multisig-field__readonly--copyable"
            :title="`Click to copy · ${defaultMarketManager}`"
            @click="copyText(defaultMarketManager, 'market_manager address copied')"
          >
            <code>{{ truncateAddress(defaultMarketManager, 8, 8) }}</code>
            <span class="multisig-field__readonly-meta">SDK default for {{ networkLabel }} · click to copy</span>
          </button>
          <p
            v-else-if="!marketManagerOverride && !defaultMarketManager"
            class="multisig-field__hint"
          >
            No SDK default for {{ networkLabel }} — enter the market_manager contract below.
          </p>
          <j-input
            v-if="marketManagerOverride || !defaultMarketManager"
            v-model="marketManagerManual"
            placeholder="CABC… (56-char Soroban contract)"
          />
          <p
            v-if="addressErrors.marketManager"
            class="multisig-field__hint multisig-field__hint--err"
          >
            {{ addressErrors.marketManager }}
          </p>
        </div>

        <div
          v-if="requiresMarket"
          class="multisig-field"
        >
          <label class="multisig-field__label">
            Market
            <span
              v-if="marketsLoading"
              class="multisig-field__kind"
            >loading…</span>
          </label>
          <p
            v-if="marketsLoading"
            class="multisig-field__readonly"
          >
            <code>Loading markets…</code>
            <span class="multisig-field__readonly-meta">picker enables once loaded</span>
          </p>
          <address-picker
            v-else
            v-model="marketAddress"
            :known-options="marketKnownOptions"
            :labels="marketLabels"
            select-label="Pick a market"
            manual-placeholder="CABC… (56-char Soroban contract)"
            :error="addressErrors.market"
          />
        </div>
      </div>
    </section>

    <section
      v-if="selectedFn"
      class="multisig-section"
    >
      <header class="multisig-section__header">
        <h2 class="multisig-section__title">
          Arguments
        </h2>
        <p class="multisig-section__subtitle">
          Provide the values this action will be hashed and signed against.
          Inputs are validated client-side; the contract validates again at simulate time.
        </p>
      </header>

      <div class="multisig-card">
        <div
          v-for="(field, name) in selectedFn.argSchema"
          :key="name"
          class="multisig-field"
        >
          <label class="multisig-field__label">
            {{ name }}
            <span class="multisig-field__kind">{{ field.kind }}</span>
          </label>
          <j-input
            class="multisig-field__mono-input"
            :model-value="args[name] ?? ''"
            :placeholder="field.kind === 'wasm-hash' ? '64 hex characters' : ''"
            @update:model-value="(v) => onArgInput(String(name), String(v ?? ''), field.kind)"
          />
          <p
            v-if="field.kind === 'wasm-hash' && args[name]"
            class="multisig-field__hint"
            :class="{ 'multisig-field__hint--err': argErrors[name] }"
          >
            {{ args[name].length }} / 64 hex chars{{ argErrors[name] ? ` — ${argErrors[name]}` : '' }}
          </p>
          <p
            v-else-if="argErrors[name]"
            class="multisig-field__hint multisig-field__hint--err"
          >
            {{ argErrors[name] }}
          </p>

          <wasm-hash-verifier
            v-if="field.kind === 'wasm-hash'"
            :claimed-hash="args[name] ?? ''"
          />
        </div>
      </div>
    </section>

    <section
      v-if="selectedFn"
      class="multisig-section"
    >
      <header class="multisig-section__header">
        <h2 class="multisig-section__title">
          Review &amp; build
        </h2>
        <p class="multisig-section__subtitle">
          Final check before broadcasting. The summary below is what signers will see in their
          decoded view; the proposal hash is computed from this exact set of values plus the
          network passphrase.
        </p>
      </header>

      <div class="multisig-card">
        <div
          v-if="disabledReason"
          class="multisig-banner multisig-banner--warn"
        >
          <span class="multisig-banner__title">Not ready to build</span>
          <span class="multisig-banner__body">{{ disabledReason }}</span>
        </div>

        <div class="preview-card">
          <header class="preview-card__header">
            <span class="preview-card__title">Pre-build review</span>
            <span class="preview-card__sub">
              This is what signers will see. Composer:
              <code>{{ wallet.publicKey ? truncateAddress(wallet.publicKey, 6, 6) : '— not connected' }}</code>
            </span>
          </header>
          <ul class="preview-card__rows">
            <li
              v-for="row in previewRows"
              :key="row.label"
              class="preview-row"
              :class="{ 'preview-row--err': row.error }"
            >
              <span class="preview-row__label">{{ row.label }}</span>
              <span class="preview-row__value">
                <code>{{ row.value }}</code>
                <span
                  v-if="row.kind"
                  class="preview-row__kind"
                >{{ row.kind }}</span>
                <span
                  v-if="row.error"
                  class="preview-row__err"
                >{{ row.error }}</span>
              </span>
            </li>
          </ul>
        </div>

        <div class="multisig-actions">
          <j-btn
            variant="primary"
            :loading="building"
            :disabled="!canBuild || building"
            :title="disabledReason ?? undefined"
            @click="build"
          >
            {{ building ? 'Building…' : 'Build proposal' }}
          </j-btn>
        </div>

        <div
          v-if="buildError"
          class="multisig-banner multisig-banner--err"
        >
          <span class="multisig-banner__title">Build failed</span>
          <span class="multisig-banner__body">{{ buildError }}</span>
        </div>
      </div>
    </section>

    <section
      v-if="builtProposal"
      class="multisig-section"
    >
      <header class="multisig-section__header">
        <h2 class="multisig-section__title">
          Share with signers
        </h2>
        <p class="multisig-section__subtitle">
          Send this URL to the <strong>{{ capitalize(role) }}</strong> signers. Each one signs in
          their own browser; you submit once the threshold is met.
        </p>
      </header>

      <div class="multisig-card">
        <div class="kv">
          <span class="kv__k">Hash</span>
          <button
            type="button"
            class="kv__v kv__v--copyable"
            :title="`Click to copy · ${builtProposal.proposal_hash}`"
            @click="copyText(builtProposal.proposal_hash, 'Proposal hash copied')"
          >
            {{ builtProposal.proposal_hash }}
          </button>
        </div>
        <div
          v-if="simulateResult"
          class="multisig-banner"
          :class="simulateResult.ok
            ? 'multisig-banner--ok'
            : 'multisig-banner--err'"
        >
          <span class="multisig-banner__title">
            {{ simulateResult.ok ? 'Simulation passed' : 'Simulation failed' }}
            <span
              v-if="simulatedAgo"
              class="multisig-banner__age"
              :title="`Last simulated at ${new Date(simulatedAt ?? 0).toLocaleTimeString()}`"
            >· {{ simulatedAgo }}</span>
          </span>
          <span class="multisig-banner__body">
            <template v-if="simulateResult.ok">
              The call would execute against current chain state. That snapshot expires when the
              next ledger closes, so re-simulate just before submitting in case the chain has moved
              (another proposal queued, admin rotated, target market closed).
            </template>
            <template v-else>
              {{ simulateResult.error }}
              <br>
              <em>Common causes:</em> the chosen multisig isn't the manager's admin; another upgrade is already queued; argument values are out-of-range. Sharing the URL is still possible but the tx will likely fail at submit.
            </template>
          </span>
          <div class="multisig-banner__actions">
            <j-btn
              size="sm"
              variant="ghost"
              :loading="resimulating"
              :disabled="resimulating"
              @click="resimulate"
            >
              {{ resimulating ? 'Re-simulating…' : 'Re-simulate' }}
            </j-btn>
          </div>
        </div>

        <textarea
          class="proposal-share__url"
          readonly
          rows="3"
          :value="fragmentUrl"
        />
        <p class="proposal-share__note">
          The full proposal travels inside the URL fragment (after the <code>#</code>) — fragments
          are never sent to the relay or any server. Safe to paste into chat.
        </p>

        <div class="multisig-actions">
          <j-btn
            :variant="simulateResult && !simulateResult.ok ? 'outlined-brand' : 'primary'"
            :title="simulateResult && !simulateResult.ok
              ? 'Simulation failed — copying anyway. Verify before sharing with signers.'
              : 'Copy the share URL to clipboard'"
            @click="copyText(fragmentUrl, simulateResult && !simulateResult.ok
              ? 'Share URL copied — note: simulation failed'
              : 'Share URL copied to clipboard')"
          >
            {{ simulateResult && !simulateResult.ok ? 'Copy URL anyway' : 'Copy URL' }}
          </j-btn>
          <j-btn
            variant="ghost"
            title="Verify the URL decodes correctly before sharing"
            @click="openInNewTab"
          >
            Open in new tab
          </j-btn>
        </div>
      </div>
    </section>
  </main>
</template>

<style lang="scss">
.multisig-compose-page {
  padding: 32px 16px 64px;
  display: flex;
  flex-direction: column;
  gap: 32px;

  &__hero {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-bottom: 24px;
    border-bottom: 1px solid $border-primary;
  }

  &__eyebrow {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: $cyan;
    text-transform: uppercase;
  }

  &__title {
    font-size: 32px;
    font-weight: 700;
    color: $navi-25;
    margin: 0;
    line-height: 1.15;

    @media (max-width: 640px) {
      font-size: 26px;
    }
  }

  &__lead {
    font-size: 14px;
    color: $text-secondary;
    line-height: 1.55;
    max-width: 640px;
    margin: 0;
  }
}

.multisig-section {
  display: flex;
  flex-direction: column;
  gap: 16px;

  &__header {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  &__title {
    font-size: 18px;
    font-weight: 700;
    color: $text-primary;
    margin: 0;
  }

  &__subtitle {
    font-size: 12px;
    color: $text-tertiary;
    margin: 0;
    line-height: 1.5;
    max-width: 640px;

    code {
      font-family: $font-JetBrainsMono;
      color: $text-primary;
    }
  }
}

// Single-column flex stack — every section card here is read top-down.
// A 2-col grid was tried earlier but no section's content paired
// cleanly across columns (the warning row, market_manager toggle, and
// preview-card all want full width), and the result was awkward
// half-cards. Kept simple here; revisit if a future card has truly
// independent left/right content.
.multisig-card {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: $radius-lg;
}

.multisig-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;

  &__label {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: $text-tertiary;
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  &__label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  &__kind {
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.04em;
    color: $text-tertiary;
    padding: 1px 6px;
    border: 1px solid $border-secondary;
    border-radius: 999px;
    text-transform: lowercase;
  }

  &__role-chip {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    padding: 2px 8px;
    border: 1px solid $border-secondary;
    border-radius: 999px;
    color: $text-tertiary;
    background-color: color-mix(in oklab, $navi-700 40%, transparent);

    &--ops {
      color: $cyan;
      border-color: color-mix(in oklab, $cyan 35%, $border-secondary);
      background-color: color-mix(in oklab, $cyan 12%, transparent);
    }
    &--program {
      color: $warning;
      border-color: color-mix(in oklab, $warning 40%, $border-secondary);
      background-color: color-mix(in oklab, $warning 12%, transparent);
    }
    &--upgrade {
      color: $indigo;
      border-color: color-mix(in oklab, $indigo 45%, $border-secondary);
      background-color: color-mix(in oklab, $indigo 14%, transparent);
    }
  }

  &__hint {
    font-size: 11px;
    color: $text-tertiary;
    margin: 0;
    line-height: 1.5;

    strong {
      color: $text-secondary;
    }

    &--err {
      color: $danger;
    }

    &--warn {
      color: $warning;
      padding: 8px 12px;
      background-color: color-mix(in oklab, $warning 10%, transparent);
      border-left: 2px solid color-mix(in oklab, $warning 60%, $border-secondary);
      border-radius: 0 $radius-md $radius-md 0;

      strong {
        color: $warning;
        font-weight: 700;
      }
    }

    &--lead {
      font-size: 12px;
      color: $text-secondary;
      line-height: 1.55;
      padding: 8px 12px;
      background-color: color-mix(in oklab, $navi-700 50%, transparent);
      border-left: 2px solid color-mix(in oklab, $cyan 50%, $border-secondary);
      border-radius: 0 $radius-md $radius-md 0;
    }
  }

  &__readonly {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 12px;
    background-color: color-mix(in oklab, $navi-700 60%, transparent);
    border: 1px solid $border-secondary;
    border-radius: $radius-md;
    margin: 0;
    text-align: left;

    code {
      font-family: $font-JetBrainsMono;
      font-size: 12px;
      color: $text-primary;
    }

    &--copyable {
      width: 100%;
      cursor: pointer;
      transition:
        border-color 0.12s ease,
        color 0.12s ease;

      &:hover {
        border-color: color-mix(in oklab, $cyan 35%, $border-secondary);
        code {
          color: $cyan;
        }
      }
    }
  }

  // Apply monospace to wasm-hash and other arg inputs. Targeted via
  // class on the <j-input> wrapper so it doesn't bleed into pickers
  // that already get mono via AddressPicker's own scoped rule.
  &__mono-input :deep(input) {
    font-family: $font-JetBrainsMono;
    font-size: 12px;
    letter-spacing: 0.01em;
  }

  &__readonly-meta {
    font-size: 10px;
    color: $text-tertiary;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  &__toggle {
    background: none;
    border: 1px solid $border-secondary;
    border-radius: 999px;
    padding: 1px 8px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: $text-tertiary;
    cursor: pointer;
    transition:
      color 0.12s ease,
      border-color 0.12s ease;

    &:hover {
      color: $cyan;
      border-color: color-mix(in oklab, $cyan 35%, $border-secondary);
    }
  }
}

.fn-option {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;

  &__name {
    font-size: 13px;
    font-weight: 600;
    color: $text-primary;
    line-height: 1.3;
  }

  &__id {
    font-size: 10px;
    color: $text-tertiary;
    font-family: $font-JetBrainsMono;
    line-height: 1.3;
    word-break: break-all;
  }
}

.multisig-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.preview-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px 16px;
  background-color: color-mix(in oklab, $navi-700 50%, transparent);
  border: 1px solid $border-secondary;
  border-left: 3px solid $cyan;
  border-radius: $radius-md;

  &__header {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  &__title {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: $cyan;
  }

  &__sub {
    font-size: 11px;
    color: $text-tertiary;

    code {
      font-family: $font-JetBrainsMono;
      color: $text-secondary;
    }
  }

  &__rows {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
}

.preview-row {
  display: grid;
  grid-template-columns: minmax(120px, 160px) 1fr;
  gap: 12px;
  font-size: 12px;
  align-items: baseline;

  &__label {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: $text-tertiary;
  }

  &__value {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    min-width: 0;

    code {
      font-family: $font-JetBrainsMono;
      color: $text-primary;
      word-break: break-all;
    }
  }

  &__kind {
    font-size: 9px;
    font-weight: 500;
    letter-spacing: 0.04em;
    color: $text-tertiary;
    padding: 1px 6px;
    border: 1px solid $border-secondary;
    border-radius: 999px;
    text-transform: lowercase;
  }

  &__err {
    font-size: 11px;
    color: $danger;
    font-style: italic;
  }

  &--err {
    .preview-row__value code {
      color: $danger;
    }
  }
}

.multisig-banner {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 14px;
  border: 1px solid $border-secondary;
  border-radius: $radius-md;
  background-color: color-mix(in oklab, $navi-700 70%, transparent);

  &__title {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  &__age {
    margin-left: 6px;
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.04em;
    color: $text-tertiary;
    text-transform: none;
  }

  &__body {
    font-size: 12px;
    color: $text-secondary;
    word-break: break-word;
  }

  &--err {
    border-color: color-mix(in oklab, $danger 45%, $border-secondary);
    .multisig-banner__title {
      color: $danger;
    }
  }

  &--ok {
    border-color: color-mix(in oklab, $success 40%, $border-secondary);
    .multisig-banner__title {
      color: $success;
    }
  }

  &--warn {
    border-color: color-mix(in oklab, $warning 45%, $border-secondary);
    background-color: color-mix(in oklab, $warning 8%, transparent);
    .multisig-banner__title {
      color: $warning;
    }
  }

  &__actions {
    display: flex;
    gap: 8px;
    margin-top: 6px;
  }
}

.proposal-share__url {
  width: 100%;
  padding: 10px 12px;
  background-color: color-mix(in oklab, $navi-700 70%, transparent);
  border: 1px solid $border-secondary;
  border-radius: $radius-md;
  font-family: $font-JetBrainsMono;
  font-size: 12px;
  color: $text-primary;
  word-break: break-all;
  resize: vertical;
  cursor: default;
}

.proposal-share__note {
  margin: 0;
  font-size: 11px;
  color: $text-tertiary;
  line-height: 1.5;

  code {
    font-family: $font-JetBrainsMono;
    font-size: 10px;
    padding: 1px 4px;
    border-radius: $radius-sm;
    background-color: color-mix(in oklab, $navi-700 60%, transparent);
    color: $text-secondary;
  }
}

.kv {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 12px;

  &__k {
    color: $text-tertiary;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-weight: 600;
    font-size: 10px;
    flex-shrink: 0;
  }

  &__v {
    color: $text-primary;
    font-family: $font-JetBrainsMono;
    word-break: break-all;
    text-align: right;
    min-width: 0;

    &--copyable {
      background: none;
      border: none;
      padding: 0;
      cursor: pointer;
      transition: color 0.12s ease;

      &:hover {
        color: $cyan;
      }
    }
  }
}
</style>
