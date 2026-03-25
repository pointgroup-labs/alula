<script lang="ts" setup>
const filterStore = useMarketFilterStore()

const uniqueAssets = computed(() => filterStore.uniqueAssets)

function toggle(symbol: string) {
  filterStore.marketToggle(filterStore.collateralFilter, symbol)
}
</script>

<template>
  <j-popover
    v-if="uniqueAssets.length > 0"
    variant="ghost"
    position="bottom"
    :teleport-to-body="false"
  >
    <div class="markets-filter-menu">
      <div
        v-for="asset in uniqueAssets"
        :key="asset.symbol"
        class="filter-item"
        @click="toggle(asset.symbol)"
      >
        <j-checkbox
          v-model="filterStore.collateralFilter[asset.symbol]"
          size="md"
          @click.stop
        />
        <img
          :src="asset.icon"
          alt="token icon"
        >
        <span>{{ asset.symbol }}</span>
      </div>
    </div>
    <template #target="{ active }">
      <j-btn
        :variant="filterStore.isActiveCollateralFilter ? 'brand-outlined' : 'ghost'"
        size="md"
        class="market-filter-btn"
      >
        Collateral Assets <i-app-accordion-arrow-down
          class="arrow-icon"
          :class="{ active }"
        />
      </j-btn>
    </template>
  </j-popover>
</template>
