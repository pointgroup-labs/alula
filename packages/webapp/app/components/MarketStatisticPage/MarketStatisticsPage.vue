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
</script>

<template>
  <div class="market-statistic-page ">

    <div class="statistic-title">
      <h1>Statistics</h1>

      <div
        v-if="marketName"
        class="market-name-pill"
      >
        {{ marketName }} Market
      </div>
    </div>

    <div
      v-if="(isLoading && !isHasState) || !isReady"
      class="statistic-wrapper"
    >
      <market-statistics-card-skeleton
        v-for="v in 3"
        :key="v"
      />

    </div>
    <div
      v-else-if="isHasState"
      class="statistic-wrapper"
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
.market-statistic-page {
  display: flex;
  flex-direction: column;
  gap: 32px;

  .statistic-title {
    display: flex;
    align-items: center;
    gap: 24px;

    h1 {
      font-size: 32px;
      margin: 0;
    }

    .market-name-pill {
      display: flex;
      align-items: center;
      flex-direction: column;
      gap: 2px;
      padding: 4px 12px;
      font-size: $text-xs;
      color: $text-tertiary;
      letter-spacing: 0.05em;
      font-weight: 500;
      text-transform: capitalize;
      background-color: color-mix(in oklab, $secondary 60%, transparent);
      border-radius: $radius-full;

      span {
        color: $text-primary;
      }
    }
  }
  .statistic-wrapper {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: $spacing-lg;
  }

  .no-data {
    padding: 32px;
    margin: 0 auto;
    color: #c6ccd9;
    font-size: 12px;
    font-style: normal;
    font-weight: 400;
    line-height: 16px;
  }
}
</style>
