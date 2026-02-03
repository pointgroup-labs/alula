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
          Price  {{ price < 1000 ? formatCompactUSD(price, 2, 5) : formatCompactUSD(price) }}
        </div>
        <template #content>
          Pool Price: {{ formatCompactUSD(price, 2, 6) }}
        </template>
      </j-tooltip>

    </div>
  </div>
</template>
