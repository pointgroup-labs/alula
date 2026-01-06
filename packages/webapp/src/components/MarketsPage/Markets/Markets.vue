<script lang="ts" setup>
const marketsStore = useMarketsStore()
const isMarkets = computed(() => Object.keys(marketsStore.state.markets).length > 0)
const searchAsses = ref()
</script>

<template>
  <div class="markets">
    <market-search v-model="searchAsses" />
    <markets-list :search-asses="searchAsses" />
    <div
      v-if="searchAsses && isMarkets"
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

    .market-assets {
      display: flex;
      align-items: center;
      padding: $spacing-4 $spacing-12;
      background-color: rgba(255, 255, 255, 0.496);
      border-radius: 100px;
      margin-left: auto;
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

      &:has(.market-assets) {
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

body.body--dark {
  .markets {
    .table-wrapper {
      .market-assets {
        background-color: rgba(0, 0, 0, 0.259);
      }
    }
  }
}
</style>
