<script lang="ts" setup>
import { calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk'
import { formatPrice, truncatePercent } from '~/utils'

const router = useRouter()

const userStore = useUserStore()
const marketsStore = useMarketsStore()

const multiplyObligations = computed(() => Object.entries(userStore.state.multiplyObligations))

const positionsCount = computed(() => {
  if (multiplyObligations.value.length === 0) {
    return null
  }

  const count = multiplyObligations.value.reduce((acc, [, obligationRaw]) => {
    const obligations = Object.values(obligationRaw).filter(Boolean)
    if (obligations.length === 0) {
      return acc
    }
    acc += obligations.length
    return acc
  }, 0)

  return count
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
  const result = multiplyObligations.value.reduce((acc, [marketName, obligationRaw]) => {
    const marketState = marketStatesByName.value[marketName]
    const obligations = Object.values(obligationRaw).filter(Boolean)

    if (obligations.length === 0 || !marketState) {
      return acc
    }

    const oraclePriceDecimals = marketState.oracle_price_decimals
    const poolsData = marketState.pools_data

    for (const obligation of obligations) {
      if (!obligation) {
        continue
      }
      const depositPoolAddr = obligation.deposits?.[0]?.[0]
      const borrowPoolAddr = obligation.borrows?.[0]?.[0]

      const depositPoolData = poolsData.find(data => data.pool.pool_address === depositPoolAddr)
      const borrowPoolData = poolsData.find(data => data.pool.pool_address === borrowPoolAddr)

      const depositAssetDecimals = depositPoolData?.pool.token_decimals ?? 7
      const borrowAssetDecimals = borrowPoolData?.pool.token_decimals ?? 7

      const supplyApy = (depositPoolData?.apy.supply_bps ?? 0) / 100
      const borrowApy = (borrowPoolData?.apy.borrow_bps ?? 0) / 100

      const suppliedUsd
        = calcUserTotalStakeInUsd(obligation, poolsData, depositAssetDecimals, oraclePriceDecimals) ?? 0

      const borrowedUsd
        = calcUserTotalBorrowedInUsd(obligation, poolsData, borrowAssetDecimals, oraclePriceDecimals) ?? 0

      acc.supplied += suppliedUsd
      acc.borrowed += borrowedUsd

      const netApy = calcMultiplyObligationNetApy({
        suppliedUsd,
        borrowedUsd,
        supplyApy,
        borrowApy,
      })

      acc.weightedApySum += suppliedUsd * netApy
    }

    return acc
  }, {
    supplied: 0,
    borrowed: 0,
    weightedApySum: 0,
  })

  const netValue = result.supplied - result.borrowed

  const multiplier = netValue > 0
    ? result.supplied / netValue
    : Infinity

  const netApy = result.supplied > 0
    ? result.weightedApySum / result.supplied
    : 0

  return {
    supplied: result.supplied,
    borrowed: result.borrowed,
    netValue,
    multiplier,
    netApy,
  }
})

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
        <span class="my-positions__metric-value">${{ formatPrice(metrics.netValue, 2, 2) }}</span>
      </div>

      <div class="my-positions__metric">
        <span class="my-positions__metric-label">My Collateral</span>
        <span class="my-positions__metric-value">${{ formatPrice(metrics.supplied, 2, 2) }}</span>
      </div>

      <div class="my-positions__metric">
        <span class="my-positions__metric-label">My Debt</span>
        <span class="my-positions__metric-value">${{ formatPrice(metrics.borrowed, 2, 2) }}</span>
      </div>

      <div class="my-positions__metric">
        <span class="my-positions__metric-label">Multiplier</span>
        <span class="my-positions__metric-value text-positive">{{ truncatePercent(metrics.multiplier, 2) }}%</span>
      </div>

      <div class="my-positions__metric">
        <span class="my-positions__metric-label">Net APY</span>
        <span
          class="my-positions__metric-value"
          :class="{
            'my-positions__metric-value--positive': metrics.netApy > 0,
            'my-positions__metric-value--negative': metrics.netApy < 0,
          }"
        >{{ truncatePercent(metrics.netApy, 2) }}%</span>
      </div>
    </div>

    <i-app-arrow-right class="arrow-icon" />
  </div>
</template>
