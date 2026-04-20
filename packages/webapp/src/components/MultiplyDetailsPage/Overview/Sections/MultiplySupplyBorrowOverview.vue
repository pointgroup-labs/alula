<template>
  <section id="multiply-detail-overview">
    <multiply-supply-overview />
    <multiply-borrow-overview />
  </section>
</template>

<style lang="scss">
section#multiply-detail-overview {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;

  @media (max-width: $breakpoint-sm) {
    grid-template-columns: 1fr;
  }

  .pool-card {
    .stat-card__header {
      gap: 12px;
    }

    .stat-card__body {
      display: flex;
      flex-direction: column;
      gap: 8px;
    }

    &-title {
      font-size: 14px;
    }
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
        border-bottom: 1px solid $border-primary;
      }

      &__label {
        color: $text-tertiary;
        display: flex;
        align-items: center;
        gap: 6px;
      }

      &__value {
        color: $text-primary;
        font-family: $font-JetBrainsMono;
        opacity: 0.9;
      }

      a {
        width: 14px;
        height: 14px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: $text-primary;
      }
    }
  }
}
</style>
