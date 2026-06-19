<script lang="ts" setup>
const marketTableStore = useMarketTableStore()
const marketsStore = useMarketsStore()

const route = useRoute()

const marketName = computed(() => {
  const marketAdderess = route.params?.market as string
  const currentMarket = Object.entries(marketsStore.state.markets).find(([, data]) => data.address === marketAdderess)
  if (!currentMarket) {
    return
  }
  return currentMarket[0]
})

const pools = computed(() => {
  if (!marketName.value) {
    return []
  }
  const items = marketTableStore.marketWithTableItems
  const pools = items.find(item => item.marketName === marketName.value)?.tableItems
  return pools ?? []
})
const isHasState = computed(() => pools.value?.length > 0)
const isLoading = computed(() => marketsStore.state.loading)

const isReady = computed(() => {
  return !isLoading.value && marketTableStore.marketWithTableItems.length > 0
})

const backRoutePath = computed(() => {
  const backPath = globalThis.history.state.back
  let isStatistics = false
  if (backPath.startsWith('/statistics/')) {
    isStatistics = true
  }
  const path = isStatistics ? '/statistics' : globalThis.history.state.back ?? `/statistics`
  return path
})
</script>

<template>
  <div class="statistics-page marker-pools-page">
    <div class="statistics-page-title">
      <back-btn
        :to="backRoutePath"
      />

      <h1>Statistics</h1>

      <market-pill v-if="marketName">
        {{ marketName }} Market
      </market-pill>
    </div>

    <div
      v-if="(isLoading && !isHasState) || !isReady"
      class="statistics-wrapper"
    >
      <market-statistics-card-skeleton
        v-for="v in 3"
        :key="v"
      />

    </div>
    <div
      v-else-if="isHasState"
      class="statistics-wrapper"
    >
      <market-statistics-card
        v-for="pool in pools"
        :key="pool.pool_address"
        :pool="pool"
      />
    </div>
    <div
      v-else
      class="no-data"
    >
      No data
    </div>
  </div>
</template>

<style lang="scss">
@import url(~/assets/styles/components/statistics-page.scss);
</style>
