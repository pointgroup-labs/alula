<script lang="ts" setup>
const statisticChartTypes = ['total_supplied', 'total_borrowed', 'supply_apy_bps', 'borrow_apy_bps', 'tvl_usd_cents', 'utilization_bps', 'oracle_price_usd'] as const

const route = useRoute()

const { getFullTokenData } = useTokensStore()

const statisticsStore = useMarketStatisticsStore()
const pool = computed(() => statisticsStore.state.pool)

const assetData = computed(() => pool.value?.symbol ? getFullTokenData(pool.value.symbol) : undefined)

const backRoutePath = computed(() => {
  const path = globalThis.history.state.back ?? `/statistics/${route.params.market}`
  return path
})
</script>

<template>
  <div class="pool-statistic-page">
    <div class="asset-statistic__top">
      <back-btn
        :to="backRoutePath"
      />

      <div
        v-if="assetData"
        class="asset-data"
      >
        <img
          :src="assetData?.icon"
          alt="asset icon"
        >
        <div class="asset-data__coin">
          <span class="symbol">{{ assetData?.symbol }}</span>
          <span class="name">{{ assetData?.name }}</span>
        </div>
      </div>
    </div>
    <div class="asset-statistic-wrapper">
      <pool-statistics-card
        v-for="chartType in statisticChartTypes"
        :key="chartType"
        :chart-type="chartType"
      />
    </div>
  </div>
</template>

<style lang="scss">
.pool-statistic-page {
  display: flex;
  flex-direction: column;
  gap: 32px;

  .asset-statistic__top {
    min-height: 52px;
    display: flex;
    align-items: center;
    gap: 16px;

    @media (max-width: $breakpoint-xs) {
      min-height: 45px;
      gap: 12px;
    }

    .asset-data {
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 18px;
      font-weight: 500;

      img {
        width: 38px;
        height: 38px;
        object-fit: contain;
        border-radius: 50%;
      }

      &__coin {
        display: flex;
        flex-direction: column;
        align-items: flex-start;

        .name {
          color: $text-tertiary;
          font-size: 12px;
          opacity: 0.8;
        }
      }
    }
  }
  .asset-statistic-wrapper {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 32px;

    @media (max-width: $breakpoint-sm) {
      grid-template-columns: 1fr;
    }

    .pool-statistic-card {
      &:last-child {
        grid-column: span 2;

        @media (max-width: $breakpoint-sm) {
          grid-column: span 1;
        }
      }
    }
  }
}
</style>
