<script lang="ts" setup>
import type { MultiplyVaultItem } from '~/types/table'

defineProps<{
  selectedVault: MultiplyVaultItem
}>()
</script>

<template>
  <section class="multiply-overview">
    <div class="overview-wrapper">
      <multiply-apy-history-chart />
      <multiply-supply-borrow-overview />
      <multiply-overview-bottom-tabs />
    </div>

    <multiply-supply-card :vault="selectedVault" />
  </section>
</template>

<style lang="scss">
section.multiply-overview {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;

  @media (max-width: $breakpoint-sm) {
    flex-direction: column-reverse;
  }

  .overview-wrapper {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 24px;

    @media (max-width: $breakpoint-sm) {
      flex-direction: column;
    }
  }

  .supply-card {
    @media (max-width: $breakpoint-sm) {
      max-width: initial;
      width: 100%;
      position: relative;
      top: 0;
    }
  }

  .stat-card {
    background-color: $bg-card;
    border: 1px solid $border-primary;
    border-radius: $radius-2xl;

    &__header {
      display: flex;
      align-items: center;
      border-bottom: 1px solid $border-primary;
      padding: 14px 20px;

      @media (max-width: $breakpoint-sm) {
        flex-wrap: wrap;
        gap: 6px;
      }

      .title {
        font-size: 14px;
        color: $text-primary;
      }
    }

    &__body {
      padding: 16px;
    }
  }
}
</style>
