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
  >
    <div class="total-card__title">
      {{ title }}
    </div>
    <div
      class="total-card__body"
    >
      {{ body }}
    </div>
  </div>
</template>

<style lang="scss">
.total-card {
  height: 120px;
  border-radius: 12px;
  background: $surface-neutral-04;
  padding: $spacing-24;
  display: flex;
  flex-direction: column;
  gap: 16px;

  &__title {
    color: $text-secondary;
    font-size: 14px;
    font-style: normal;
    font-weight: 500;
    line-height: 16px;
  }

  &__body {
    color: $text-primary;
    font-size: 32px;
    font-style: normal;
    font-weight: 700;
    line-height: 28px;
  }
}
</style>
