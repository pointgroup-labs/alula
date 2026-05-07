<script lang="ts" setup>
/**
 * Cross-checks a `.wasm` file against a claimed SHA-256 hash so the
 * composer (and signers, if reused on the sign page) can prove that the
 * hash being voted on actually corresponds to the file they audited.
 *
 * Self-contained: owns its own per-arg display state (file name, ok note,
 * error). Emits nothing; the parent only needs the hash value, which it
 * already owns via v-model on the input above this widget.
 */

import { verifyWasmFile } from '~/utils/multisig'

const props = defineProps<{
  /** The hash currently typed/pasted into the parent's input. */
  claimedHash: string
}>()

const fileName = ref<string | null>(null)
const okNote = ref<string | null>(null)
const errNote = ref<string | null>(null)

// The verifier's contract: "✓ shown" ⇒ the last uploaded file's SHA-256
// equals the hash currently displayed above. If the operator edits the
// hash after a successful upload, the previous result no longer proves
// anything — clear it so a stale green checkmark can't endorse a hash
// the file doesn't actually match. Same applies to a stale red note
// against a hash that's since been corrected.
watch(() => props.claimedHash, () => {
  fileName.value = null
  okNote.value = null
  errNote.value = null
})

async function onFileChange(event: Event) {
  fileName.value = null
  okNote.value = null
  errNote.value = null

  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  // Always reset the input so re-picking the same file refires `change`.
  input.value = ''
  if (!file) { return }

  if (!props.claimedHash) {
    errNote.value = 'Paste the claimed wasm hash above first, then upload to verify.'
    return
  }

  const result = await verifyWasmFile(file, props.claimedHash)
  fileName.value = `${file.name} · ${result.byte_size} bytes`
  if (result.matches) {
    okNote.value = 'file SHA-256 matches the claimed hash'
  } else {
    errNote.value = `hash mismatch — file SHA-256 = ${result.computed_hash}`
  }
}
</script>

<template>
  <div class="wasm-verify">
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
      @change="onFileChange"
    >
    <p
      v-if="fileName"
      class="wasm-verify__file"
    >
      {{ fileName }}
    </p>
    <p
      v-if="okNote"
      class="wasm-verify__note wasm-verify__note--ok"
    >
      {{ okNote }}
    </p>
    <p
      v-if="errNote"
      class="wasm-verify__note wasm-verify__note--err"
    >
      {{ errNote }}
    </p>
  </div>
</template>

<style lang="scss" scoped>
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
</style>
