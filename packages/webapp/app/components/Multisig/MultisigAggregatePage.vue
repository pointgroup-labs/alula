<script lang="ts" setup>
/**
 * Aggregate page — operator view. Polls the relay for sigs, lets the
 * operator paste sigs as a fallback, and submits to the network when the
 * threshold weight is reached.
 *
 * The relay is untrusted: every fetched sig is re-validated against the
 * proposal hash + signer-set snapshot before it counts toward the
 * threshold.
 */

import type { FunctionDef, SigPayload } from '~/utils/multisig'
import { Networks } from '@stellar/stellar-sdk'
import { decodeProposal, extractProposalAddresses, extractSigPayloads } from '~/utils/multisig'
import MarketManagerInspector from './MarketManagerInspector.vue'
import MultisigAccountInspector from './MultisigAccountInspector.vue'
import WasmHashVerifier from './WasmHashVerifier.vue'

const route = useRoute()
const multisig = useMultisigStore()
const wallet = useWallet()
const rpcStore = useRpcStore()
const toast = useToast()

const fnEntry = ref<FunctionDef<any, any> | null>(null)
const pasteBlob = ref('')
const pasteResult = ref<string | null>(null)
const submitError = ref<string | null>(null)

let pollHandle: number | null = null

onMounted(async () => {
  const fragment = extractFragment()
  if (!fragment) {
    multisig.decodingError = 'Missing #p=… proposal fragment in URL'
    return
  }
  await multisig.loadFromFragment(fragment)
  if (multisig.proposal) {
    try {
      const decoded = await decodeProposal(multisig.proposal)
      fnEntry.value = decoded.fn
    } catch (error) {
      multisig.decodingError = (error as Error).message
    }
  }
  // Poll the relay every 5s. Cheap (KV GET); silent on failure — the
  // paste fallback covers the case where the relay is truly down, and
  // a transient `Failed to fetch` in the UI just spooks operators.
  pollHandle = globalThis.setInterval(() => { multisig.refreshSigs() }, 5000) as unknown as number
})

onBeforeUnmount(() => {
  if (pollHandle !== null) { globalThis.clearInterval(pollHandle) }
})

function extractFragment(): string | null {
  const hash = route.hash || (import.meta.client ? globalThis.location.hash : '')
  if (!hash.startsWith('#p=')) { return null }
  return hash.slice(3)
}

function pasteSigs() {
  pasteResult.value = null
  const sigs = extractSigPayloads(pasteBlob.value)
  if (sigs.length === 0) { pasteResult.value = 'No well-formed sigs found in paste'; return }

  let added = 0
  let rejected = 0
  for (const s of sigs) {
    const r = multisig.addSigPayload(s)
    if (r.ok) { added++ } else { rejected++ }
  }
  pasteResult.value = `Added ${added}, rejected ${rejected}`
  pasteBlob.value = ''
}

async function submit() {
  submitError.value = null
  try {
    await multisig.submitCurrent()
  } catch (error) {
    submitError.value = (error as Error).message
  }
}

// Classify Soroban RPC's `sendTransaction` status. PENDING/DUPLICATE
// are the only outcomes that mean "the network has it"; everything
// else is either retryable (TRY_AGAIN_LATER) or a hard reject (ERROR).
// Treating the banner uniformly as success hid TRY_AGAIN_LATER from
// operators, who'd then walk away assuming the tx was in flight.
type SubmitOutcome = 'success' | 'retry' | 'error'
const submitOutcome = computed<SubmitOutcome | null>(() => {
  const s = multisig.lastSubmit?.status
  if (!s) { return null }
  if (s === 'PENDING' || s === 'DUPLICATE') { return 'success' }
  if (s === 'TRY_AGAIN_LATER') { return 'retry' }
  return 'error'
})

// Once the network has the tx (PENDING/DUPLICATE), disable Submit so
// an operator can't fire a duplicate and get a confusing
// TRY_AGAIN_LATER for the *retry* rather than the original.
const submitBlocked = computed(() => submitOutcome.value === 'success')

const summary = computed(() => {
  if (!fnEntry.value || !multisig.proposal) { return null }
  return fnEntry.value.renderSummary(multisig.proposal.args, multisig.proposal.snapshot)
})

// Cross-reference the snapshot with verified sigs to build the full
// signer roster. Operators need to see *who hasn't signed* as
// prominently as who has — otherwise they have to mentally diff the
// two lists, which is exactly the kind of work that gets glossed
// over right before submit.
type RosterRow = {
  key: string
  weight: number
  signed: boolean
}
const signerRoster = computed<RosterRow[]>(() => {
  const snapshot = multisig.proposal?.signer_set_snapshot ?? []
  const signedSet = new Set(multisig.sigs.map((s: SigPayload) => s.signer_pubkey))
  const rows: RosterRow[] = snapshot.map(s => ({
    key: s.key,
    weight: s.weight,
    signed: signedSet.has(s.key),
  }))
  // Signed first, then heaviest unsigned at the top so the operator
  // can see at a glance which missing signatures would close the gap.
  rows.sort((a, b) => {
    if (a.signed !== b.signed) { return a.signed ? -1 : 1 }
    return b.weight - a.weight
  })
  return rows
})

const progressPct = computed(() => {
  if (multisig.requiredThreshold === 0) { return 0 }
  return Math.min(100, Math.round((multisig.collectedWeight / multisig.requiredThreshold) * 100))
})

const remainingWeight = computed(() =>
  Math.max(0, multisig.requiredThreshold - multisig.collectedWeight),
)

function truncateAddress(addr: string, head = 6, tail = 6): string {
  if (addr.length <= head + tail + 1) { return addr }
  return `${addr.slice(0, head)}…${addr.slice(-tail)}`
}

// Long opaque tokens (wasm hashes, strkeys, base64) blow out row width.
// Render head…tail with the full value in `title=` so hover reveals
// everything. Same threshold as sign page.
function isLongOpaque(s: string | null | undefined): boolean {
  return typeof s === 'string' && s.length > 24
}
function shortOpaque(s: string, head = 10, tail = 10): string {
  if (s.length <= head + tail + 1) { return s }
  return `${s.slice(0, head)}…${s.slice(-tail)}`
}

async function copyValue(s: string | null | undefined, label: string) {
  if (!s) { return }
  try {
    await navigator.clipboard.writeText(s)
    toast.create({ title: 'Copied', body: `${label} copied`, modelValue: 1800 })
  } catch (error) {
    toast.create({
      title: 'Copy failed',
      body: String((error as Error)?.message ?? error),
      variant: 'danger',
      modelValue: 4000,
    })
  }
}

// --- Verification section data --------------------------------------------
//
// Mirrors the sign page. Operators submit transactions whose snapshot
// may be hours old — drift surfaces here, not just on the signer side.

const proposalAddresses = computed(() => {
  if (!multisig.proposal) { return null }
  try {
    return extractProposalAddresses(
      multisig.proposal.unsigned_xdr,
      multisig.proposal.network_passphrase,
    )
  } catch {
    return null
  }
})

const targetIsMarketManager = computed(() =>
  fnEntry.value?.contract === 'market_manager'
  && Boolean(proposalAddresses.value?.targetContract),
)

const managerFlow = computed<'queue-market' | 'queue-manager' | 'apply-market' | 'apply-manager' | undefined>(() => {
  switch (multisig.proposal?.function_id) {
    case 'market_manager.queue_in_market_upgrade': return 'queue-market'
    case 'market_manager.apply_market_upgrade': return 'apply-market'
    case 'market_manager.queue_in_manager_upgrade': return 'queue-manager'
    case 'market_manager.apply_manager_upgrade': return 'apply-manager'
    default: return undefined
  }
})

const proposedWasmHash = computed<string>(() => {
  const args = multisig.proposal?.args
  if (!args) { return '' }
  const named = args.new_wasm_hash
  if (typeof named === 'string') { return named }
  const fn = fnEntry.value
  if (!fn) { return '' }
  for (const [name, field] of Object.entries(fn.argSchema)) {
    if ((field as { kind: string }).kind === 'wasm-hash') {
      const v = args[name]
      if (typeof v === 'string') { return v }
    }
  }
  return ''
})

// Same fingerprint table the compose + sign pages use. Duplicated by
// design — importing across pages would pull each page's deps into
// the others' bundles.
const EXPECTED_EXPORTS_BY_FN_ID: Record<string, string[]> = {
  'market_manager.queue_in_market_upgrade': ['deposit', 'borrow', 'liquidate'],
  'market_manager.queue_in_manager_upgrade': [
    'queue_in_market_upgrade',
    'apply_market_upgrade',
    'register_market',
  ],
}
function expectedExportsFor(fnId: string | undefined): string[] {
  if (!fnId) { return [] }
  return EXPECTED_EXPORTS_BY_FN_ID[fnId] ?? []
}

const wasmHashArgs = computed<Array<{ name: string, value: string }>>(() => {
  const fn = fnEntry.value
  const args = multisig.proposal?.args
  if (!fn || !args) { return [] }
  const out: Array<{ name: string, value: string }> = []
  for (const [name, field] of Object.entries(fn.argSchema)) {
    if ((field as { kind: string }).kind === 'wasm-hash') {
      const v = args[name]
      if (typeof v === 'string') { out.push({ name, value: v }) }
    }
  }
  return out
})

const proposalNetworkLabel = computed(() => {
  const p = multisig.proposal?.network_passphrase
  if (p === Networks.PUBLIC) { return 'Mainnet' }
  if (p === Networks.TESTNET) { return 'Testnet' }
  return p ? `Custom (${p})` : 'Unknown'
})
const currentNetworkLabel = computed(() =>
  rpcStore.network === 'public' ? 'Mainnet' : 'Testnet',
)
const networkMismatch = computed(() => {
  const proposalPassphrase = multisig.proposal?.network_passphrase
  if (!proposalPassphrase) { return false }
  const currentPassphrase = rpcStore.network === 'public' ? Networks.PUBLIC : Networks.TESTNET
  return proposalPassphrase !== currentPassphrase
})

const proposalAgeText = computed(() => {
  const t = multisig.proposal?.created_at
  if (!t) { return '' }
  const seconds = Math.max(0, Math.floor(Date.now() / 1000 - t))
  if (seconds < 60) { return `${seconds}s ago` }
  if (seconds < 3600) { return `${Math.floor(seconds / 60)}m ago` }
  if (seconds < 86400) { return `${Math.floor(seconds / 3600)}h ago` }
  return `${Math.floor(seconds / 86400)}d ago`
})
const proposalCreatedAtIso = computed(() => {
  const t = multisig.proposal?.created_at
  if (!t) { return '' }
  try { return new Date(t * 1000).toISOString().replace('T', ' ').slice(0, 19) + ' UTC' }
  catch { return String(t) }
})
</script>

<template>
  <main class="multisig-aggregate-page container">
    <header class="multisig-aggregate-page__hero">
      <span class="multisig-aggregate-page__eyebrow">Multisig</span>
      <h1 class="multisig-aggregate-page__title">
        Aggregate &amp; submit
      </h1>
      <p class="multisig-aggregate-page__lead">
        Collect signatures, verify on-chain state hasn't drifted from the proposal's snapshot, and
        submit when the threshold is met. Every signature is re-validated against the proposal
        hash and the snapshot signer set before it counts.
      </p>
    </header>

    <section
      v-if="multisig.loading"
      class="multisig-section"
    >
      <div class="multisig-empty">
        Loading proposal…
      </div>
    </section>

    <section
      v-if="multisig.decodingError"
      class="multisig-section"
    >
      <div class="multisig-banner multisig-banner--err">
        <span class="multisig-banner__title">Cannot decode proposal</span>
        <span class="multisig-banner__body">{{ multisig.decodingError }}</span>
      </div>
    </section>

    <template v-if="multisig.proposal && summary && !multisig.decodingError">
      <section class="multisig-section">
        <header class="multisig-section__header">
          <h2 class="multisig-section__title">
            {{ summary.title }}
          </h2>
          <p class="multisig-section__subtitle">
            Multisig role <strong>{{ multisig.proposal.multisig }}</strong> ·
            <code>{{ fnEntry?.contract }}.{{ fnEntry?.function }}</code>
          </p>
        </header>

        <ul class="diff-list">
          <li
            v-for="row in summary.rows"
            :key="row.label"
            class="diff-row"
            :class="row.severity ? `diff-row--${row.severity}` : null"
          >
            <span class="diff-row__label">{{ row.label }}</span>
            <div class="diff-row__values">
              <template v-if="!row.before">
                <span class="diff-row__chip diff-row__chip--new">new</span>
                <button
                  type="button"
                  class="diff-row__after diff-row__after--copy"
                  :title="`Click to copy · ${row.after}`"
                  @click="copyValue(row.after, row.label)"
                >
                  <code>{{ isLongOpaque(row.after) ? shortOpaque(row.after) : row.after }}</code>
                </button>
              </template>
              <template v-else>
                <button
                  type="button"
                  class="diff-row__before diff-row__before--copy"
                  :title="`Previous value · click to copy · ${row.before}`"
                  @click="copyValue(row.before, `Previous ${row.label}`)"
                >
                  <code>{{ isLongOpaque(row.before) ? shortOpaque(row.before) : row.before }}</code>
                </button>
                <span class="diff-row__arrow">→</span>
                <button
                  type="button"
                  class="diff-row__after diff-row__after--copy"
                  :title="`New value · click to copy · ${row.after}`"
                  @click="copyValue(row.after, row.label)"
                >
                  <code>{{ isLongOpaque(row.after) ? shortOpaque(row.after) : row.after }}</code>
                </button>
              </template>
            </div>
          </li>
        </ul>
      </section>

      <section class="multisig-section">
        <header class="multisig-section__header">
          <h2 class="multisig-section__title">
            Identity
          </h2>
        </header>
        <div class="multisig-card multisig-card--stack">
          <div class="kv">
            <span class="kv__k">Proposal hash</span>
            <button
              type="button"
              class="kv__v kv__v--copy"
              :title="`Click to copy · ${multisig.proposal.proposal_hash}`"
              @click="copyValue(multisig.proposal.proposal_hash, 'Proposal hash')"
            >{{ multisig.proposal.proposal_hash }}</button>
          </div>
          <div class="kv">
            <span class="kv__k">Composer</span>
            <button
              type="button"
              class="kv__v kv__v--copy"
              :title="`Click to copy · ${multisig.proposal.created_by}`"
              @click="copyValue(multisig.proposal.created_by, 'Composer address')"
            >{{ multisig.proposal.created_by }}</button>
          </div>
          <div class="kv">
            <span class="kv__k">Created</span>
            <span
              class="kv__v"
              :title="proposalCreatedAtIso"
            >{{ proposalAgeText }} <span class="kv__note">({{ proposalCreatedAtIso }})</span></span>
          </div>
          <div class="kv">
            <span class="kv__k">Network</span>
            <span class="kv__v">{{ proposalNetworkLabel }}</span>
          </div>
        </div>
      </section>

      <section class="multisig-section">
        <header class="multisig-section__header">
          <h2 class="multisig-section__title">
            Verification
          </h2>
          <p class="multisig-section__subtitle">
            Live on-chain state vs. the proposal's snapshot.
          </p>
        </header>

        <div
          v-if="networkMismatch"
          class="multisig-banner multisig-banner--err"
        >
          <span class="multisig-banner__title">Network mismatch</span>
          <span class="multisig-banner__body">
            This proposal was composed for <strong>{{ proposalNetworkLabel }}</strong> but your
            wallet/RPC is set to <strong>{{ currentNetworkLabel }}</strong>. Switch networks before
            submitting — the transaction will be rejected as a hash mismatch otherwise.
          </span>
        </div>

        <div
          v-if="proposalAddresses?.multisigAccount"
          class="verification-block"
        >
          <header class="verification-block__header">
            <h3 class="verification-block__title">
              Multisig account
            </h3>
          </header>
          <multisig-account-inspector
            :address="proposalAddresses.multisigAccount"
            :rpc-url="rpcStore.sorobanRPCUrl ?? undefined"
            :composer="wallet.publicKey"
            :network-label="proposalNetworkLabel"
            :snapshot-signers="multisig.proposal.signer_set_snapshot"
            :snapshot-thresholds="multisig.proposal.thresholds_snapshot"
          />
        </div>

        <div
          v-if="targetIsMarketManager && proposalAddresses?.targetContract"
          class="verification-block"
        >
          <header class="verification-block__header">
            <h3 class="verification-block__title">
              Target contract · Market Manager
            </h3>
          </header>
          <market-manager-inspector
            :address="proposalAddresses.targetContract"
            :rpc-url="rpcStore.sorobanRPCUrl ?? undefined"
            :network-label="proposalNetworkLabel"
            :expected-admin="proposalAddresses.multisigAccount"
            :proposed-wasm-hash="proposedWasmHash || null"
            :flow="managerFlow"
            :affects-all-markets="fnEntry?.affectsAllMarkets"
          />
        </div>

        <div
          v-for="arg in wasmHashArgs"
          :key="`wasm-${arg.name}`"
          class="verification-block"
        >
          <header class="verification-block__header">
            <h3 class="verification-block__title">
              WASM hash
              <code
                v-if="wasmHashArgs.length > 1"
                class="verification-block__arg"
              >{{ arg.name }}</code>
            </h3>
          </header>
          <wasm-hash-verifier
            :claimed-hash="arg.value"
            :rpc-url="rpcStore.sorobanRPCUrl ?? undefined"
            :network-label="proposalNetworkLabel"
            :expected-exports="expectedExportsFor(multisig.proposal.function_id)"
          />
        </div>
      </section>

      <section class="multisig-section">
        <header class="multisig-section__header">
          <h2 class="multisig-section__title">
            Threshold
          </h2>
          <p class="multisig-section__subtitle">
            Collected signing weight vs. the multisig's medium threshold from the snapshot.
          </p>
        </header>

        <div class="multisig-card threshold-card">
          <div class="threshold-card__head">
            <span class="threshold-card__weight">
              <strong>{{ multisig.collectedWeight }}</strong>
              <span class="threshold-card__sep">/</span>
              {{ multisig.requiredThreshold }}
            </span>
            <span
              v-if="multisig.thresholdMet"
              class="threshold-card__pill threshold-card__pill--ok"
            >
              <span class="threshold-card__dot" />
              Threshold met
            </span>
            <span
              v-else
              class="threshold-card__pill"
            >
              <span class="threshold-card__dot" />
              Need {{ remainingWeight }} more weight
            </span>
          </div>
          <div
            class="threshold-card__bar"
            :title="`${progressPct}%`"
          >
            <div
              class="threshold-card__bar-fill"
              :class="{ 'threshold-card__bar-fill--ok': multisig.thresholdMet }"
              :style="{ width: `${progressPct}%` }"
            />
          </div>
        </div>
      </section>

      <section class="multisig-section">
        <header class="multisig-section__header">
          <h2 class="multisig-section__title">
            Signers
          </h2>
          <p class="multisig-section__subtitle">
            Every snapshot signer and whether their signature has been collected. Heaviest
            unsigned keys are listed first so you can see which signatures would close the gap.
          </p>
        </header>

        <ul
          v-if="signerRoster.length > 0"
          class="roster"
        >
          <li
            v-for="row in signerRoster"
            :key="row.key"
            class="roster__row"
            :class="row.signed ? 'roster__row--signed' : 'roster__row--waiting'"
          >
            <span class="roster__status">
              <span
                v-if="row.signed"
                class="roster__dot roster__dot--ok"
              />
              <span
                v-else
                class="roster__dot"
              />
              {{ row.signed ? 'Signed' : 'Waiting' }}
            </span>
            <button
              type="button"
              class="roster__addr"
              :title="`Click to copy · ${row.key}`"
              @click="copyValue(row.key, 'Signer address')"
            >
              <code>{{ truncateAddress(row.key) }}</code>
            </button>
            <span class="roster__weight">w {{ row.weight }}</span>
          </li>
        </ul>
        <div
          v-else
          class="multisig-empty"
        >
          Snapshot has no signers.
        </div>
      </section>

      <section class="multisig-section">
        <header class="multisig-section__header">
          <h2 class="multisig-section__title">
            Paste signatures
          </h2>
          <p class="multisig-section__subtitle">
            For signers who couldn't reach the relay. Paste their <code>alula-sig:v1:…</code>
            line(s); each is independently re-verified before being added.
          </p>
        </header>
        <div class="multisig-card multisig-card--stack">
          <textarea
            v-model="pasteBlob"
            class="proposal-share__url"
            rows="3"
            placeholder="alula-sig:v1:…"
          />
          <div class="multisig-actions">
            <j-btn
              variant="outlined-brand"
              @click="pasteSigs"
            >
              Add pasted sigs
            </j-btn>
            <span
              v-if="pasteResult"
              class="multisig-actions__hint"
            >
              {{ pasteResult }}
            </span>
          </div>
        </div>
      </section>

      <section class="multisig-section">
        <header class="multisig-section__header">
          <h2 class="multisig-section__title">
            Submit
          </h2>
        </header>
        <div class="multisig-card multisig-card--stack">
          <div class="multisig-actions">
            <j-btn
              variant="primary"
              :loading="multisig.submitting"
              :disabled="!multisig.thresholdMet || multisig.submitting || networkMismatch || submitBlocked"
              @click="submit"
            >
              {{ multisig.submitting ? 'Submitting…' : 'Submit to network' }}
            </j-btn>
            <span
              v-if="submitError"
              class="multisig-actions__err"
            >
              {{ submitError }}
            </span>
          </div>

          <!-- Outcome banner colored by what the RPC actually said.
               PENDING/DUPLICATE → green; the operator should poll
               getTransaction to confirm landing.
               TRY_AGAIN_LATER → amber; tx wasn't accepted, retry.
               Anything else → red; submission failed. -->
          <div
            v-if="multisig.lastSubmit && submitOutcome"
            class="multisig-banner"
            :class="{
              'multisig-banner--ok': submitOutcome === 'success',
              'multisig-banner--warn': submitOutcome === 'retry',
              'multisig-banner--err': submitOutcome === 'error',
            }"
          >
            <span class="multisig-banner__title">
              {{ submitOutcome === 'success'
                ? 'Submitted'
                : submitOutcome === 'retry'
                  ? 'Network busy'
                  : 'Submission rejected' }}
            </span>
            <span class="multisig-banner__body">
              <button
                type="button"
                class="kv__v kv__v--copy"
                :title="`Click to copy · ${multisig.lastSubmit.txHash}`"
                @click="copyValue(multisig.lastSubmit.txHash, 'Transaction hash')"
              ><code>{{ multisig.lastSubmit.txHash }}</code></button>
              · status <strong>{{ multisig.lastSubmit.status }}</strong>
              <template v-if="submitOutcome === 'success'">
                · poll <code>getTransaction</code> to confirm landing
              </template>
              <template v-else-if="submitOutcome === 'retry'">
                · the RPC didn't accept this tx — click Submit again
              </template>
            </span>
          </div>
        </div>
      </section>
    </template>
  </main>
</template>

<style lang="scss">
.multisig-aggregate-page {
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

.multisig-empty {
  padding: 24px;
  text-align: center;
  color: $text-tertiary;
  font-size: 13px;
  border: 1px dashed $border-primary;
  border-radius: $radius-lg;
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

    code {
      font-family: $font-JetBrainsMono;
      font-size: 11px;
      color: $text-primary;
      word-break: break-all;
    }
  }

  &--err {
    border-color: color-mix(in oklab, $danger 45%, $border-secondary);
    .multisig-banner__title {
      color: $danger;
    }
  }

  &--ok {
    border-color: color-mix(in oklab, $success 35%, $border-secondary);
    .multisig-banner__title {
      color: $success;
    }
  }

  &--warn {
    border-color: color-mix(in oklab, $warning 40%, $border-secondary);
    .multisig-banner__title {
      color: $warning;
    }
  }
}

.diff-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.diff-row {
  display: grid;
  grid-template-columns: minmax(140px, 200px) 1fr;
  gap: 16px;
  padding: 10px 14px;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: $radius-md;

  &__label {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: $text-tertiary;
    align-self: center;
  }

  &__values {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    min-width: 0;
    font-size: 12px;

    code {
      font-family: $font-JetBrainsMono;
      color: $text-primary;
      word-break: break-all;
    }
  }

  &__before {
    color: $text-tertiary;

    &--empty {
      font-family: $font-JetBrainsMono;
    }

    &--copy {
      background: none;
      border: none;
      padding: 0;
      color: $text-tertiary;
      cursor: pointer;
      transition: color 0.12s ease;

      code { color: inherit; }

      &:hover { color: $cyan; }
    }
  }

  &__arrow {
    color: $text-tertiary;
    font-family: $font-JetBrainsMono;
  }

  &__after {
    color: $text-primary;
    font-weight: 600;

    &--copy {
      background: none;
      border: none;
      padding: 0;
      cursor: pointer;
      color: $text-primary;
      transition: color 0.12s ease;

      code { color: inherit; }

      &:hover { color: $cyan; }
    }
  }

  &__chip {
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    padding: 2px 7px;
    border-radius: 999px;
    line-height: 1;

    &--new {
      color: $text-tertiary;
      background-color: color-mix(in oklab, $navi-700 70%, transparent);
      border: 1px solid $border-secondary;
    }
  }

  // See sign page for the rationale on indigo vs red here. Short
  // version: red conflicts with the err banner palette and makes
  // "look hard at this row" indistinguishable from "broken."
  &--warning {
    border-color: color-mix(in oklab, $warning 40%, $border-secondary);
    background-color: color-mix(in oklab, $warning 6%, $bg-card);
  }

  &--critical {
    border-color: color-mix(in oklab, $indigo 45%, $border-secondary);
    background-color: color-mix(in oklab, $indigo 8%, $bg-card);
  }
}

.threshold-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  grid-template-columns: minmax(0, 1fr);

  &__head {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  &__weight {
    font-family: $font-JetBrainsMono;
    font-size: 22px;
    font-weight: 600;
    color: $text-primary;

    strong { color: $cyan; }
  }

  &__sep {
    color: $text-tertiary;
    margin: 0 4px;
  }

  &__pill {
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

    &--ok {
      color: $success;
      border-color: color-mix(in oklab, $success 35%, $border-secondary);
    }
  }

  &__dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background-color: currentColor;
    box-shadow: 0 0 0 1px currentColor;
    opacity: 0.85;
  }

  &__bar {
    width: 100%;
    height: 6px;
    background-color: color-mix(in oklab, $navi-700 80%, transparent);
    border-radius: 999px;
    overflow: hidden;
  }

  &__bar-fill {
    height: 100%;
    background-color: $cyan;
    transition: width $transition-base ease;

    &--ok { background-color: $success; }
  }
}

// Roster lists every snapshot signer, not just the ones who have
// signed. Operators need to see what's *missing* at a glance — the
// previous chip list buried that under "go cross-reference the
// snapshot yourself." Sorted signed-first, then heaviest-unsigned
// next so the top of the waiting block is the most leverage-y.
.roster {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;

  &__row {
    display: grid;
    grid-template-columns: 110px 1fr auto;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background-color: $bg-card;
    border: 1px solid $border-secondary;
    border-radius: $radius-md;
    font-size: 12px;

    &--signed {
      border-color: color-mix(in oklab, $success 25%, $border-secondary);
    }
    &--waiting {
      // Slightly muted so the eye lands on the signed rows first;
      // the *count* of waiting rows is what matters, not each one.
      opacity: 0.92;
    }
  }

  &__status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: $text-tertiary;
  }

  &__dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: $text-tertiary;
    opacity: 0.6;

    &--ok {
      background-color: $success;
      opacity: 1;
    }
  }

  &__addr {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    text-align: left;
    color: $text-primary;
    transition: color 0.12s ease;
    min-width: 0;

    code {
      font-family: $font-JetBrainsMono;
      color: inherit;
    }

    &:hover { color: $cyan; }
  }

  &__weight {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: $cyan;
    padding: 1px 6px;
    border: 1px solid color-mix(in oklab, $cyan 35%, $border-secondary);
    border-radius: 999px;
  }
}

.multisig-section {
  display: flex;
  flex-direction: column;
  gap: 12px;

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

.multisig-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;

  &__err {
    font-size: 12px;
    color: $danger;
    margin: 0;
  }

  &__hint {
    font-size: 12px;
    color: $text-tertiary;
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

    &--copy {
      background: none;
      border: none;
      padding: 0;
      cursor: pointer;
      color: $text-primary;
      transition: color 0.12s ease;

      &:hover { color: $cyan; }
    }
  }

  &__note {
    font-size: 10px;
    color: $text-tertiary;
    font-family: $font-JetBrainsMono;
    margin-left: 4px;
  }
}

.verification-block {
  display: flex;
  flex-direction: column;
  gap: 10px;

  &__header {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  &__title {
    font-size: 13px;
    font-weight: 600;
    color: $text-primary;
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  &__arg {
    font-family: $font-JetBrainsMono;
    font-size: 11px;
    font-weight: 500;
    color: $text-tertiary;
    padding: 2px 6px;
    border-radius: $radius-sm;
    background-color: color-mix(in oklab, $navi-700 70%, transparent);
    border: 1px solid $border-secondary;
  }

  &__subtitle {
    font-size: 11px;
    color: $text-tertiary;
    margin: 0;
    line-height: 1.5;
    max-width: 620px;
  }
}
</style>
