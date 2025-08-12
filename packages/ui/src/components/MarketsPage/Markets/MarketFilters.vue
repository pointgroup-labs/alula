<script lang="ts" setup>
const marketsStore = useMarketsStore()
const selectedMarket = toRef(marketsStore, 'selectedMarket')
</script>

<template>
  <div class="market-filters">
    <j-btn
      v-for="market in marketsStore.state.markets"
      :key="market"
      pill
      :variant="market.toLowerCase() === selectedMarket.toLowerCase() ? 'secondary' : 'light'"
      size="sm"
      @click="selectedMarket = market"
    >
      {{ market }}
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
