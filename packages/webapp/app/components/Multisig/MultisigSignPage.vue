<script lang="ts" setup>
/**
 * Sign page — opened from a `#p=…` URL. Decodes the proposal, runs the
 * args-↔-xdr drift check before showing anything, and lets the connected
 * wallet append a signature. The sig is POSTed to the relay so the
 * aggregate page picks it up automatically.
 *
 * Security invariants:
 *  - Never display unsigned XDR opaquely. We only render the catalog
 *    function and decoded args after `decodeProposal` succeeds.
 *  - Verify the wallet's returned sig locally before relaying — protects
 *    against a wallet that signs but lies about which key it used.
 */

import type { FunctionDef } from '~/utils/multisig'
import { Networks } from '@stellar/stellar-sdk'
import { decodeProposal, extractProposalAddresses, serializeSigPayload } from '~/utils/multisig'
import MarketManagerInspector from './MarketManagerInspector.vue'
import MultisigAccountInspector from './MultisigAccountInspector.vue'
import WasmHashVerifier from './WasmHashVerifier.vue'

const route = useRoute()
const wallet = useWallet()
const connection = useConnectionStore()
const multisig = useMultisigStore()
const rpcStore = useRpcStore()
const toast = useToast()

const fnEntry = ref<FunctionDef<any, any> | null>(null)
const signing = ref(false)
const signError = ref<string | null>(null)
const localSigB64 = ref<string | null>(null)
const showSigDetail = ref(false)

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
})

function extractFragment(): string | null {
  const hash = route.hash || (import.meta.client ? globalThis.location.hash : '')
  if (!hash.startsWith('#p=')) { return null }
  return hash.slice(3)
}

const summary = computed(() => {
  if (!fnEntry.value || !multisig.proposal) { return null }
  return fnEntry.value.renderSummary(multisig.proposal.args, multisig.proposal.snapshot)
})

// Long opaque tokens (wasm hashes, strkeys, base64) blow out the row
// width and wrap awkwardly. Render head…tail with the full value in
// `title=` so hover reveals everything. Threshold of 24 chars catches
// 56-char strkeys, 64-char hashes, and base64 blobs without touching
// short human values like numbers, BPS, or short enum names.
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

async function sign() {
  signError.value = null
  if (!wallet.publicKey) { signError.value = 'Connect a wallet first'; return }
  if (!multisig.proposal) { signError.value = 'No proposal loaded'; return }
  signing.value = true
  // Snapshot so we can tell *this* sign's relay outcome apart from a
  // stale error left by an earlier fetch. On failure we auto-expand
  // the signature artifact so the user can hand it over manually
  // without any extra prompt.
  const relayErrorBefore = multisig.lastRelayError?.when ?? 0
  try {
    const sig = await multisig.signCurrent()
    localSigB64.value = serializeSigPayload(sig)
    const relayErrorAfter = multisig.lastRelayError?.when ?? 0
    if (relayErrorAfter > relayErrorBefore) { showSigDetail.value = true }
  } catch (error) {
    signError.value = (error as Error).message
  } finally {
    signing.value = false
  }
}

async function copySig() {
  if (!localSigB64.value) { return }
  try {
    await navigator.clipboard.writeText(localSigB64.value)
    toast.create({ title: 'Copied', body: 'Signature copied to clipboard', modelValue: 2000 })
  } catch (error) {
    toast.create({
      title: 'Copy failed',
      body: String((error as Error)?.message ?? error),
      variant: 'danger',
      modelValue: 4000,
    })
  }
}

const alreadySigned = computed(() =>
  Boolean(wallet.publicKey)
  && multisig.sigs.some(s => s.signer_pubkey === wallet.publicKey),
)

const isAllowedSigner = computed(() =>
  Boolean(wallet.publicKey)
  && multisig.allowedSigners.includes(wallet.publicKey as string),
)

// --- Verification section data --------------------------------------------
//
// Everything below derives from the loaded proposal. Computeds, not
// async helpers, so the section reactively re-renders if the user
// changes networks or reconnects a wallet mid-page (rare but possible).

// Pull the two addresses out of the unsigned envelope on demand. Wrap
// in try/catch because a malformed proposal would already have set
// `decodingError` upstream — we just want to fail soft here so the
// verification UI doesn't crash the whole page on a degraded payload.
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

// Map the function id to the inspector's `flow` discriminator the same
// way the compose page does. Kept inline (not extracted) because this
// page is the only other consumer — moving it to a shared helper would
// scatter what's effectively a 4-line dispatch.
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

// Same fingerprint table the compose page uses, kept in sync by hand.
// Resist the urge to import from the compose page — the coupling would
// pull the entire compose page's deps into the sign bundle.
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

// List of wasm-hash args to render verifiers for. Most upgrade actions
// have exactly one, but we drive off the schema so future multi-hash
// actions surface every hash without code changes.
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

// Network mismatch is a footgun: the wallet may happily sign a proposal
// hash that bakes the wrong passphrase. Compare the proposal's
// passphrase against the page's current network and warn loudly.
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

// Snapshot creation age — surfaced in the Identity section so signers
// know how stale the snapshot they're being asked to ratify is.
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
  <main class="multisig-sign-page container">
    <header class="multisig-sign-page__hero">
      <span class="multisig-sign-page__eyebrow">Multisig</span>
      <h1 class="multisig-sign-page__title">
        Sign proposal
      </h1>
      <p class="multisig-sign-page__lead">
        Review the decoded action, then sign with the wallet that holds your signer key. Your
        signature is bound to this proposal hash and cannot be replayed against any other call.
      </p>
    </header>

    <section
      v-if="multisig.loading"
      class="multisig-section"
    >
      <div class="multisig-empty">
        Decoding proposal…
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
              <!-- "Setting" case: no prior value. Drop the dash + arrow
                   entirely — there's no transition to show, just a fresh
                   assignment. The NEW chip tells the eye "this is being
                   set" so the layout reads correctly. -->
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
              <!-- True diff: keep the before → after form, but truncate
                   long values so the row doesn't wrap into a column of
                   broken hash characters. -->
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
            Live on-chain checks against the proposal's snapshot. Signing without reading these
            is signing blind — the proposal hash was bound at compose time, but the world may
            have moved since.
          </p>
        </header>

        <div
          v-if="networkMismatch"
          class="multisig-banner multisig-banner--err"
        >
          <span class="multisig-banner__title">Network mismatch</span>
          <span class="multisig-banner__body">
            This proposal was composed for <strong>{{ proposalNetworkLabel }}</strong> but your
            wallet/RPC is set to <strong>{{ currentNetworkLabel }}</strong>. Switch networks
            before signing — a wrong-network signature will never satisfy the multisig.
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
            <p class="verification-block__subtitle">
              The Stellar account that owns this proposal. Verifies its current signer set and
              thresholds match the snapshot this proposal was bound to.
            </p>
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
            <p class="verification-block__subtitle">
              The contract this transaction invokes. Confirms its admin is the multisig above and
              shows current queued upgrades so this action's effect is visible in context.
            </p>
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
              <!-- Only show the raw schema field name when it adds
                   signal. For a single hash arg there's no ambiguity
                   so the bare "WASM hash" reads cleaner. -->
              <code
                v-if="wasmHashArgs.length > 1"
                class="verification-block__arg"
              >{{ arg.name }}</code>
            </h3>
            <p class="verification-block__subtitle">
              The proposed code. Fetches the on-chain WASM that hashes to this value and inspects
              its exports so you can confirm what's actually being deployed.
            </p>
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
            Sign
          </h2>
        </header>

        <div class="multisig-card multisig-card--stack">
          <div v-if="!wallet.publicKey">
            <j-btn
              variant="primary"
              @click="connection.connectWallet()"
            >
              Connect wallet
            </j-btn>
          </div>

          <div
            v-else-if="!isAllowedSigner"
            class="multisig-banner multisig-banner--err"
          >
            <span class="multisig-banner__title">Not in signer set</span>
            <span class="multisig-banner__body">
              Connected wallet <code>{{ wallet.publicKey }}</code> is not a signer on the snapshot
              for this multisig.
            </span>
          </div>

          <div
            v-else-if="alreadySigned || localSigB64"
            class="multisig-banner multisig-banner--ok signed-banner"
          >
            <span class="multisig-banner__title">Signed</span>
            <span class="multisig-banner__body">Your signature is recorded for this proposal.</span>

            <!-- On relay failure we just auto-expand the artifact
                 (see `sign()` setting `showSigDetail = true`). The
                 signature being right there, open, is the signal —
                 prose telling the user what the relay is would only
                 add noise. -->
            <div
              v-if="localSigB64"
              class="signed-banner__detail"
            >
              <button
                type="button"
                class="signed-banner__toggle"
                @click="showSigDetail = !showSigDetail"
              >
                {{ showSigDetail ? 'Hide signature' : 'View signature' }}
              </button>
              <div
                v-if="showSigDetail"
                class="signed-banner__artifact"
              >
                <textarea
                  class="proposal-share__url"
                  readonly
                  rows="2"
                  :value="localSigB64"
                />
                <j-btn
                  variant="outlined-brand"
                  @click="copySig"
                >
                  Copy signature
                </j-btn>
              </div>
            </div>
          </div>

          <div v-else>
            <j-btn
              variant="primary"
              :loading="signing"
              :disabled="signing"
              @click="sign"
            >
              {{ signing ? 'Signing…' : 'Sign with wallet' }}
            </j-btn>
          </div>

          <p
            v-if="signError"
            class="multisig-actions__err"
          >
            {{ signError }}
          </p>
        </div>
      </section>
    </template>
  </main>
</template>

<style lang="scss">
.multisig-sign-page {
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

      &:hover {
        color: $cyan;
      }
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

      &:hover {
        color: $cyan;
      }
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

    // Neutral, low-contrast chip. The hash on the same row is what
    // operators need to verify; the chip is a structural hint ("no
    // prior value — this is being set, not changed"). Anything more
    // saturated competes with content that actually matters.
    &--new {
      color: $text-tertiary;
      background-color: color-mix(in oklab, $navi-700 70%, transparent);
      border: 1px solid $border-secondary;
    }
  }

  // Severity classes color the *change* (this is the consequential
  // field in the proposal), not the page's verdict (this looks broken).
  // Keep them visually distinct from `.multisig-banner--err/--warn`:
  //   --warning  → amber (caution-worthy change)
  //   --critical → indigo (high-stakes structural change, e.g. a wasm
  //                swap or admin rotation). Reusing red here would
  //                conflict with the err/warn banner palette and make
  //                "look hard at this row" indistinguishable from
  //                "something is broken."
  &--warning {
    border-color: color-mix(in oklab, $warning 40%, $border-secondary);
    background-color: color-mix(in oklab, $warning 6%, $bg-card);
  }

  &--critical {
    border-color: color-mix(in oklab, $indigo 45%, $border-secondary);
    background-color: color-mix(in oklab, $indigo 8%, $bg-card);
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

.multisig-actions__err {
  font-size: 12px;
  color: $danger;
  margin: 0;
}

// The signed banner doubles as the "your sig artifact lives here"
// surface. Happy path is a one-liner; the disclosure only appears
// when there's an artifact to reveal, and `sign()` auto-expands it
// on relay failure so the user can hand the signature over without
// any extra prompt.
.signed-banner {
  &__detail {
    margin-top: 8px;
  }

  &__toggle {
    background: none;
    border: none;
    padding: 0;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: $text-tertiary;
    cursor: pointer;
    transition: color 0.12s ease;

    &:hover { color: $cyan; }
  }

  &__artifact {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 8px;
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

  // Raw schema field name shown only when multiple wasm hashes exist
  // on a single proposal. Muted so it reads as a disambiguator, not
  // as a heading in its own right.
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
