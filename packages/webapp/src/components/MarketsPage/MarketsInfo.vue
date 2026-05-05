<script lang="ts" setup>
import { formatPrice } from '~/utils'

const GUIDE_LINK = 'https://docs.alula.finance/guides/'

const marketTable = useMarketTableStore()
const marketsStore = useMarketsStore()

const loading = computed(() => marketsStore.state.loading)

const marketsInfo = computed(() => {
  const activeMarkets = marketTable.filteredMarkets
  return Object.values(activeMarkets)?.reduce((acc, el) => {
    for (const data of el.tableItems) {
      acc.total_collateral += data.total_supply * data.price
      acc.total_borrowed += data.total_borrowed * data.price
    }
    return acc
  }, { total_collateral: 0, total_borrowed: 0 })
})
</script>

<template>
  <div class="markets-info">
    <div class="markets-info__content">
      <h1>Markets</h1>

      <div class="markets-info__content__desc">
        Earn yield or borrow against your assets.
        <a
          :href="GUIDE_LINK"
          target="_blank"
          rel="noopener noreferrer nofollow"
        >How it works</a>
      </div>
    </div>
    <div class="d-flex gap-2">
      <template v-if="loading && marketsInfo.total_collateral === 0">
        <market-info-skeleton
          v-for="i in 2"
          :key="i"
        />
      </template>
      <template v-else>
        <total-card
          title="Total Supply"
          :body="`$${formatPrice(marketsInfo.total_collateral, 0, 0)}`"
          :loading="loading"
        />
        <total-card
          title="Total Borrowed"
          :body="`$${formatPrice(marketsInfo.total_borrowed, 0, 0)}`"
          :loading="loading"
        />
      </template>
    </div>
  </div>
</template>
