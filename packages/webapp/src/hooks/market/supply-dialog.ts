import type { WatchStopHandle } from 'vue'
import type { MarketTableItem } from '~/types/table'
import { calcFee } from '@alula/client-sdk'
import { calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { calcHealthFactor, calcWeightedBorrowedUsd } from '~/utils'

export function useSupplyDialog(data: MaybeRef<MarketTableItem | undefined>, isOpen: Ref<boolean>) {
  const router = useRouter()
  const route = useRoute()

  const wallet = useWallet()
  const publicKey = computed(() => wallet.publicKey)

  const marketsStore = useMarketsStore()
  const market = useMarketActions()

  const userStore = useUserStore()

  const marketClient = computed(() => marketsStore.marketClient)
  const collateralOnly = toRef(market, 'collateralOnly')

  const poolData = computed(() => unref(data))

  const amount = toRef(market, 'depositAmount')

  const isLoadingFee = ref(false)
  const reloadFee = ref(false)
  const txFee = ref(0)

  const isLoading = computed(() => marketsStore.poolActiveAddress === poolData.value?.raw.pool.pool_address)

  const balance = computed(() => {
    if (!poolData.value) {
      return 0
    }
    if (poolData.value.raw.pool.token_symbol === 'native') {
      return wallet.nativeBalance
    }
    const [, asset_issuer] = destructurePoolAsset(poolData.value?.raw.pool.name)
    return wallet.getAssetBalance(String(asset_issuer))
  })

  const isSupplyLimited = computed(() => poolData.value?.supply_limit && !collateralOnly.value && poolData.value?.supply_limit > 0)
  const supplyLimit = computed(() => isSupplyLimited.value ? Math.max(Number(poolData.value?.supply_limit) - Number(poolData.value?.total_supply), 0) : 0)
  const limitLabel = computed(() => isSupplyLimited.value ? shortenNumber(Number(supplyLimit.value), 2, 2) : '-')

  const contractAddress = computed(() => poolData.value?.raw.pool.pool_address || '')

  const reserveAmount = computed(() => poolData.value?.raw.pool.token_symbol === 'native' ? 2 : 0)

  const isCanSupply = computed(() => {
    const borrowObligations = userStore.state.obligations[String(poolData.value?.market)]?.borrows ?? []
    return checkIsCanUsePool(borrowObligations, poolData.value?.raw.pool.pool_address)
  })

  const marketFee = computed(() => {
    const marketFeeBps = collateralOnly.value
      ? poolData.value?.raw.pool.config.fee_config.add_collateral_fee_bps
      : poolData.value?.raw.pool.config.fee_config.deposit_fee_bps
    return calcFee(Number(amount.value || 0), marketFeeBps || 0)
  })

  const currentLtv = computed(() => {
    const marketName = String(poolData.value?.market)
    const obligation = userStore.state.obligations[marketName]
    const marketState = marketsStore.state.markets[marketName]?.marketState

    if (!obligation || !marketState) {
      return 0
    }

    const assetDecimals = marketState.asset_decimals ?? 7
    const oraclePriceDecimals = marketState.oracle_price_decimals ?? 0
    const poolsData = marketState.pools_data
    const collateralValueUsd = calcUserTotalStakeInUsd(
      obligation,
      poolsData,
      assetDecimals,
      oraclePriceDecimals,
    )

    if (collateralValueUsd <= 0) {
      return 0
    }

    const weightedBorrowedValueUsd = calcWeightedBorrowedUsd(
      obligation,
      poolsData,
      assetDecimals,
      oraclePriceDecimals,
    )

    return (weightedBorrowedValueUsd / collateralValueUsd) * 100
  })

  const dynamicLtv = computed(() => {
    const marketName = String(poolData.value?.market)
    const obligation = userStore.state.obligations[marketName]
    const marketState = marketsStore.state.markets[marketName]?.marketState

    if (!obligation || !marketState) {
      return 0
    }

    const assetDecimals = marketState.asset_decimals ?? 7
    const oraclePriceDecimals = marketState.oracle_price_decimals ?? 0
    const poolsData = marketState.pools_data
    const collateralValueUsd = calcUserTotalStakeInUsd(
      obligation,
      poolsData,
      assetDecimals,
      oraclePriceDecimals,
    )
    const weightedBorrowedValueUsd = calcWeightedBorrowedUsd(
      obligation,
      poolsData,
      assetDecimals,
      oraclePriceDecimals,
    )
    const collateralAdjustUsd = (Number(amount.value) || 0) * Number(poolData.value?.price ?? 0)
    const nextCollateralValueUsd = collateralValueUsd + collateralAdjustUsd

    if (nextCollateralValueUsd <= 0) {
      return 0
    }

    return (weightedBorrowedValueUsd / nextCollateralValueUsd) * 100
  })

  const currentHealthFactor = computed(() => {
    const marketName = String(poolData.value?.market)
    const obligation = userStore.state.obligations[marketName]
    const marketState = marketsStore.state.markets[marketName]?.marketState

    if (!obligation || !marketState) {
      return 10
    }

    const assetDecimals = marketState.asset_decimals ?? 7
    const oraclePriceDecimals = marketState.oracle_price_decimals ?? 0
    const poolsData = marketState.pools_data

    return calcHealthFactor(obligation, poolsData, assetDecimals, oraclePriceDecimals)
  })

  const dynamicHealthFactor = computed(() => {
    const marketName = String(poolData.value?.market)
    const obligation = userStore.state.obligations[marketName]
    const marketState = marketsStore.state.markets[marketName]?.marketState

    if (!obligation || !marketState) {
      return 10
    }

    const assetDecimals = marketState.asset_decimals ?? 7
    const oraclePriceDecimals = marketState.oracle_price_decimals ?? 0
    const poolsData = marketState.pools_data
    const closeLtvRatio = Number(poolData.value?.raw.pool.config.health_config.close_ltv_bps ?? 0) / 10_000
    const depositAdjustUsd = -((Number(amount.value) || 0) * Number(poolData.value?.price ?? 0) * closeLtvRatio)

    return calcHealthFactor(obligation, poolsData, assetDecimals, oraclePriceDecimals, depositAdjustUsd)
  })

  const dynamicUtilizationRate = computed(() => {
    const pool = poolData?.value?.raw.pool
    const assetDecimals = poolData?.value?.assetDecimals ?? 7
    if (!pool) {
      return poolData?.value?.utilization_rate ?? '-'
    }

    const totalBorrowed = Number(bigintToNumber(pool.total_borrowed, assetDecimals))
    const totalAvailable = Number(bigintToNumber(pool.total_available, assetDecimals))
    const depositAmount = collateralOnly.value ? 0 : (Number(amount.value) || 0)

    const newTotalAvailable = totalAvailable + depositAmount
    const newUtil = totalBorrowed > 0 ? totalBorrowed / (newTotalAvailable + totalBorrowed) * 100 : 0

    return `${truncatePercent(newUtil, 2)}%`
  })

  const infoPanelData = computed(() => {
    if (!poolData.value) {
      return {}
    }

    return {
      poolInfo: {
        title: 'Pool Info',
        data: [
          {
            label: 'Supply Limit',
            value: `${limitLabel.value} ${limitLabel.value === '-' ? '' : poolData.value?.asset.symbol}`,
          },
          {
            label: 'Open LTV',
            value: poolData.value.open_ltv,
          },
          {
            label: 'Util. Rate',
            value: dynamicUtilizationRate.value,
          },
        ],
      },
      fees: {
        title: 'Fees',
        data: [
          {
            label: 'Operation Fee',
            value: `${formatPrice(marketFee.value)} ${poolData.value?.asset.symbol}`,
          },
          {
            label: 'Transaction Fee',
            value: txFee.value,
            slotName: 'txFee',
            className: 'fee-cell',
          },
        ],
      },
    }
  })

  async function supply() {
    try {
      if (!publicKey.value || !poolData.value?.raw.pool.pool_address) {
        return
      }
      if (!amount.value || amount.value <= 0) {
        focusInput('.supply-dialog .dialog-default__input')
        return
      }
      marketsStore.poolActiveAddress = poolData.value?.raw.pool.pool_address

      const marketProps = {
        market: marketsStore.selectedMarketName,
        client: marketClient.value!,
        pool_address: poolData.value?.raw.pool.pool_address,
        amount: amount.value,
        asset_data: poolData.value?.raw.pool.name,
      }
      collateralOnly.value
        ? await market.addCollateral(marketProps)
        : await market.deposit(marketProps)
    } finally {
      marketsStore.poolActiveAddress = undefined
    }
  }

  const attentionText = 'You cannot deposit funds into a pool where you have an active loan. Please repay your loan before depositing funds.'

  let stopFeeWatcher: WatchStopHandle | undefined
  let stopCollateralToQuery: WatchStopHandle | undefined
  let stopQueryToCollateral: WatchStopHandle | undefined

  let feeInterval: ReturnType<typeof setInterval> | undefined
  function startFeeInterval() {
    clearInterval(feeInterval)
    feeInterval = setInterval(() => {
      reloadFee.value = true
      nextTick(() => {
        reloadFee.value = false
      })
    }, RELOAD_FEE_INTERVAL)
    return feeInterval
  }

  function stopFeeInterval() {
    clearInterval(feeInterval)
    feeInterval = undefined
  }

  function startSupplyWatchers() {
    stopSupplyWatchers()

    // 1) fee calc (buildDepositTx)
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
          const tx = await marketClient.value.lending.buildDepositTx(
            publicKey.value,
            d.raw.pool.pool_address || '',
            0.01,
          )
          txFee.value = marketClient.value.lending.getTransactionFee(tx)
        } finally {
          isLoadingFee.value = false
        }
      },
      { immediate: true, debounce: 300 },
    )

    // 2) collateralOnly -> query
    stopCollateralToQuery = watchDebounced(
      collateralOnly,
      (c) => {
        if (!isOpen.value) {
          return
        }
        const query = { ...route.query }
        if (c) {
          query['collateral-only'] = 'true'
        } else {
          delete query['collateral-only']
        }
        router.replace({ query })
      },
      { debounce: 100 },
    )

    // 3) query -> collateralOnly
    stopQueryToCollateral = watch(
      () => route.query,
      (q) => {
        if (!isOpen.value) {
          return
        }
        collateralOnly.value = Boolean(q['collateral-only'])
      },
      { immediate: true },
    )

    startFeeInterval()
  }

  function stopSupplyWatchers() {
    stopFeeWatcher?.()
    stopFeeWatcher = undefined

    stopCollateralToQuery?.()
    stopCollateralToQuery = undefined

    stopQueryToCollateral?.()
    stopQueryToCollateral = undefined

    stopFeeInterval()
  }

  watch(isOpen, (open) => {
    if (open) {
      startSupplyWatchers()
    } else {
      stopSupplyWatchers()
    }
  }, { immediate: true })

  onUnmounted(() => {
    stopSupplyWatchers()
  })

  return {
    balance,
    marketClient,
    collateralOnly,
    txFee,
    reloadFee,
    isLoadingFee,
    supplyLimit,
    limitLabel,
    contractAddress,
    amount,
    reserveAmount,
    isLoading,
    isCanSupply,
    attentionText,
    infoPanelData,
    marketFee,
    currentLtv,
    dynamicLtv,
    currentHealthFactor,
    dynamicHealthFactor,
    dynamicUtilizationRate,
    supply,
    startFeeInterval,
    stopSupplyWatchers,
  }
}
