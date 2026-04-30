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
import { decodeProposal, serializeSigPayload } from '~/utils/multisig'

const route = useRoute()
const wallet = useWallet()
const connection = useConnectionStore()
const multisig = useMultisigStore()
const toast = useToast()

const fnEntry = ref<FunctionDef<any, any> | null>(null)
const signing = ref(false)
const signError = ref<string | null>(null)
const localSigB64 = ref<string | null>(null)

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
    }
    catch (error) {
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

async function sign() {
  signError.value = null
  if (!wallet.publicKey) { signError.value = 'Connect a wallet first'; return }
  if (!multisig.proposal) { signError.value = 'No proposal loaded'; return }
  signing.value = true
  try {
    const sig = await multisig.signCurrent()
    localSigB64.value = serializeSigPayload(sig)
  }
  catch (error) {
    signError.value = (error as Error).message
  }
  finally {
    signing.value = false
  }
}

async function copySig() {
  if (!localSigB64.value) { return }
  try {
    await navigator.clipboard.writeText(localSigB64.value)
    toast.create({ title: 'Copied', body: 'Signature copied to clipboard', modelValue: 2000 })
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

const alreadySigned = computed(() =>
  Boolean(wallet.publicKey)
  && multisig.sigs.some(s => s.signer_pubkey === wallet.publicKey),
)

const isAllowedSigner = computed(() =>
  Boolean(wallet.publicKey)
  && multisig.allowedSigners.includes(wallet.publicKey as string),
)
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
              <span
                v-if="row.before"
                class="diff-row__before"
              >
                <code>{{ row.before }}</code>
              </span>
              <span
                v-else
                class="diff-row__before diff-row__before--empty"
              >—</span>
              <span class="diff-row__arrow">→</span>
              <span class="diff-row__after">
                <code>{{ row.after }}</code>
              </span>
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
            <span class="kv__v">{{ multisig.proposal.proposal_hash }}</span>
          </div>
          <div class="kv">
            <span class="kv__k">Composer</span>
            <span class="kv__v">{{ multisig.proposal.created_by }}</span>
          </div>
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
            v-else-if="alreadySigned"
            class="multisig-banner multisig-banner--ok"
          >
            <span class="multisig-banner__title">Already signed</span>
            <span class="multisig-banner__body">Your signature is recorded for this proposal.</span>
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

      <section
        v-if="localSigB64"
        class="multisig-section"
      >
        <header class="multisig-section__header">
          <h2 class="multisig-section__title">
            Fallback share
          </h2>
          <p class="multisig-section__subtitle">
            If the relay was offline when you signed, paste this string into chat so the operator
            can add it manually.
          </p>
        </header>
        <div class="multisig-card multisig-card--stack">
          <textarea
            class="proposal-share__url"
            readonly
            rows="2"
            :value="localSigB64"
          />
          <j-btn
            variant="outline-primary"
            @click="copySig"
          >
            Copy signature
          </j-btn>
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
    .multisig-banner__title { color: $danger; }
  }

  &--ok {
    border-color: color-mix(in oklab, $success 35%, $border-secondary);
    .multisig-banner__title { color: $success; }
  }

  &--warn {
    border-color: color-mix(in oklab, $warning 40%, $border-secondary);
    .multisig-banner__title { color: $warning; }
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
  }

  &__arrow {
    color: $text-tertiary;
    font-family: $font-JetBrainsMono;
  }

  &__after {
    color: $text-primary;
    font-weight: 600;
  }

  &--warning {
    border-color: color-mix(in oklab, $warning 40%, $border-secondary);
    background-color: color-mix(in oklab, $warning 6%, $bg-card);
  }

  &--critical {
    border-color: color-mix(in oklab, $danger 45%, $border-secondary);
    background-color: color-mix(in oklab, $danger 8%, $bg-card);
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
  }
}
</style>
