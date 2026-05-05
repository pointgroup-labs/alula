import type { WatchStopHandle } from 'vue'
import type { MarketTableItem } from '~/types/table'
import { bpsToNumber, calcFee } from '@alula/client-sdk'
import { calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { calcHealthFactor, calcWeightedBorrowedUsd, truncatePercent } from '~/utils'

export function useBorrowDialog(data: MaybeRef<MarketTableItem | undefined>, isOpen: Ref<boolean>) {
  const { publicKey } = useWalletComposable()

  const marketsStore = useMarketsStore()
  const market = useMarketActions()

  const userStore = useUserStore()

  const marketClient = computed(() => marketsStore.marketClient)

  const poolData = computed(() => unref(data))

  const tokenDecimals = computed(() => poolData.value?.raw.pool.token_decimals ?? 0)

  const agreeToBorrow = useLocalStorage('agreeToBorrow', false)
  const agree = ref(false)

  const reloadFee = ref(false)
  const isLoadingFee = ref(false)
  const txFee = ref(0)

  const amount = toRef(market, 'borrowAmount')

  const isLoading = computed(() => marketsStore.poolActiveAddress === poolData.value?.raw.pool.pool_address)

  const marketName = computed(() => poolData.value?.market)
  const obligation = computed(() => userStore.state.obligations[String(marketName.value)])
  const marketState = computed(() => marketsStore.state.markets[String(marketName.value)]?.marketState)

  const poolBorrowLimit = computed(() => {
    if (!poolData.value) {
      return 0
    }

    const bpsFactor = 10_000n
    const totalAvailableAdjusted = BigInt(poolData.value.raw.total_available_adjusted)
    const totalSupply = BigInt(poolData.value.raw.total_supply)
    const totalBorrow = BigInt(poolData.value.raw.pool.total_borrowed)
    const utilRatioLimitBps = BigInt(poolData.value.raw.pool.config.health_config.utilization_ratio_limit_bps || 0)

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

    return Number(bigintToNumber(borrowLimit, poolData.value.assetDecimals))
  })

  const healthBorrowLimit = computed(() => {
    if (!poolData.value) {
      return 0
    }

    if (!obligation.value || !marketState.value) {
      return 0
    }

    const assetDecimals = tokenDecimals.value
    const oraclePriceDecimals = marketState.value.oracle_price_decimals ?? 0
    const poolsData = marketState.value.pools_data

    const depositWithOpenLTV = calcUserTotalStakeInUsd(obligation.value, poolsData, assetDecimals, oraclePriceDecimals, 'open')

    let borrowedWithLF = 0
    for (const [borrowedPoolAddress, data] of obligation.value.borrows) {
      const borrowedPool = poolsData.find(p => p.pool.pool_address === borrowedPoolAddress)
      if (!borrowedPool) {
        continue
      }
      const price = borrowedPool.oracle_asset_price
        ? bigintToNumber(borrowedPool.oracle_asset_price, oraclePriceDecimals)
        : 0
      const borrowBps = bigintToNumber(data.d_tokens * BigInt(borrowedPool.d_token_rate_ceil_bps), assetDecimals)
      const lf = bpsToNumber(Number(borrowedPool.pool.config.health_config.liability_factor_bps))
      borrowedWithLF += bpsToNumber(Number(borrowBps)) * Number(price) * lf
    }

    const positionsWithNonZeroLTV = obligation.value.deposits.filter(([poolAddr]) => {
      const pool = poolsData.find(p => p.pool.pool_address === poolAddr)
      return pool && Number(pool.pool.config.health_config.close_ltv_bps) > 0
    }).length
    const minCollateralUsd = (Number(marketState.value.global_state.min_collateral_value_cents) / 100) * positionsWithNonZeroLTV

    const borrowingCapacityUsd = Math.max(depositWithOpenLTV - borrowedWithLF - minCollateralUsd, 0)

    const price = Number(poolData.value.price)
    const liabilityFactor = bpsToNumber(Number(poolData.value.raw.pool.config.health_config.liability_factor_bps))

    const maxAvailableAssets = price > 0 && liabilityFactor > 0
      ? borrowingCapacityUsd / (price * liabilityFactor)
      : 0

    return Number(truncatePercent(maxAvailableAssets, assetDecimals))
  })

  const availableToBorrow = computed(() => {
    if (!poolData.value) {
      return 0
    }

    return Number(truncatePercent(Math.min(healthBorrowLimit.value, poolBorrowLimit.value), poolData.value.assetDecimals))
  })

  const isCanBorrow = computed(() => {
    const depositObligations = userStore.state.obligations[String(poolData.value?.market)]?.deposits ?? []
    return checkIsCanUsePool(depositObligations, poolData.value?.raw.pool.pool_address)
  })

  const dynamicHealthFactor = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }
    const assetDecimals = tokenDecimals.value
    const oraclePriceDecimals = marketState.value.oracle_price_decimals ?? 0
    const poolsData = marketState.value.pools_data

    const price = Number(poolData.value?.price || 0)
    const liabilityFactor = bpsToNumber(Number(poolData.value?.raw.pool.config.health_config.liability_factor_bps || 0))
    const borrowAdjustUsd = (amount.value || 0) * price * liabilityFactor

    return calcHealthFactor(obligation.value, poolsData, assetDecimals, oraclePriceDecimals, 0, borrowAdjustUsd)
  })

  const currentHealthFactor = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 10
    }

    const assetDecimals = tokenDecimals.value
    const oraclePriceDecimals = marketState.value.oracle_price_decimals ?? 0
    const poolsData = marketState.value.pools_data

    return calcHealthFactor(obligation.value, poolsData, assetDecimals, oraclePriceDecimals)
  })

  const collateralValueUsd = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    const assetDecimals = tokenDecimals.value
    const oraclePriceDecimals = marketState.value.oracle_price_decimals ?? 0
    const poolsData = marketState.value.pools_data

    return calcUserTotalStakeInUsd(
      obligation.value,
      poolsData,
      assetDecimals,
      oraclePriceDecimals,
    )
  })

  const currentLtv = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    const assetDecimals = tokenDecimals.value
    const oraclePriceDecimals = marketState.value.oracle_price_decimals ?? 0
    const poolsData = marketState.value.pools_data
    const collateralValue = collateralValueUsd.value

    if (collateralValue <= 0) {
      return 0
    }

    const weightedBorrowedValueUsd = calcWeightedBorrowedUsd(
      obligation.value,
      poolsData,
      assetDecimals,
      oraclePriceDecimals,
    )

    return (weightedBorrowedValueUsd / collateralValue) * 100
  })

  const maxLtv = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    const assetDecimals = tokenDecimals.value
    const oraclePriceDecimals = marketState.value.oracle_price_decimals ?? 0
    const poolsData = marketState.value.pools_data
    const collateralValue = collateralValueUsd.value

    if (collateralValue <= 0) {
      return 0
    }

    const weightedBorrowedValueUsd = calcWeightedBorrowedUsd(
      obligation.value,
      poolsData,
      assetDecimals,
      oraclePriceDecimals,
    )
    const price = Number(poolData.value?.price || 0)
    const liabilityFactor = bpsToNumber(Number(poolData.value?.raw.pool.config.health_config.liability_factor_bps || 0))
    const maxBorrowAdjustUsd = availableToBorrow.value * price * liabilityFactor

    return ((weightedBorrowedValueUsd + maxBorrowAdjustUsd) / collateralValue) * 100
  })

  const dynamicLtv = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    const assetDecimals = tokenDecimals.value
    const oraclePriceDecimals = marketState.value.oracle_price_decimals ?? 0
    const poolsData = marketState.value.pools_data
    const collateralValue = collateralValueUsd.value

    if (collateralValue <= 0) {
      return 0
    }

    const weightedBorrowedValueUsd = calcWeightedBorrowedUsd(
      obligation.value,
      poolsData,
      assetDecimals,
      oraclePriceDecimals,
    )
    const price = Number(poolData.value?.price || 0)
    const liabilityFactor = bpsToNumber(Number(poolData.value?.raw.pool.config.health_config.liability_factor_bps || 0))
    const borrowAdjustUsd = (Number(amount.value) || 0) * price * liabilityFactor

    return ((weightedBorrowedValueUsd + borrowAdjustUsd) / collateralValue) * 100
  })

  const borrowLimitUsedUsd = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    const price = Number(poolData.value?.price || 0)
    const liabilityFactor = bpsToNumber(Number(poolData.value?.raw.pool.config.health_config.liability_factor_bps || 0))
    const borrowAdjustUsd = (Number(amount.value) || 0) * price * liabilityFactor

    return Math.max(currentLtv.value >= 0
      ? calcWeightedBorrowedUsd(
        obligation.value,
        marketState.value.pools_data,
        tokenDecimals.value,
        marketState.value.oracle_price_decimals ?? 0,
      ) + borrowAdjustUsd
      : 0, 0)
  })

  const borrowLimitTotalUsd = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    const weightedBorrowedValueUsd = calcWeightedBorrowedUsd(
      obligation.value,
      marketState.value.pools_data,
      tokenDecimals.value,
      marketState.value.oracle_price_decimals ?? 0,
    )
    const price = Number(poolData.value?.price || 0)
    const liabilityFactor = bpsToNumber(Number(poolData.value?.raw.pool.config.health_config.liability_factor_bps || 0))
    const maxBorrowAdjustUsd = availableToBorrow.value * price * liabilityFactor

    return Math.max(weightedBorrowedValueUsd + maxBorrowAdjustUsd, 0)
  })

  const marketFee = computed(() => {
    const marketFeeBps = poolData.value?.raw.pool.config.fee_config.borrow_fee_bps
    return calcFee(Number(amount.value || 0), marketFeeBps || 0)
  })

  const attentionText = computed(() => {
    if (!isCanBorrow.value) {
      return 'You need to supply collateral before you can borrow.'
    }
    return availableToBorrow.value <= 0 && isCanBorrow.value ? 'You don\'t have enough collateral or available borrow limit.' : null
  })

  async function borrow() {
    if (!publicKey.value || !poolData.value?.raw.pool.pool_address) {
      return
    }
    if (!amount.value || amount.value <= 0) {
      focusInput('.borrow-input')
      return
    }

    try {
      marketsStore.poolActiveAddress = poolData.value?.raw.pool.pool_address

      const marketProps = {
        market: marketsStore.selectedMarketName,
        client: marketClient.value!,
        pool_address: poolData.value?.raw.pool.pool_address,
        amount: amount.value,
        asset_data: poolData.value?.raw.pool.name,
        poolBorrowLimit: poolBorrowLimit.value,
        withBuffer: healthBorrowLimit.value < poolBorrowLimit.value - (1 / 10 ** poolData.value.assetDecimals)
          && Number(amount.value) >= Number(availableToBorrow.value) - (1 / 10 ** poolData.value.assetDecimals),
      }

      await market.borrow(marketProps)
    } finally {
      marketsStore.poolActiveAddress = undefined
    }
  }

  let stopFeeWatcher: WatchStopHandle | undefined
  let feeInterval: ReturnType<typeof setInterval> | undefined

  function startFeeInterval() {
    clearInterval(feeInterval)
    feeInterval = setInterval(() => {
      reloadFee.value = true
      nextTick(() => { reloadFee.value = false })
    }, RELOAD_FEE_INTERVAL)
    return feeInterval
  }

  function stopFeeInterval() {
    clearInterval(feeInterval)
    feeInterval = undefined
  }

  function startBorrowWatchers() {
    stopBorrowWatchers()

    stopFeeWatcher = watchDebounced(
      [poolData, reloadFee, publicKey],
      async ([d]) => {
        if (!isOpen.value) {
          return
        }
        if (!d || !publicKey.value || !marketClient.value) {
          return
        }

        try {
          isLoadingFee.value = true
          const tx = await marketClient.value.borrowing.buildBorrowTx(
            {
              user: publicKey.value,
              seed: undefined,
            },
            d.raw.pool.pool_address || '',
            0.01,
            tokenDecimals.value,
          )
          txFee.value = marketClient.value.borrowing.getTransactionFee(tx, tokenDecimals.value)
        } finally {
          isLoadingFee.value = false
        }
      },
      { immediate: true, debounce: 300 },
    )

    startFeeInterval()
  }

  function stopBorrowWatchers() {
    stopFeeWatcher?.()
    stopFeeWatcher = undefined
    stopFeeInterval()
  }

  watch(isOpen, (open) => {
    if (open) {
      startBorrowWatchers()
    } else {
      stopBorrowWatchers()
    }
  }, { immediate: true })

  onScopeDispose(() => {
    stopBorrowWatchers()
  })

  onMounted(() => {
    agree.value = agreeToBorrow.value
  })

  return {
    agree,
    agreeToBorrow,
    obligation,
    isLoading,
    isLoadingFee,
    marketFee,
    txFee,
    amount,
    currentHealthFactor,
    dynamicHealthFactor,
    poolBorrowLimit,
    availableToBorrow,
    collateralValueUsd,
    borrowLimitUsedUsd,
    borrowLimitTotalUsd,
    currentLtv,
    maxLtv,
    dynamicLtv,
    isCanBorrow,
    attentionText,
    borrow,
  }
}
