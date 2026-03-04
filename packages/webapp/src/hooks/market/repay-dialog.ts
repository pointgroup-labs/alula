import type { WatchStopHandle } from 'vue'
import { calculateBorrow, calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { shortenNumber, truncatePercent } from '~/utils'

export function useRepayDialog(isOpen: Ref<boolean>) {
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
  const asset_issuer = computed(() => poolData.value?.pool.name ? destructurePoolAsset(poolData.value.pool.name)[1] : '')
  const price = computed(() =>
    poolData.value?.oracle_asset_price ? bigintToNumber(poolData.value.oracle_asset_price, oraclePriceDecimals.value) : 0,
  )

  const debt = computed(() => {
    const obligation = userStore.state.obligations[String(marketKey.value)]
    const borrow = obligation?.borrows?.find(([addr]) => addr === pool_address.value)
    if (!borrow || !poolData.value) {
      return 0
    }
    const [, bor] = borrow
    return Number(calculateBorrow(bor.d_tokens, {
      total_borrowed: poolData.value.pool.total_borrowed,
      total_d_tokens: poolData.value.pool.total_d_tokens,
    }, assetDecimals.value))
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

  const loading = ref(false)
  const reloadFee = ref(false)
  const isLoadingFee = ref(false)
  const txFee = ref(0)

  const amount = toRef(market, 'repayAmount')

  const balance = computed(() => {
    if (!poolData.value) {
      return 0
    }
    if (asset.value.symbol === 'XLM') {
      return wallet.nativeBalance
    }
    return wallet.getAssetBalance(asset_issuer.value)
  })

  const healthFactor = computed(() => {
    const amountInUsd = Number(amount.value || 0) * Number(price.value)
    const deposited = userTotalDepositByMarket.value
    const borrowed = Math.max(Number(userTotalBorrowByMarket.value) - amountInUsd, 0)
    const result = Math.max(deposited / borrowed, 0)
    return Math.min(result, 10)
  })

  const infoPanelData = computed(() => {
    if (!poolData.value) {
      return {}
    }
    const debtVal = debt.value
    const borrowBalanceAfterRepay = Math.max(debtVal - (amount.value || 0), 0)
    return {
      balances: {
        title: 'Balances / Health',
        data: [
          {
            label: 'Health Factor',
            value: truncatePercent(healthFactor.value, 2),
          },
          {
            label: 'Debt',
            value: `${shortenNumber(debtVal, 2, maxDecimalsForShortenNumber(debtVal))} ${asset.value.symbol}`,
          },
          {
            label: 'Debt Balance After Repayment',
            value: `${shortenNumber(borrowBalanceAfterRepay, 2, maxDecimalsForShortenNumber(borrowBalanceAfterRepay))} ${asset.value.symbol}`,
          },
        ],
      },
      fees: {
        title: 'Fees',
        data: [
          {
            label: 'Transaction Fee',
            value: `${txFee.value} XLM`,
            slotName: 'txFee',
          },
        ],
      },
    }
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

      const withBuffer = debt.value === amount.value && Number(balance.value) !== amount.value

      const marketProps = {
        market: activeMarket.value!.marketName,
        client: activeMarket.value!.client,
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
          const tx = await activeMarket.value?.client.borrowing.buildRepayTx(
            publicKey.value,
            r.pool.pool_address,
            0,
          )
          txFee.value = activeMarket.value?.client.borrowing.getTransactionFee(tx) ?? 0
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

  return {
    poolData,
    asset,
    price,
    debt,
    balance,
    healthFactor,
    infoPanelData,
    isLoadingFee,
    txFee,
    amount,
    loading,
    reloadFee,
    repay,
    stopRepayWatcher,
  }
}
