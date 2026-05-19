<script lang="ts" setup>
import { calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk'
import { formatPrice } from '~/utils'

const marketsStore = useMarketsStore()
const userStore = useUserStore()

const loading = computed(() => marketsStore.state.loading)

const obligations = computed(() => Object.entries(userStore.state.obligations).filter(([, obligation]) => Boolean(obligation)))
const multiplyObligations = computed(() => Object.entries(userStore.state.multiplyObligations))

const marketStatesByName = computed(() => {
  return Object.values(marketsStore.state.markets).reduce<Record<string, typeof marketsStore.state.markets[string]['marketState']>>((acc, market) => {
    if (!acc[market.marketName]) {
      acc[market.marketName] = market.marketState
    }
    return acc
  }, {})
})

const regularMetrics = computed(() => {
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

    acc.positions += obligation.positions_count ?? 0
    acc.supplied += calcUserTotalStakeInUsd(obligation, poolsData, depositAssetDecimals, oraclePriceDecimals) ?? 0
    acc.borrowed += calcUserTotalBorrowedInUsd(obligation, poolsData, borrowAssetDecimals, oraclePriceDecimals) ?? 0

    return acc
  }, {
    positions: 0,
    supplied: 0,
    borrowed: 0,
  })
})

const multiplyMetrics = computed(() => {
  return multiplyObligations.value.reduce((acc, [marketName, obligationRaw]) => {
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

      acc.positions += 1
      acc.supplied += calcUserTotalStakeInUsd(obligation, poolsData, depositAssetDecimals, oraclePriceDecimals) ?? 0
      acc.borrowed += calcUserTotalBorrowedInUsd(obligation, poolsData, borrowAssetDecimals, oraclePriceDecimals) ?? 0
    }

    return acc
  }, {
    positions: 0,
    supplied: 0,
    borrowed: 0,
  })
})

const portfolioMetrics = computed(() => {
  const supplied = regularMetrics.value.supplied + multiplyMetrics.value.supplied
  const borrowed = regularMetrics.value.borrowed + multiplyMetrics.value.borrowed

  return {
    positions: regularMetrics.value.positions + multiplyMetrics.value.positions,
    netValue: supplied - borrowed,
  }
})
</script>

<template>
  <div class="markets-info">
    <div class="markets-info__content">
      <h1>Portfolio</h1>

      <div class="markets-info__content__desc">
        Manage your positions across all strategies. Track performance, risk, and portfolio value in real time.
      </div>
    </div>
    <div class="d-flex gap-2">
      <template v-if="loading && portfolioMetrics.positions === 0 && portfolioMetrics.netValue === 0">
        <market-info-skeleton
          v-for="i in 2"
          :key="i"
        />
      </template>
      <template v-else>
        <total-card
          title="Net Value"
          :body="`$${formatPrice(portfolioMetrics.netValue, 0, 0)}`"
          :loading="loading"
        />
        <total-card
          title="Total Positions"
          :body="String(portfolioMetrics.positions)"
          :loading="loading"
        />
      </template>
    </div>
  </div>
</template>
