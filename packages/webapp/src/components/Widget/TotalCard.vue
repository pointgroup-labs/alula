<script lang="ts" setup>
defineProps<{
  title: string
  body: string
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
    >
      {{ body }}
    </div>
  </div>
</template>

<style lang="scss">
.total-card {
  border-radius: $radius-md;
  background-color: $bg-card;
  padding: $spacing-md $spacing-xl;
  border: 1px solid $border-primary;
  display: flex;
  align-items: center;
  gap: 8px;

  &__title {
    color: $text-tertiary;
    font-size: 12px;
    font-style: normal;
    font-weight: 500;
  }

  &__body {
    color: $text-primary;
    font-size: 14px;
    font-style: normal;
    font-weight: 600;
  }
}
</style>
