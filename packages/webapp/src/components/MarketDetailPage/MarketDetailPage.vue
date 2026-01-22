<script lang="ts" setup>
const route = useRoute()

const marketsStore = useMarketsStore()
const loading = computed(() => marketsStore.state.loading)

const {
  selectedMarketName,
  selectedPoolAddress,
  selectedPool,
  dialogSupply,
  dialogBorrow,
} = useMarketTable()

const marketAddress = route.params?.market as string
const poolAddress = route.params?.pool as string

async function dialogHandler(marketName: string, poolAddress: string, action: 'supply' | 'borrow') {
  selectedMarketName.value = marketName
  selectedPoolAddress.value = poolAddress
  action === 'supply' ? dialogSupply.value = true : dialogBorrow.value = true
}

watch(() => marketsStore.state.markets, (storeMarkets) => {
  if (!storeMarkets) {
    return
  }
  const markets = Object.entries(storeMarkets)
  const market = markets.find(([, data]) => data.address === marketAddress)
  const pool = market?.[1]?.marketState?.pools_data?.find(p => p.pool.pool_address === poolAddress)

  selectedMarketName.value = market?.[0]
  selectedPoolAddress.value = pool?.pool.pool_address
}, { immediate: true })
</script>

<template>
  <main>
    <div class="market-detail-page container">
      <market-detail-top :pool-data="selectedPool" />

      <j-loading-spinner
        v-if="loading"
        class="table-loading-spinner"
      >
        Loading market data...
      </j-loading-spinner>

      <template v-else-if="selectedPool && !loading">
        <market-detail-actions
          :pool-data="selectedPool"
          @dialog-handler="dialogHandler"
        />
      </template>

      <div
        v-else
        class="no-data"
      >
        Market or Pool not found
      </div>

      <supply-dialog
        v-model="dialogSupply"
        :data="selectedPool"
      />

      <borrow-dialog
        v-model="dialogBorrow"
        :data="selectedPool"
      />
    </div>
  </main>
</template>

<style lang="scss">
.market-detail-page {
  display: flex;
  flex-direction: column;
  gap: $spacing-32;

  .no-data {
    padding: $spacing-32;
    margin: 0 auto;
  }
}
</style>
