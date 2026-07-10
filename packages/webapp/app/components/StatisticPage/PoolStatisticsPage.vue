<script lang="ts" setup>
const statisticChartTypes = ['total_supplied', 'total_borrowed', 'supply_apy_bps', 'borrow_apy_bps', 'tvl_usd_cents', 'utilization_bps', 'oracle_price_usd'] as const

const route = useRoute()

const { getFullTokenData } = useTokensStore()

const statisticsStore = useMarketStatisticsStore()
const pool = computed(() => statisticsStore.state.pool)
const pairPool = computed(() => statisticsStore.state.pairPool)

const assetData = computed(() => pool.value?.symbol ? getFullTokenData(pool.value.symbol) : undefined)
const pairAssetData = computed(() => pairPool.value?.symbol ? getFullTokenData(pairPool.value.symbol) : undefined)

const backRoutePath = computed(() => {
  const backPath = globalThis.history.state.back?.includes('statistics') ? undefined : globalThis.history.state.back
  const path = backPath ?? `/statistics/${route.params.market}`
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
        <img
          v-if="pairAssetData?.icon"
          :src="pairAssetData?.icon"
          alt="asset icon"
        >
        <div class="asset-data__coin">
          <span class="symbol">
            {{ assetData?.symbol }}
            <template v-if="pairAssetData?.symbol">
              / {{ pairAssetData?.symbol }}
            </template>
          </span>
          <span class="name">
            {{ assetData?.name }}
            <template v-if="pairAssetData?.symbol">
              / {{ pairAssetData?.name }}
            </template>
          </span>
        </div>
      </div>

      <compare-asset-select />
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
    flex-wrap: wrap;
    gap: 16px;

    @media (max-width: $breakpoint-xs) {
      min-height: 45px;
      gap: 12px;
    }

    .asset-data {
      display: flex;
      align-items: center;
      gap: 12px;
      font-size: 18px;
      font-weight: 500;

      img {
        width: 38px;
        height: 38px;
        object-fit: contain;
        border-radius: 50%;

        &:not(:first-child) {
          margin-left: -18px;
        }
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
