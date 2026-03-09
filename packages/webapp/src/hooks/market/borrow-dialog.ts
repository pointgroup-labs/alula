import type { WatchStopHandle } from 'vue'
import type { MarketTableItem } from '~/types/table'
import { bpsToNumber, calcFee } from '@alula/client-sdk'
import { calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { calcHealthFactor, truncatePercent } from '~/utils'

export function useBorrowDialog(data: MaybeRef<MarketTableItem | undefined>, isOpen: Ref<boolean>) {
  const wallet = useWallet()
  const publicKey = computed(() => wallet.publicKey)

  const marketsStore = useMarketsStore()
  const market = useMarketActions()

  const userStore = useUserStore()

  const marketClient = computed(() => marketsStore.marketClient)

  const poolData = computed(() => unref(data))

  const agree = ref(false)

  const reloadFee = ref(false)
  const isLoadingFee = ref(false)
  const txFee = ref(0)

  const amount = toRef(market, 'borrowAmount')

  const isLoading = computed(() => marketsStore.poolActiveAddress === poolData.value?.raw.pool.pool_address)

  const poolBorrowLimit = computed(() => {
    if (!poolData.value) {
      return 0
    }
    const utilRatioLimit = bpsToNumber(Number(poolData.value?.raw.pool.config.health_config.utilization_ratio_limit_bps || 0))
    const totalSupply = Number(bigintToNumber(poolData.value.raw.total_supply, poolData.value.assetDecimals))
    const totalBorrow = Number(bigintToNumber(poolData.value.raw.pool.total_borrowed, poolData.value.assetDecimals))
    const availableByRatioLimit = totalSupply * utilRatioLimit
    return Math.max(availableByRatioLimit - totalBorrow, 0)
  })

  const availableToBorrow = computed(() => {
    if (!poolData.value) {
      return 0
    }
    const marketName = String(poolData.value.market)
    const obligation = userStore.state.obligations[marketName]
    const marketState = marketsStore.state.markets[marketName]?.marketState

    if (!obligation || !marketState) {
      return 0
    }

    const assetDecimals = marketState.asset_decimals ?? 7
    const oraclePriceDecimals = marketState.oracle_price_decimals ?? 0
    const poolsData = marketState.pools_data

    // Σ(Vc_i × oLTV_i)
    const depositWithOpenLTV = calcUserTotalStakeInUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals, 'open')

    // Σ(Vb_j × LF_j)
    let borrowedWithLF = 0
    for (const [borrowedPoolAddress, data] of obligation.borrows) {
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

    // Vmin × count of deposit positions with non-zero close LTV
    const positionsWithNonZeroLTV = obligation.deposits.filter(([poolAddr]) => {
      const pool = poolsData.find(p => p.pool.pool_address === poolAddr)
      return pool && Number(pool.pool.config.health_config.close_ltv_bps) > 0
    }).length
    const minCollateralUsd = (Number(marketState.global_state.min_collateral_value_cents) / 100) * positionsWithNonZeroLTV

    // BC = Σ(Vc_i × oLTV_i) − Σ(Vb_j × LF_j) − Vmin × count
    const borrowingCapacityUsd = Math.max(depositWithOpenLTV - borrowedWithLF - minCollateralUsd, 0)

    // maxTokens = BC / (price × LF) for the target borrow pool
    const price = Number(poolData.value.price)
    const liabilityFactor = bpsToNumber(Number(poolData.value.raw.pool.config.health_config.liability_factor_bps))
    const maxAvailableAssets = price > 0 && liabilityFactor > 0
      ? borrowingCapacityUsd / (price * liabilityFactor)
      : 0

    return Number(truncatePercent(Math.min(maxAvailableAssets, poolBorrowLimit.value), assetDecimals))
  })

  const closeLTV = computed(() => Number(poolData.value?.raw.pool.config.health_config.close_ltv_bps || 0) / 100)

  const liquidationPenalty = computed(() => Number(poolData.value?.raw.pool.config.health_config.liquidation_close_factor_bps || 0) / 100)

  const isCanBorrow = computed(() => {
    const depositObligations = userStore.state.obligations[String(poolData.value?.market)]?.deposits ?? []
    return checkIsCanUsePool(depositObligations, poolData.value?.raw.pool.pool_address)
  })

  const healthFactor = computed(() => {
    const marketName = String(poolData.value?.market)
    const obligation = userStore.state.obligations[marketName]
    const marketState = marketsStore.state.markets[marketName]?.marketState
    if (!obligation || !marketState) {
      return 0
    }
    const assetDecimals = marketState.asset_decimals ?? 7
    const oraclePriceDecimals = marketState.oracle_price_decimals ?? 0
    const poolsData = marketState.pools_data

    const price = Number(poolData.value?.price || 0)
    const liabilityFactor = bpsToNumber(Number(poolData.value?.raw.pool.config.health_config.liability_factor_bps || 0))
    const borrowAdjustUsd = (amount.value || 0) * price * liabilityFactor

    return calcHealthFactor(obligation, poolsData, assetDecimals, oraclePriceDecimals, 0, borrowAdjustUsd)
  })

  const marketFee = computed(() => {
    const marketFeeBps = poolData.value?.raw.pool.config.fee_config.borrow_fee_bps
    return calcFee(Number(amount.value || 0), marketFeeBps || 0)
  })

  const dynamicUtilizationRate = computed(() => {
    const pool = poolData?.value?.raw.pool
    const assetDecimals = poolData?.value?.assetDecimals ?? 7
    if (!pool) {
      return poolData?.value?.utilization_rate ?? '-'
    }

    const totalBorrowed = Number(bigintToNumber(pool.total_borrowed, assetDecimals))
    const totalAvailable = Number(bigintToNumber(pool.total_available, assetDecimals))
    const borrowAmount = Number(amount.value) || 0

    const newTotalBorrowed = totalBorrowed + borrowAmount
    const newTotalAvailable = Math.max(totalAvailable - borrowAmount, 0)
    const denominator = newTotalAvailable + newTotalBorrowed
    const newUtil = denominator > 0 ? newTotalBorrowed / denominator * 100 : 0

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
            label: 'Pool Liquidity Available',
            value: shortenNumber(poolBorrowLimit.value || 0),
          },
          {
            label: 'Open LTV',
            value: poolData.value.open_ltv,
          },
          {
            label: 'Close LTV',
            value: truncatePercent(closeLTV.value || 0, 2),
          },
        ],
      },
      health: {
        title: 'Health',
        data: [
          {
            label: 'Health Factor',
            value: truncatePercent(healthFactor.value),
            slotName: 'hf',
          },
          {
            label: 'Borrowing Capacity',
            value: shortenNumber(availableToBorrow.value || 0),
          },
          {
            label: 'Util. Rate',
            value: dynamicUtilizationRate.value,
          },
        ],
      },
      fees: {
        title: 'Operation Fee',
        data: [
          {
            label: 'Operation Fee',
            value: `${formatPrice(marketFee.value, 0, 5)} ${poolData.value?.asset.symbol}`,
          },
          {
            label: 'Transaction Fee',
            value: `${txFee.value} ${poolData.value?.asset.symbol}`,
            slotName: 'txFee',
          },
        ],
      },
    }
  })

  const attentionText = computed(() =>
    isCanBorrow.value
      ? 'Parameter changes via governance can alter your account health factor and risk of liquidation.'
      : 'You cannot open a loan in the same pool where you have a deposit.')

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
        withBuffer: Number(amount.value) >= Number(availableToBorrow.value),
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
            publicKey.value,
            d.raw.pool.pool_address || '',
            0.01,
          )
          txFee.value = marketClient.value.borrowing.getTransactionFee(tx)
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

  onUnmounted(() => {
    stopBorrowWatchers()
  })

  return {
    marketClient,
    agree,
    isLoading,
    isLoadingFee,
    reloadFee,
    marketFee,
    txFee,
    amount,
    healthFactor,
    poolBorrowLimit,
    availableToBorrow,
    closeLTV,
    liquidationPenalty,
    dynamicUtilizationRate,
    isCanBorrow,
    attentionText,
    infoPanelData,
    borrow,
    startFeeInterval,
    stopBorrowWatchers,
  }
}
