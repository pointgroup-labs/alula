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
  dialogSupply,
  dialogBorrow,
} = useMarketTable()

const marketTabs = [{
  label: 'Market Overview',
  value: 'overview',
},
{
  label: 'Info & Risk',
  value: 'info',
}]

const activeTab = ref(marketTabs[0])

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

provide('selectedPool', selectedPool)
provide('selectedMarketDetails', selectedPool)
</script>

<template>
  <main>
    <div class="market-detail-page container">
      <market-detail-top />

      <j-loading-spinner
        v-if="loading"
        class="table-loading-spinner"
      >
        Loading market data...
      </j-loading-spinner>

      <template v-else-if="selectedPool && !loading">
        <div class="market-detail-header">
          <j-line-tab
            v-model="activeTab"
            :tabs="marketTabs"
          />
          <market-detail-actions @dialog-handler="dialogHandler" />
        </div>

        <market-overview v-if="activeTab?.value === 'overview'" />
        <div
          v-else
          class="market-info"
        >
          info
        </div>
      </template>

      <div
        v-else
        class="no-data"
      >
        Market or Pool not found
      </div>

      <supply-dialog
        v-model="dialogBorrow"
        :data="selectedPool"
      />

      <borrow-dialog
        v-model="dialogBorrow"
        :data="selectedPool"
      />

      <market-info-dialog
        v-if="dialogSupply"
        v-model="dialogSupply"
      />
    </div>
  </main>
</template>

<style lang="scss">
.market-detail-page {
  display: flex;
  flex-direction: column;
  gap: $spacing-32;

  .market-detail-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    border-bottom: 1px solid $neutral-5;
  }

  .no-data {
    padding: $spacing-32;
    margin: 0 auto;
  }
}
</style>
