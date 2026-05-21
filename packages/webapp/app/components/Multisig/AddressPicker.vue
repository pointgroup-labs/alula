<script lang="ts" setup>
const props = defineProps<{
  /** Effective address (computed by parent: dropdown choice or manual buffer). */
  modelValue: string
  /** Known options to surface in the dropdown, in display order. */
  knownOptions: readonly string[]
  /** label[address] -> human-readable row text. */
  labels: Record<string, string>
  /** Placeholder for the manual input. */
  manualPlaceholder?: string
  /** Closed-state default text when nothing is selected. */
  selectLabel?: string
  /** Inline error to render under the input. */
  error?: string | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

// Root element ref; used to locate the manual input for autofocus when
// the operator picks "Enter address manually…". JInput doesn't expose
// .focus() so we reach through the rendered DOM rather than plumb a ref
// through every wrapper component.
const rootEl = ref<HTMLElement | null>(null)

/**
 * Reusable picker for "pick from a known set, or paste manually".
 *
 * Used by the compose page for the multisig account and the market
 * fields, both of which share the shape: dropdown of known addresses
 * (each a string keyed in `labels`) plus a special MANUAL sentinel that
 * unhides a free-form input.
 *
 * The parent owns the *effective address* via v-model; this component
 * owns the local picker state (which option is selected, what's typed in
 * the manual field) and reconciles the two through internal computeds.
 */

const MANUAL = '__manual__'

// Local state: what's currently picked in the dropdown, and the buffer
// for manual entry. We never *derive* these from `modelValue` — that would
// fight the user when they type into the manual field. Instead we pick a
// sane initial state and emit changes upward.
const selection = ref<string>(
  props.modelValue && props.knownOptions.includes(props.modelValue)
    ? props.modelValue
    : MANUAL,
)
const manualBuffer = ref<string>(
  props.modelValue && !props.knownOptions.includes(props.modelValue)
    ? props.modelValue
    : '',
)

// If the parent reassigns modelValue (e.g. network change wipes it,
// or a watcher snaps it to a known address), reflect that into our state.
watch(() => props.modelValue, (next) => {
  if (next && props.knownOptions.includes(next)) {
    selection.value = next
    return
  }
  // External clear or external manual value.
  if (selection.value !== MANUAL) {
    selection.value = MANUAL
  }
  if (manualBuffer.value !== next) {
    manualBuffer.value = next
  }
})

// If the known-options list changes and our current pick disappears
// (e.g. role change yields no known multisig), fall back to MANUAL so the
// dropdown isn't pointing at a phantom value.
watch(() => props.knownOptions, (next) => {
  if (selection.value !== MANUAL && !next.includes(selection.value)) {
    selection.value = MANUAL
  }
})

const dropdownOptions = computed<string[]>(() => [...props.knownOptions, MANUAL])

const dropdownLabels = computed<Record<string, string>>(() => ({
  ...props.labels,
  [MANUAL]: 'Enter address manually…',
}))

function onSelectionChange(next: string) {
  selection.value = next
  emit('update:modelValue', next === MANUAL ? manualBuffer.value : next)
  // Autofocus the manual input the moment the operator switches to
  // MANUAL — the input is freshly mounted on the same tick, so wait
  // one nextTick for it to appear in the DOM before grabbing it.
  if (next === MANUAL) {
    nextTick(() => {
      rootEl.value?.querySelector<HTMLInputElement>('input')?.focus()
    })
  }
}

function onManualInput(next: string) {
  manualBuffer.value = next
  if (selection.value === MANUAL) {
    emit('update:modelValue', next)
  }
}

defineExpose({ MANUAL })
</script>

<template>
  <div
    ref="rootEl"
    class="address-picker"
  >
    <j-select
      :model-value="selection"
      :options="dropdownOptions"
      :unselected="false"
      :label="selectLabel ?? 'Pick an address'"
      @update:model-value="(v) => onSelectionChange(v as string)"
    >
      <template #default>
        {{ dropdownLabels[selection] ?? (selectLabel ?? 'Pick an address') }}
      </template>
      <template #option="{ option }">
        {{ dropdownLabels[option as string] ?? option }}
      </template>
    </j-select>
    <j-input
      v-if="selection === MANUAL"
      :model-value="manualBuffer"
      :placeholder="manualPlaceholder"
      @update:model-value="(v) => onManualInput(String(v))"
    />
    <p
      v-if="error"
      class="address-picker__err"
    >
      {{ error }}
    </p>
  </div>
</template>

<style lang="scss" scoped>
.address-picker {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;

  // Address strings (G…/C…) are pure base32 and visually fragile in
  // sans-serif — `1`/`l`, `0`/`O` confusions matter at audit time.
  // Force the inner input to mono so the operator can eyeball-compare
  // against an external source. Targets the b-form-input that JInput
  // renders.
  :deep(input) {
    font-family: $font-JetBrainsMono;
    font-size: 12px;
    letter-spacing: 0.01em;
  }

  &__err {
    font-size: 11px;
    color: $danger;
    margin: 0;
    line-height: 1.5;
  }
}
</style>
