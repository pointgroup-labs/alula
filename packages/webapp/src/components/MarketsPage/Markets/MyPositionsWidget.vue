<script lang="ts" setup>
import { calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk'
import { calcWeightedBorrowedUsd, formatPrice, ltvColor, truncatePercent } from '~/utils'

const router = useRouter()

const userStore = useUserStore()
const marketsStore = useMarketsStore()
const { marketWithTableItems } = useMarketTable()

const obligations = computed(() => Object.entries(userStore.state.obligations).filter(([, obligation]) => Boolean(obligation)))

const positionsCount = computed(() => {
  if (obligations.value.length === 0) {
    return null
  }

  return obligations.value.reduce((acc, [, obligation]) => acc += obligation?.positions_count ?? 0, 0)
})

const marketStatesByName = computed(() => {
  return Object.values(marketsStore.state.markets).reduce<Record<string, typeof marketsStore.state.markets[string]['marketState']>>((acc, market) => {
    if (!acc[market.marketName]) {
      acc[market.marketName] = market.marketState
    }
    return acc
  }, {})
})

const metrics = computed(() => {
  return obligations.value.reduce((acc, [marketName, obligation]) => {
    const marketState = marketStatesByName.value[marketName]

    if (!obligation || !marketState) {
      return acc
    }

    const oraclePriceDecimals = marketState.oracle_price_decimals
    const poolsData = marketState.pools_data

    const depositPoolAddr = obligation.deposits?.[0]?.[0]
    const borrowPoolAddr = obligation.borrows?.[0]?.[0]

    const depositAssetDecimals = poolsData.find(data => data.pool.pool_address === depositPoolAddr)?.pool.token_decimals ?? 7
    const borrowAssetDecimals = poolsData.find(data => data.pool.pool_address === borrowPoolAddr)?.pool.token_decimals ?? 7

    acc.supplied += calcUserTotalStakeInUsd(obligation, poolsData, depositAssetDecimals, oraclePriceDecimals) ?? 0
    acc.borrowed += calcUserTotalBorrowedInUsd(obligation, poolsData, borrowAssetDecimals, oraclePriceDecimals) ?? 0
    acc.weightedBorrowed += calcWeightedBorrowedUsd(obligation, poolsData, borrowAssetDecimals, oraclePriceDecimals) ?? 0
    acc.liquidationCollateral += calcUserTotalStakeInUsd(obligation, poolsData, depositAssetDecimals, oraclePriceDecimals, 'close') ?? 0

    return acc
  }, {
    supplied: 0,
    borrowed: 0,
    weightedBorrowed: 0,
    liquidationCollateral: 0,
  })
})

const netApy = computed(() => {
  let totalSuppliedUsd = 0
  let totalEarningsUsd = 0
  let totalBorrowUsd = 0

  for (const market of marketWithTableItems.value) {
    for (const item of market.tableItems) {
      const suppliedUsd = Number(item.position.supplied || 0) * Number(item.price || 0)
      const borrowedUsd = Number(item.position.borrowed || 0) * Number(item.price || 0)
      const supplyApy = Number(String(item.deposit_apy).replace('%', '')) / 100
      const borrowApy = Number(String(item.borrow_apy).replace('%', '')) / 100

      totalSuppliedUsd += suppliedUsd
      totalEarningsUsd += suppliedUsd * supplyApy
      totalBorrowUsd += borrowedUsd * borrowApy
    }
  }

  if (totalSuppliedUsd <= 0) {
    return 0
  }

  return ((totalEarningsUsd - totalBorrowUsd) / totalSuppliedUsd) * 100
})

const netValue = computed(() => metrics.value.supplied - metrics.value.borrowed)
const currentLtv = computed(() => metrics.value.supplied > 0
  ? (metrics.value.weightedBorrowed / metrics.value.supplied) * 100
  : 0)
const liquidationLtv = computed(() => metrics.value.supplied > 0
  ? (metrics.value.liquidationCollateral / metrics.value.supplied) * 100
  : 0)
const ltvValueColor = computed(() => ltvColor(currentLtv.value, liquidationLtv.value) ?? 'inherit')

function goToPortfolio() {
  router.push('/portfolio')
}
</script>

<template>
  <div
    v-if="positionsCount"
    class="my-positions-widget"
    @click="goToPortfolio"
  >
    <div class="my-positions__info">
      <div class="my-positions__title">
        My Positions
      </div>

      <div class="my-positions__count">
        {{ positionsCount }}
      </div>
    </div>

    <div class="my-positions__metrics">
      <div class="my-positions__metric">
        <span class="my-positions__metric-label">Net Value</span>
        <span class="my-positions__metric-value">${{ formatPrice(netValue, 2, 2) }}</span>
      </div>

      <div class="my-positions__metric">
        <span class="my-positions__metric-label">Supplied</span>
        <span class="my-positions__metric-value">${{ formatPrice(metrics.supplied, 2, 2) }}</span>
      </div>

      <div class="my-positions__metric">
        <span class="my-positions__metric-label">Borrowed</span>
        <span class="my-positions__metric-value">${{ formatPrice(metrics.borrowed, 2, 2) }}</span>
      </div>

      <div class="my-positions__metric">
        <span class="my-positions__metric-label">LTV</span>
        <span
          class="my-positions__metric-value"
          :style="{ color: ltvValueColor }"
        >{{ truncatePercent(currentLtv, 2) }}%</span>
      </div>

      <div class="my-positions__metric">
        <span class="my-positions__metric-label">Net APY</span>
        <span
          class="my-positions__metric-value"
          :class="{
            'my-positions__metric-value--positive': netApy > 0,
            'my-positions__metric-value--negative': netApy < 0,
          }"
        >{{ truncatePercent(netApy, 2) }}%</span>
      </div>

      <i-app-arrow-right class="arrow-icon" />
    </div>

  </div>
</template>
