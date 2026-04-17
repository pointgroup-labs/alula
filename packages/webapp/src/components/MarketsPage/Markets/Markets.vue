<script lang="ts" setup>
const marketsStore = useMarketsStore()

const isMarkets = ref(true)
const loading = computed(() => marketsStore.state.loading)
</script>

<template>
  <div class="markets">
    <my-positions-widget />
    <div class="markets-control">
      <div class="market-filters">
        <collateral-assets-filter />
        <debt-assets-filter />
        <clear-filters-btn />
      </div>
      <collapse-all-btn />
      <market-search />
    </div>
    <markets-list v-model:is-markets="isMarkets" />
    <div
      v-if="!isMarkets && !loading"
      class="no-markets-found"
    >
      No Markets found
    </div>
  </div>
</template>

<style lang="scss">
.markets {
  .collapse-btn {
    @media (max-width: $breakpoint-xs) {
      display: none;
    }
  }
}
</style>
