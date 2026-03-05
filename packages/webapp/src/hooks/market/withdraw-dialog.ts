import type { WatchStopHandle } from 'vue'
import { bpsToNumber, calculateTotalStake, calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { shortenNumber, truncatePercent } from '~/utils'

export function useWithdrawDialog(isOpen: Ref<boolean>) {
  const marketsStore = useMarketsStore()
  const market = useMarketActions()
  const wallet = useWallet()
  const userStore = useUserStore()

  const publicKey = computed(() => wallet.publicKey)

  const marketKey = computed(() => marketsStore.selectedMarketName)
  const poolData = computed(() =>
    marketsStore.state.markets[String(marketKey.value)]?.marketState.pools_data
      ?.find(p => p.pool.pool_address === marketsStore.selectedPoolAddress),
  )

  const activeMarket = computed(() => marketsStore.state.markets[String(marketKey.value)])

  const assetDecimals = computed(() => activeMarket.value?.marketState.asset_decimals ?? 7)
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

  const userTotalBorrowByMarket = computed(() => {
    const obligation = userStore.state.obligations[String(activeMarket.value?.marketName)]
    const pools = activeMarket.value?.marketState.pools_data
    if (!obligation || !pools) {
      return 0
    }
    return calcUserTotalBorrowedInUsd(obligation, pools, assetDecimals.value, oraclePriceDecimals.value) ?? 0
  })

  const collateralBalance = computed(() => userDeposit.value.collateral)
  const supplyBalance = computed(() => userDeposit.value.balance - collateralBalance.value)
  const totalSuppliedBalance = computed(() => userDeposit.value.balance)
  const remainingBalance = computed(() => (collateralOnly.value ? collateralBalance.value : supplyBalance.value) - amount.value)

  const openLtv = computed(() =>
    poolData.value?.pool.config.health_config.open_ltv_bps
      ? bpsToNumber(Number(poolData.value.pool.config.health_config.open_ltv_bps))
      : 0,
  )

  const healthFactor = computed(() => {
    const withdrawUsd = Number(amount.value || 0) * Number(price.value) * openLtv.value
    const depositedAfterWithdraw = Math.max(userTotalDepositByMarket.value - withdrawUsd, 0)
    const borrowed = userTotalBorrowByMarket.value
    const result = borrowed === 0 ? 10 : Math.max(depositedAfterWithdraw / borrowed, 0)
    return Math.min(result, 10)
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
    const depositWithOpenLtv = userTotalDepositByMarket.value
    const borrowed = userTotalBorrowByMarket.value
    const poolOpenLtv = openLtv.value

    const maxWithdrawUsd = poolOpenLtv > 0
      ? Math.max(depositWithOpenLtv - borrowed * 1.05, 0) / poolOpenLtv
      : 0
    const maxWithdrawAmount = maxWithdrawUsd / priceVal
    const balance = collateralOnly.value ? collateralBalance.value : supplyBalance.value
    return Math.min(balance, maxWithdrawAmount)
  })

  const availableToWithdrawWithPoolLimit = computed(() =>
    Math.min(Number(truncatePercent(availableToWithdraw.value, 7)), Number(poolLimit.value)),
  )

  const infoPanelData = computed(() => {
    if (!poolData.value) {
      return {}
    }
    const sym = asset.value.symbol
    return {
      balances: {
        title: 'Balances',
        data: [
          {
            label: 'Total Supply',
            value: `${shortenNumber(totalSuppliedBalance.value || 0, 2, maxDecimalsForShortenNumber(totalSuppliedBalance.value))} ${sym}`,
          },
          {
            label: 'Supply Balance',
            value: `${shortenNumber(supplyBalance.value || 0, 2, maxDecimalsForShortenNumber(supplyBalance.value))} ${sym}`,
          },
          {
            label: 'Collateral Balance',
            value: `${shortenNumber(collateralBalance.value || 0, 2, maxDecimalsForShortenNumber(collateralBalance.value))} ${sym}`,
          },
        ],
      },
      health: {
        title: 'Health',
        data: [
          {
            label: 'Health Factor',
            value: truncatePercent(healthFactor.value || 0, 2),
          },
          {
            label: 'Remaining Supply',
            value: `${shortenNumber(Math.max(remainingBalance.value || 0, 0), 2, maxDecimalsForShortenNumber(remainingBalance.value))} ${sym}`,
          },
          {
            label: 'Available for Withdrawal',
            value: `${shortenNumber(availableToWithdraw.value || 0, 2, maxDecimalsForShortenNumber(availableToWithdraw.value))} ${sym}`,
          },
        ],
      },
      poolInfo: {
        title: 'Info / Fee',
        data: [
          {
            label: 'Pool Withdrawal Limit',
            value: `${shortenNumber(poolLimit.value || 0, 2, maxDecimalsForShortenNumber(poolLimit.value))} ${sym}`,
          },
          {
            label: 'Transaction Fee',
            value: `${txFee.value || 0} XLM`,
            slotName: 'txFee',
          },
        ],
      },
    }
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
        client: activeMarket.value!.client,
        pool_address: pool_address.value,
        amount: amount.value,
        asset_data: poolData.value.pool.name,
        limit: collateralBalance.value,
        withBuffer: Number(availableToWithdraw.value) === Number(amount.value),
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
        if (!isOpen.value) { return }
        if (!a || Number(a) <= 0) {
          poolFee.value = 0
          return
        }
        if (!publicKey.value || !pool_address.value) { return }

        const feeData = await activeMarket.value?.client.market.simulateWithdraw(
          publicKey.value,
          pool_address.value,
          a,
        )
        const feeSum = feeData?.operation_fees?.fee_sum
        poolFee.value = feeSum ? Number(bigintToNumber(feeSum, assetDecimals.value)) : 0
      },
      { debounce: 500 },
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
          const tx = await activeMarket.value?.client.lending.buildWithdrawTx(
            publicKey.value,
            r.pool.pool_address,
            0,
          )
          txFee.value = activeMarket.value?.client.lending.getTransactionFee(tx) ?? 0
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

  return {
    poolData,
    asset,
    price,
    collateralBalance,
    supplyBalance,
    availableToWithdraw,
    availableToWithdrawWithPoolLimit,
    infoPanelData,
    isLoadingFee,
    txFee,
    amount,
    collateralOnly,
    loading,
    reloadFee,
    withdraw,
    startFeeInterval,
    stopWithdrawWatchers,
  }
}
