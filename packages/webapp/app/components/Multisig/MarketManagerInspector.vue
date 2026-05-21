<script lang="ts" setup>
/**
 * Inspects a market_manager contract at compose time: fetches admin,
 * current Market wasm hash, and any queued upgrades from instance
 * storage. Catches the upgrade-class footguns that operators most often
 * trip over:
 *
 *  - "wrong contract" — the address resolves but the admin is some
 *    *other* address than the multisig we're about to authorize with.
 *  - "queue collision" — a queue_* proposal targeting a slot that
 *    already holds an upgrade (the contract panics on `set_*` over an
 *    existing queue).
 *  - "no-op upgrade" — queuing a wasm hash that matches what's already
 *    live (wastes a multisig round).
 *  - "apply doesn't match queue" — an apply_* proposal whose hash arg
 *    doesn't match what's actually queued (the contract panics).
 *
 * None of these are blocking client-side — they surface as red/yellow
 * verdicts so the operator can stop *before* gathering signatures.
 */

import type { ManagerState } from '~/utils/multisig'
import { loadManagerState } from '~/utils/multisig'

const props = defineProps<{
  /** market_manager contract id (C…). */
  address: string
  /** Soroban RPC URL — without it we can't run a check. */
  rpcUrl?: string
  /** Human label for the network ("Testnet"/"Mainnet"). */
  networkLabel?: string
  /**
   * Multisig account address the operator is composing under. We
   * cross-check this against the manager's on-chain admin — a mismatch
   * means signatures from this multisig won't satisfy `require_auth`.
   */
  expectedAdmin?: string | null
  /**
   * 64-hex `new_wasm_hash` arg the operator typed. Used for queue-vs-current
   * and queue-already-set checks. Optional — only relevant for upgrade flows.
   */
  proposedWasmHash?: string | null
  /**
   * Which slot the proposal targets:
   *  - `queue-market` / `queue-manager`: a `queue_in_*_upgrade` call
   *  - `apply-market` / `apply-manager`: an `apply_*_upgrade` call
   *  - undefined: not an upgrade flow; skip slot-aware checks
   */
  flow?: 'queue-market' | 'queue-manager' | 'apply-market' | 'apply-manager'
  /**
   * True when the catalog marks this action as propagating to every
   * market governed by the manager. Surfaced inline in the verdict body
   * (quantified by `marketsCount`) instead of a separate warning block,
   * so the operator gets one concrete sentence instead of a generic
   * "double-check the address" banner above a "verified" pill.
   */
  affectsAllMarkets?: boolean
}>()

const C_RE = /^C[A-Z2-7]{55}$/

const state = ref<ManagerState | null>(null)
const loading = ref(false)
const fetchError = ref<string | null>(null)

let reqToken = 0
let debounceTimer: ReturnType<typeof setTimeout> | null = null

function resetState() {
  state.value = null
  fetchError.value = null
  loading.value = false
}

async function run(addr: string) {
  if (!props.rpcUrl) { return }
  if (!C_RE.test(addr)) { return }
  loading.value = true
  fetchError.value = null
  const myToken = ++reqToken
  try {
    const result = await loadManagerState(props.rpcUrl, addr)
    if (myToken !== reqToken) { return }
    state.value = result
  } catch (error) {
    if (myToken !== reqToken) { return }
    fetchError.value = (error as Error).message ?? String(error)
    state.value = null
  } finally {
    if (myToken === reqToken) { loading.value = false }
  }
}

watch(
  () => [props.address, props.rpcUrl] as const,
  ([next]) => {
    resetState()
    if (debounceTimer) { clearTimeout(debounceTimer) }
    if (!next || !C_RE.test(next)) { return }
    debounceTimer = setTimeout(() => { run(next) }, 400)
  },
  { immediate: true },
)

onUnmounted(() => {
  if (debounceTimer) { clearTimeout(debounceTimer) }
})

function retry() {
  if (props.address) { run(props.address) }
}

function truncate(addr: string, head = 6, tail = 6): string {
  if (!addr) { return '' }
  if (addr.length <= head + tail + 1) { return addr }
  return `${addr.slice(0, head)}…${addr.slice(-tail)}`
}

function truncateHash(hex: string, head = 8, tail = 8): string {
  if (!hex) { return '' }
  if (hex.length <= head + tail + 1) { return hex }
  return `${hex.slice(0, head)}…${hex.slice(-tail)}`
}

function formatTs(unix?: number): string {
  if (!unix) { return '—' }
  try { return new Date(unix * 1000).toISOString().replace('T', ' ').slice(0, 19) + ' UTC' }
  catch { return String(unix) }
}

// Normalize the proposed hash exactly the way the compose form does, so
// any case/whitespace drift between the typed arg and what's already on
// chain doesn't show as a spurious mismatch.
const proposedHash = computed(() =>
  (props.proposedWasmHash ?? '').trim().toLowerCase(),
)

const adminMismatch = computed(() => {
  if (!state.value?.admin || !props.expectedAdmin) { return false }
  return state.value.admin !== props.expectedAdmin
})

// Queue-slot semantics:
//  - For `queue-*` flows, the slot MUST be empty (contract panics on
//    `set_*` over existing).
//  - For `apply-*` flows, the slot MUST be non-empty AND the queued
//    hash must match the proposal's arg.
const targetedSlot = computed(() => {
  switch (props.flow) {
    case 'queue-market':
    case 'apply-market':
      return state.value?.queuedMarketUpgrade ?? null
    case 'queue-manager':
    case 'apply-manager':
      return state.value?.queuedManagerUpgrade ?? null
    default:
      return null
  }
})

const slotAlreadyQueued = computed(() => {
  if (props.flow !== 'queue-market' && props.flow !== 'queue-manager') { return false }
  return targetedSlot.value !== null
})

const slotEmptyForApply = computed(() => {
  if (props.flow !== 'apply-market' && props.flow !== 'apply-manager') { return false }
  return targetedSlot.value === null
})

const applyHashMismatch = computed(() => {
  if (props.flow !== 'apply-market' && props.flow !== 'apply-manager') { return false }
  if (!targetedSlot.value || !proposedHash.value) { return false }
  return targetedSlot.value.wasmHash !== proposedHash.value
})

const queuedHashEqualsCurrent = computed(() => {
  if (props.flow !== 'queue-market') { return false }
  if (!proposedHash.value || !state.value?.marketWasmHash) { return false }
  return proposedHash.value === state.value.marketWasmHash
})

type Verdict
  = | { kind: 'idle' }
    | { kind: 'invalid' }
    | { kind: 'no-rpc' }
    | { kind: 'loading' }
    | { kind: 'error', msg: string }
    | { kind: 'not-deployed' }
    | { kind: 'parse-error', msg: string }
    | { kind: 'admin-mismatch' }
    | { kind: 'queue-collision' }
    | { kind: 'apply-empty-slot' }
    | { kind: 'apply-hash-mismatch' }
    | { kind: 'noop-upgrade' }
    | { kind: 'ok' }

const verdict = computed<Verdict>(() => {
  if (!props.address) { return { kind: 'idle' } }
  if (!C_RE.test(props.address)) { return { kind: 'invalid' } }
  if (!props.rpcUrl) { return { kind: 'no-rpc' } }
  if (loading.value) { return { kind: 'loading' } }
  if (fetchError.value) { return { kind: 'error', msg: fetchError.value } }
  if (!state.value) { return { kind: 'loading' } }
  if (!state.value.exists) { return { kind: 'not-deployed' } }
  if (state.value.parseError && !state.value.admin) {
    return { kind: 'parse-error', msg: state.value.parseError }
  }
  if (adminMismatch.value) { return { kind: 'admin-mismatch' } }
  if (slotAlreadyQueued.value) { return { kind: 'queue-collision' } }
  if (slotEmptyForApply.value) { return { kind: 'apply-empty-slot' } }
  if (applyHashMismatch.value) { return { kind: 'apply-hash-mismatch' } }
  if (queuedHashEqualsCurrent.value) { return { kind: 'noop-upgrade' } }
  return { kind: 'ok' }
})

const verdictTone = computed<'ok' | 'warn' | 'err' | 'info'>(() => {
  switch (verdict.value.kind) {
    case 'ok': return 'ok'
    case 'noop-upgrade': return 'warn'
    case 'admin-mismatch':
    case 'queue-collision':
    case 'apply-empty-slot':
    case 'apply-hash-mismatch':
    case 'not-deployed':
    case 'invalid':
    case 'error':
    case 'parse-error': return 'err'
    default: return 'info'
  }
})

const verdictTitle = computed<string>(() => {
  switch (verdict.value.kind) {
    case 'idle': return ''
    case 'invalid': return 'Contract id format invalid'
    case 'no-rpc': return 'No RPC configured'
    case 'loading': return 'Inspecting market_manager…'
    case 'error': return 'Lookup failed'
    case 'not-deployed': return 'No contract at this address'
    case 'parse-error': return 'Could not decode contract state'
    case 'admin-mismatch': return 'Admin does not match the selected multisig'
    case 'queue-collision': return 'Slot already holds a queued upgrade'
    case 'apply-empty-slot': return 'Nothing queued — apply would revert'
    case 'apply-hash-mismatch': return 'Apply hash does not match the queued upgrade'
    case 'noop-upgrade': return 'Proposed hash equals current Market wasm — no-op'
    case 'ok': return 'Verified market_manager'
  }
})

const verdictBody = computed<string>(() => {
  const v = verdict.value
  switch (v.kind) {
    case 'idle': return ''
    case 'invalid': return 'Soroban contracts are 56 characters, base32, starting with C.'
    case 'no-rpc': return 'Configure a Soroban RPC URL to inspect the contract on chain.'
    case 'loading': return 'Resolving admin, current wasm hash, and queued upgrades from the live ledger.'
    case 'error': return v.msg
    case 'parse-error': return v.msg
    case 'not-deployed':
      return 'No contract instance exists at this address on '
        + `${props.networkLabel ?? 'this network'}. Double-check the market_manager address.`
    case 'admin-mismatch':
      return `On-chain admin is ${truncate(state.value?.admin ?? '', 6, 6)}, but the selected multisig is `
        + `${truncate(props.expectedAdmin ?? '', 6, 6)}. Signatures from this multisig won't satisfy `
        + '`require_auth` — the tx will revert at submit.'
    case 'queue-collision':
      return 'A queued upgrade is already pending in this slot. The contract panics if you queue '
        + 'a second one — cancel the existing queue first, or wait for it to apply.'
    case 'apply-empty-slot':
      return 'No upgrade is queued in this slot. apply_* requires a queued upgrade — the call would revert.'
    case 'apply-hash-mismatch':
      return `The arg you typed (${truncateHash(proposedHash.value)}) does not match the queued hash `
        + `(${truncateHash(targetedSlot.value?.wasmHash ?? '')}). The contract panics on mismatch; `
        + 'use the queued hash or cancel and re-queue.'
    case 'noop-upgrade':
      return 'The proposed wasm hash is already the current Market wasm. Queuing it is a no-op — '
        + 'check that you copied the new hash, not the existing one.'
    case 'ok': {
      const adminBit = state.value?.admin
        ? props.expectedAdmin && state.value.admin === props.expectedAdmin
          ? 'admin matches multisig'
          : `admin ${truncate(state.value.admin, 6, 6)}`
        : 'admin unknown'
      // Quantified propagation: only mention scale when the action
      // actually broadcasts, and use the live count from chain rather
      // than the static "every market" phrasing. With 1 market, "1
      // market" is honest; with 7, "all 7 markets" earns its weight.
      const count = state.value?.marketsCount ?? 0
      const propagationBit = props.affectsAllMarkets
        ? count === 0
          ? 'no markets registered yet — nothing to propagate to'
          : count === 1
            ? 'propagates to the 1 registered market'
            : `propagates to all ${count} registered markets`
        : `${count} market(s) registered`
      return `${adminBit} · ${propagationBit}.`
    }
  }
})

// Two-layer compactness:
//  - `canCompact` is structural: does this view have a useful compact form
//    at all? Compact mode only makes sense when nothing actionable is
//    pending (queues / pending admin / upgrade-in-flight). Those states
//    carry information the operator needs to see — collapsing them would
//    hide the very thing they came here to check.
//  - `isCompact` adds the user's preference on top: when compactable AND
//    the user hasn't asked for details, hide the grid.
//
// Splitting these prevents the "dead toggle" bug: when `canCompact` is
// false (e.g. an upgrade flow is in progress), the Show details button
// hides entirely instead of flipping state nothing reads.
const showDetails = ref(false)
const canCompact = computed(() =>
  verdict.value.kind === 'ok'
  && !state.value?.queuedMarketUpgrade
  && !state.value?.queuedManagerUpgrade
  && !state.value?.pendingAdmin
  && !props.flow,
)
const isCompact = computed(() => canCompact.value && !showDetails.value)
const detailsVisible = computed(() => Boolean(state.value?.exists) && !isCompact.value)

</script>

<template>
  <div
    v-if="verdict.kind !== 'idle'"
    class="mm-inspect"
  >
    <header class="mm-inspect__head">
      <div
        class="mm-inspect__pill"
        :class="`mm-inspect__pill--${verdictTone}`"
      >
        <span class="mm-inspect__pill-title">
          {{ verdictTitle }}
          <span
            v-if="networkLabel && state"
            class="mm-inspect__pill-net"
          >· {{ networkLabel }}</span>
        </span>
        <span
          v-if="verdictBody"
          class="mm-inspect__pill-body"
        >{{ verdictBody }}</span>
        <button
          v-if="verdict.kind === 'error'"
          type="button"
          class="mm-inspect__retry"
          @click="retry"
        >
          Retry
        </button>
        <button
          v-if="state && state.exists && canCompact"
          type="button"
          class="mm-inspect__retry"
          @click="showDetails = !showDetails"
        >
          {{ showDetails ? 'Hide details' : 'Show details' }}
        </button>
      </div>
    </header>

    <div
      v-if="state && state.exists && detailsVisible"
      class="mm-inspect__grid"
    >
      <div class="mm-inspect__cell">
        <span class="mm-inspect__cell-k">Admin</span>
        <code
          class="mm-inspect__cell-v"
          :class="{ 'mm-inspect__cell-v--err': adminMismatch }"
          :title="state.admin ?? ''"
        >{{ state.admin ? truncate(state.admin, 8, 8) : '—' }}</code>
        <span
          v-if="adminMismatch"
          class="mm-inspect__cell-note mm-inspect__cell-note--err"
        >mismatch</span>
        <span
          v-else-if="expectedAdmin && state.admin === expectedAdmin"
          class="mm-inspect__cell-note mm-inspect__cell-note--ok"
        >matches multisig</span>
      </div>

      <div class="mm-inspect__cell">
        <span class="mm-inspect__cell-k">Current Market wasm</span>
        <code
          class="mm-inspect__cell-v"
          :title="state.marketWasmHash ?? ''"
        >{{ state.marketWasmHash ? truncateHash(state.marketWasmHash) : '—' }}</code>
        <span
          v-if="queuedHashEqualsCurrent"
          class="mm-inspect__cell-note mm-inspect__cell-note--warn"
        >equals proposed</span>
      </div>

      <div class="mm-inspect__cell">
        <span class="mm-inspect__cell-k">Markets registered</span>
        <span class="mm-inspect__cell-v">{{ state.marketsCount ?? 0 }}</span>
      </div>

      <div
        v-if="state.pendingAdmin"
        class="mm-inspect__cell"
      >
        <span class="mm-inspect__cell-k">Pending admin</span>
        <code
          class="mm-inspect__cell-v"
          :title="state.pendingAdmin"
        >{{ truncate(state.pendingAdmin, 8, 8) }}</code>
      </div>

      <div
        v-if="state.queuedMarketUpgrade"
        class="mm-inspect__cell mm-inspect__cell--wide"
      >
        <span class="mm-inspect__cell-k">Queued Market upgrade</span>
        <code
          class="mm-inspect__cell-v"
          :title="state.queuedMarketUpgrade.wasmHash"
        >{{ truncateHash(state.queuedMarketUpgrade.wasmHash) }}</code>
        <span class="mm-inspect__cell-note">queued {{ formatTs(state.queuedMarketUpgrade.queuedAtUnix) }}</span>
      </div>

      <div
        v-if="state.queuedManagerUpgrade"
        class="mm-inspect__cell mm-inspect__cell--wide"
      >
        <span class="mm-inspect__cell-k">Queued Manager self-upgrade</span>
        <code
          class="mm-inspect__cell-v"
          :title="state.queuedManagerUpgrade.wasmHash"
        >{{ truncateHash(state.queuedManagerUpgrade.wasmHash) }}</code>
        <span class="mm-inspect__cell-note">queued {{ formatTs(state.queuedManagerUpgrade.queuedAtUnix) }}</span>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.mm-inspect {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 10px 12px;
  background-color: color-mix(in oklab, $navi-700 60%, transparent);
  border: 1px dashed $border-secondary;
  border-radius: $radius-md;

  &__head {
    display: flex;
    flex-direction: column;
  }

  &__pill {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px 12px;
    border: 1px solid $border-secondary;
    border-radius: $radius-md;
    background-color: color-mix(in oklab, $navi-800 50%, transparent);

    &--ok {
      border-color: color-mix(in oklab, $success 45%, $border-secondary);
      background-color: color-mix(in oklab, $success 10%, transparent);
    }
    &--warn {
      border-color: color-mix(in oklab, $warning 45%, $border-secondary);
      background-color: color-mix(in oklab, $warning 10%, transparent);
    }
    &--err {
      border-color: color-mix(in oklab, $danger 45%, $border-secondary);
      background-color: color-mix(in oklab, $danger 10%, transparent);
    }
  }

  &__pill-title {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;

    .mm-inspect__pill--ok & { color: $success; }
    .mm-inspect__pill--warn & { color: $warning; }
    .mm-inspect__pill--err & { color: $danger; }
    .mm-inspect__pill--info & { color: $text-secondary; }
  }

  &__pill-net {
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.03em;
    color: $text-tertiary;
    text-transform: none;
    margin-left: 4px;
  }

  &__pill-body {
    font-size: 11px;
    color: $text-secondary;
    line-height: 1.5;
  }

  &__retry {
    align-self: flex-start;
    margin-top: 4px;
    background: none;
    border: 1px solid $border-secondary;
    border-radius: 999px;
    padding: 2px 10px;
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

  &__grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;

    @media (max-width: 540px) {
      grid-template-columns: 1fr;
    }
  }

  &__cell {
    display: flex;
    align-items: baseline;
    flex-wrap: wrap;
    gap: 6px;
    padding: 6px 8px;
    border: 1px solid $border-secondary;
    border-radius: $radius-sm;
    background-color: color-mix(in oklab, $navi-800 40%, transparent);

    &--wide { grid-column: 1 / -1; }
  }

  &__cell-k {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: $text-tertiary;
    flex-basis: 100%;
  }

  &__cell-v {
    font-family: $font-JetBrainsMono;
    font-size: 12px;
    color: $text-primary;
    font-weight: 600;

    &--err { color: $danger; }
  }

  &__cell-note {
    font-size: 10px;
    color: $text-tertiary;
    font-style: italic;

    &--ok { color: $success; font-style: normal; font-weight: 600; }
    &--warn { color: $warning; font-style: normal; font-weight: 600; }
    &--err { color: $danger; font-style: normal; font-weight: 700; }
  }
}
</style>
