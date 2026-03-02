import type { MarketTableItem } from '~/types/table'
import { calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'

export function useBorrowDialog(data: MaybeRef<MarketTableItem | undefined>, isCalcFee: boolean = true) {
  const wallet = useWallet()
  const publicKey = computed(() => wallet.publicKey)

  const marketsStore = useMarketsStore()

  const userStore = useUserStore()

  const marketClient = computed(() => marketsStore.marketClient)

  const poolData = computed(() => unref(data))

  const agree = ref(false)

  const reloadFee = ref(false)
  const txFee = ref(0)

  const isLoading = computed(() => marketsStore.poolActiveAddress === poolData.value?.raw.pool.pool_address)

  const poolBorrowLimit = computed(() => {
    if (!poolData.value) {
      return 0
    }
    const utilRatioLimit = Number(poolData.value?.raw.pool.config.health_config.utilization_ratio_limit_bps || 0) / 10_000
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
    const userTotalBorrowedInUsd = Number(userStore.userTotalBorrowedInUsd) || 0
    const marketAvailableInUsd = Number(poolBorrowLimit.value) * Number(poolData.value.price)

    let userAvailableByLTV = 0
    if (obligation && marketState) {
      const assetDecimals = marketState.asset_decimals ?? 7
      const oraclePriceDecimals = marketState.oracle_price_decimals ?? 0
      userAvailableByLTV = calcUserTotalStakeInUsd(obligation, marketState.pools_data, assetDecimals, oraclePriceDecimals, 'open')
    }

    const userAvailable = Math.max(userAvailableByLTV - userTotalBorrowedInUsd, 0)
    const maxAvailableUsd = Math.min(userAvailable, marketAvailableInUsd)
    const maxAvailableAssets = maxAvailableUsd / Number(poolData.value.price)

    return marketAvailableInUsd > userAvailable ? maxAvailableAssets : Math.floor(maxAvailableAssets)
  })

  const closeLTV = computed(() => Number(poolData.value?.raw.pool.config.health_config.close_ltv_bps || 0) / 100)

  const liquidationPenalty = computed(() => Number(poolData.value?.raw.pool.config.health_config.liquidation_close_factor_bps || 0) / 100)

  const isCanBorrow = computed(() => {
    const depositObligations = userStore.state.obligations[String(poolData.value?.market)]?.deposits ?? []
    return checkIsCanUsePool(depositObligations, poolData.value?.raw.pool.pool_address)
  })

  const attentionText = computed(() =>
    isCanBorrow.value
      ? 'Parameter changes via governance can alter your account health factor and risk of liquidation.'
      : 'You cannot open a loan in the same pool where you have a deposit.')

  if (isCalcFee) {
    watchDebounced([
      poolData,
      reloadFee,
      publicKey,
    ], async ([d, _r]) => {
      if (!d || !publicKey.value || !marketClient.value) {
        return
      }
      const tx = await marketClient.value?.borrowing.buildBorrowTx(
        publicKey.value,
        d?.raw.pool.pool_address || '',
        0,
      )
      txFee.value = marketClient.value.borrowing.getTransactionFee(tx)
    }, { immediate: true, debounce: 300 })
  }

  return {
    marketClient,
    agree,
    isLoading,
    reloadFee,
    txFee,
    poolBorrowLimit,
    availableToBorrow,
    closeLTV,
    liquidationPenalty,
    isCanBorrow,
    attentionText,
  }
}
