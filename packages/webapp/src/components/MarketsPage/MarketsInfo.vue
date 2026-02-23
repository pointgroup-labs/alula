<script lang="ts" setup>
import borrowingIcon from '~/assets/img/icons/percentage-square-icon.svg?raw'
import { formatPrice, shortenNumber } from '~/utils'

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

const marketSize = computed(() => shortenNumber(poolsInfo.value.total_collateral + poolsInfo.value.total_borrowed))

function normalizeAmount(price: number) {
  return price < 1_000_000 ? formatPrice(price, 2, 2) : shortenNumber(price)
}
</script>

<template>
  <div class="markets-info">
    <template v-if="loading">
      <market-info-skeleton
        v-for="i in 3"
        :key="i"
      />
    </template>
    <template v-else>
      <total-card
        title="Total Supply"
        :body="`$${normalizeAmount(poolsInfo.total_collateral)}`"
        bg="#006ce4"
        icon-color="#006CE4"
      />
      <total-card
        title="Total Borrow"
        :body="`$${normalizeAmount(poolsInfo.total_borrowed)}`"
        color="#111"
        bg="#ffd101"
        :icon="borrowingIcon"
        icon-color="#FFD101"
      />
         <total-card
        title="Global Market Size"
        :body="`$${marketSize}`"
        color="#111"
        bg="#ffd101"
        :icon="borrowingIcon"
        icon-color="#FFD101"
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
