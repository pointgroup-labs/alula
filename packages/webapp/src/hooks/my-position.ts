import type { ComputedRef, Ref } from 'vue'
import type { MarketTableItem } from '~/types/table'
import { calcUserTotalBorrowedInUsd } from '@alula/client-sdk'
import { bpsToNumber, calculateBorrow, calculateTotalStake, calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'
import { calcHealthFactor, calcWeightedBorrowedUsd } from '~/utils'

type PositionAsset = ReturnType<typeof getFullTokenData> & {
  address: string
  amount: number
  usd: number
}

type PositionsData = {
  borrows: PositionAsset[]
  deposits: PositionAsset[]
} | null

type MyPositionState = {
  selectedPool: Ref<MarketTableItem>
  marketKey: ComputedRef<string>
  positions: ComputedRef<PositionsData>
  suppliedAmount: ComputedRef<number>
  suppliedUsd: ComputedRef<number>
  collateralOnly: ComputedRef<number>
  collateralOnlyUsd: ComputedRef<number>
  currentPoolInterestBearingAmount: ComputedRef<number>
  currentPoolInterestBearingUsd: ComputedRef<number>
  currentPoolYearlyYieldUsd: ComputedRef<number>
  borrowedAmount: ComputedRef<number>
  borrowLimit: ComputedRef<number>
  availableToBorrow: ComputedRef<number>
  currentLtv: ComputedRef<number>
  liquidationLtv: ComputedRef<number>
  liquidationBufferPercent: ComputedRef<number>
  liquidationBufferUsd: ComputedRef<number>
  weightedBorrowedValueUsd: ComputedRef<number>
  totalBorrowedValueUsd: ComputedRef<number>
  netApy: ComputedRef<number>
  netYearlyUsd: ComputedRef<number>
  collateralValueUsd: ComputedRef<number>
  healthFactor: ComputedRef<number | null>
  liquidationPrice: ComputedRef<number | null>
  hasSupply: ComputedRef<boolean>
  hasBorrow: ComputedRef<boolean>
  hasPosition: ComputedRef<boolean>
}

const MY_POSITION_KEY = 'myPosition'

export function createMyPositionState(): MyPositionState {
  const selectedPool = inject('selectedPool') as Ref<MarketTableItem>

  const userStore = useUserStore()
  const marketsStore = useMarketsStore()

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

  const positions = computed<PositionsData>(() => {
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
  const collateralOnlyUsd = computed(() => collateralOnly.value * Number(selectedPool.value?.price ?? 0))
  const currentPoolInterestBearingAmount = computed(() => Math.max(suppliedAmount.value - collateralOnly.value, 0))
  const currentPoolInterestBearingUsd = computed(() => currentPoolInterestBearingAmount.value * Number(selectedPool.value?.price ?? 0))
  const currentPoolYearlyYieldUsd = computed(() => currentPoolInterestBearingUsd.value * bpsToNumber(Number(selectedPool.value?.raw?.apy.supply_bps || 0)))

  const interestBearingSupplyUsd = computed(() => {
    if (!obligation.value) {
      return Math.max(suppliedUsd.value - collateralOnly.value * Number(selectedPool.value?.price ?? 0), 0)
    }

    let total = 0

    for (const [address, data] of obligation.value.deposits) {
      const poolData = poolsData.value.find(p => p.pool.pool_address === address)
      if (!poolData || !data?.j_tokens) {
        continue
      }

      const amount = Number(calculateTotalStake(
        data.j_tokens,
        {
          total_j_tokens: poolData.pool.total_j_tokens,
          total_borrowed: poolData.pool.total_borrowed,
          total_available: poolData.total_available_adjusted,
        },
        assetDecimals.value,
      ))
      const price = poolData.oracle_asset_price
        ? Number(bigintToNumber(poolData.oracle_asset_price, oraclePriceDecimals.value))
        : 0

      total += amount * price
    }

    return total
  })

  const totalBorrowedValueUsd = computed(() => {
    if (!obligation.value || !marketState.value) {
      return borrowedUsd.value
    }

    return calcUserTotalBorrowedInUsd(
      obligation.value,
      poolsData.value,
      assetDecimals.value,
      oraclePriceDecimals.value,
    ) ?? 0
  })

  const supplyYearlyYieldUsd = computed(() => {
    if (!obligation.value) {
      return 0
    }

    let total = 0

    for (const [address, data] of obligation.value.deposits) {
      const poolData = poolsData.value.find(p => p.pool.pool_address === address)
      if (!poolData || !data?.j_tokens) {
        continue
      }

      const amount = Number(calculateTotalStake(
        data.j_tokens,
        {
          total_j_tokens: poolData.pool.total_j_tokens,
          total_borrowed: poolData.pool.total_borrowed,
          total_available: poolData.total_available_adjusted,
        },
        assetDecimals.value,
      ))
      const price = poolData.oracle_asset_price
        ? Number(bigintToNumber(poolData.oracle_asset_price, oraclePriceDecimals.value))
        : 0
      const supplyApy = bpsToNumber(Number(poolData.apy.supply_bps || 0))

      total += amount * price * supplyApy
    }

    return total
  })

  const borrowYearlyCostUsd = computed(() => {
    if (!obligation.value) {
      return 0
    }

    let total = 0

    for (const [address, data] of obligation.value.borrows) {
      const poolData = poolsData.value.find(p => p.pool.pool_address === address)
      if (!poolData || !data?.d_tokens) {
        continue
      }

      const amount = Number(calculateBorrow(
        data.d_tokens,
        {
          total_d_tokens: poolData.pool.total_d_tokens,
          total_borrowed: poolData.pool.total_borrowed,
        },
        assetDecimals.value,
      ))
      const price = poolData.oracle_asset_price
        ? Number(bigintToNumber(poolData.oracle_asset_price, oraclePriceDecimals.value))
        : 0
      const borrowApy = bpsToNumber(Number(poolData.apy.borrow_bps || 0))

      total += amount * price * borrowApy
    }

    return total
  })

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

  const netYearlyUsd = computed(() => supplyYearlyYieldUsd.value - borrowYearlyCostUsd.value)

  const netApy = computed(() => {
    const equityBaseUsd = collateralValueUsd.value - totalBorrowedValueUsd.value
    const fallbackBaseUsd = interestBearingSupplyUsd.value || collateralValueUsd.value
    const baseUsd = equityBaseUsd > 0 ? equityBaseUsd : fallbackBaseUsd

    if (baseUsd <= 0) {
      return 0
    }

    return (netYearlyUsd.value / baseUsd) * 100
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

  const borrowLimit = computed(() => availableToBorrow.value + borrowedAmount.value)

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

  const liquidationBufferPercent = computed(() => Math.max(liquidationLtv.value - currentLtv.value, 0))
  const liquidationBufferUsd = computed(() => Math.max(liquidationCollateralValueUsd.value - weightedBorrowedValueUsd.value, 0))

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

  return {
    selectedPool,
    marketKey,
    positions,
    suppliedAmount,
    suppliedUsd,
    collateralOnly,
    collateralOnlyUsd,
    currentPoolInterestBearingAmount,
    currentPoolInterestBearingUsd,
    currentPoolYearlyYieldUsd,
    borrowedAmount,
    borrowLimit,
    availableToBorrow,
    currentLtv,
    liquidationLtv,
    liquidationBufferPercent,
    liquidationBufferUsd,
    weightedBorrowedValueUsd,
    totalBorrowedValueUsd,
    netApy,
    netYearlyUsd,
    collateralValueUsd,
    healthFactor,
    liquidationPrice,
    hasSupply,
    hasBorrow,
    hasPosition,
  }
}

export function provideMyPosition() {
  const state = createMyPositionState()
  provide(MY_POSITION_KEY, state)
  return state
}

export function useMyPosition() {
  const state = inject<MyPositionState>(MY_POSITION_KEY)
  if (!state) {
    throw new Error('My position state was not provided')
  }
  return state
}
