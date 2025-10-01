<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const selectedMarketDetails = inject('selectedMarketDetails') as Ref<MarketTableItem>

const dialog = defineModel({
  default: false,
})
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="market-info-dialog"
  >
    <template #header>
      <div class="market-info">
        <img
          :src="selectedMarketDetails?.asset.icon"
          :alt="selectedMarketDetails?.asset.symbol"
        >
        {{ selectedMarketDetails?.asset.symbol }} Pool
        <!-- <j-pill-label
          variant="secondary"
          size="md"
          bg-color="#08b57680"
        >
          Can by collateral
        </j-pill-label> -->
      </div>
    </template>

    <div class="market-info__body">
      <market-details-supply />
      <div class="separator-vert" />
      <market-details-borrow />
    </div>

    <div class="separator" />

    <market-details-bottom />
  </j-dialog>
</template>

<style lang="scss">
.market-info-dialog {
  .modal-content {
    max-width: 1104px;

    @media (max-width: $breakpoint-xs) {
      max-width: 100dvw;
      overflow-y: auto;
    }
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    gap: $spacing-16;
    overflow: initial;
  }

  .market-info__body {
    display: flex;
    gap: $spacing-24;
    padding-top: $spacing-16;

    @media (max-width: $breakpoint-xs) {
      flex-direction: column;
      gap: $spacing-8;
    }
  }

  .market-info {
    font-size: 20px;
    font-style: normal;
    font-weight: 500;
    line-height: 20px;
    display: flex;
    align-items: center;
    gap: $spacing-8;

    img {
      width: 40px;
      height: 40px;
      object-fit: contain;
      border-radius: 50%;
    }

    .j-pill-label {
      margin-left: 2px;
      font-size: 11px;
      font-style: normal;
      font-weight: 500;
      line-height: 12px;
    }
  }
}
</style>
