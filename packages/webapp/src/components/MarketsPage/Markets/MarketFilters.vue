<script lang="ts" setup>
import { capitalize } from 'lodash-es'

const marketsStore = useMarketsStore()
const activeMarketFilter = toRef(marketsStore, 'activeMarketFilter')

const markets = computed(() => Object.values(marketsStore.state.markets).map(m => m.marketName) ?? [])

watch([
  activeMarketFilter,
  markets,
], ([active, markets]) => {
  if ((!active || active?.length === 0) && markets.length > 0) {
    activeMarketFilter.value = String(markets[0])
  }
}, { immediate: true })
</script>

<template>
  <div
    class="market-filters"
  >
    <j-btn
      v-for="market in markets"
      :key="market"
      pill
      :variant="market?.toLowerCase() === activeMarketFilter.toLowerCase() ? 'secondary' : 'light'"
      size="sm"
      @click="activeMarketFilter = market"
    >
      {{ capitalize(market) }}
    </j-btn>
  </div>
</template>

<style lang="scss">
.market-filters {
  font-size: 20px;
  font-style: normal;
  font-weight: 600;
  line-height: 20px;
  display: flex;
  align-items: center;
  gap: $spacing-8;

  .btn {
    padding-top: $spacing-4;
    padding-bottom: $spacing-4;

    &-light {
      background: transparent;
      border-color: transparent;

      &:active {
        background: transparent;
      }
    }

    .btn-content {
      font-size: 16px;
      font-style: normal;
      font-weight: 600;
      line-height: 20px;
      text-transform: capitalize;
    }
  }
}

body.body--dark {
  .market-filters {
    .btn-secondary {
      background: $neutral-16;
      border-color: $neutral-16;
      color: $neutral-7;
    }

    .btn-light {
      color: $neutral-7;
    }
  }
}
</style>
