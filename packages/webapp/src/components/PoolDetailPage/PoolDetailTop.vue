<script lang="ts" setup>
import { bpsToNumber } from '@alula/client-sdk'

const route = useRoute()

const { width } = useWindowSize()

const marketsStore = useMarketsStore()

const marketAddress = route.params?.market as string
const poolAddress = route.params?.pool as string

const selectedMarketName = ref()
const selectedPoolAddress = ref()

const marketTableStore = useMarketTableStore()

const marketWithTableItems = computed(() => marketTableStore.marketWithTableItems)

const selectedMarket = computed(() => marketWithTableItems.value.find(m => m.marketName === selectedMarketName.value))
const selectedPool = computed(() => selectedMarket.value?.tableItems.find(p => p.pool_address === selectedPoolAddress.value))

watch(() => marketsStore.state.markets, (storeMarkets) => {
  if (!storeMarkets || Object.keys(storeMarkets).length === 0) {
    return
  }
  const markets = Object.entries(storeMarkets)
  const market = markets.find(([, data]) => data.address === marketAddress)
  const pool = market?.[1]?.marketState?.pools_data?.find(p => p.pool.pool_address === poolAddress)

  selectedMarketName.value = market?.[0]
  selectedPoolAddress.value = pool?.pool.pool_address
}, { immediate: true })

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
      <div class="market-pill">
        {{ selectedPool?.market ?? '' }} Market
      </div>

      <div
        v-if="width > 650"
        class="market-pill"
      >
        <div class="market-name">
          Price: <span class="text-num"> ${{ formatPrice(price, 2, 2) }}</span>
        </div>
      </div>
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
        <span class="supply">{{ detailCardsData.depositApy }}</span>
      </div>

      <div class="separator-vert" />

      <div class="pool-metrics__item">
        <span>Borrow APY
          <info-tooltip>
            Annual interest rate paid for borrowing assets.
            <br>
            Rates increase as pool utilization grows.
          </info-tooltip>
        </span>
        <span class="borrow">{{ detailCardsData.borrowAPY }}</span>
      </div>
    </div>
  </div>
</template>
