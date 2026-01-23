<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { capitalize } from 'vue'

const poolData = inject<Ref<MarketTableItem>>('selectedPool')

const asset = computed(() => poolData?.value?.asset)
const price = computed(() => poolData?.value?.price ?? 0)
</script>

<template>
  <div class="market-details-top">
    <back-btn />

    <div
      v-if="asset"
      class="asset-data"
    >
      <img
        :src="asset?.icon"
        alt="asset icon"
      >
      {{ asset?.symbol }}
    </div>

    <div
      v-if="poolData"
      class="market-pills"
    >
      <j-tooltip>
        <div
          class="market-pill"
          style="text-transform: capitalize;"
        >
          {{ poolData?.market }} Market
        </div>
        <template #content>
          Market: {{ capitalize(poolData?.market ?? '') }}
          <br>
          Pool Address: {{ poolData?.pool_address }}
        </template>
      </j-tooltip>

      <j-tooltip>
        <div
          class="market-pill"
          style="text-transform: capitalize;"
        >
          Price  {{ price < 1000 ? formatPrice(price, 2, 6) : shortenNumber(price) }}
        </div>
        <template #content>
          Pool Price: {{ formatPrice(price, 2, 6) }}
        </template>
      </j-tooltip>

    </div>
  </div>
</template>

<style lang="scss">
.market-details-top {
  display: flex;
  align-items: center;
  gap: $spacing-16;

  .asset-data {
    display: flex;
    align-items: center;
    gap: $spacing-6;
    font-size: 22px;
    font-weight: 500;

    img {
      width: 38px;
      height: 38px;
    }
  }

  .market-pills {
    display: flex;
    align-items: center;
    gap: $spacing-8;
    margin-left: auto;

    .market-pill {
      padding: $spacing-4 $spacing-12;
      border: 1px solid;
      border-radius: 100px;
      font-size: 14px;
    }
  }
}
</style>
