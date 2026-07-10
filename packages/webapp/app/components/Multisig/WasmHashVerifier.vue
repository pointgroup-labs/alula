<script lang="ts" setup>
/**
 * Resolves a claimed wasm hash against the live ledger and surfaces a
 * compact verdict + structural details an operator can use to confirm
 * the hash refers to the kind of contract they intend to upgrade.
 */

import type { OnChainWasmInfo } from '~/utils/multisig'
import { verifyHashOnChain } from '~/utils/multisig'

const props = defineProps<{
  /** The hash currently typed/pasted into the parent's input. */
  claimedHash: string
  /** Soroban RPC URL. Without it we can't run any check; render an info row. */
  rpcUrl?: string
  /** Optional human label for the network ("Testnet"/"Mainnet"). */
  networkLabel?: string
  /**
   * Function names this proposal *expects* to find in the wasm. Drives
   * the "Expected present" / "Missing" panes and the top-level verdict.
   * Pass a small set (2–4) of names that uniquely identify the target
   * contract (e.g. for a market upgrade: `deposit`, `borrow`, `liquidate`).
   */
  expectedExports?: string[]
}>()

const toast = useToast()

const HEX_64_RE = /^[0-9a-f]{64}$/i

const info = ref<OnChainWasmInfo | null>(null)
const loading = ref(false)
const fetchError = ref<string | null>(null)

// Monotonic request token so a slow RPC resolve can't overwrite a
// fresh check the user already triggered by editing the hash field.
let reqToken = 0
let debounceTimer: ReturnType<typeof setTimeout> | null = null

function resetState() {
  info.value = null
  fetchError.value = null
  loading.value = false
}

async function run(hash: string) {
  if (!props.rpcUrl) { return }
  if (!HEX_64_RE.test(hash)) { return }
  loading.value = true
  fetchError.value = null
  const myToken = ++reqToken
  try {
    const result = await verifyHashOnChain(props.rpcUrl, hash)
    if (myToken !== reqToken) { return }
    info.value = result
  } catch (error) {
    if (myToken !== reqToken) { return }
    fetchError.value = (error as Error).message ?? String(error)
  } finally {
    if (myToken === reqToken) { loading.value = false }
  }
}

watch(
  () => props.claimedHash,
  (next) => {
    resetState()
    if (debounceTimer) { clearTimeout(debounceTimer) }
    if (!next || !HEX_64_RE.test(next)) { return }
    debounceTimer = setTimeout(() => { run(next) }, 400)
  },
  { immediate: true },
)

onUnmounted(() => {
  if (debounceTimer) { clearTimeout(debounceTimer) }
})

function retry() {
  if (props.claimedHash) { run(props.claimedHash) }
}

// Skip allocator / runtime symbols so the "contract-ish" view is only
// what an operator would recognise as public entry points.
function isContractishExport(name: string): boolean {
  if (!name || name.startsWith('_')) { return false }
  if (name === 'memory') { return false }
  return /^[a-z][a-z0-9_]*$/.test(name)
}

const exports = computed<string[]>(() => info.value?.exports ?? [])
const contractish = computed<string[]>(() => exports.value.filter(isContractishExport))

const expectedPresent = computed<string[]>(() => {
  if (!props.expectedExports?.length || !info.value?.exists) { return [] }
  const present = new Set(contractish.value)
  return props.expectedExports.filter(name => present.has(name))
})

const expectedMissing = computed<string[]>(() => {
  if (!props.expectedExports?.length || !info.value?.exists) { return [] }
  const present = new Set(contractish.value)
  return props.expectedExports.filter(name => !present.has(name))
})

const otherExports = computed<string[]>(() => {
  if (!props.expectedExports?.length) { return contractish.value }
  const expected = new Set(props.expectedExports)
  return contractish.value.filter(name => !expected.has(name))
})

const showAllOther = ref(false)
const OTHER_PREVIEW_LIMIT = 12
const visibleOther = computed<string[]>(() =>
  showAllOther.value ? otherExports.value : otherExports.value.slice(0, OTHER_PREVIEW_LIMIT),
)
const hiddenOtherCount = computed(() =>
  Math.max(0, otherExports.value.length - OTHER_PREVIEW_LIMIT),
)

// Top-level verdict. Drives the status pill and its colour. The order
// here matters — we want the *least good* state to win when multiple
// flags fire at once (e.g. error wins over loading).
type Verdict
  = | { kind: 'idle' }
    | { kind: 'invalid' }
    | { kind: 'no-rpc' }
    | { kind: 'loading' }
    | { kind: 'error', msg: string }
    | { kind: 'not-found' }
    | { kind: 'parse-error', msg: string }
    | { kind: 'shape-mismatch', missing: string[] }
    | { kind: 'verified', soft?: boolean }

const verdict = computed<Verdict>(() => {
  if (!props.rpcUrl) { return { kind: 'no-rpc' } }
  if (!props.claimedHash) { return { kind: 'idle' } }
  if (!HEX_64_RE.test(props.claimedHash)) { return { kind: 'invalid' } }
  if (loading.value) { return { kind: 'loading' } }
  if (fetchError.value) { return { kind: 'error', msg: fetchError.value } }
  const i = info.value
  if (!i) { return { kind: 'loading' } }
  if (!i.exists) { return { kind: 'not-found' } }
  if (i.parse_error) { return { kind: 'parse-error', msg: i.parse_error } }
  if (expectedMissing.value.length > 0) {
    return { kind: 'shape-mismatch', missing: expectedMissing.value }
  }
  // No expected list → can't strongly verify shape; still mark verified
  // but flag it as "soft" so the UI can word it more cautiously.
  return { kind: 'verified', soft: !props.expectedExports?.length }
})

const verdictTone = computed<'ok' | 'warn' | 'err' | 'info'>(() => {
  switch (verdict.value.kind) {
    case 'verified': return 'ok'
    case 'shape-mismatch':
    case 'parse-error': return 'warn'
    case 'not-found':
    case 'error':
    case 'invalid': return 'err'
    default: return 'info'
  }
})

const verdictTitle = computed<string>(() => {
  switch (verdict.value.kind) {
    case 'idle': return 'Awaiting hash'
    case 'invalid': return 'Hash format invalid'
    case 'no-rpc': return 'No RPC configured'
    case 'loading': return 'Looking up on chain…'
    case 'error': return 'Lookup failed'
    case 'not-found': return 'Not uploaded on chain'
    case 'parse-error': return 'Found, but couldn\'t introspect'
    case 'shape-mismatch': return 'Found, but exports look wrong'
    case 'verified': return verdict.value.soft
      ? 'Found on chain'
      : 'Verified — exports match expected shape'
  }
})

const verdictBody = computed<string>(() => {
  const v = verdict.value
  switch (v.kind) {
    case 'idle':
      return 'Paste the 64-character SHA-256 of the new wasm above to begin verification.'
    case 'invalid':
      return 'Wasm hashes are 64 lowercase hex characters (the SHA-256 of the bytes).'
    case 'no-rpc':
      return 'Configure a Soroban RPC URL to run on-chain verification.'
    case 'loading':
      return 'Resolving the ContractCode ledger entry and parsing the wasm…'
    case 'error':
      return v.msg
    case 'not-found':
      return 'No ContractCode entry exists for this hash. Upload the wasm first '
        + '(install_contract_wasm) — otherwise apply will fail after the queue delay.'
    case 'parse-error':
      return `${v.msg}. The bytes are on chain, but we couldn't decode them. `
        + 'Confirm out-of-band before queuing.'
    case 'shape-mismatch':
      return `Expected function(s) not found in the wasm: ${v.missing.join(', ')}. `
        + 'This binary may belong to a different contract — review carefully before queuing.'
    case 'verified':
      return v.soft
        ? 'The ContractCode entry exists. No expected-exports list was provided, so '
          + 'we can\'t cross-check the contract type.'
        : 'The ContractCode entry exists and all expected entry points are present.'
  }
})

function formatBytes(n: number | undefined): string {
  if (n == null) { return '—' }
  if (n < 1024) { return `${n} B` }
  if (n < 1024 * 1024) { return `${(n / 1024).toFixed(1)} KB` }
  return `${(n / 1024 / 1024).toFixed(2)} MB`
}

async function copyHash() {
  if (!props.claimedHash) { return }
  try {
    await navigator.clipboard.writeText(props.claimedHash)
    toast.create({ title: 'Copied', body: 'Wasm hash copied to clipboard', modelValue: 2000 })
  } catch (error) {
    toast.create({
      title: 'Copy failed',
      body: String((error as Error)?.message ?? error),
      variant: 'danger',
      modelValue: 3000,
    })
  }
}

// Custom-section display: human-readable count summary + collapsed list.
const showCustomSections = ref(false)
const customSections = computed(() => info.value?.custom_sections ?? [])

const specSectionCount = computed(() =>
  customSections.value.filter(s => s.name.startsWith('contractspecv0')).length,
)
const hasEnvMeta = computed(() =>
  customSections.value.some(s => s.name === 'contractenvmetav0'),
)
const hasContractMeta = computed(() =>
  customSections.value.some(s => s.name === 'contractmetav0'),
)
</script>

<template>
  <div
    v-if="verdict.kind !== 'idle'"
    class="wasm-verify"
  >
    <header class="wasm-verify__head">
      <div
        class="wasm-verify__pill"
        :class="`wasm-verify__pill--${verdictTone}`"
      >
        <span class="wasm-verify__pill-title">
          {{ verdictTitle }}
          <span
            v-if="networkLabel && info?.exists"
            class="wasm-verify__pill-net"
          >· {{ networkLabel }}</span>
        </span>
        <span class="wasm-verify__pill-body">{{ verdictBody }}</span>
        <button
          v-if="verdict.kind === 'error'"
          type="button"
          class="wasm-verify__retry"
          @click="retry"
        >
          Retry
        </button>
      </div>
    </header>

    <div
      v-if="info?.exists"
      class="wasm-verify__stats"
    >
      <div class="wasm-verify__stat">
        <span class="wasm-verify__stat-k">Size</span>
        <span class="wasm-verify__stat-v">{{ formatBytes(info.byte_size) }}</span>
      </div>
      <div class="wasm-verify__stat">
        <span class="wasm-verify__stat-k">Functions</span>
        <span class="wasm-verify__stat-v">{{ contractish.length }} contract-ish · {{ exports.length }} total</span>
      </div>
      <div
        v-if="info.sdk_interface_version"
        class="wasm-verify__stat"
      >
        <span class="wasm-verify__stat-k">SDK iface</span>
        <span class="wasm-verify__stat-v">v{{ info.sdk_interface_version }}</span>
      </div>
      <div
        v-if="customSections.length"
        class="wasm-verify__stat"
      >
        <span class="wasm-verify__stat-k">Metadata</span>
        <span class="wasm-verify__stat-v">
          <span v-if="hasEnvMeta">env-meta</span>
          <span v-if="hasContractMeta">{{ hasEnvMeta ? ' · ' : '' }}contract-meta</span>
          <span v-if="specSectionCount">
            {{ (hasEnvMeta || hasContractMeta) ? ' · ' : '' }}{{ specSectionCount }} spec sections
          </span>
        </span>
      </div>
    </div>

    <div
      v-if="expectedPresent.length || expectedMissing.length"
      class="wasm-verify__shape"
    >
      <span class="wasm-verify__block-title">Expected entry points</span>
      <ul class="wasm-verify__pill-list">
        <li
          v-for="name in expectedPresent"
          :key="`p-${name}`"
          class="wasm-verify__chip wasm-verify__chip--ok"
          :title="`Present in wasm`"
        >
          ✓ {{ name }}
        </li>
        <li
          v-for="name in expectedMissing"
          :key="`m-${name}`"
          class="wasm-verify__chip wasm-verify__chip--err"
          :title="`Missing from wasm`"
        >
          ✗ {{ name }}
        </li>
      </ul>
    </div>

    <div
      v-if="info?.exists && otherExports.length"
      class="wasm-verify__shape"
    >
      <button
        type="button"
        class="wasm-verify__disclosure"
        @click="showAllOther = !showAllOther"
      >
        <span class="wasm-verify__block-title">
          Other exports ({{ otherExports.length }})
        </span>
        <span class="wasm-verify__disclosure-arrow">{{ showAllOther ? '▾' : '▸' }}</span>
      </button>
      <ul
        v-if="showAllOther || hiddenOtherCount === 0"
        class="wasm-verify__pill-list"
      >
        <li
          v-for="name in visibleOther"
          :key="`o-${name}`"
          class="wasm-verify__chip"
        >
          {{ name }}
        </li>
      </ul>
      <p
        v-else
        class="wasm-verify__hint"
      >
        Showing {{ OTHER_PREVIEW_LIMIT }} of {{ otherExports.length }}.
        <button
          type="button"
          class="wasm-verify__inline-btn"
          @click="showAllOther = true"
        >
          Show all
        </button>
      </p>
    </div>

    <div
      v-if="customSections.length"
      class="wasm-verify__shape"
    >
      <button
        type="button"
        class="wasm-verify__disclosure"
        @click="showCustomSections = !showCustomSections"
      >
        <span class="wasm-verify__block-title">
          Custom sections ({{ customSections.length }})
        </span>
        <span class="wasm-verify__disclosure-arrow">{{ showCustomSections ? '▾' : '▸' }}</span>
      </button>
      <ul
        v-if="showCustomSections"
        class="wasm-verify__section-list"
      >
        <li
          v-for="s in customSections"
          :key="s.name"
          class="wasm-verify__section-row"
        >
          <code>{{ s.name }}</code>
          <span class="wasm-verify__section-size">{{ formatBytes(s.byte_size) }}</span>
        </li>
      </ul>
    </div>

    <footer
      v-if="claimedHash && HEX_64_RE.test(claimedHash)"
      class="wasm-verify__footer"
    >
      <button
        type="button"
        class="wasm-verify__inline-btn"
        @click="copyHash"
      >
        Copy hash
      </button>
      <span class="wasm-verify__footer-hint">
        Forward to a co-signer to cross-check the same verdict before signing.
      </span>
    </footer>
  </div>
</template>

<style lang="scss" scoped>
.wasm-verify {
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
    &--info {
      // default
    }
  }

  &__pill-title {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;

    .wasm-verify__pill--ok & { color: $success; }
    .wasm-verify__pill--warn & { color: $warning; }
    .wasm-verify__pill--err & { color: $danger; }
    .wasm-verify__pill--info & { color: $text-secondary; }
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

  &__stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 8px;
    padding: 8px 10px;
    border: 1px solid $border-secondary;
    border-radius: $radius-sm;
    background-color: color-mix(in oklab, $navi-800 50%, transparent);
  }

  &__stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  &__stat-k {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: $text-tertiary;
  }

  &__stat-v {
    font-size: 12px;
    color: $text-primary;
    font-family: $font-JetBrainsMono;
    word-break: break-word;
  }

  &__shape {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  &__block-title {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: $text-tertiary;
  }

  &__pill-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  &__chip {
    font-family: $font-JetBrainsMono;
    font-size: 10px;
    padding: 2px 8px;
    border: 1px solid $border-secondary;
    border-radius: 999px;
    color: $text-secondary;
    background-color: color-mix(in oklab, $navi-700 40%, transparent);

    &--ok {
      color: $success;
      border-color: color-mix(in oklab, $success 45%, $border-secondary);
      background-color: color-mix(in oklab, $success 12%, transparent);
    }
    &--err {
      color: $danger;
      border-color: color-mix(in oklab, $danger 45%, $border-secondary);
      background-color: color-mix(in oklab, $danger 12%, transparent);
    }
  }

  &__disclosure {
    background: none;
    border: none;
    padding: 0;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    color: $text-tertiary;

    &:hover {
      color: $cyan;
    }
  }

  &__disclosure-arrow {
    font-size: 10px;
    color: $text-tertiary;
  }

  &__hint {
    font-size: 11px;
    color: $text-tertiary;
    margin: 0;
  }

  &__inline-btn {
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
    margin-left: 4px;
    transition:
      color 0.12s ease,
      border-color 0.12s ease;

    &:hover {
      color: $cyan;
      border-color: color-mix(in oklab, $cyan 35%, $border-secondary);
    }
  }

  &__section-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  &__section-row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    font-size: 11px;
    padding: 2px 0;
    border-bottom: 1px solid color-mix(in oklab, $border-secondary 60%, transparent);

    &:last-child { border-bottom: none; }

    code {
      font-family: $font-JetBrainsMono;
      color: $text-secondary;
      word-break: break-all;
    }
  }

  &__section-size {
    font-family: $font-JetBrainsMono;
    color: $text-tertiary;
    flex-shrink: 0;
  }

  &__footer {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    padding-top: 6px;
    border-top: 1px solid color-mix(in oklab, $border-secondary 60%, transparent);
  }

  &__footer-hint {
    font-size: 10px;
    color: $text-tertiary;
    line-height: 1.5;
  }
}
</style>
