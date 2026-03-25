<script lang="ts" setup>
const filterStore = useMarketFilterStore()

const isHasActiveFilters = computed(() => {
  return filterStore.isActiveCollateralFilter || filterStore.isActiveDebtFilter
})

function clearFilters() {
  const { collateralFilter, debtFilter } = filterStore
  filterStore.clearFilter(collateralFilter)
  filterStore.clearFilter(debtFilter)
}
</script>

<template>
  <j-tooltip v-if="isHasActiveFilters">
    <j-btn
      variant="ghost"
      class="market-filter-btn clear-filter-btn"
      @click="clearFilters"
    >
      <i-app-cross-icon />
    </j-btn>

    <template #content>
      Clear Filters
    </template>
  </j-tooltip>
</template>

<style lang="scss" scoped>
.clear-filter-btn {
  padding: $spacing-lg;
  svg {
    width: 16px;
    height: 16px;
  }
}
</style>
