import type { WatchStopHandle } from 'vue'
import { bpsToNumber } from '@alula/client-sdk'
import { calculateBorrow, calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { calcHealthFactor, calcWeightedBorrowedUsd, truncatePercent } from '~/utils'

export function useRepayDialog(isOpen: Ref<boolean>) {
  const marketsStore = useMarketsStore()
  const market = useMarketActions()
  const userStore = useUserStore()
  const { getFullTokenData } = useTokensStore()

  const {
    publicKey,
    nativeBalance,
    getAssetBalance } = useWalletComposable()

  const loading = ref(false)
  const reloadFee = ref(false)
  const isLoadingFee = ref(false)
  const txFee = ref(0)

  const amount = toRef(market, 'repayAmount')

  const marketKey = computed(() => marketsStore.selectedMarketName)
  const obligation = computed(() => userStore.state.obligations[String(marketKey.value)])
  const poolData = computed(() =>
    marketsStore.state.markets[String(marketKey.value)]?.marketState.pools_data
      ?.find(p => p.pool.pool_address === marketsStore.selectedPoolAddress),
  )

  const activeMarket = computed(() => marketsStore.state.markets[String(marketKey.value)])
  const marketState = computed(() => activeMarket.value?.marketState)

  const assetDecimals = computed(() => poolData.value?.pool.token_decimals ?? 0)
  const oraclePriceDecimals = computed(() => activeMarket.value?.marketState.oracle_price_decimals ?? 0)

  const pool_address = computed(() => poolData.value?.pool.pool_address ?? '')
  const asset = computed(() => getFullTokenData(poolData.value?.pool.token_symbol ?? ''))
  const asset_issuer = computed(() => poolData.value?.pool.name ? destructurePoolAsset(poolData.value.pool.name)[1] : '')
  const price = computed(() =>
    poolData.value?.oracle_asset_price ? bigintToNumber(poolData.value.oracle_asset_price, oraclePriceDecimals.value) : 0,
  )

  const debt = computed(() => {
    const borrow = obligation.value?.borrows?.find(([addr]) => addr === pool_address.value)
    if (!borrow || !poolData.value) {
      return 0
    }
    const [, bor] = borrow
    return Number(calculateBorrow(bor.d_tokens, {
      total_borrowed: poolData.value.pool.total_borrowed,
      total_d_tokens: poolData.value.pool.total_d_tokens,
    }, assetDecimals.value))
  })

  const debtAfterRepay = computed(() => Math.max(debt.value - (amount.value || 0), 0))

  const balance = computed(() => {
    if (!poolData.value) {
      return 0
    }
    if (asset.value.symbol === 'XLM') {
      return nativeBalance.value
    }
    return getAssetBalance(asset_issuer.value)
  })

  const liabilityFactor = computed(() => bpsToNumber(Number(poolData.value?.pool.config.health_config.liability_factor_bps || 0)))

  const currentWeightedBorrowedUsd = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    return calcWeightedBorrowedUsd(
      obligation.value,
      marketState.value.pools_data,
      assetDecimals.value,
      oraclePriceDecimals.value,
    )
  })

  const repayAdjustUsd = computed(() => (Number(amount.value) || 0) * Number(price.value) * liabilityFactor.value)

  const nextWeightedBorrowedUsd = computed(() => Math.max(currentWeightedBorrowedUsd.value - repayAdjustUsd.value, 0))

  const collateralValueUsd = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    return calcUserTotalStakeInUsd(
      obligation.value,
      marketState.value.pools_data,
      assetDecimals.value,
      oraclePriceDecimals.value,
    )
  })

  const currentHealthFactor = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 10
    }

    return calcHealthFactor(
      obligation.value,
      marketState.value.pools_data,
      assetDecimals.value,
      oraclePriceDecimals.value,
    )
  })

  const dynamicHealthFactor = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    const repayLF = bpsToNumber(Number(poolData.value?.pool.config.health_config.liability_factor_bps || 0))
    const borrowAdjustUsd = -(Number(amount.value || 0) * Number(price.value) * repayLF)

    return calcHealthFactor(
      obligation.value,
      marketState.value.pools_data,
      assetDecimals.value,
      oraclePriceDecimals.value,
      0,
      borrowAdjustUsd,
    )
  })

  const currentLtv = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    const collateralValue = collateralValueUsd.value

    if (collateralValue <= 0) {
      return 0
    }

    const weightedBorrowedValueUsd = calcWeightedBorrowedUsd(
      obligation.value,
      marketState.value.pools_data,
      assetDecimals.value,
      oraclePriceDecimals.value,
    )

    return (weightedBorrowedValueUsd / collateralValue) * 100
  })

  const dynamicLtv = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    const collateralValue = collateralValueUsd.value

    if (collateralValue <= 0) {
      return 0
    }

    return (nextWeightedBorrowedUsd.value / collateralValue) * 100
  })

  const poolBorrowLimitAfterRepay = computed(() => {
    if (!poolData.value) {
      return 0
    }

    const totalSupply = Number(bigintToNumber(poolData.value.total_supply, assetDecimals.value))
    const totalBorrow = Math.max(Number(bigintToNumber(poolData.value.pool.total_borrowed, assetDecimals.value)) - (Number(amount.value) || 0), 0)
    const totalAvailableAdjusted = Number(bigintToNumber(poolData.value.total_available_adjusted, assetDecimals.value)) + (Number(amount.value) || 0)
    const utilRatioLimitBps = Number(poolData.value.pool.config.health_config.utilization_ratio_limit_bps || 0)

    if (totalSupply <= 0) {
      return 0
    }

    const utilizationRatioBps = Math.ceil((totalBorrow * 10_000) / totalSupply)
    if (utilizationRatioBps > utilRatioLimitBps) {
      return 0
    }

    const availablePercentageToBorrowBps = utilRatioLimitBps - utilizationRatioBps
    const maxBorrowByUtilization = (totalSupply * availablePercentageToBorrowBps) / 10_000

    return Math.min(totalAvailableAdjusted, maxBorrowByUtilization)
  })

  const healthBorrowLimitAfterRepay = computed(() => {
    if (!obligation.value || !marketState.value || !poolData.value) {
      return 0
    }

    const poolsData = marketState.value.pools_data
    const depositWithOpenLTV = calcUserTotalStakeInUsd(
      obligation.value,
      poolsData,
      assetDecimals.value,
      oraclePriceDecimals.value,
      'open',
    )
    const positionsWithNonZeroLTV = obligation.value.deposits.filter(([poolAddr]) => {
      const pool = poolsData.find(p => p.pool.pool_address === poolAddr)
      return pool && Number(pool.pool.config.health_config.close_ltv_bps) > 0
    }).length
    const minCollateralUsd = (Number(marketState.value.global_state.min_collateral_value_cents) / 100) * positionsWithNonZeroLTV
    const borrowingCapacityUsd = Math.max(depositWithOpenLTV - nextWeightedBorrowedUsd.value - minCollateralUsd, 0)

    if (Number(price.value) <= 0 || liabilityFactor.value <= 0) {
      return 0
    }

    const maxAvailableAssets = borrowingCapacityUsd / (Number(price.value) * liabilityFactor.value)

    return Number(truncatePercent(maxAvailableAssets, assetDecimals.value))
  })

  const availableToBorrowAfterRepay = computed(() => {
    if (!poolData.value) {
      return 0
    }

    return Number(truncatePercent(
      Math.min(healthBorrowLimitAfterRepay.value, poolBorrowLimitAfterRepay.value),
      assetDecimals.value,
    ))
  })

  const maxLtv = computed(() => {
    if (!obligation.value || !marketState.value || !poolData.value) {
      return 0
    }

    const collateralValue = collateralValueUsd.value

    if (collateralValue <= 0) {
      return 0
    }

    const maxBorrowAdjustUsd = availableToBorrowAfterRepay.value * Number(price.value) * liabilityFactor.value

    return ((nextWeightedBorrowedUsd.value + maxBorrowAdjustUsd) / collateralValue) * 100
  })

  const borrowLimitUsedUsd = computed(() => Math.max(nextWeightedBorrowedUsd.value, 0))

  const borrowLimitTotalUsd = computed(() => {
    if (!obligation.value || !marketState.value || !poolData.value) {
      return 0
    }

    const maxBorrowAdjustUsd = availableToBorrowAfterRepay.value * Number(price.value) * liabilityFactor.value

    return Math.max(nextWeightedBorrowedUsd.value + maxBorrowAdjustUsd, 0)
  })

  async function repay() {
    if (!poolData.value) {
      return
    }
    if (!amount.value || amount.value <= 0 || amount.value > Number(balance.value)) {
      focusInput('.repay-dialog__input')
      return
    }
    try {
      loading.value = true

      const withBuffer = debt.value === Number(amount.value) && Number(balance.value) > Number(amount.value)

      const marketProps = {
        market: activeMarket.value!.marketName,
        client: activeMarket.value!.client!,
        pool_address: pool_address.value,
        amount: amount.value,
        asset_data: poolData.value.pool.name,
        limit: balance.value,
        withBuffer,
      }

      await market.repay(marketProps)
    } finally {
      loading.value = false
    }
  }

  let stopRepayWatcherHandle: WatchStopHandle | undefined
  let feeInterval: ReturnType<typeof setInterval> | undefined

  function startRepayWatcher() {
    stopRepayWatcherHandle?.()
    stopRepayWatcherHandle = watchDebounced(
      [poolData, reloadFee, publicKey],
      async ([r]) => {
        if (!r?.pool.pool_address || !publicKey.value) {
          return
        }

        try {
          isLoadingFee.value = true

          const oblKey = buildObligationKey({ pablicKey: publicKey.value })

          const tx = await activeMarket.value?.client!.borrowing.buildRepayTx(
            oblKey,
            r.pool.pool_address,
            0.01,
            assetDecimals.value,
          )
          txFee.value = activeMarket.value?.client!.borrowing.getTransactionFee(tx, assetDecimals.value) ?? 0
        } finally {
          isLoadingFee.value = false
        }
      },
      { immediate: true, debounce: 300 },
    )

    startFeeInterval()
  }

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

  function stopRepayWatcher() {
    stopRepayWatcherHandle?.()
    stopRepayWatcherHandle = undefined
    stopFeeInterval()
  }

  watch(isOpen, (open) => {
    if (open) {
      startRepayWatcher()
    } else {
      stopRepayWatcher()
    }
  }, { immediate: true })

  onUnmounted(() => {
    stopRepayWatcher()
  })

  return {
    asset,
    price,
    debt,
    debtAfterRepay,
    balance,
    currentHealthFactor,
    dynamicHealthFactor,
    borrowLimitUsedUsd,
    borrowLimitTotalUsd,
    currentLtv,
    dynamicLtv,
    maxLtv,
    isLoadingFee,
    txFee,
    amount,
    loading,
    repay,
  }
}
