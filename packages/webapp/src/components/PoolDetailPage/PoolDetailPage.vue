<script lang="ts" setup>
const route = useRoute()
const router = useRouter()

const marketAddress = route.params?.market as string
const poolAddress = route.params?.pool as string

const marketsStore = useMarketsStore()
const loading = computed(() => marketsStore.state.loading)
const isMarketsLoaded = computed(() => Object.keys(marketsStore.state.markets).length > 0)

const marketTableStore = useMarketTableStore()
const selectedPool = computed(() => marketTableStore.selectedPool)
const selectedPoolAddress = toRef(marketTableStore, 'selectedPoolAddress')
const selectedMarketName = toRef(marketTableStore, 'selectedMarketName')

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

const activeTab = ref(resolveTab(route.params.page as string | undefined))

watch(activeTab, (tab) => {
  if (!tab?.value) {
    return
  }

  if (route.params.page === tab.value) {
    return
  }

  router.push({
    params: {
      ...route.params,
      page: tab.value,
    },
  })
}, { deep: true })

watch(() => route.params.page, (tabValue) => {
  const nextTab = resolveTab(tabValue as string | undefined)
  if (activeTab.value?.value === nextTab?.value) {
    return
  }

  activeTab.value = nextTab
}, { immediate: true })
</script>

<template>
  <pool-detail-skeleton v-if="loading && !isMarketsLoaded" />
  <template
    v-else
  >
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
  </template>
</template>
