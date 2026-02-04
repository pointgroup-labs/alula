<script lang="ts" setup>
const route = useRoute()

const marketAddress = route.params?.market as string
const poolAddress = route.params?.pool as string

const marketsStore = useMarketsStore()
const loading = computed(() => marketsStore.state.loading)

const {
  selectedPoolAddress,
  selectedPool,
  activeLeverageMarket,
} = useMultiplyTable()

const marketTabs = [{
  label: 'Overview',
  value: 'overview',
},
{
  label: 'My Position',
  value: 'position',
},
{
  label: 'Info & FAQs',
  value: 'info',
}]

const activeTab = ref(marketTabs[0])

watch(() => marketsStore.state.markets, (storeMarkets) => {
  if (!storeMarkets || Object.keys(storeMarkets).length === 0) {
    return
  }

  const markets = Object.entries(storeMarkets)
  const marketName = markets.find(([, data]) => data.address === marketAddress)?.[0]

  if (marketName && poolAddress) {
    activeLeverageMarket.value = marketName
    selectedPoolAddress.value = poolAddress
  }
}, { immediate: true })

provide('selectedPool', selectedPool)
</script>

<template>
  <main>
    <div class="market-detail-page container">
      <multiply-detail-top />

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
        </div>

        <multiply-overview v-if="activeTab?.value === 'overview'" />
        <multiply-position v-else-if="activeTab?.value === 'position'" />
        <!-- <market-overview v-if="activeTab?.value === 'overview'" />
        <info-risks-faq v-else /> -->
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
