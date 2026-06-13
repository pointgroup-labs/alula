<script lang="ts" setup>
import { capitalize } from 'vue'

const { market } = defineProps<{
  market: MarketWithTableItems
}>()

const status = {
  idle: 'Idle',
  healthy: 'Healthy',
  active: 'Active',
  high: 'High',
  critical: 'Critical',
}

const router = useRouter()

const marketsStore = useMarketsStore()

const marketStats = computed(() => {
  const supplied = market.marketSize.supplied
  const borrowed = market.marketSize.borrowed

  return {
    supplied,
    borrowed,
    utilization: borrowed / supplied * 100,
  }
})

const marketAssets = computed(() => {
  return market.tableItems.map(p => p.asset)
})

const largestPool = computed(() => {
  const pools = market.tableItems
  const prepared = pools?.map(p => ({ name: p.asset.symbol, value: p.total_supply * p.price }))
    ?.toSorted((a, b) => b.value - a.value)
  const bestPool = prepared[0]
  const bestPoolValue = bestPool?.value ?? 0
  const totalValue = prepared?.reduce((acc, p) => acc + p.value, 0)
  const percentage = (bestPoolValue / totalValue) * 100
  return {
    name: bestPool?.name ?? '-',
    value: percentage,
  }
})

const marketAssetsExtraCount = computed(() => marketAssets.value.length - 2)

const marketStatus = computed(() => {
  const utilization = marketStats.value.utilization
  switch (true) {
    case utilization >= 95: return status.critical
    case utilization >= 80: return status.high
    case utilization >= 50: return status.active
    case utilization >= 20: return status.healthy
    default: return status.idle
  }
})

function handleStats() {
  const currentMarket = marketsStore.state.markets[market.marketName]
  if (!currentMarket) {
    return
  }
  router.push(`/statistics/${currentMarket.address}`)
}
</script>

<template>
  <div class="market-info-main-card">
    <div class="market-info__top">
      <div class="market-name">
        {{ capitalize(market.marketName) }}
      </div>

      <div class="market-assets">
        <img
          v-for="asset in marketAssets.slice(0, 2)"
          :key="asset.symbol"
          :src="asset.icon"
          alt="pool asset"
        >
        <span v-if="marketAssetsExtraCount > 0">
          +{{ marketAssetsExtraCount }}
        </span>
      </div>
    </div>

    <div class="market-info__body">
      <div class="market-main-stats">
        <div
          v-for="k in Object.keys(marketStats)"
          :key="k"
          class="stats-item"
        >
          <div
            class="stats-item__title"
            :class="`stats-item__title--${k}`"
          >
            {{ k }}
          </div>
          <div class="stats-item__value">
            <template
              v-if="k === 'utilization'"
            >
              <j-circular-progress
                :progress="marketStats[k] ?? 0"
                :width="18"
                :stroke-width="30"
                stroke-bg="#262729"
                :stroke-color="utilRateColor(marketStats[k], 100)"
                background="transparent"
                color="#fff"
                :with-progress="false"
              />  {{ truncatePercent(marketStats[k]) }}%
            </template>
            <template v-else>
              ${{ shortenNumber(marketStats[k as keyof typeof marketStats]) }}
            </template>
          </div>
        </div>
      </div>

      <div class="market-sub-stats">
        <div class="sub-stats-item">
          <div class="sub-stats-item__title">
            Pools
          </div>
          <div class="sub-stats-item__value">
            {{ market.tableItems.length }}
          </div>
        </div>

        <div class="sub-stats-item">
          <div class="sub-stats-item__title">
            Largest Pool
          </div>
          <div class="sub-stats-item__value">
            {{ largestPool.name }} · {{ truncatePercent(largestPool.value) }}%
          </div>
        </div>

        <div class="sub-stats-item">
          <div class="sub-stats-item__title">
            Market Status
          </div>
          <div
            class="sub-stats-item__value"
            :class="`market-status market-status--${marketStatus.toLowerCase()}`"
          >
            {{ marketStatus }}
          </div>
        </div>
      </div>
    </div>

    <div class="market-info__action">
      <j-btn
        size="xs"
        variant="outlined-brand"
        @click="handleStats"
      >
        Market Stats <i-app-export-icon />
      </j-btn>
    </div>

  </div>
</template>
