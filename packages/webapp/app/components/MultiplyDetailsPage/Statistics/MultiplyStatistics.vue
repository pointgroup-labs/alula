<script lang="ts" setup>
import type { ApiHistoryData } from '~/services/api'

const chartTypes: { type: keyof ApiHistoryData, onlyMarketAsset?: boolean, onlyPairAsset?: boolean }[] = [{
  type: 'supply_apy_bps',
  onlyMarketAsset: true,
},
{
  type: 'borrow_apy_bps',
  onlyPairAsset: true,
},
{
  type: 'oracle_price_usd',
  onlyMarketAsset: true,
}]
</script>

<template>
  <section class="multiply-statistics">
    <pool-statistics-card
      v-for="chart in chartTypes"
      :key="chart.type"
      :chart-type="chart.type"
      :only-market-asset="chart?.onlyMarketAsset"
      :only-pair-asset="chart?.onlyPairAsset"
    />
  </section>
</template>

<style lang="scss">
.multiply-statistics {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;

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
</style>
