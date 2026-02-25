<script lang="ts" setup>
const marketsStore = useMarketsStore()
const isMarkets = computed(() => Object.keys(marketsStore.state.markets).length > 0)
const isHasMarkets = ref(true)
const searchAsset = ref()
</script>

<template>
  <div class="markets">
    <market-search
      v-model="searchAsset"
    />
    <markets-list
      v-model:is-has-markets="isHasMarkets"
      :search-asset="searchAsset"
    />
    <div
      v-if="searchAsset && isMarkets && !isHasMarkets"
      class="no-markets-found"
    >
      No Markets found
    </div>
  </div>
</template>

<style lang="scss">
.markets {
  .table-wrapper {
    display: flex;
    flex-direction: column;
    gap: $spacing-16;

    @media (max-width: $breakpoint-xs) {
      gap: 32px;
    }

    .market-info-wrapper {
      display: flex;
      gap: $spacing-16;
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
        padding: $spacing-12 0 0;
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
