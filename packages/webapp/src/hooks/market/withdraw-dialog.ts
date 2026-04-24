import type { WatchStopHandle } from 'vue'
import { bpsToNumber, calculateTotalStake, calcUserTotalStakeInUsd } from '@alula/client-sdk'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { calcHealthFactor, calcWeightedBorrowedUsd, truncatePercent } from '~/utils'

export function useWithdrawDialog(isOpen: Ref<boolean>) {
  const marketsStore = useMarketsStore()
  const market = useMarketActions()
  const userStore = useUserStore()
  const { getFullTokenData } = useTokensStore()

  const { publicKey } = useWalletComposable()

  const marketKey = computed(() => marketsStore.selectedMarketName)
  const poolData = computed(() =>
    marketsStore.state.markets[String(marketKey.value)]?.marketState.pools_data
      ?.find(p => p.pool.pool_address === marketsStore.selectedPoolAddress),
  )

  const activeMarket = computed(() => marketsStore.state.markets[String(marketKey.value)])

  const assetDecimals = computed(() => poolData.value?.pool.token_decimals ?? 0)
  const oraclePriceDecimals = computed(() => activeMarket.value?.marketState.oracle_price_decimals ?? 0)

  const pool_address = computed(() => poolData.value?.pool.pool_address ?? '')
  const asset = computed(() => getFullTokenData(poolData.value?.pool.token_symbol ?? ''))
  const price = computed(() =>
    poolData.value?.oracle_asset_price ? bigintToNumber(poolData.value.oracle_asset_price, oraclePriceDecimals.value) : 0,
  )

  const amount = toRef(market, 'withdrawAmount')
  const collateralOnly = toRef(market, 'collateralOnly')

  const loading = ref(false)
  const reloadFee = ref(false)
  const isLoadingFee = ref(false)
  const txFee = ref(0)
  const poolFee = ref(0)

  const userDeposit = computed(() => {
    const obligation = userStore.state.obligations[String(marketKey.value)]
    const deposit = obligation?.deposits?.find(([addr]) => addr === pool_address.value)
    if (!deposit || !poolData.value) {
      return { balance: 0, collateral: 0 }
    }
    const [, dep] = deposit
    const deposited = calculateTotalStake(dep.j_tokens, {
      total_j_tokens: poolData.value.pool.total_j_tokens,
      total_borrowed: poolData.value.pool.total_borrowed,
      total_available: poolData.value.total_available_adjusted,
    })
    const collateral = Number(bigintToNumber(dep.collateral, assetDecimals.value))
    const balance = Number(deposited) + collateral
    return { balance, collateral }
  })

  const userTotalDepositByMarket = computed(() => {
    const obligation = userStore.state.obligations[String(activeMarket.value?.marketName)]
    const pools = activeMarket.value?.marketState.pools_data
    if (!obligation || !pools) {
      return 0
    }
    return calcUserTotalStakeInUsd(obligation, pools, assetDecimals.value, oraclePriceDecimals.value, 'open') ?? 0
  })

  const collateralBalance = computed(() => userDeposit.value.collateral)
  const supplyBalance = computed(() => userDeposit.value.balance - collateralBalance.value)
  const remainingBalance = computed(() => (collateralOnly.value ? collateralBalance.value : supplyBalance.value) - (amount.value ?? 0))
  const obligation = computed(() => userStore.state.obligations[String(activeMarket.value?.marketName)])
  const marketState = computed(() => activeMarket.value?.marketState)

  const openLtv = computed(() =>
    poolData.value?.pool.config.health_config.open_ltv_bps
      ? bpsToNumber(Number(poolData.value.pool.config.health_config.open_ltv_bps))
      : 0,
  )

  const closeLtv = computed(() =>
    poolData.value?.pool.config.health_config.close_ltv_bps
      ? bpsToNumber(Number(poolData.value.pool.config.health_config.close_ltv_bps))
      : 0,
  )

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

  const withdrawAdjustUsd = computed(() => (Number(amount.value) || 0) * Number(price.value || 0))

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
      return 10
    }

    const depositAdjustUsd = withdrawAdjustUsd.value * closeLtv.value

    return calcHealthFactor(
      obligation.value,
      marketState.value.pools_data,
      assetDecimals.value,
      oraclePriceDecimals.value,
      depositAdjustUsd,
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

    return (currentWeightedBorrowedUsd.value / collateralValue) * 100
  })

  const dynamicLtv = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    const nextCollateralValueUsd = Math.max(collateralValueUsd.value - withdrawAdjustUsd.value, 0)

    if (nextCollateralValueUsd <= 0) {
      return 0
    }

    return (currentWeightedBorrowedUsd.value / nextCollateralValueUsd) * 100
  })

  const poolLimit = computed(() => {
    if (!poolData.value) {
      return 0
    }
    const limit = collateralOnly.value ? poolData.value.pool.total_collateral : poolData.value.total_available_adjusted
    return Math.max(Number(bigintToNumber(limit, assetDecimals.value)), 0)
  })

  const availableToWithdraw = computed(() => {
    const priceVal = Number(price.value) || 1
    const balance = collateralOnly.value ? collateralBalance.value : supplyBalance.value
    const poolsData = marketState.value?.pools_data

    if (!poolsData || !obligation.value || !marketState.value) {
      return 0
    }

    if (obligation.value.borrows.length === 0) {
      return balance
    }

    const depositWithOpenLTV = userTotalDepositByMarket.value

    const borrowedWithLF = currentWeightedBorrowedUsd.value

    const positionsWithNonZeroLTV = obligation.value.deposits.filter(([poolAddr]) => {
      const pool = poolsData.find(p => p.pool.pool_address === poolAddr)
      return pool && Number(pool.pool.config.health_config.close_ltv_bps) > 0
    }).length
    const minCollateralUsd = (Number(marketState.value.global_state.min_collateral_value_cents) / 100) * positionsWithNonZeroLTV

    const borrowingCapacityUsd = Math.max(depositWithOpenLTV - borrowedWithLF - minCollateralUsd, 0)

    const poolOpenLtv = openLtv.value
    const maxWithdrawAmount = poolOpenLtv > 0 ? (borrowingCapacityUsd / poolOpenLtv) / priceVal : 0

    return Math.min(balance, maxWithdrawAmount)
  })

  const availableToWithdrawWithPoolLimit = computed(() =>
    Math.min(Number(truncatePercent(availableToWithdraw.value, 7)), Number(poolLimit.value)),
  )

  const maxLtv = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    const nextCollateralValueUsd = Math.max(collateralValueUsd.value - withdrawAdjustUsd.value, 0)

    if (nextCollateralValueUsd <= 0) {
      return 0
    }

    const poolsData = marketState.value.pools_data
    const nextDepositWithOpenLtvUsd = Math.max(userTotalDepositByMarket.value - (withdrawAdjustUsd.value * openLtv.value), 0)
    const positionsWithNonZeroLTV = obligation.value.deposits.filter(([poolAddr]) => {
      const pool = poolsData.find(p => p.pool.pool_address === poolAddr)
      return pool && Number(pool.pool.config.health_config.close_ltv_bps) > 0
    }).length
    const minCollateralUsd = (Number(marketState.value.global_state.min_collateral_value_cents) / 100) * positionsWithNonZeroLTV
    const borrowingCapacityUsd = Math.max(nextDepositWithOpenLtvUsd - currentWeightedBorrowedUsd.value - minCollateralUsd, 0)

    return ((currentWeightedBorrowedUsd.value + borrowingCapacityUsd) / nextCollateralValueUsd) * 100
  })

  const borrowLimitUsedUsd = computed(() => Math.max(currentWeightedBorrowedUsd.value, 0))

  const borrowLimitTotalUsd = computed(() => {
    if (!obligation.value || !marketState.value) {
      return 0
    }

    const poolsData = marketState.value.pools_data
    const nextDepositWithOpenLtvUsd = Math.max(userTotalDepositByMarket.value - (withdrawAdjustUsd.value * openLtv.value), 0)
    const positionsWithNonZeroLTV = obligation.value.deposits.filter(([poolAddr]) => {
      const pool = poolsData.find(p => p.pool.pool_address === poolAddr)
      return pool && Number(pool.pool.config.health_config.close_ltv_bps) > 0
    }).length
    const minCollateralUsd = (Number(marketState.value.global_state.min_collateral_value_cents) / 100) * positionsWithNonZeroLTV
    const borrowingCapacityUsd = Math.max(nextDepositWithOpenLtvUsd - currentWeightedBorrowedUsd.value - minCollateralUsd, 0)

    return Math.max(currentWeightedBorrowedUsd.value + borrowingCapacityUsd, 0)
  })

  async function withdraw() {
    if (!poolData.value) {
      return
    }
    if (!amount.value || amount.value <= 0 || amount.value > availableToWithdrawWithPoolLimit.value) {
      focusInput('.withdraw-dialog__input')
      return
    }
    try {
      loading.value = true

      const marketProps = {
        market: activeMarket.value!.marketName,
        client: activeMarket.value!.client!,
        pool_address: pool_address.value,
        amount: amount.value,
        asset_data: poolData.value.pool.name,
        limit: collateralBalance.value,
        withBuffer: Number(amount.value) >= availableToWithdrawWithPoolLimit.value,
      }

      collateralOnly.value
        ? await market.removeCollateral(marketProps)
        : await market.withdraw({ ...marketProps, limit: supplyBalance.value })
    } finally {
      loading.value = false
    }
  }

  let stopSimulateWatcher: WatchStopHandle | undefined
  let stopTxFeeWatcher: WatchStopHandle | undefined
  let feeInterval: ReturnType<typeof setInterval> | undefined

  function startWithdrawWatchers() {
    stopWithdrawWatchers()

    stopSimulateWatcher = watchDebounced(
      amount,
      async (a) => {
        if (!isOpen.value) {
          return
        }
        if (!a || Number(a) <= 0) {
          poolFee.value = 0
          return
        }
        if (!publicKey.value || !pool_address.value) {
          return
        }

        const oblKey = buildObligationKey({ pablicKey: publicKey.value })

        const feeData = await activeMarket.value?.client!.market.simulateWithdraw(
          oblKey,
          pool_address.value,
          a,
          assetDecimals.value,
        )
        const feeSum = feeData?.operation_fees?.fee_sum
        poolFee.value = feeSum ? Number(bigintToNumber(feeSum, assetDecimals.value)) : 0
      },
      { debounce: 1000 },
    )

    stopTxFeeWatcher = watchDebounced(
      [poolData, reloadFee, publicKey],
      async ([r]) => {
        if (!isOpen.value) {
          return
        }
        if (!r?.pool.pool_address || !publicKey.value) {
          return
        }

        try {
          isLoadingFee.value = true

          const oblKey = buildObligationKey({ pablicKey: publicKey.value })

          const tx = await activeMarket.value?.client!.lending.buildWithdrawTx(
            oblKey,
            r.pool.pool_address,
            0.1,
            assetDecimals.value,
          )
          txFee.value = activeMarket.value?.client!.lending.getTransactionFee(tx, assetDecimals.value) ?? 0
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

  function stopWithdrawWatchers() {
    stopSimulateWatcher?.()
    stopSimulateWatcher = undefined
    stopTxFeeWatcher?.()
    stopTxFeeWatcher = undefined
    stopFeeInterval()
  }

  watch(isOpen, (open) => {
    if (open) {
      startWithdrawWatchers()
    } else {
      stopWithdrawWatchers()
    }
  }, { immediate: true })

  watch(collateralBalance, (b) => {
    if (b <= 0) {
      collateralOnly.value = false
    }
  }, { immediate: true })

  onScopeDispose(() => {
    stopWithdrawWatchers()
  })

  return {
    poolData,
    poolLimit,
    asset,
    price,
    collateralBalance,
    supplyBalance,
    remainingBalance,
    availableToWithdrawWithPoolLimit,
    isLoadingFee,
    poolFee,
    txFee,
    amount,
    collateralOnly,
    currentHealthFactor,
    dynamicHealthFactor,
    borrowLimitUsedUsd,
    borrowLimitTotalUsd,
    currentLtv,
    dynamicLtv,
    maxLtv,
    loading,
    withdraw,
  }
}
