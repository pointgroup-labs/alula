<script lang="ts" setup>
</script>

<template>
  <div class="market-details">
    <div class="market-details__title">
      Supply Info
    </div>

    <div class="market-stats">
      <div class="market-stats__apy">
        <div class="stats-apy">
          Supply APY
          <span>7.28%</span>
        </div>
        <div class="stats-params">
          <div class="stats-params__item">
            Trust Ratio:
            <span>45%</span>
          </div>
          <div class="stats-params__item">
            Risk Floor:
            <span>24%</span>
          </div>
        </div>
      </div>

      <div class="separator-vert" />

      <market-progress :progress="52.56">
        <div class="market-progress__info">
          <div class="market-progress__info__title">
            Total Supplied
          </div>
          <div class="market-progress__info__data">
            327.92K / 800K

            <span>$1327.92K / $1800K</span>
          </div>
        </div>
      </market-progress>

      <div class="separator-vert" />

      <div class="market-penalty">
        Liquidation Penalty:

        <span>45%</span>
      </div>
    </div>

    <div class="separator" />

    <market-history-chart-supply />
  </div>
</template>

<style lang="scss">
.market-info-dialog {
  .market-details {
    width: 504px;
    display: flex;
    flex-direction: column;
    gap: $spacing-16;

    &__title {
      padding: 0 $spacing-12;
      font-size: 14px;
      font-style: normal;
      font-weight: 700;
      line-height: 16px;
    }
  }

  .market-stats {
    display: flex;
    justify-content: space-between;
    align-items: stretch;
    gap: $spacing-16;
    padding: 0 $spacing-12;

    &__apy {
      width: 96px;
      display: flex;
      flex-direction: column;
      gap: 6px;

      .stats-apy {
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

      .stats-params {
        display: flex;
        flex-direction: column;

        &__item {
          color: $neutral-6;
          font-size: 11px;
          font-style: normal;
          font-weight: 500;
          line-height: 12px;
          display: flex;
          align-items: center;
          justify-content: space-between;

          span {
            color: $dark;
            font-size: 12px;
            font-style: normal;
            font-weight: 500;
            line-height: 16px;
          }
        }
      }
    }

    .market-penalty {
      width: 105px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      color: $neutral-6;
      font-size: 11px;
      font-style: normal;
      font-weight: 500;
      line-height: 12px;

      span {
        color: $dark;
        font-size: 12px;
        font-weight: 500;
        line-height: 16px;
      }
    }
  }
}
</style>
