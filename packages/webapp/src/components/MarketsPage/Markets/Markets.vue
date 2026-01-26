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
    gap: $spacing-12;

    .market-info-wrapper {
      display: flex;
      gap: $spacing-16;
      margin-left: auto;
    }

    .market-info-pill {
      display: flex;
      align-items: center;
      padding: $spacing-4 $spacing-12;
      background-color: rgba(255, 255, 255, 0.496);
      border-radius: 100px;
      font-size: 14px;

      p {
        padding-right: 4px;
      }

      span {
        font-size: 12px;
      }

      img {
        width: 20px;
        height: 20px;
        object-fit: contain;

        & + img {
          margin-left: -10px;
        }
      }
    }

    .market-size {
      @media (max-width: $breakpoint-xs) {
        display: none;
      }
    }

    .accordion-body {
      @media (max-width: $breakpoint-sm) {
        display: flex;
        flex-direction: column;
        gap: $spacing-16;
        padding-bottom: 1rem;
      }
    }

    .j-accordion .accordion-button {
      height: 60px;

      &:has(.market-info-wrapper) {
        i {
          margin-left: 12px;
        }
      }
    }
  }

  .no-markets-found {
    padding: $spacing-32;
    text-align: center;
  }
}

.theme-dark {
  .markets {
    .table-wrapper {
      .market-info-pill {
        background-color: rgba(0, 0, 0, 0.259);
      }
    }
  }
}
</style>
