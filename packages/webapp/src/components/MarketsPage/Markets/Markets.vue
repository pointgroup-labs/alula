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
    <client-only>
      <div
        v-if="!isMarkets && !loading"
        class="no-markets-found"
      >
        No Markets found
      </div>
    </client-only>
  </div>
</template>
