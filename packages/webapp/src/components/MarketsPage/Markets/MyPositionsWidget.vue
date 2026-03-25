<script lang="ts" setup>
import { calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk'
import { calcWeightedBorrowedUsd, formatPrice, ltvColor, truncatePercent } from '~/utils'

const router = useRouter()

const userStore = useUserStore()
const marketsStore = useMarketsStore()

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

    const assetDecimals = marketState.asset_decimals
    const oraclePriceDecimals = marketState.oracle_price_decimals
    const poolsData = marketState.pools_data

    acc.supplied += calcUserTotalStakeInUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals) ?? 0
    acc.borrowed += calcUserTotalBorrowedInUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals) ?? 0
    acc.weightedBorrowed += calcWeightedBorrowedUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals) ?? 0
    acc.liquidationCollateral += calcUserTotalStakeInUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals, 'close') ?? 0

    return acc
  }, {
    supplied: 0,
    borrowed: 0,
    weightedBorrowed: 0,
    liquidationCollateral: 0,
  })
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
  router.push('/account')
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
    </div>

    <i-app-arrow-right class="arrow-icon" />
  </div>
</template>

<style lang="scss">
.my-positions-widget {
  margin-bottom: 24px;
  height: 54px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  background-color: $bg-card;
  padding: $spacing-lg $spacing-xl;
  border-radius: $radius-xl;
  border: 1px solid $border-primary;
  cursor: pointer;

  @media (max-width: $breakpoint-md) {
    height: auto;
    flex-wrap: wrap;
    align-items: flex-start;
  }

  .my-positions {
    &__info {
      display: flex;
      align-items: center;
      gap: 16px;
      flex-shrink: 0;
    }

    &__title {
      color: $text-primary;
      font-size: $text-sm;
      font-style: normal;
      font-weight: 500;
      line-height: normal;
    }

    &__count {
      min-width: 20px;
      height: 20px;
      border-radius: $radius-full;
      background-color: $navi-400;
      color: $text-tertiary;
      font-size: $text-xs;
      font-family: $font-JetBrainsMono;
      font-style: normal;
      font-weight: 500;
      line-height: normal;
      display: flex;
      align-items: center;
      justify-content: center;
    }

    &__metrics {
      margin-left: auto;
      display: flex;
      align-items: center;
      justify-content: flex-end;
      gap: 0;

      @media (max-width: $breakpoint-md) {
        width: 100%;
        margin-left: 0;
        justify-content: space-between;
        flex-wrap: wrap;
        row-gap: 12px;
      }
    }

    &__metric {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 0 24px;
      white-space: nowrap;

      &:not(:first-child) {
        border-left: 1px solid $border-primary;
      }

      &:last-child {
        padding-right: 0;
      }

      @media (max-width: $breakpoint-md) {
        padding: 0 16px 0 0;

        &:not(:first-child) {
          border-left: 0;
        }
      }

      @media (max-width: $breakpoint-xs) {
        padding-right: 0;
      }
    }

    &__metric-label {
      color: $text-tertiary;
      font-size: $text-xs;
      font-style: normal;
      font-weight: 400;
      line-height: normal;
    }

    &__metric-value {
      color: $text-primary;
      font-size: $text-sm;
      font-family: $font-JetBrainsMono;
      font-style: normal;
      font-weight: 700;
      line-height: normal;
    }
  }

  .arrow-icon {
    width: 14px;
    height: 14px;

    @media (max-width: $breakpoint-xs) {
      display: none;
    }

    path {
      stroke: $text-tertiary;
    }
  }
}
</style>
