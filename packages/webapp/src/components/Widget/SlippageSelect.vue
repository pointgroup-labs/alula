<script lang="ts" setup>
// Mirrors MAX_SLIPPAGE_PERCENT in client-sdk (services/multiply.ts). The SDK throws
// "Slippage percent must be in [0, MAX_SLIPPAGE_PERCENT]" if a request comes in above this.
// Clamping here is the single source of truth for every consumer.
const MAX_SLIPPAGE_PERCENT = 50

const slippageModel = defineModel<number | string | undefined>({ default: '0.05' })

// Local copy backs the input so the user always sees what they typed (including out-of-range
// values that trip the inline validator). Only the clamped numeric value reaches the parent.
const inputValue = ref<number | string | undefined>(slippageModel.value)

watch(slippageModel, (next) => {
  if (next === inputValue.value) {
    return
  }
  inputValue.value = next
})

watch(inputValue, (next) => {
  if (next === '' || next === null || next === undefined) {
    slippageModel.value = 0
    return
  }
  const parsed = Number(next)
  if (!Number.isFinite(parsed)) {
    slippageModel.value = 0
    return
  }
  slippageModel.value = Math.max(0, Math.min(MAX_SLIPPAGE_PERCENT, parsed))
})

const slippageRules = [
  (value: string | number) => {
    if (value === '' || value === null || value === undefined) {
      return true
    }

    const nextValue = Number(value)
    if (!Number.isFinite(nextValue) || nextValue < 0) {
      return 'Slippage cannot be negative'
    }

    if (nextValue > MAX_SLIPPAGE_PERCENT) {
      return `Slippage must be ${MAX_SLIPPAGE_PERCENT}% or less`
    }

    return true
  },
]
</script>

<template>
  <div class="slippage-select">
    <div class="slippage-select-label">Slippage
      <info-tooltip>
        Slippage is the maximum percentage of the swap price that can be exceeded.
        <br>
        Max slippage is {{ MAX_SLIPPAGE_PERCENT }}%.
      </info-tooltip>
    </div>
    <j-input
      v-model="inputValue"
      class="slippage-select-input"
      size="md"
      only-numbers
      :rules="slippageRules"
      placeholder="0.05"
    >
      <template #append>
        <span class="slippage-select-suffix text-cyan">%</span>
      </template>
    </j-input>
  </div>
</template>

<style lang="scss">
.slippage-select {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 4px;

  &-label {
    font-size: 11px;
    font-weight: 500;
    color: $text-tertiary;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    line-height: normal;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  &-input {
    width: 76px;

    &:has(.validate-label) {
      margin-bottom: 18px;
    }

    .input-group {
      height: 28px;
      border-radius: 6px;
      border-color: $navi-400;

      input {
        font-size: 12px;
        margin-bottom: -2px;
        padding-right: 2px;
      }
    }

    .j-input__append {
      display: flex;
      align-items: center;
      padding-left: 2px;
    }

    .validate-label {
      position: absolute;
      right: 0;
      left: auto;
      bottom: 0;
      white-space: nowrap;
    }
  }

  &-suffix {
    color: $text-brand;
    font-size: 10px;
    font-weight: 700;
  }
}
</style>
