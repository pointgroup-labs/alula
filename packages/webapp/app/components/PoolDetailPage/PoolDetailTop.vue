<script lang="ts" setup>
import { bpsToNumber } from '@alula/client-sdk'

const { width } = useWindowSize()

const marketsStore = useMarketsStore()

const selectedMarketName = computed(() => marketsStore.selectedMarketName)
const selectedPoolAddress = computed(() => marketsStore.selectedPoolAddress)

const marketTableStore = useMarketTableStore()

const marketWithTableItems = computed(() => marketTableStore.marketWithTableItems)

const selectedMarket = computed(() => marketWithTableItems.value.find(m => m.marketName === selectedMarketName.value))
const selectedPool = computed(() => selectedMarket.value?.tableItems.find(p => p.pool_address === selectedPoolAddress.value))

const pool = computed(() => selectedPool.value?.raw?.pool)

const asset = computed(() => selectedPool?.value?.asset)

const price = computed(() => selectedPool.value?.price ?? 0)

const borrowCapacity = computed(() => {
  if (!pool.value) {
    return 0
  }
  const utilRatePercent = selectedPool.value?.utilization_rate_percent ?? 0
  const utilLimit = bpsToNumber(Number(selectedPool.value?.raw?.pool?.config.health_config.utilization_ratio_limit_bps) || 0) * 100
  return utilRatePercent / utilLimit * 100
})

const detailCardsData = computed(() => {
  if (!pool.value) {
    return {
      depositApy: '0.00%',
      borrowAPY: '0.00%',
      supplied: '0.00',
      utilRate: '0.00%',
    }
  }
  const depositApy = selectedPool.value?.deposit_apy ?? '0.00%'
  const borrowAPY = selectedPool.value?.borrow_apy ?? '0.00%'
  return {
    depositApy,
    borrowAPY,
    supplied: selectedPool?.value?.total_supply ?? 0,
    utilRate: `${truncatePercent(borrowCapacity.value, 2)}%`,
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

    <template v-if="selectedPool">
      <market-pill>
        {{ selectedPool?.market ?? '' }} Market
      </market-pill>

      <market-pill v-if="width > 650">
        Price: <span class="text-num"> ${{ formatPrice(price, 2, 2) }}</span>
      </market-pill>

    </template>

    <div class="pool-metrics">
      <div class="pool-metrics__item">
        <span>Supply APY
          <info-tooltip>
            Estimated annual yield earned by supplying assets.
            <br>
            Rates adjust dynamically based on pool utilization.
          </info-tooltip>
        </span>
        <j-incentive-apy
          :market-name="selectedMarketName"
          :pool-data="selectedPool?.raw"
          farm-type="supply"
        >
          {{ detailCardsData.depositApy }}
        </j-incentive-apy>
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
        <j-incentive-apy
          :market-name="selectedMarketName"
          :pool-data="selectedPool?.raw"
          farm-type="borrow"
          variant="indigo"
        >
          {{ detailCardsData.borrowAPY }}
        </j-incentive-apy>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.farms-badge {
  background-color: transparent;
}
</style>