<script lang="ts" setup>
const marketTableStore = useMarketTableStore()
const marketsStore = useMarketsStore()

const preparedMarkets = computed(() => marketTableStore.marketWithTableItems)

const marketsCount = computed(() => preparedMarkets.value.length)
const poolsCount = computed(() => preparedMarkets.value.reduce((acc, market) => acc += market.tableItems.length, 0))

const isHasState = computed(() => preparedMarkets.value?.length > 0)
const isLoading = computed(() => marketsStore.state.loading)
const isReady = computed(() => {
  return !isLoading.value && marketTableStore.marketWithTableItems.length > 0
})
</script>

<template>
  <div class="statistics-page">
    <div class="statistics-page-title">
      <back-btn
        to="/"
      />

      <h1>Statistics</h1>

      <div class="d-flex gap-2">
        <market-pill>
          Markets: <span class="text-num">{{ marketsCount }}</span>
        </market-pill>

        <market-pill>
          Pools: <span class="text-num">{{ poolsCount }}</span>
        </market-pill>
      </div>
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
      <market-statistics-main-card
        v-for="market in preparedMarkets"
        :key="market.marketName"
        :market="market"
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
