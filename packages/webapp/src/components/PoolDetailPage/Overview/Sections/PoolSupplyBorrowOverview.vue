<script lang="ts" setup></script>

<template>
  <section id="pool-detail-overview">
    <supply-overview />
    <borrow-overview />
  </section>
</template>

<style lang="scss">
section#pool-detail-overview {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;

  .pool-card {
    .stat-card__header {
      gap: 12px;
    }

    &-title {
      font-size: 12px;
    }

    &-pill {
      font-size: 10px;
      padding: 2px 6px;
      border-radius: 50px;
      color: var(--color);
      background-color: var(--background-color);
      margin-left: auto;
    }

    .header-icon {
      width: 24px;
      height: 24px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: 50%;
      background-color: var(--icon-bg);
      color: var(--icon-color);

      svg {
        width: 12px;
        height: 12px;
      }
    }
  }

  .market-progress__wrapper {
    padding: 10px;
    display: flex;
    align-items: flex-start;
    background-color: color-mix(in oklab, $new-secondary 20%, transparent);
    border-radius: 16px;
  }

  .detail-list {
    display: flex;
    flex-direction: column;

    &__item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 8px 0;
      font-size: 12px;

      &:not(:last-child) {
        border-bottom: 1px solid $border-color;
      }

      &__label {
        color: $muted-foreground;
      }

      &__value {
        color: $foreground;
        font-family: $font-JetBrainsMono;
        opacity: 0.9;
      }

      a {
        width: 14px;
        height: 14px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: $foreground;
      }
    }
  }
}
</style>
