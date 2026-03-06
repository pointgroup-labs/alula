<script lang="ts" setup>
import borrowingIcon from '~/assets/img/icons/percentage-square-icon.svg?raw'
import { formatPrice } from '~/utils'

const marketsStore = useMarketsStore()

const loading = computed(() => marketsStore.state.loading)

const poolsInfo = computed(() => {
  return Object.values(marketsStore.state.markets)?.reduce((acc, { marketState }) => {
    const assetDecimals = marketState?.asset_decimals ?? 0
    const oraclePriceDecimale = marketState?.oracle_price_decimals ?? 0

    for (const data of marketState?.pools_data) {
      const price = Number(bigintToNumber(data.oracle_asset_price, oraclePriceDecimale))
      const totalSupplied = data.total_supply + data.pool.total_collateral
      const supplied = Number(bigintToNumber(totalSupplied, assetDecimals)) * price
      const borrowed = Number(bigintToNumber(data.pool.total_borrowed, assetDecimals)) * price
      acc.total_borrowed += borrowed
      acc.total_collateral += supplied
    }
    return acc
  }, { total_collateral: 0, total_borrowed: 0 })
})

const marketSize = computed(() => formatPrice(poolsInfo.value.total_collateral + poolsInfo.value.total_borrowed, 0, 0))
</script>

<template>
  <div class="markets-info">
    <template v-if="loading && poolsInfo.total_collateral === 0">
      <market-info-skeleton
        v-for="i in 3"
        :key="i"
      />
    </template>
    <template v-else>
      <total-card
        title="Total Supply"
        :body="`$${formatPrice(poolsInfo.total_collateral, 0, 0)}`"
        bg="#006ce4"
        icon-color="#006CE4"
        :loading="loading"
      />
      <total-card
        title="Total Borrow"
        :body="`$${formatPrice(poolsInfo.total_borrowed, 0, 0)}`"
        bg="#ffd101"
        :icon="borrowingIcon"
        icon-color="#FFD101"
        :loading="loading"
      />
      <total-card
        title="Global Market Size"
        :body="`$${marketSize}`"
        bg="#ffd101"
        :icon="borrowingIcon"
        icon-color="#FFD101"
        :loading="loading"
      />
    </template>
  </div>
</template>

<style lang="scss">
.markets-info {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: $spacing-24;
}
</style>
