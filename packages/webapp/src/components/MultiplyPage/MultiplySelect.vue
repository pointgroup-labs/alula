<script lang="ts" setup>
import type { PoolData } from '@alula/market-sdk'

const {
  multiplier,
  maxMultiply,
  netApy,
  pool,
} = defineProps<{
  multiplier: number | string
  maxMultiply: number | string
  netApy: number
  pool?: PoolData
}>()

const percent = defineModel<number>({ default: 85 })

const netApyDisplay = computed(() => truncatePercent(Number(netApy) || 0, 2))

const netApyPrefix = computed(() => Number(netApy) > 0 ? '+' : '')
const depositApy = computed(() => (pool?.apy.supply_bps || 0) / 100)

const netApyClass = computed(() => {
  if (netApy <= 0) {
    return 'negative'
  }
  if (netApy < depositApy.value) {
    return 'warning'
  }
  return 'positive'
})

const minPercent = computed(() => {
  const max = Number(maxMultiply)
  if (!max) { return 0 }

  const raw = (1.1 / max) * 100

  return Math.ceil(raw * 10) / 10
})

watch([() => minPercent.value, () => maxMultiply], ([nextMin, nextMax]) => {
  if (!nextMax) {
    percent.value = 0
    return
  }

  if (percent.value < nextMin) {
    percent.value = nextMin
  }
}, { immediate: true })
</script>

<template>
  <div class="multiply-select">
    <div class="multiply-select__header">
      <div>
        <div class="multiply-select__label">
          Target multiplier
          <info-tooltip>
            Adjusts the leverage of your position.
            <br>
            Increasing the multiplier boosts exposure and potential returns, but also raises borrowing costs and liquidation risk.
          </info-tooltip>
        </div>
      </div>

      <div class="multiply-select__value">
        x{{ truncatePercent(Number(multiplier) || 0, 2) }}
      </div>
    </div>

    <input
      v-model.number="percent"
      class="multiply-select__range"
      type="range"
      :min="minPercent"
      :max="100"
      :step="0.001"
    >

    <div class="multiply-select__limits">
      <span>x1.10</span>
      <span>x{{ truncatePercent(Number(maxMultiply) || 0, 2) }}</span>
    </div>

    <div class="separator" />

    <div class="net-apy">
      <div class="net-apy__label">
        Net APY
      </div>

      <div
        class="net-apy__value"
        :class="[`net-apy--${netApyClass}`]"
      >
        {{ netApyPrefix }}{{ netApyDisplay }}%
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.multiply-select {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  border: 1px solid $border-primary;
  border-radius: 16px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.04) 0%, rgba(255, 255, 255, 0.02) 100%);

  &__header {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: flex-start;
  }

  &__label {
    font-size: 13px;
    font-weight: 700;
    color: $text-primary;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  &__value {
    padding: 4px 10px;
    border-radius: 999px;
    background: rgba(24, 185, 119, 0.16);
    border: 1px solid rgba(24, 185, 119, 0.22);
    color: $text-success;
    font-size: 12px;
    font-weight: 700;
    font-family: $font-JetBrainsMono;
    white-space: nowrap;
  }

  &__range {
    width: 100%;
    accent-color: #18b977;
    filter: saturate(1.15) brightness(0.95);
  }

  &__limits {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: $text-tertiary;
  }

  .net-apy {
    display: flex;
    align-items: center;
    justify-content: space-between;

    &__label {
      font-size: 12px;
      color: $text-tertiary;
    }

    &__value {
      font-size: 13px;
      font-weight: 500;
      font-family: $font-JetBrainsMono;

      &--positive {
        color: $text-success;
      }

      &--negative {
        color: $text-danger;
      }
    }

    &--positive {
      color: $text-success;
    }

    &--negative {
      color: $text-danger;
    }

    &--warning {
      color: $text-warning;
    }
  }
}
</style>
