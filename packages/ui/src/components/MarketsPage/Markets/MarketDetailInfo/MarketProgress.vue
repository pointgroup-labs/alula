<script lang="ts" setup>
const {
  progress,
  color = '#006CE4',
}
= defineProps<{
  color?: string
  progress: number
}>()
</script>

<template>
  <div class="market-progress">
    <j-circular-progress
      :progress="progress"
      :width="60"
      :stroke-width="25"
      stroke-bg="#EAECF0"
      :stroke-color="color"
    />

    <slot />
  </div>
</template>

<style lang="scss">
.market-progress {
  display: flex;
  align-items: center;
  gap: $spacing-12;

  .j-circular-progress {
    font-size: 10px;
    font-style: normal;
    font-weight: 500;
    line-height: 12px;
  }

  &__info {
    display: flex;
    flex-direction: column;
    gap: $spacing-8;

    &__title {
      font-size: 12px;
      font-style: normal;
      font-weight: 700;
      line-height: 16px;
    }

    &__data {
      display: flex;
      flex-direction: column;
      font-size: 11px;
      font-style: normal;
      font-weight: 500;
      line-height: 12px;

      span {
        color: $neutral-9;
        font-size: 10px;
        font-weight: 500;
        line-height: 12px;
      }
    }
  }
}
</style>
