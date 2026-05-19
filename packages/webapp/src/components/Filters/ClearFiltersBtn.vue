<script lang="ts" setup>
const { scope = 'markets' } = defineProps<{ scope?: FilterScope }>()

const filterStore = useMarketFilterStore()

const isHasActiveFilters = computed(() => {
  return filterStore.isActiveCollateralFilter(scope) || filterStore.isActiveDebtFilter(scope)
})

function clearFilters() {
  filterStore.clearFilter(scope, 'collateral')
  filterStore.clearFilter(scope, 'debt')
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

  @media (max-width: $breakpoint-xs) {
    display: none;
  }
  svg {
    width: 16px;
    height: 16px;
  }
}
</style>
