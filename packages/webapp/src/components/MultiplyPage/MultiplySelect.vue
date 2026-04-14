<script lang="ts" setup>
const {
  multiplier,
  maxMultiply,
} = defineProps<{
  multiplier: number | string
  maxMultiply: number | string
}>()

const percent = defineModel<number>({ default: 85 })

const minPercent = computed(() => {
  const max = Number(maxMultiply)
  if (!max) {
    return 0
  }
  return Math.min(100, Math.ceil((1.1 / max) * 100))
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
        </div>
        <div class="multiply-select__hint">
          The slider stays below the computed safe maximum.
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
      :step="0.1"
    >

    <div class="multiply-select__limits">
      <span>x1.10</span>
      <span>x{{ truncatePercent(Number(maxMultiply) || 0, 2) }}</span>
    </div>
  </div>
</template>

<style lang="scss">
.multiply-select {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px;
  border: 1px solid $border-primary;
  border-radius: 20px;
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
  }

  &__hint {
    margin-top: 4px;
    font-size: 12px;
    line-height: 1.4;
    color: $text-tertiary;
  }

  &__value {
    padding: 4px 12px;
    border-radius: 999px;
    background: rgba(24, 185, 119, 0.16);
    border: 1px solid rgba(24, 185, 119, 0.22);
    color: $text-success;
    font-size: 12px;
    font-weight: 700;
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
}
</style>
