import type { MarketTableItem } from '~/types/table'

export function useSupplyDialog(data: MaybeRef<MarketTableItem | undefined>) {
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

  const isSupplyLimited = computed(() => poolData.value?.supply_limit && poolData.value?.supply_limit > 0)
  const supplyLimit = computed(() => isSupplyLimited.value ? Math.max(Number(poolData.value?.supply_limit) || 0 - Number(poolData.value?.total_supply), 0) : 0)
  const limitLabel = computed(() => isSupplyLimited.value ? formatPrice(Number(poolData.value?.supply_limit) || 0, 2, 2) : '-')

  const contractAddress = computed(() => poolData.value?.raw.pool.pool_address || '')

  const isCanSupply = computed(() => {
    const borrowObligations = userStore.state.obligations[String(poolData.value?.market)]?.borrows ?? []
    return checkIsCanUsePool(borrowObligations, poolData.value?.raw.pool.pool_address)
  })

  watchDebounced([
    poolData,
    reloadFee,
    publicKey,
  ], async ([d, _r]) => {
    try {
      isLoadingFee.value = true

      if (!d || !publicKey.value || !marketClient.value) {
        return
      }

      const tx = await marketClient.value.marketSdk.depositTx(
        publicKey.value,
        d?.raw.pool.pool_address || '',
        0,
      )
      txFee.value = marketClient.value.marketSdk.getTransactionFee(tx)
    } finally {
      isLoadingFee.value = false
    }
  }, { immediate: true, debounce: 300 })

  watchDebounced(collateralOnly, (c) => {
    const query = { ...route.query }
    if (c) {
      query['collateral-only'] = 'true'
    } else {
      delete query['collateral-only']
    }
    router.replace({ query })
  }, { debounce: 100 })

  watch(() => route.query, (q) => {
    if (q['collateral-only']) {
      collateralOnly.value = true
    }
  }, { immediate: true, once: true })

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
    isLoading,
    isCanSupply,
  }
}
