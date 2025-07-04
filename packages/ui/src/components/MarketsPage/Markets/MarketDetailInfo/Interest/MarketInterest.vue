<script lang="ts" setup>
</script>

<template>
  <div class="interest-wrapper">
    <div class="interest-wrapper__title">
      Interest rate
    </div>
    <div class="interest-wrapper__details">
      <div class="interest-rate">
        Utilization Rate
        <span>7.28%</span>
      </div>

      <div class="separator-vert" />

      <a href="#" target="_blank" class="interest-link">
        Rate Strategy <i-app-export-icon />
      </a>

      <div class="separator-vert" />
    </div>
    <div class="separator" />

    <div class="interest-legend">
      <div class="interest-legend__item" :style="{ '--legend-color': '#006CE4' }">
        Borrow APR
      </div>
      <div class="interest-legend__item" :style="{ '--legend-color': '#FFD101' }">
        Utilization Rate
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.market-info-dialog {
  .interest-wrapper {
    display: flex;
    flex-direction: column;
    gap: $spacing-16;

    &__title {
      font-size: 14px;
      font-style: normal;
      font-weight: 700;
      line-height: 16px;
    }

    &__details {
      padding: 0 $spacing-12;
      display: flex;
      gap: $spacing-24;
    }

    .interest-rate {
      display: flex;
      flex-direction: column;
      gap: 2px;
      color: $neutral-12;
      font-size: 12px;
      font-style: normal;
      font-weight: 500;
      line-height: 16px;

      span {
        color: $dark;
        font-size: 20px;
        font-style: normal;
        font-weight: 700;
        line-height: 20px;
      }
    }

    .interest-link {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 10px;
      color: $neutral-6;
      font-size: 11px;
      font-style: normal;
      font-weight: 500;
      line-height: 12px;
      text-decoration: none;

      svg {
        color: $dark;
        margin-bottom: -2px;
      }
    }

    .interest-legend {
      display: flex;
      align-items: center;
      justify-content: flex-end;
      gap: $spacing-12;
      font-size: 11px;
      font-style: normal;
      font-weight: 500;
      line-height: 12px;

      &__item {
        display: flex;
        align-items: center;
        gap: $spacing-4;

        &::before {
          content: '';
          display: block;
          width: 12px;
          height: 12px;
          border-radius: 50px;
          background-color: var(--legend-color);
        }
      }
    }
  }
}
</style>
