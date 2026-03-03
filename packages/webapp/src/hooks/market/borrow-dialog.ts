import type { MarketTableItem } from '~/types/table'
import { bpsToNumber } from '@alula/client-sdk'
import { calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'
import { truncatePercent } from '~/utils'

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
    const marketAvailableInUsd = Number(poolBorrowLimit.value) * Number(poolData.value.price)

    if (!obligation || !marketState) {
      return 0
    }

    const assetDecimals = marketState.asset_decimals ?? 7
    const oraclePriceDecimals = marketState.oracle_price_decimals ?? 0
    const poolsData = marketState.pools_data

    const userDepositWithCloseLtv = calcUserTotalStakeInUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals, 'open')
    const userTotalBorrowedInUsd = calcUserTotalBorrowedInUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals) ?? 0

    // max borrow so that HF stays >= 1.1: extra = depositWithOpenLtv / 1.1 - borrowed
    const userAvailableUsd = Math.max(userDepositWithCloseLtv / 1.1 - userTotalBorrowedInUsd, 0)
    const maxAvailableUsd = Math.min(userAvailableUsd, marketAvailableInUsd)
    const maxAvailableAssets = maxAvailableUsd / Number(poolData.value.price)

    return Number(truncatePercent(maxAvailableAssets, assetDecimals))
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
