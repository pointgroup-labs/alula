<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { bpsToNumber, calculateBorrow, calculateTotalStake, calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'
import { calcHealthFactor, calcWeightedBorrowedUsd } from '~/utils'

const selectedPool = inject('selectedPool') as Ref<MarketTableItem>

const userStore = useUserStore()
const marketsStore = useMarketsStore()
const marketActions = useMarketActions()

const pool = computed(() => selectedPool.value?.raw?.pool)
const marketKey = computed(() => String(selectedPool.value?.market ?? ''))
const obligation = computed(() => userStore.state.obligations[marketKey.value])
const marketState = computed(() => marketsStore.state.markets[marketKey.value]?.marketState)
const assetDecimals = computed(() => selectedPool.value?.assetDecimals ?? marketState.value?.asset_decimals ?? 7)
const oraclePriceDecimals = computed(() => marketState.value?.oracle_price_decimals ?? 0)
const poolsData = computed(() => marketState.value?.pools_data ?? [])

const depositPosition = computed(() =>
  obligation.value?.deposits?.find(([address]) => address === pool.value?.pool_address)?.[1],
)

const borrowPosition = computed(() =>
  obligation.value?.borrows?.find(([address]) => address === pool.value?.pool_address)?.[1],
)

const positions = computed(() => {
  if (!obligation.value) {
    return null
  }
  const borrows = obligation.value.borrows?.map(([address, data]) => {
    const poolData = poolsData.value.find(p => p.pool.pool_address === address)
    const symbol = poolData?.pool?.token_symbol ?? ''
    const asset = getFullTokenData(symbol)
    const price = poolData?.oracle_asset_price
      ? Number(bigintToNumber(poolData.oracle_asset_price, oraclePriceDecimals.value))
      : 0
    const amount = data?.d_tokens && poolData
      ? Number(calculateBorrow(
          data.d_tokens,
          {
            total_d_tokens: poolData.pool.total_d_tokens,
            total_borrowed: poolData.pool.total_borrowed,
          },
          assetDecimals.value,
        ))
      : 0

    return {
      address,
      amount,
      usd: amount * price,
      ...asset,
    }
  }) ?? []
  const deposits = obligation.value.deposits?.map(([address, data]) => {
    const poolData = poolsData.value.find(p => p.pool.pool_address === address)
    const symbol = poolData?.pool?.token_symbol ?? ''
    const asset = getFullTokenData(symbol)
    const price = poolData?.oracle_asset_price
      ? Number(bigintToNumber(poolData.oracle_asset_price, oraclePriceDecimals.value))
      : 0
    const suppliedFromJTokens = data?.j_tokens && poolData
      ? Number(calculateTotalStake(
          data.j_tokens,
          {
            total_j_tokens: poolData.pool.total_j_tokens,
            total_borrowed: poolData.pool.total_borrowed,
            total_available: poolData.total_available_adjusted,
          },
          assetDecimals.value,
        ))
      : 0
    const suppliedCollateral = data?.collateral
      ? Number(bigintToNumber(BigInt(data.collateral), assetDecimals.value))
      : 0
    const amount = suppliedFromJTokens + suppliedCollateral

    return {
      address,
      amount,
      usd: amount * price,
      ...asset,
    }
  }) ?? []
  return {
    borrows,
    deposits,
  }
})

const collateralOnly = computed(() => {
  if (!depositPosition.value?.collateral) {
    return 0
  }
  return Number(bigintToNumber(BigInt(depositPosition.value.collateral), assetDecimals.value))
})

const suppliedAmount = computed(() => {
  if (!pool.value || !depositPosition.value) {
    return 0
  }

  const suppliedFromJTokens = depositPosition.value.j_tokens
    ? Number(calculateTotalStake(
        depositPosition.value.j_tokens,
        {
          total_j_tokens: pool.value.total_j_tokens,
          total_borrowed: pool.value.total_borrowed,
          total_available: selectedPool.value.raw.total_available_adjusted,
        },
        assetDecimals.value,
      ))
    : 0

  return suppliedFromJTokens + collateralOnly.value
})

const borrowedAmount = computed(() => {
  if (!pool.value || !borrowPosition.value?.d_tokens) {
    return 0
  }

  return Number(calculateBorrow(
    borrowPosition.value.d_tokens,
    {
      total_d_tokens: pool.value.total_d_tokens,
      total_borrowed: pool.value.total_borrowed,
    },
    assetDecimals.value,
  ))
})

const hasSupply = computed(() => suppliedAmount.value > 0)
const hasBorrow = computed(() => borrowedAmount.value > 0)
const hasPosition = computed(() => hasSupply.value || hasBorrow.value)

const suppliedUsd = computed(() => suppliedAmount.value * Number(selectedPool.value?.price ?? 0))
const borrowedUsd = computed(() => borrowedAmount.value * Number(selectedPool.value?.price ?? 0))

const collateralValueUsd = computed(() => {
  if (!obligation.value || !marketState.value) {
    return suppliedUsd.value
  }

  return calcUserTotalStakeInUsd(
    obligation.value,
    poolsData.value,
    assetDecimals.value,
    oraclePriceDecimals.value,
  )
})

const weightedBorrowedValueUsd = computed(() => {
  if (!obligation.value || !marketState.value) {
    return borrowedUsd.value
  }

  return calcWeightedBorrowedUsd(
    obligation.value,
    poolsData.value,
    assetDecimals.value,
    oraclePriceDecimals.value,
  )
})

const liquidationCollateralValueUsd = computed(() => {
  if (!obligation.value || !marketState.value) {
    return 0
  }

  return calcUserTotalStakeInUsd(
    obligation.value,
    poolsData.value,
    assetDecimals.value,
    oraclePriceDecimals.value,
    'close',
  )
})

const healthFactor = computed<number | null>(() => {
  if (!obligation.value || !marketState.value || obligation.value.borrows.length === 0) {
    return null
  }

  return calcHealthFactor(
    obligation.value,
    poolsData.value,
    assetDecimals.value,
    oraclePriceDecimals.value,
  )
})

const poolBorrowLimit = computed(() => {
  if (!selectedPool.value) {
    return 0
  }

  const bpsFactor = 10_000n
  const totalAvailableAdjusted = BigInt(selectedPool.value.raw.total_available_adjusted)
  const totalSupply = BigInt(selectedPool.value.raw.total_supply)
  const totalBorrow = BigInt(selectedPool.value.raw.pool.total_borrowed)
  const utilRatioLimitBps = BigInt(selectedPool.value.raw.pool.config.health_config.utilization_ratio_limit_bps || 0)

  if (totalSupply <= 0n) {
    return 0
  }

  const utilizationRatioBps = (totalBorrow * bpsFactor + totalSupply - 1n) / totalSupply
  if (utilizationRatioBps > utilRatioLimitBps) {
    return 0
  }

  const availablePercentageToBorrowBps = utilRatioLimitBps - utilizationRatioBps
  const maxBorrowByUtilization = (totalSupply * availablePercentageToBorrowBps) / bpsFactor
  let borrowLimit = totalAvailableAdjusted
  if (maxBorrowByUtilization < borrowLimit) {
    borrowLimit = maxBorrowByUtilization
  }

  return Number(bigintToNumber(borrowLimit, assetDecimals.value))
})

const healthBorrowLimit = computed(() => {
  if (!selectedPool.value || !obligation.value || !marketState.value) {
    return 0
  }

  const depositWithOpenLTV = calcUserTotalStakeInUsd(
    obligation.value,
    poolsData.value,
    assetDecimals.value,
    oraclePriceDecimals.value,
    'open',
  )

  let borrowedWithLF = 0
  for (const [borrowedPoolAddress, data] of obligation.value.borrows) {
    const borrowedPool = poolsData.value.find(poolData => poolData.pool.pool_address === borrowedPoolAddress)
    if (!borrowedPool) {
      continue
    }

    const price = borrowedPool.oracle_asset_price
      ? bigintToNumber(borrowedPool.oracle_asset_price, oraclePriceDecimals.value)
      : 0
    const borrowBps = bigintToNumber(data.d_tokens * BigInt(borrowedPool.d_token_rate_ceil_bps), assetDecimals.value)
    const liabilityFactor = bpsToNumber(Number(borrowedPool.pool.config.health_config.liability_factor_bps))
    borrowedWithLF += bpsToNumber(Number(borrowBps)) * Number(price) * liabilityFactor
  }

  const positionsWithNonZeroLTV = obligation.value.deposits.filter(([poolAddress]) => {
    const depositPool = poolsData.value.find(poolData => poolData.pool.pool_address === poolAddress)
    return depositPool && Number(depositPool.pool.config.health_config.close_ltv_bps) > 0
  }).length

  const minCollateralUsd = (Number(marketState.value.global_state.min_collateral_value_cents) / 100) * positionsWithNonZeroLTV
  const borrowingCapacityUsd = Math.max(depositWithOpenLTV - borrowedWithLF - minCollateralUsd, 0)

  const price = Number(selectedPool.value.price)
  const liabilityFactor = bpsToNumber(Number(selectedPool.value.raw.pool.config.health_config.liability_factor_bps))
  const maxAvailableAssets = price > 0 && liabilityFactor > 0
    ? borrowingCapacityUsd / (price * liabilityFactor)
    : 0

  return Number(truncatePercent(maxAvailableAssets, assetDecimals.value))
})

const availableToBorrow = computed(() =>
  Number(truncatePercent(Math.min(healthBorrowLimit.value, poolBorrowLimit.value), assetDecimals.value)),
)

const borrowLimit = computed(() => {
  return availableToBorrow.value + borrowedAmount.value
})

const currentLtv = computed(() => {
  if (collateralValueUsd.value <= 0) {
    return 0
  }

  return (weightedBorrowedValueUsd.value / collateralValueUsd.value) * 100
})

const liquidationLtv = computed(() => {
  if (collateralValueUsd.value <= 0) {
    return 0
  }

  return (liquidationCollateralValueUsd.value / collateralValueUsd.value) * 100
})

const liquidationBufferPercent = computed(() => {
  return Math.max(liquidationLtv.value - currentLtv.value, 0)
})

const liquidationBufferUsd = computed(() => {
  return Math.max(liquidationCollateralValueUsd.value - weightedBorrowedValueUsd.value, 0)
})

const currentPoolCloseCollateralUsd = computed(() => {
  const currentPrice = Number(selectedPool.value?.price ?? 0)
  const closeLtv = bpsToNumber(Number(selectedPool.value?.raw?.pool?.config?.health_config?.close_ltv_bps ?? 0))

  if (suppliedAmount.value <= 0 || currentPrice <= 0 || closeLtv <= 0) {
    return 0
  }

  return suppliedAmount.value * currentPrice * closeLtv
})

const currentPoolWeightedBorrowUsd = computed(() => {
  if (!borrowPosition.value?.d_tokens) {
    return 0
  }

  const currentPrice = Number(selectedPool.value?.price ?? 0)
  const dTokenRateCeilBps = selectedPool.value?.raw?.d_token_rate_ceil_bps
  const liabilityFactor = bpsToNumber(Number(selectedPool.value?.raw?.pool?.config?.health_config?.liability_factor_bps ?? 0))

  if (!dTokenRateCeilBps || currentPrice <= 0 || liabilityFactor <= 0) {
    return 0
  }

  const borrowedBaseAmount = bpsToNumber(Number(bigintToNumber(
    borrowPosition.value.d_tokens * BigInt(dTokenRateCeilBps),
    assetDecimals.value,
  )))

  return borrowedBaseAmount * currentPrice * liabilityFactor
})

const currentPoolWeightedBorrowPerPrice = computed(() => {
  if (!borrowPosition.value?.d_tokens) {
    return 0
  }

  const dTokenRateCeilBps = selectedPool.value?.raw?.d_token_rate_ceil_bps
  const liabilityFactor = bpsToNumber(Number(selectedPool.value?.raw?.pool?.config?.health_config?.liability_factor_bps ?? 0))

  if (!dTokenRateCeilBps || liabilityFactor <= 0) {
    return 0
  }

  const borrowedBaseAmount = bpsToNumber(Number(bigintToNumber(
    borrowPosition.value.d_tokens * BigInt(dTokenRateCeilBps),
    assetDecimals.value,
  )))

  return borrowedBaseAmount * liabilityFactor
})

const liquidationPrice = computed<number | null>(() => {
  if (healthFactor.value === null || (!hasSupply.value && !hasBorrow.value)) {
    return null
  }

  const closeLtv = bpsToNumber(Number(selectedPool.value?.raw?.pool?.config?.health_config?.close_ltv_bps ?? 0))
  const otherCloseCollateralUsd = Math.max(liquidationCollateralValueUsd.value - currentPoolCloseCollateralUsd.value, 0)
  const otherWeightedBorrowUsd = Math.max(weightedBorrowedValueUsd.value - currentPoolWeightedBorrowUsd.value, 0)

  const collateralPerPrice = suppliedAmount.value * closeLtv
  const borrowPerPrice = currentPoolWeightedBorrowPerPrice.value

  const priceSensitivity = collateralPerPrice - borrowPerPrice
  if (priceSensitivity === 0) {
    return null
  }

  const price = (otherWeightedBorrowUsd - otherCloseCollateralUsd) / priceSensitivity
  const liqPrice = Number.isFinite(price) && price > 0 ? price : null
  return Math.max(liqPrice ?? 0, 0)
})

const positionLabel = computed(() => {
  if (hasBorrow.value) {
    return 'Borrow'
  }
  if (hasSupply.value) {
    return 'Supply'
  }
  return 'No Active Position'
})

const statusPillStyle = computed(() => {
  if (hasSupply.value && hasBorrow.value) {
    return {
      '--color': '#8a8df4',
      '--background-color': 'rgb(138 141 244 / 10%)',
    }
  }
  if (hasBorrow.value) {
    return {
      '--color': '#8a8df4',
      '--background-color': 'rgb(138 141 244 / 10%)',
    }
  }
  if (hasSupply.value) {
    return {
      '--color': '#22d3ee',
      '--background-color': 'rgb(34 211 238 / 10%)',
    }
  }
  return {
    '--color': '#6b7994',
    '--background-color': 'rgb(107 121 148 / 15%)',
  }
})

const healthIndicatorStyle = computed(() => ({
  '--indicator-width': `${Math.min(Math.max((((healthFactor.value ?? 1) - 1) * 100), 0), 100)}%`,
  '--indicator-color': healthFactorColor(healthFactor.value),
}))

function openAction(action: 'supply' | 'withdraw' | 'borrow' | 'repay') {
  marketsStore.selectedMarketName = marketKey.value
  marketsStore.selectedPoolAddress = selectedPool.value.pool_address

  if (action === 'supply') {
    marketsStore.dialogSupply = true
    return
  }
  if (action === 'withdraw') {
    marketsStore.dialogWithdraw = true
    return
  }
  if (action === 'borrow') {
    marketsStore.dialogBorrow = true
    return
  }
  marketsStore.dialogRepay = true
}
</script>

<template>
  <section id="my-position">
    <div class="pool-card stat-card position-card">
      <div class="stat-card__header">
        <div class="position-card__heading">
          <div class="position-card__asset">
            <img
              :src="selectedPool?.asset.icon"
              :alt="selectedPool?.asset.symbol"
            >
            <div class="position-card__title-wrap">
              <h3 class="pool-card-title">
                My Position
              </h3>
            </div>
          </div>

          <div
            class="pool-card-pill"
            :style="statusPillStyle"
          >
            {{ positionLabel }}
          </div>

          <div
            v-if="hasPosition"
            class="position-actions"
          >
            <j-btn
              v-if="hasSupply"
              size="sm"
              variant="brand-outlined"
              :disabled="marketActions.isDisabled(selectedPool.pool_address, 'deposit', selectedPool.market!)"
              :loading="marketActions.isLoading(selectedPool.pool_address, 'deposit', selectedPool.market!)"
              @click="openAction('supply')"
            >
              Supply
            </j-btn>
            <j-btn
              v-else
              size="sm"
              variant="brand-secondary-outlined"
              :disabled="marketActions.isDisabled(selectedPool.pool_address, 'repay', selectedPool.market!)"
              :loading="marketActions.isLoading(selectedPool.pool_address, 'repay', selectedPool.market!)"
              @click="openAction('repay')"
            >
              Repay
            </j-btn>
            <j-btn
              v-if="hasBorrow"
              size="sm"
              variant="brand-secondary-outlined"
              :disabled="marketActions.isDisabled(selectedPool.pool_address, 'borrow', selectedPool.market!)"
              :loading="marketActions.isLoading(selectedPool.pool_address, 'borrow', selectedPool.market!)"
              @click="openAction('borrow')"
            >
              Borrow
            </j-btn>
            <j-btn
              v-else
              size="sm"
              variant="brand-outlined"
              :disabled="marketActions.isDisabled(selectedPool.pool_address, 'withdraw', selectedPool.market!)"
              :loading="marketActions.isLoading(selectedPool.pool_address, 'withdraw', selectedPool.market!)"
              @click="openAction('withdraw')"
            >
              Withdraw
            </j-btn>
          </div>
        </div>
      </div>

      <div class="stat-card__body">
        <template v-if="hasPosition">
          <div class="position-layout">

            <position-overview
              :collateral-value-usd="collateralValueUsd"
              :positions="positions"
              :weighted-borrowed-value-usd="weightedBorrowedValueUsd"
              :health-factor="healthFactor"
              :health-indicator-style="healthIndicatorStyle"
            />

            <position-borrow-limits
              :borrow-limit="borrowLimit"
              :selected-pool="selectedPool"
              :borrowed-amount="borrowedAmount"
              :available-to-borrow="availableToBorrow"
            />

            <position-risk-metrics
              :current-ltv="currentLtv"
              :liquidation-ltv="liquidationLtv"
              :liquidation-buffer-percent="liquidationBufferPercent"
              :liquidation-buffer-usd="liquidationBufferUsd"
              :health-factor="healthFactor"
              :liquidation-price="liquidationPrice"
            />

          </div>
        </template>

        <template v-else>
          <div class="empty-state">
            <div class="empty-state__title">
              No active position in this pool
            </div>
            <div class="empty-state__text">
              Supply to start earning yield, or borrow once you have collateral in this market.
            </div>
          </div>
        </template>

      </div>
    </div>

    <client-only>
      <supply-dialog
        v-model="marketsStore.dialogSupply"
        :data="selectedPool"
      />

      <borrow-dialog
        v-model="marketsStore.dialogBorrow"
        :data="selectedPool"
      />

      <withdraw-dialog v-model="marketsStore.dialogWithdraw" />
      <repay-dialog v-model="marketsStore.dialogRepay" />
    </client-only>
  </section>
</template>

<style lang="scss">
section#my-position {
  .pool-card {
    background-color: $bg-card;
    .stat-card__header {
      gap: 12px;
    }

    &-title {
      font-size: 14px;
    }

    &-pill {
      font-size: $text-xs;
      padding: 2px 6px;
      border-radius: 50px;
      color: var(--color);
      background-color: var(--background-color);
      white-space: nowrap;
    }
  }

  .position-card__heading {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 16px;

    @media (max-width: $breakpoint-sm) {
      flex-direction: column;
      align-items: flex-start;
    }
  }

  .position-card__asset {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;

    img {
      width: 38px;
      height: 38px;
      border-radius: 50%;
      flex-shrink: 0;
    }
  }

  .position-card__title-wrap {
    min-width: 0;

    .subtitle {
      margin: 4px 0 0;
      color: $text-tertiary;
      font-size: 12px;
    }
  }

  .position-layout {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: auto auto;
    gap: 16px;
    margin-bottom: 16px;

    @media (max-width: $breakpoint-md) {
      grid-template-columns: 1fr;
    }
  }

  .position-panel {
    padding: $spacing-xl;
    display: flex;
    flex-direction: column;
    background-color: $bg-tertiary;
    border-color: $border-secondary;

    &--overview {
      grid-row: span 2;

      @media (max-width: $breakpoint-md) {
        grid-column: span 1;
      }
    }

    &__eyebrow {
      color: $text-primary;
      font-size: 12px;
      font-weight: 700;
      padding-bottom: $spacing-md;
    }
  }

  .overview-metric {
    display: flex;
    flex-direction: column;
    gap: 8px;

    &__title {
      color: $text-tertiary;
      font-size: 12px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
    }

    &__value {
      font-family: $font-JetBrainsMono;
      font-size: 14px;
      font-weight: 700;
      color: $text-primary;
    }

    &__list {
      display: flex;
      flex-direction: column;
      max-height: 120px;
      overflow: auto;
    }

    &__item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: $spacing-md 0 $spacing-md $spacing-lg;

      img {
        width: 24px;
        height: 24px;
        object-fit: contain;
        border-radius: 50%;
      }

      .value {
        color: $text-tertiary;
        font-family: $font-JetBrainsMono;
        font-size: 12px;
        font-style: normal;
        font-weight: 400;
        line-height: normal;
      }
    }

    [class*='tooltip'] {
      cursor: default;
    }
  }

  .health-highlight {
    padding-top: 12px;
    border-top: 1px solid $border-primary;
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: auto;

    &__meta {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
    }

    &__label {
      color: $text-tertiary;
      font-size: 12px;
      display: flex;
      align-items: center;
      gap: 6px;
    }

    &__status {
      font-size: 12px;
      font-weight: 600;
      letter-spacing: 0.06em;
    }

    &__value-row {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
    }

    &__value {
      font-family: $font-JetBrainsMono;
      font-size: 14px;
      font-weight: 700;
      line-height: 1;
      color: $text-primary;
    }
  }

  .metric-list {
    display: flex;
    flex-direction: column;
    gap: 0;

    &__item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
      padding: $spacing-md 0;

      &:last-child {
        padding-bottom: 0;

        .metric-list__value {
          font-size: 14px;
          font-weight: 500;
        }
      }
    }

    &__label {
      color: $text-tertiary;
      font-size: 12px;
    }

    &__value {
      color: $text-primary;
      font-family: $font-JetBrainsMono;
      font-size: 12px;
      font-weight: 400;
      text-align: right;

      &--stacked {
        display: flex;
        flex-direction: column;
        align-items: flex-end;

        span {
          color: $text-primary;
          font-family: $font-JetBrainsMono;
          font-size: 12px;
          font-weight: 700;
        }

        small {
          color: $text-tertiary;
          font-family: $font-JetBrainsMono;
          font-size: 12px;
          font-style: normal;
          font-weight: 400;
          line-height: normal;
        }
      }
    }

    .separator {
      margin: $spacing-md 0;
    }
  }

  .hf-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .hf-indicator {
    position: relative;
    width: 120px;
    height: 4px;
    border-radius: $radius-lg;
    background-color: color-mix(in oklab, $border-primary 70%, transparent);
    overflow: hidden;
    flex-shrink: 0;

    &::after {
      content: '';
      position: absolute;
      top: 0;
      right: 0;
      width: var(--indicator-width, 0%);
      height: 100%;
      border-radius: $radius-lg;
      background-color: var(--indicator-color, var(--hf-success));
      transition:
        width 0.3s ease,
        background-color 0.3s ease;
    }
  }

  .supply-color {
    color: $cyan !important;
  }

  .borrow-color {
    color: $indigo !important;
  }

  .empty-state {
    padding: 16px;
    border: 1px solid $border-primary;
    border-radius: $radius-xl;
    margin-bottom: 18px;
    text-align: center;

    &__title {
      color: $text-primary;
      font-size: 16px;
      font-weight: 600;
      margin-bottom: 4px;
    }

    &__text {
      color: $text-tertiary;
      font-size: 13px;
      line-height: 1.5;
    }
  }

  .no-borrow-card {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    border: 1px solid $border-secondary;
    height: 100%;
    border-radius: $radius-md;
    color: $text-tertiary;
    font-size: 16px;
    font-style: normal;
    font-weight: 500;
    line-height: 20px;
    gap: 12px;
  }

  .position-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
    flex-wrap: wrap;
    margin-left: auto;
  }
}
</style>
