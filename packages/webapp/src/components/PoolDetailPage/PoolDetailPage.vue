<script lang="ts" setup>
const route = useRoute()

const marketAddress = route.params?.market as string
const poolAddress = route.params?.pool as string

const marketsStore = useMarketsStore()
const loading = computed(() => marketsStore.state.loading)

const {
  selectedMarketName,
  selectedPoolAddress,
  selectedPool,
} = useMarketTable()

watch(() => marketsStore.state.markets, (storeMarkets) => {
  if (!storeMarkets || Object.keys(storeMarkets).length === 0) {
    return
  }
  const markets = Object.entries(storeMarkets)
  const market = markets.find(([, data]) => data.address === marketAddress)
  const pool = market?.[1]?.marketState?.pools_data?.find(p => p.pool.pool_address === poolAddress)

  selectedMarketName.value = market?.[0]
  selectedPoolAddress.value = pool?.pool.pool_address
}, { immediate: true })

provide('selectedPool', selectedPool)
</script>

<template>
  <main>
    <div class="market-detail-page container">
      <pool-detail-top />

      <j-loading-spinner
        v-if="loading && Object.keys(marketsStore.state.markets).length === 0"
        class="table-loading-spinner"
      >
        Loading market data...
      </j-loading-spinner>

      <template v-else-if="selectedPool">
        <pool-overview />
      </template>

      <div
        v-else
        class="no-data"
      >
        Market or Pool not found
      </div>
    </div>
  </main>
</template>
