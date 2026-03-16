<script lang="ts" setup>
const route = useRoute()
const router = useRouter()

const marketAddress = route.params?.market as string
const poolAddress = route.params?.pool as string

const marketsStore = useMarketsStore()
const loading = computed(() => marketsStore.state.loading)
const isMarketsLoaded = computed(() => Object.keys(marketsStore.state.markets).length > 0)

const {
  selectedMarketName,
  selectedPoolAddress,
  selectedPool,
} = useMarketTable()

provide('selectedPool', selectedPool)

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

const tabs = [{
  label: 'Pool',
  value: 'pool',
},
{
  label: 'Info / Risks',
  value: 'info',
},
{
  label: 'Position',
  value: 'position',
}]

const defaultTab = tabs[0]!

function resolveTab(tabValue?: string | null) {
  return tabs.find(tab => tab.value === tabValue) ?? defaultTab
}

const activeTab = ref(resolveTab(route.query.activeTab as string | undefined))

watch(activeTab, (tab) => {
  if (!tab?.value) {
    return
  }

  if (route.query.activeTab === tab.value) {
    return
  }

  router.replace({
    path: route.path,
    query: {
      ...route.query,
      activeTab: tab.value,
    },
  })
}, { deep: true })

watch(() => route.query.activeTab, (tabValue) => {
  const nextTab = resolveTab(tabValue as string | undefined)
  if (activeTab.value?.value === nextTab?.value) {
    return
  }

  activeTab.value = nextTab
}, { immediate: true })
</script>

<template>
  <main>
    <pool-detail-skeleton v-if="loading && !isMarketsLoaded" />
    <div
      v-else
      class="market-detail-page container"
    >
      <pool-detail-top />

      <template v-if="selectedPool">
        <j-line-tab
          v-model="activeTab"
          :tabs="tabs"
          style="margin-bottom: -12px;"
        />
        <pool-overview v-if="activeTab?.value === 'pool'" />
        <pool-info-risks v-if="activeTab?.value === 'info'" />
        <my-position v-if="activeTab?.value === 'position'" />
      </template>

      <div
        v-else-if="!selectedPool && isMarketsLoaded"
        class="no-data"
      >
        Market or Pool not found
      </div>
    </div>
  </main>
</template>
