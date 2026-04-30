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
import { decodeProposal, extractSigPayloads } from '~/utils/multisig'

const route = useRoute()
const multisig = useMultisigStore()

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
    }
    catch (error) {
      multisig.decodingError = (error as Error).message
    }
  }
  // Poll the relay every 5s. Cheap (KV GET) and the operator usually has
  // this tab open while signers go through the flow.
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
    if (r.ok) { added++ }
    else { rejected++ }
  }
  pasteResult.value = `Added ${added}, rejected ${rejected}`
  pasteBlob.value = ''
}

async function submit() {
  submitError.value = null
  try {
    await multisig.submitCurrent()
  }
  catch (error) {
    submitError.value = (error as Error).message
  }
}

const summary = computed(() => {
  if (!fnEntry.value || !multisig.proposal) { return null }
  return fnEntry.value.renderSummary(multisig.proposal.args, multisig.proposal.snapshot)
})

const sigRows = computed(() => {
  return multisig.sigs.map((sig: SigPayload) => {
    const weight = multisig.proposal?.signer_set_snapshot.find(s => s.key === sig.signer_pubkey)?.weight ?? 0
    return { signer: sig.signer_pubkey, weight }
  })
})

const progressPct = computed(() => {
  if (multisig.requiredThreshold === 0) { return 0 }
  return Math.min(100, Math.round((multisig.collectedWeight / multisig.requiredThreshold) * 100))
})

function truncateAddress(addr: string, head = 6, tail = 6): string {
  if (addr.length <= head + tail + 1) { return addr }
  return `${addr.slice(0, head)}…${addr.slice(-tail)}`
}
</script>

<template>
  <main class="multisig-aggregate-page container">
    <header class="multisig-aggregate-page__hero">
      <span class="multisig-aggregate-page__eyebrow">Multisig</span>
      <h1 class="multisig-aggregate-page__title">
        Aggregate &amp; submit
      </h1>
      <p class="multisig-aggregate-page__lead">
        The relay is polled every five seconds. Each signature is re-validated against the
        proposal hash and the on-chain signer-set snapshot before it counts toward the threshold.
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
            Threshold
          </h2>
          <p class="multisig-section__subtitle">
            Collected weight against the multisig medium threshold.
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
              Awaiting signatures
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

        <p
          v-if="multisig.lastRelayError"
          class="multisig-actions__err"
        >
          Relay error: {{ multisig.lastRelayError.message }}
        </p>
      </section>

      <section class="multisig-section">
        <header class="multisig-section__header">
          <h2 class="multisig-section__title">
            Signatures
          </h2>
          <p class="multisig-section__subtitle">
            Each row is a verified signature from a snapshot signer.
          </p>
        </header>

        <ul
          v-if="sigRows.length"
          class="sig-list"
        >
          <li
            v-for="row in sigRows"
            :key="row.signer"
            class="sig-chip"
          >
            <span class="sig-chip__addr">
              <code :title="row.signer">{{ truncateAddress(row.signer) }}</code>
            </span>
            <span class="sig-chip__weight">w {{ row.weight }}</span>
          </li>
        </ul>
        <div
          v-else
          class="multisig-empty"
        >
          No verified signatures yet.
        </div>
      </section>

      <section class="multisig-section">
        <header class="multisig-section__header">
          <h2 class="multisig-section__title">
            Paste signatures (fallback)
          </h2>
          <p class="multisig-section__subtitle">
            If a signer can't reach the relay, paste their <code>alula-sig:v1:…</code> line(s) here.
            Each one is independently re-verified before being added.
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
              variant="outline-primary"
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
              :disabled="!multisig.thresholdMet || multisig.submitting"
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

          <div
            v-if="multisig.lastSubmit"
            class="multisig-banner multisig-banner--ok"
          >
            <span class="multisig-banner__title">Submitted</span>
            <span class="multisig-banner__body">
              <code>{{ multisig.lastSubmit.txHash }}</code> · status
              <strong>{{ multisig.lastSubmit.status }}</strong> · poll
              <code>getTransaction</code> to confirm.
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
    .multisig-banner__title { color: $danger; }
  }

  &--ok {
    border-color: color-mix(in oklab, $success 35%, $border-secondary);
    .multisig-banner__title { color: $success; }
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

    strong {
      color: $cyan;
    }
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

    &--ok {
      background-color: $success;
    }
  }
}

.sig-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.sig-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px 4px 10px;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: 999px;
  font-size: 11px;

  &__addr code {
    font-family: $font-JetBrainsMono;
    color: $text-primary;
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
</style>
