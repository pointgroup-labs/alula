<script lang="ts" setup>
/**
 * Inspects a Stellar account at compose-time: fetches its signer set
 * and thresholds and surfaces "is this actually a multisig?" / "are you
 * one of its signers?" so the operator catches single-signer, wrong-
 * account, and not-a-signer mistakes before clicking Build.
 *
 * Spec §5.3 invariants 1, 2 — signer set and thresholds are recorded
 * in the proposal at build time. This inspector lets the operator see
 * what's about to be snapshotted.
 */

import type { MultisigAccountState } from '~/utils/multisig'
import type { SignerEntry, ThresholdsSnapshot } from '~/utils/multisig/types'
import { loadMultisigState } from '~/utils/multisig'

const props = defineProps<{
  /** Account G… address the user picked. */
  address: string
  /** Soroban RPC URL — without it we can't run a check. */
  rpcUrl?: string
  /** Connected wallet pubkey. Used to highlight "you are a signer". */
  composer?: string | null
  /** Human label for the network ("Testnet"/"Mainnet"). */
  networkLabel?: string
  /**
   * Snapshot signer set captured at compose time. When provided, the
   * inspector compares it against the live ledger and emits a
   * `snapshot-drift` verdict listing the additions / removals / weight
   * changes. Drives the sign page's "snapshot is stale" warning.
   */
  snapshotSigners?: SignerEntry[] | null
  /** Snapshot thresholds; compared against live the same way. */
  snapshotThresholds?: ThresholdsSnapshot | null
}>()

const G_RE = /^G[A-Z2-7]{55}$/

const state = ref<MultisigAccountState | null>(null)
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
  if (!G_RE.test(addr)) { return }
  loading.value = true
  fetchError.value = null
  const myToken = ++reqToken
  try {
    const result = await loadMultisigState(props.rpcUrl, addr)
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
  () => props.address,
  (next) => {
    resetState()
    if (debounceTimer) { clearTimeout(debounceTimer) }
    if (!next || !G_RE.test(next)) { return }
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

const signers = computed(() => state.value?.signers ?? [])
const isSingleSigner = computed(() => state.value !== null && signers.value.length <= 1)
const totalWeight = computed(() => signers.value.reduce((s, e) => s + e.weight, 0))

const composerSigner = computed(() => {
  if (!props.composer || !state.value) { return null }
  return signers.value.find(s => s.key === props.composer) ?? null
})

// Snapshot-vs-live drift. Only meaningful when caller passes a
// snapshot (sign page does, compose page does not). We report:
//   - `removed`: signers present in snapshot but missing live (a sig
//     from one of these is unusable — the on-chain account no longer
//     recognizes them).
//   - `added`: signers present live but missing from snapshot (a sig
//     from one of these can't be added to this proposal even though
//     it would count at submit — the snapshot guards the relay).
//   - `weight-changed`: same key, different weight. Affects whether
//     collected sigs cross the threshold at submit time.
//   - `thresholds-changed`: med threshold drift is the consequential
//     one for Soroban invocations; low/high are tracked for forensics.
type DriftReport = {
  removed: SignerEntry[]
  added: SignerEntry[]
  weightChanged: Array<{ key: string, before: number, after: number }>
  thresholdsChanged: boolean
  hasAny: boolean
}
const drift = computed<DriftReport | null>(() => {
  if (!props.snapshotSigners || !state.value) { return null }
  const liveByKey = new Map(signers.value.map(s => [s.key, s.weight]))
  const snapByKey = new Map(props.snapshotSigners.map(s => [s.key, s.weight]))

  const removed: SignerEntry[] = []
  const weightChanged: Array<{ key: string, before: number, after: number }> = []
  for (const [key, snapW] of snapByKey) {
    const liveW = liveByKey.get(key)
    if (liveW === undefined) { removed.push({ key, weight: snapW }) }
    else if (liveW !== snapW) { weightChanged.push({ key, before: snapW, after: liveW }) }
  }
  const added: SignerEntry[] = []
  for (const [key, liveW] of liveByKey) {
    if (!snapByKey.has(key)) { added.push({ key, weight: liveW }) }
  }

  const snapT = props.snapshotThresholds
  const liveT = state.value.thresholds
  const thresholdsChanged = Boolean(
    snapT
    && (snapT.low !== liveT.low || snapT.med !== liveT.med || snapT.high !== liveT.high),
  )

  const hasAny = removed.length > 0 || added.length > 0
    || weightChanged.length > 0 || thresholdsChanged
  return { removed, added, weightChanged, thresholdsChanged, hasAny }
})

const medThreshold = computed(() => state.value?.thresholds.med ?? 0)
const lowThreshold = computed(() => state.value?.thresholds.low ?? 0)
const highThreshold = computed(() => state.value?.thresholds.high ?? 0)

// Pre-flight: can `med`-threshold ever be reached? If sum of all signer
// weights is below `med`, the account is effectively bricked for
// Soroban operations (which require `med`). Compose-time catch.
const thresholdUnreachable = computed(() =>
  state.value !== null && totalWeight.value < medThreshold.value,
)

type Verdict
  = | { kind: 'idle' }
    | { kind: 'invalid' }
    | { kind: 'no-rpc' }
    | { kind: 'loading' }
    | { kind: 'error', msg: string }
    | { kind: 'single-signer' }
    | { kind: 'threshold-unreachable' }
    | { kind: 'snapshot-drift' }
    | { kind: 'composer-not-signer' }
    | { kind: 'ok' }

const verdict = computed<Verdict>(() => {
  if (!props.address) { return { kind: 'idle' } }
  if (!G_RE.test(props.address)) { return { kind: 'invalid' } }
  if (!props.rpcUrl) { return { kind: 'no-rpc' } }
  if (loading.value) { return { kind: 'loading' } }
  if (fetchError.value) { return { kind: 'error', msg: fetchError.value } }
  if (!state.value) { return { kind: 'loading' } }
  if (isSingleSigner.value) { return { kind: 'single-signer' } }
  if (thresholdUnreachable.value) { return { kind: 'threshold-unreachable' } }
  // Snapshot drift is louder than composer-not-signer because it
  // invalidates assumptions about the *whole* signature collection,
  // not just the connected wallet's role. Surface it first.
  if (drift.value?.hasAny) { return { kind: 'snapshot-drift' } }
  // Composer-not-signer is informational, not blocking — operators
  // can legitimately compose for others to sign. We still flag it
  // visibly so it's never silent.
  if (props.composer && !composerSigner.value) { return { kind: 'composer-not-signer' } }
  return { kind: 'ok' }
})

const verdictTone = computed<'ok' | 'warn' | 'err' | 'info'>(() => {
  switch (verdict.value.kind) {
    case 'ok': return 'ok'
    case 'composer-not-signer':
    case 'snapshot-drift': return 'warn'
    case 'single-signer':
    case 'threshold-unreachable': return 'err'
    case 'invalid':
    case 'error': return 'err'
    default: return 'info'
  }
})

const verdictTitle = computed<string>(() => {
  switch (verdict.value.kind) {
    case 'idle': return ''
    case 'invalid': return 'Account format invalid'
    case 'no-rpc': return 'No RPC configured'
    case 'loading': return 'Inspecting account…'
    case 'error': return 'Lookup failed'
    case 'single-signer': return 'Not a multisig — single signer only'
    case 'threshold-unreachable': return 'Threshold unreachable'
    case 'snapshot-drift': return 'Snapshot is stale — account has changed on chain'
    case 'composer-not-signer': return 'You are not a signer of this multisig'
    case 'ok': return composerSigner.value
      ? `Verified · you are a signer (weight ${composerSigner.value.weight})`
      : 'Verified'
  }
})

const verdictBody = computed<string>(() => {
  const v = verdict.value
  switch (v.kind) {
    case 'idle': return ''
    case 'invalid': return 'Stellar accounts are 56 characters, base32, starting with G.'
    case 'no-rpc': return 'Configure a Soroban RPC URL to inspect the account on chain.'
    case 'loading': return 'Resolving signer set and thresholds from the live ledger.'
    case 'error': return v.msg
    case 'single-signer':
      return 'This account has at most one signer. A proposal would only need one '
        + 'signature, which defeats the purpose of multisig. Pick the correct account.'
    case 'threshold-unreachable':
      return `Sum of signer weights (${totalWeight.value}) is below the med threshold `
        + `(${medThreshold.value}). Soroban invocations can never collect enough signatures `
        + 'on this account — pick a different multisig or fix the account first.'
    case 'snapshot-drift': {
      const d = drift.value!
      const parts: string[] = []
      if (d.removed.length) { parts.push(`${d.removed.length} signer(s) removed`) }
      if (d.added.length) { parts.push(`${d.added.length} signer(s) added`) }
      if (d.weightChanged.length) { parts.push(`${d.weightChanged.length} weight change(s)`) }
      if (d.thresholdsChanged) { parts.push('thresholds changed') }
      return `Since this proposal was composed: ${parts.join(' · ')}. `
        + 'Sigs from current-but-not-snapshot signers cannot be added; sigs from '
        + 'snapshot-but-not-current signers will not count at submit. Verify the '
        + 'collected signature set still satisfies the live thresholds before submitting.'
    }
    case 'composer-not-signer':
      return 'You can still build and share the proposal, but you won\'t be able to sign it '
        + 'with the connected wallet. Make sure at least one signer is reachable.'
    case 'ok':
      return composerSigner.value
        ? `${signers.value.length} signers configured · med threshold ${medThreshold.value} `
          + `· you can contribute weight ${composerSigner.value.weight}.`
        : `${signers.value.length} signers configured · med threshold ${medThreshold.value}.`
  }
})
</script>

<template>
  <div
    v-if="verdict.kind !== 'idle'"
    class="multisig-inspect"
  >
    <header class="multisig-inspect__head">
      <div
        class="multisig-inspect__pill"
        :class="`multisig-inspect__pill--${verdictTone}`"
      >
        <span class="multisig-inspect__pill-title">
          {{ verdictTitle }}
          <span
            v-if="networkLabel && state"
            class="multisig-inspect__pill-net"
          >· {{ networkLabel }}</span>
        </span>
        <span
          v-if="verdictBody"
          class="multisig-inspect__pill-body"
        >{{ verdictBody }}</span>
        <button
          v-if="verdict.kind === 'error'"
          type="button"
          class="multisig-inspect__retry"
          @click="retry"
        >
          Retry
        </button>
      </div>
    </header>

    <div
      v-if="state"
      class="multisig-inspect__grid"
    >
      <div class="multisig-inspect__thresholds">
        <span class="multisig-inspect__block-title">Thresholds</span>
        <div class="multisig-inspect__threshold-rows">
          <div class="multisig-inspect__threshold-row">
            <span class="multisig-inspect__threshold-k">low</span>
            <span class="multisig-inspect__threshold-v">{{ lowThreshold }}</span>
          </div>
          <div
            class="multisig-inspect__threshold-row multisig-inspect__threshold-row--highlight"
            title="A Soroban contract invocation is a `med`-threshold op on Stellar — signatures on this proposal must sum to at least this number."
          >
            <span class="multisig-inspect__threshold-k">med</span>
            <span class="multisig-inspect__threshold-v">{{ medThreshold }}</span>
            <span class="multisig-inspect__threshold-note">signatures needed for this proposal</span>
          </div>
          <div class="multisig-inspect__threshold-row">
            <span class="multisig-inspect__threshold-k">high</span>
            <span class="multisig-inspect__threshold-v">{{ highThreshold }}</span>
          </div>
        </div>
      </div>

      <div class="multisig-inspect__signers">
        <span class="multisig-inspect__block-title">
          Signers ({{ signers.length }}) · total weight {{ totalWeight }}
        </span>
        <ul class="multisig-inspect__signer-list">
          <li
            v-for="s in signers"
            :key="s.key"
            class="multisig-inspect__signer-row"
            :class="{
              'multisig-inspect__signer-row--composer': composer && s.key === composer,
              'multisig-inspect__signer-row--new': drift?.added.some(a => a.key === s.key),
              'multisig-inspect__signer-row--changed': drift?.weightChanged.some(c => c.key === s.key),
            }"
            :title="s.key"
          >
            <code>{{ truncate(s.key, 6, 6) }}</code>
            <span class="multisig-inspect__signer-weight">weight {{ s.weight }}</span>
            <span
              v-if="composer && s.key === composer"
              class="multisig-inspect__signer-you"
            >you</span>
            <span
              v-if="drift?.added.some(a => a.key === s.key)"
              class="multisig-inspect__signer-tag multisig-inspect__signer-tag--new"
            >added since snapshot</span>
            <span
              v-if="drift?.weightChanged.some(c => c.key === s.key)"
              class="multisig-inspect__signer-tag multisig-inspect__signer-tag--changed"
            >weight changed</span>
          </li>
          <!-- Snapshot signers that have been removed. They're not in
               `signers` (which is live), so render them separately with
               a struck-through style so the operator can see who used
               to count. -->
          <li
            v-for="s in (drift?.removed ?? [])"
            :key="`removed-${s.key}`"
            class="multisig-inspect__signer-row multisig-inspect__signer-row--removed"
            :title="`Was in snapshot · ${s.key}`"
          >
            <code>{{ truncate(s.key, 6, 6) }}</code>
            <span class="multisig-inspect__signer-weight">was weight {{ s.weight }}</span>
            <span class="multisig-inspect__signer-tag multisig-inspect__signer-tag--removed">removed since snapshot</span>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.multisig-inspect {
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

    .multisig-inspect__pill--ok & { color: $success; }
    .multisig-inspect__pill--warn & { color: $warning; }
    .multisig-inspect__pill--err & { color: $danger; }
    .multisig-inspect__pill--info & { color: $text-secondary; }
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
    grid-template-columns: minmax(140px, 1fr) 2fr;
    gap: 14px;

    @media (max-width: 540px) {
      grid-template-columns: 1fr;
    }
  }

  &__block-title {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: $text-tertiary;
  }

  &__thresholds {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  &__threshold-rows {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  &__threshold-row {
    display: flex;
    align-items: baseline;
    gap: 6px;
    padding: 4px 8px;
    border: 1px solid $border-secondary;
    border-radius: $radius-sm;
    background-color: color-mix(in oklab, $navi-800 40%, transparent);

    &--highlight {
      border-color: color-mix(in oklab, $cyan 35%, $border-secondary);
      background-color: color-mix(in oklab, $cyan 8%, transparent);
    }
  }

  &__threshold-k {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: $text-tertiary;
    width: 32px;
  }

  &__threshold-v {
    font-family: $font-JetBrainsMono;
    font-size: 12px;
    color: $text-primary;
    font-weight: 700;
  }

  &__threshold-note {
    margin-left: auto;
    font-size: 10px;
    font-style: italic;
    color: $cyan;
  }

  &__signers {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  &__signer-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  &__signer-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border: 1px solid $border-secondary;
    border-radius: $radius-sm;
    background-color: color-mix(in oklab, $navi-800 40%, transparent);

    code {
      font-family: $font-JetBrainsMono;
      font-size: 11px;
      color: $text-primary;
      flex: 1;
      min-width: 0;
    }

    &--composer {
      border-color: color-mix(in oklab, $success 45%, $border-secondary);
      background-color: color-mix(in oklab, $success 8%, transparent);
    }

    &--new {
      border-color: color-mix(in oklab, $cyan 40%, $border-secondary);
      background-color: color-mix(in oklab, $cyan 7%, transparent);
    }

    &--changed {
      border-color: color-mix(in oklab, $warning 40%, $border-secondary);
      background-color: color-mix(in oklab, $warning 7%, transparent);
    }

    &--removed {
      border-color: color-mix(in oklab, $danger 35%, $border-secondary);
      background-color: color-mix(in oklab, $danger 6%, transparent);
      opacity: 0.75;

      code, .multisig-inspect__signer-weight {
        text-decoration: line-through;
      }
    }
  }

  &__signer-weight {
    font-family: $font-JetBrainsMono;
    font-size: 10px;
    color: $text-tertiary;
  }

  &__signer-you {
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    padding: 1px 6px;
    border-radius: 999px;
    color: $success;
    border: 1px solid color-mix(in oklab, $success 45%, $border-secondary);
    background-color: color-mix(in oklab, $success 12%, transparent);
  }

  &__signer-tag {
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    padding: 1px 6px;
    border-radius: 999px;
    line-height: 1.4;

    &--new {
      color: $cyan;
      border: 1px solid color-mix(in oklab, $cyan 45%, $border-secondary);
      background-color: color-mix(in oklab, $cyan 12%, transparent);
    }
    &--changed {
      color: $warning;
      border: 1px solid color-mix(in oklab, $warning 45%, $border-secondary);
      background-color: color-mix(in oklab, $warning 12%, transparent);
    }
    &--removed {
      color: $danger;
      border: 1px solid color-mix(in oklab, $danger 45%, $border-secondary);
      background-color: color-mix(in oklab, $danger 12%, transparent);
    }
  }
}
</style>
