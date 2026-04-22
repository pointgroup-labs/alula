<script lang="ts" setup>
const slippageModel = defineModel<number | string | undefined>({ default: '0.5' })

const slippageRules = [
  (value: string | number) => {
    if (value === '' || value === null || value === undefined) {
      return true
    }

    const nextValue = Number(value)
    if (!Number.isFinite(nextValue) || nextValue < 0) {
      return 'Slippage cannot be negative'
    }

    if (nextValue > 50) {
      return 'Slippage must be 50% or less'
    }

    return true
  },
]
</script>

<template>
  <div class="slippage-select">
    <span class="slippage-select-label">Slippage</span>
    <j-input
      v-model="slippageModel"
      class="slippage-select-input"
      size="md"
      only-numbers
      :rules="slippageRules"
      placeholder="0.5"
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
  }

  &-input {
    width: 72px;

    &:focus-within {
      .input-group {
        border-color: $navi-200;
      }
    }

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
      }
    }

    .j-input__append {
      display: flex;
      align-items: center;
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
