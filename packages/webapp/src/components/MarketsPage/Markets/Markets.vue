<script lang="ts" setup>
const marketsStore = useMarketsStore()
const isMarkets = ref(true)
const loading = computed(() => marketsStore.state.loading)
</script>

<template>
  <div class="markets">
    <div class="markets-control">
      <div class="market-filters">
        <collateral-assets-filter />
        <debt-assets-filter />
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
  .markets-control {
    margin: 8px 0 24px;
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;

    @media (max-width: $breakpoint-xs) {
      gap: 12px;
    }
  }
  .collapse-btn {
    @media (max-width: $breakpoint-xs) {
      display: none;
    }
  }

  .table-wrapper {
    display: flex;
    flex-direction: column;
    gap: 16px;

    @media (max-width: $breakpoint-xs) {
      gap: 32px;
    }

    .market-info-wrapper {
      display: flex;
      gap: 16px;
      margin-left: auto;

      @media (max-width: $breakpoint-xs) {
        display: none;
      }
    }

    .market-size {
      @media (max-width: $breakpoint-xs) {
        display: none;
      }
    }

    .j-accordion {
      .accordion-button {
        height: 54px;
      }

      .accordion-body {
        padding: 4px 0 0;
      }
    }

    tr {
      th:first-child {
        padding-left: 32px;
      }
      th:last-child {
        padding-left: 32px;
      }

      td:first-child {
        padding-left: 24px;
      }
      td:last-child {
        padding-right: 24px;
      }
    }
  }

  .no-markets-found {
    color: $text-secondary;
    font-size: 12px;
    font-style: normal;
    font-weight: 400;
    line-height: 16px;
    text-align: center;
  }
}
</style>
