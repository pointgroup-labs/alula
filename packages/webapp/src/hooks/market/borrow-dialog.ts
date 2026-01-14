import type { MarketTableItem } from '~/types/table'

export function useBorrowDialog(data: MaybeRef<MarketTableItem | undefined>) {
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
    const userTotalDepositInUsd = userStore.userTotalDepositInUsd
    const userTotalBorrowedInUsd = Number(userStore.userTotalBorrowedInUsd) || 0
    const openLTV = Number(poolData.value?.raw.pool.config.health_config.open_ltv_bps || 0) / 10_000
    const marketAvailableInUsd = Number(poolBorrowLimit.value) * Number(poolData.value.price)
    const userAvailableByLTV = Number(userTotalDepositInUsd * openLTV) || 0
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

  watchDebounced([
    poolData,
    reloadFee,
    publicKey,
  ], async ([d, _r]) => {
    if (!d || !publicKey.value || !marketClient.value) {
      return
    }
    const tx = await marketClient.value?.marketSdk.borrowTx(
      publicKey.value,
      d?.raw.pool.pool_address || '',
      0,
    )
    txFee.value = marketClient.value.marketSdk.getTransactionFee(tx)
  }, { immediate: true, debounce: 300 })

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
  }
}
