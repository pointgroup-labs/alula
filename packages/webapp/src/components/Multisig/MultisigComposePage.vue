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

import type { FunctionDef, MultisigRole, ProposalPayload } from '~/utils/multisig'
import { CONTRACT_ID } from '@alula/client-sdk'
import { Networks } from '@stellar/stellar-sdk'
import { KNOWN_MULTISIGS } from '~/config'
import {
  buildProposal,
  encodeProposalToFragment,
  listAllFunctions,
  verifyWasmFile,
} from '~/utils/multisig'

// Sentinel value used by the address pickers to switch the field into
// manual-entry mode. Any string that can't collide with a real Stellar
// address works; the underscore prefix keeps it distinct from G…/C… IDs.
const MANUAL = '__manual__'

const wallet = useWallet()
const rpcStore = useRpcStore()
const marketsStore = useMarketsStore()
const toast = useToast()

const functionId = ref<string>('')
const args = ref<Record<string, string>>({})

// Market picker: dropdown selection (an address or MANUAL) plus the
// manual-entry buffer. The effective marketAddress is computed from these.
const marketSelection = ref<string>(MANUAL)
const marketManualAddress = ref<string>('')

// Multisig picker: same shape as the market picker. Defaults to MANUAL
// because there are no known multisigs yet — the watch on `role` snaps
// it back to the known address as soon as the config is populated.
const multisigSelection = ref<string>(MANUAL)
const multisigManualAddress = ref<string>('')

// Market-manager picker: defaults to the SDK constant for the current
// network and only opens an override input when the operator asks.
const marketManagerOverride = ref(false)
const marketManagerManual = ref<string>('')

// Per-arg state for the wasm-hash verifier. Page-level singletons would
// collide if a function ever exposed more than one wasm-hash arg.
const wasmFileNotes = ref<Record<string, string>>({})
const wasmFileErrors = ref<Record<string, string>>({})
const wasmFileNames = ref<Record<string, string>>({})

const building = ref(false)
const buildError = ref<string | null>(null)
const builtProposal = ref<ProposalPayload | null>(null)

// All catalog functions, flattened across roles, sorted for the dropdown:
// role (alpha) → stage (queue → apply → cancel → other) → displayName.
// Operators pick the action they want; `role` falls out of that choice.
const STAGE_ORDER: Record<string, number> = { queue: 0, apply: 1, cancel: 2 }
function stageRank(fn: FunctionDef<any, any>): number {
  const prefix = fn.function.split('_')[0]
  return STAGE_ORDER[prefix] ?? 99
}
const allFunctions = computed<FunctionDef<any, any>[]>(() =>
  [...listAllFunctions()].sort((a, b) =>
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

// Effective market & multisig addresses, derived from the picker state.
// Treating these as computeds means downstream consumers (build, watches,
// canBuild) don't need to care which input mode produced the value.
const marketAddress = computed(() =>
  marketSelection.value === MANUAL ? marketManualAddress.value : marketSelection.value,
)

const multisigAccountAddress = computed(() =>
  multisigSelection.value === MANUAL ? multisigManualAddress.value : multisigSelection.value,
)

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
// Soroban wasm hashes are 32-byte SHA-256 → 64 hex characters.
const STELLAR_ACCOUNT_RE = /^G[A-Z2-7]{55}$/
const SOROBAN_CONTRACT_RE = /^C[A-Z2-7]{55}$/
const WASM_HASH_RE = /^[0-9a-fA-F]{64}$/

function isStellarAccount(s: string): boolean { return STELLAR_ACCOUNT_RE.test(s) }
function isSorobanContract(s: string): boolean { return SOROBAN_CONTRACT_RE.test(s) }
function isWasmHash(s: string): boolean { return WASM_HASH_RE.test(s) }

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

const marketLabels = computed<Record<string, string>>(() => {
  const out: Record<string, string> = { [MANUAL]: 'Enter address manually…' }
  for (const m of Object.values(marketsStore.state.markets)) {
    if (m) {
      out[m.address] = `${capitalize(m.marketName) || 'Unnamed'} · ${truncateAddress(m.address)}`
    }
  }
  return out
})

const marketOptions = computed<string[]>(() => {
  const addrs = Object.values(marketsStore.state.markets)
    .filter(Boolean)
    .map(m => m.address)
  return [...addrs, MANUAL]
})

const knownMultisigForRole = computed(() => {
  if (!role.value) { return undefined }
  return KNOWN_MULTISIGS[rpcStore.network]?.[role.value]
})

const multisigLabels = computed<Record<string, string>>(() => {
  const out: Record<string, string> = { [MANUAL]: 'Enter address manually…' }
  const known = knownMultisigForRole.value
  if (known) {
    out[known] = `${capitalize(role.value)} multisig · ${truncateAddress(known)}`
  }
  return out
})

const multisigOptions = computed<string[]>(() => {
  const known = knownMultisigForRole.value
  return known ? [known, MANUAL] : [MANUAL]
})

const marketsLoading = computed(() => marketsStore.state.loading)

// On role change (driven by the function picker), pre-select the known
// multisig for that role on this network (if any). Saves the operator a
// click in the common path. `flush: 'post'` keeps the trace tidy when
// other watches also react to the same function change.
watch(role, () => {
  multisigSelection.value = knownMultisigForRole.value ?? MANUAL
}, { immediate: true, flush: 'post' })

// On network change, force the market-manager override input back to the
// SDK default; otherwise switching networks could silently keep an
// override that points at a different network's contract.
watch(() => rpcStore.network, () => {
  marketManagerOverride.value = false
  marketManagerManual.value = ''
  // Reset market picker too — selected market may not exist on the new network.
  marketSelection.value = MANUAL
  marketManualAddress.value = ''
  // And re-pick multisig from the new network's known list.
  multisigSelection.value = knownMultisigForRole.value ?? MANUAL
})

// Function changes nuke args + per-arg wasm state.
watch(functionId, () => {
  args.value = {}
  wasmFileNotes.value = {}
  wasmFileErrors.value = {}
  wasmFileNames.value = {}
  builtProposal.value = null
})

// Any other form edit invalidates the previously-built proposal so the
// operator can never copy a URL that doesn't match the current form.
watch(
  [
    multisigAccountAddress,
    marketManagerAddress,
    marketAddress,
    args,
  ],
  () => { builtProposal.value = null },
  { deep: true },
)

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
  market: marketAddress.value && !isSorobanContract(marketAddress.value)
    ? 'expected C… (56 chars, base32)'
    : null,
  marketManager: marketManagerAddress.value && !isSorobanContract(marketManagerAddress.value)
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
  if (!marketManagerAddress.value) { return 'market_manager address is empty' }
  if (addressErrors.value.marketManager) { return `market_manager: ${addressErrors.value.marketManager}` }
  if (!marketAddress.value) { return 'Market address is empty' }
  if (addressErrors.value.market) { return `Market: ${addressErrors.value.market}` }
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
    { label: 'Action', value: fn.displayName },
    { label: 'Method', value: `${fn.contract}.${fn.function}` },
    { label: 'Multisig role', value: capitalize(fn.multisig) },
    { label: 'Multisig account', value: multisigAccountAddress.value || '—', error: addressErrors.value.multisig },
    { label: 'Market', value: marketAddress.value || '—', error: addressErrors.value.market },
    { label: 'market_manager', value: marketManagerAddress.value || '—', error: addressErrors.value.marketManager },
  ]
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
  }
  catch (error) {
    toast.create({
      title: 'Copy failed',
      body: String((error as Error)?.message ?? error),
      variant: 'danger',
      modelValue: 4000,
    })
  }
}

async function onWasmFileChange(event: Event, argName: string) {
  wasmFileErrors.value[argName] = ''
  wasmFileNotes.value[argName] = ''
  wasmFileNames.value[argName] = ''

  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  // Always reset the input so re-picking the same file refires `change`.
  input.value = ''
  if (!file) { return }

  const claimed = args.value[argName] ?? ''
  if (!claimed) {
    wasmFileErrors.value[argName] = 'Paste the claimed wasm hash above first, then upload to verify.'
    return
  }

  const result = await verifyWasmFile(file, claimed)
  wasmFileNames.value[argName] = `${file.name} · ${result.byte_size} bytes`
  if (result.matches) {
    wasmFileNotes.value[argName] = 'file SHA-256 matches the claimed hash'
  }
  else {
    wasmFileErrors.value[argName] = `hash mismatch — file SHA-256 = ${result.computed_hash}`
  }
}

async function build() {
  buildError.value = null
  builtProposal.value = null
  const fn = selectedFn.value
  const composer = wallet.publicKey
  if (!fn || !composer) { buildError.value = 'Form is incomplete'; return }

  building.value = true
  try {
    const payload = await buildProposal({
      fn,
      args: { ...args.value },
      multisigAccountAddress: multisigAccountAddress.value,
      env: {
        rpcUrl: rpcStore.sorobanRPCUrl,
        networkPassphrase: networkPassphrase.value,
        addresses: {
          market_manager: marketManagerAddress.value,
          market: marketAddress.value,
        },
      },
      composerAddress: composer,
    })
    builtProposal.value = payload
  }
  catch (error) {
    buildError.value = (error as Error).message
  }
  finally {
    building.value = false
  }
}

function startOver() {
  builtProposal.value = null
  args.value = {}
  marketSelection.value = MANUAL
  marketManualAddress.value = ''
  multisigSelection.value = knownMultisigForRole.value ?? MANUAL
  multisigManualAddress.value = ''
  marketManagerOverride.value = false
  marketManagerManual.value = ''
  wasmFileNotes.value = {}
  wasmFileErrors.value = {}
  wasmFileNames.value = {}
  buildError.value = null
}
</script>

<template>
  <main class="multisig-compose-page container">
    <header class="multisig-compose-page__hero">
      <div class="multisig-compose-page__eyebrow-row">
        <span class="multisig-compose-page__eyebrow">Multisig</span>
        <span
          class="multisig-compose-page__network"
          :class="`multisig-compose-page__network--${rpcStore.network === 'public' ? 'mainnet' : 'testnet'}`"
          :title="`Proposal will be built for ${networkLabel}`"
        >
          <span class="multisig-compose-page__network-dot" />
          {{ networkLabel }}
        </span>
      </div>
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

      <div class="multisig-card multisig-card--stack">
        <div class="multisig-field">
          <label class="multisig-field__label">
            Action
            <span
              v-if="role"
              class="multisig-field__role-chip"
              :class="`multisig-field__role-chip--${role}`"
            >
              {{ capitalize(role) }} multisig
            </span>
          </label>
          <j-select
            v-model="functionId"
            :options="functionOptions"
            :unselected="false"
            label="— select an action —"
          >
            <template #default>
              {{ functionLabels[functionId] ?? '— select an action —' }}
            </template>
            <template #option="{ option }">
              <span
                v-if="functionDefsById[option as string]"
                class="fn-option"
              >
                <span class="fn-option__name">{{ functionDefsById[option as string].displayName }}</span>
                <span class="fn-option__id">{{ functionDefsById[option as string].id }}</span>
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
          Pick from existing on-chain entities, or enter addresses manually.
        </p>
      </header>

      <div class="multisig-card multisig-card--stack">
        <div class="multisig-field">
          <label class="multisig-field__label">Multisig account</label>
          <j-select
            v-model="multisigSelection"
            :options="multisigOptions"
            :unselected="false"
            label="Pick a multisig account"
          >
            <template #default>
              {{ multisigLabels[multisigSelection] ?? 'Pick a multisig account' }}
            </template>
            <template #option="{ option }">
              {{ multisigLabels[option as string] ?? option }}
            </template>
          </j-select>
          <p
            v-if="!knownMultisigForRole && multisigSelection === MANUAL"
            class="multisig-field__hint"
          >
            No known multisig for <strong>{{ capitalize(role) }}</strong> on {{ networkLabel }} —
            paste the account G… below.
          </p>
          <j-input
            v-if="multisigSelection === MANUAL"
            v-model="multisigManualAddress"
            placeholder="GABC… (56-char Stellar account)"
          />
          <p
            v-if="addressErrors.multisig"
            class="multisig-field__hint multisig-field__hint--err"
          >
            {{ addressErrors.multisig }}
          </p>
        </div>

        <div class="multisig-field">
          <label class="multisig-field__label">
            Market
            <span
              v-if="marketsLoading"
              class="multisig-field__kind"
            >loading…</span>
          </label>
          <j-select
            v-model="marketSelection"
            :options="marketOptions"
            :unselected="false"
            label="Pick a market"
          >
            <template #default>
              {{ marketLabels[marketSelection] ?? 'Pick a market' }}
            </template>
            <template #option="{ option }">
              {{ marketLabels[option as string] ?? option }}
            </template>
          </j-select>
          <j-input
            v-if="marketSelection === MANUAL"
            v-model="marketManualAddress"
            placeholder="CABC… (56-char Soroban contract)"
          />
          <p
            v-if="addressErrors.market"
            class="multisig-field__hint multisig-field__hint--err"
          >
            {{ addressErrors.market }}
          </p>
        </div>

        <div class="multisig-field multisig-field--inline">
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
          <p
            v-if="!marketManagerOverride && defaultMarketManager"
            class="multisig-field__readonly"
            :title="defaultMarketManager"
          >
            <code>{{ truncateAddress(defaultMarketManager, 8, 8) }}</code>
            <span class="multisig-field__readonly-meta">SDK default for {{ networkLabel }}</span>
          </p>
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
          {{ selectedFn.description }}
        </p>
      </header>

      <div class="multisig-card multisig-card--stack">
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
            v-model="args[name]"
            :placeholder="field.kind === 'wasm-hash' ? '64 hex characters' : ''"
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

          <div
            v-if="field.kind === 'wasm-hash'"
            class="wasm-verify"
          >
            <div class="wasm-verify__head">
              <span class="wasm-verify__label">Cross-check hash against local .wasm</span>
              <span class="wasm-verify__sub">
                Confirms the file you have matches the hash you pasted —
                catches transcription, copy-paste, and substitution errors.
                Does <strong>not</strong> certify the file's contents are correct;
                review the source separately.
              </span>
            </div>
            <input
              type="file"
              accept=".wasm"
              class="wasm-verify__input"
              aria-label="Upload .wasm file to cross-check against the hash above"
              @change="(e) => onWasmFileChange(e, String(name))"
            >
            <p
              v-if="wasmFileNames[name]"
              class="wasm-verify__file"
            >
              {{ wasmFileNames[name] }}
            </p>
            <p
              v-if="wasmFileNotes[name]"
              class="wasm-verify__note wasm-verify__note--ok"
            >
              {{ wasmFileNotes[name] }}
            </p>
            <p
              v-if="wasmFileErrors[name]"
              class="wasm-verify__note wasm-verify__note--err"
            >
              {{ wasmFileErrors[name] }}
            </p>
          </div>
        </div>

        <div
          v-if="selectedFn"
          class="preview-card"
        >
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
          <span
            v-if="disabledReason"
            class="multisig-actions__hint"
          >
            {{ disabledReason }}
          </span>
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
          Proposal built
        </h2>
        <p class="multisig-section__subtitle">
          Share this URL with co-signers. The proposal payload travels in the URL fragment — the
          relay never sees it.
        </p>
      </header>

      <div class="multisig-card multisig-card--stack">
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
        <div class="kv">
          <span class="kv__k">Network</span>
          <span class="kv__v">{{ networkLabel }}</span>
        </div>

        <textarea
          class="proposal-share__url"
          readonly
          rows="3"
          :value="fragmentUrl"
        />

        <div class="multisig-actions">
          <j-btn
            variant="primary"
            @click="copyText(fragmentUrl, 'Share URL copied to clipboard')"
          >
            Copy URL
          </j-btn>
          <a
            class="multisig-actions__link"
            :href="fragmentUrl"
            target="_blank"
            rel="noopener"
            title="Verify the URL decodes correctly before sharing"
          >
            Open in new tab ↗
          </a>
          <j-btn
            variant="outline-primary"
            @click="startOver"
          >
            Start over
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

  &__eyebrow-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  &__eyebrow {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: $cyan;
    text-transform: uppercase;
  }

  &__network {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 8px;
    border: 1px solid $border-secondary;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: $text-tertiary;

    &--mainnet {
      color: $success;
      border-color: color-mix(in oklab, $success 35%, $border-secondary);
    }

    &--testnet {
      color: $warning;
      border-color: color-mix(in oklab, $warning 40%, $border-secondary);
    }
  }

  &__network-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background-color: currentColor;
    box-shadow: 0 0 0 1px currentColor;
    opacity: 0.85;
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

.multisig-card {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
  padding: 16px;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: $radius-lg;

  &--stack {
    grid-template-columns: minmax(0, 1fr);
  }

  @media (max-width: 640px) {
    grid-template-columns: minmax(0, 1fr);
  }
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
    padding: 1px 8px;
    border: 1px solid $border-secondary;
    border-radius: 999px;
    color: $text-tertiary;

    &--ops {
      color: $cyan;
      border-color: color-mix(in oklab, $cyan 35%, $border-secondary);
    }
    &--program {
      color: $warning;
      border-color: color-mix(in oklab, $warning 40%, $border-secondary);
    }
    &--upgrade {
      color: $danger;
      border-color: color-mix(in oklab, $danger 45%, $border-secondary);
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

    code {
      font-family: $font-JetBrainsMono;
      font-size: 12px;
      color: $text-primary;
    }
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
    transition: color 0.12s ease, border-color 0.12s ease;

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

.wasm-verify {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 12px;
  background-color: color-mix(in oklab, $navi-700 60%, transparent);
  border: 1px dashed $border-secondary;
  border-radius: $radius-md;

  &__head {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  &__label {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    color: $text-secondary;
  }

  &__sub {
    font-size: 11px;
    color: $text-tertiary;
    line-height: 1.5;

    strong {
      color: $text-secondary;
      font-weight: 600;
    }
  }

  &__input {
    font-size: 11px;
    color: $text-secondary;
  }

  &__file {
    font-size: 11px;
    color: $text-tertiary;
    font-family: $font-JetBrainsMono;
    margin: 0;
  }

  &__note {
    font-size: 11px;
    margin: 0;
    font-family: $font-JetBrainsMono;

    &--ok {
      color: $success;
    }
    &--err {
      color: $danger;
    }
  }
}

.multisig-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;

  &__hint {
    font-size: 12px;
    color: $text-tertiary;
  }

  &__link {
    font-size: 12px;
    color: $cyan;
    text-decoration: none;
    padding: 4px 10px;
    border: 1px solid color-mix(in oklab, $cyan 30%, $border-secondary);
    border-radius: $radius-md;
    transition: color 0.12s ease, border-color 0.12s ease;

    &:hover {
      color: $navi-25;
      border-color: $cyan;
    }
  }
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

  &__body {
    font-size: 12px;
    color: $text-secondary;
    word-break: break-word;
  }

  &--err {
    border-color: color-mix(in oklab, $danger 45%, $border-secondary);
    .multisig-banner__title { color: $danger; }
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
