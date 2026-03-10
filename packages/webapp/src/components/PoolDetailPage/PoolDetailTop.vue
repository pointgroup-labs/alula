<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { bpsToNumber } from '@alula/client-sdk'
import { capitalize } from 'vue'

const selectedPool = inject('selectedPool') as Ref<MarketTableItem>

const pool = computed(() => selectedPool.value?.raw?.pool)

const asset = computed(() => selectedPool?.value?.asset)

const detailCardsData = computed(() => {
  if (!pool.value) {
    return {
      depositApy: '0.00%',
      borrowAPY: '0.00%',
      utilRate: '0.00%',
    }
  }
  const depositApy = selectedPool.value?.deposit_apy ?? '0.00%'
  const borrowAPY = selectedPool.value?.borrow_apy ?? '0.00%'
  const utilRatePercent = selectedPool.value?.utilization_rate_percent
  const utilLimit = bpsToNumber(Number(selectedPool.value?.raw?.pool?.config.health_config.utilization_ratio_limit_bps) || 0) * 100
  const utilRate = utilRatePercent / utilLimit * 100
  return {
    depositApy,
    borrowAPY,
    utilRate: `${truncatePercent(utilRate, 2)}%`,
  }
})
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
      <div class="asset-data__coin">
        <span data="name">{{ asset?.symbol }}</span>
        <span data="symbol">{{ asset?.name }}</span>
      </div>
    </div>

    <div
      v-if="selectedPool"
      class="market-pills"
    >
      <div class="market-name">{{ capitalize(selectedPool?.market ?? '') }} market</div>

    </div>

    <div class="pool-metrics">
      <div class="pool-metrics__item">
        <span>Supply APY
          <info-tooltip>
            Estimated annual yield earned by supplying assets.
            <br>
            Rates adjust dynamically based on pool utilization.
          </info-tooltip>
        </span>
        <span class="supply">{{ detailCardsData.depositApy }}</span>
      </div>

      <div class="separator-vert" />

      <div class="pool-metrics__item">
        <span>Borrow APY
          <info-tooltip>
            Annual interest rate paid for borrowing assets.
            <br>
            Rates increase as pool utilization grows
          </info-tooltip>
        </span>
        <span class="borrow">{{ detailCardsData.borrowAPY }}</span>
      </div>

      <div class="separator-vert" />

      <div class="pool-metrics__item">
        <span>Borrow Capacity
          <info-tooltip>
            Percentage of the pool's borrow limit currently used.
            <br>
            At 100% no additional borrowing is possible
          </info-tooltip>
        </span>
        <span class="">{{ detailCardsData.utilRate }}</span>
      </div>
    </div>
  </div>
</template>
