<script lang="ts" setup>
defineProps<{
  title: string
  body: string
  icon?: string
  color?: string
  loading?: boolean
}>()
</script>

<template>
  <div
    class="total-card"
    :class="{ 'total-card--loading': !loading }"
  >
    <div class="total-card__title">
      {{ title }}
    </div>
    <div
      class="total-card__body text-num"
      :style="{ '--color': color }"
    >
      {{ body }}
    </div>
  </div>
</template>

<style lang="scss">
.total-card {
  border-radius: $radius-xl;
  background-color: $bg-card;
  padding: $spacing-xl;
  border: 1px solid $border-primary;
  display: flex;
  flex-direction: column;
  gap: 8px;

  &__title {
    color: $text-tertiary;
    font-size: 0.75rem;
    font-style: normal;
    font-weight: 500;
    line-height: 16px;
  }

  &__body {
    color: var(--color, '#f5f5f5');
    font-size: 1.5rem;
    font-style: normal;
    font-weight: 600;
    line-height: 28px;
  }
}
</style>
