<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'
import { capitalize } from 'vue'

const poolData = inject<Ref<MultiplyTableItem>>('selectedPool')

const asset = computed(() => poolData?.value?.asset)
const price = computed(() => poolData?.value?.price ?? 0)
const borrowAsset = computed(() => poolData?.value?.borrowAsset)
const borrowPrice = computed(() => poolData?.value.borrowPoolPrice ?? 0)
</script>

<template>
  <div class="market-details-top">
    <back-btn to="/multiply" />

    <div
      v-if="asset"
      class="asset-data"
    >
      <img
        :src="asset?.icon"
        alt="asset icon"
      >
      {{ asset?.symbol }}/{{ borrowAsset?.symbol }} Multiply
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
          Supply Pool Address: {{ poolData?.depositPoolData.pool.pool_address }}
          <br>
          Borrow Pool Address: {{ poolData?.borrowPoolData.pool.pool_address }}
        </template>
      </j-tooltip>

      <j-tooltip>
        <div
          class="market-pill"
          style="text-transform: capitalize;"
        >
          Deposit Price  {{ price < 1000 ? formatCompactUSD(price, 2, 2) : formatCompactUSD(price) }}
        </div>
        <template #content>
          Supply: {{ formatCompactUSD(price, 2, 5) }}
        </template>
      </j-tooltip>

      <j-tooltip>
        <div
          class="market-pill"
          style="text-transform: capitalize;"
        >
          Borrow Price  {{ borrowPrice < 1000 ? formatCompactUSD(borrowPrice, 2, 2) : formatCompactUSD(borrowPrice) }}
        </div>
        <template #content>
          Price: {{ formatCompactUSD(borrowPrice, 2, 5) }}
        </template>
      </j-tooltip>

    </div>
  </div>
</template>
