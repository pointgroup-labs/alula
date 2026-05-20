export function useMarket(state: MarketsState) {
  const dialogSupply = ref(false)
  const dialogBorrow = ref(false)
  const dialogRepay = ref(false)
  const dialogWithdraw = ref(false)
  const dialogLeverage = ref(false)
  const dialogLeverageWithdraw = ref(false)

  const marketInfoDialog = ref(false)

  const activeActionPool = ref<{
    market?: string
    poolAddress?: string
    poolActionType?: TableActionType
  }>({
    market: undefined,
    poolAddress: undefined,
    poolActionType: undefined,
  })

  const route = useRoute()
  const router = useRouter()

  const { publicKey } = useWalletComposable()

  const clientStore = useClientStore()

  const activeLeverageMarket = ref<string>('')

  const selectedMarketName = ref()
  const selectedPoolAddress = ref()

  const activeMarket = computed(() => state.markets[selectedMarketName.value])
  const marketClient = computed(() => activeMarket.value?.client)

  const selectedMarketPools = computed(() => activeMarket.value?.marketState.pools_data ?? [])
  const assetDecimals = computed(() => {
    const pool = selectedMarketPools.value?.find(p => p.pool.pool_address === selectedPoolAddress.value)
    return pool?.pool.token_decimals || 7
  })

  async function regenerateMarketClient() {
    const markets = Object.entries(state.markets)
    for (const [name, market] of markets) {
      market.client = await clientStore.initClient(market.address)
      state.markets[name] = market
    }
  }

  watchDebounced([
    publicKey,
    () => state.markets], async () => {
    if (Object.keys(state.markets).length === 0) {
      return
    }
    await regenerateMarketClient()
    console.log('%c[Regenerated market clients]', 'color: #FFB726', state.markets)
  }, { immediate: true, debounce: 200 })

  watch([
    dialogSupply,
    dialogBorrow,
    marketInfoDialog,
  ], ([supply, borrow, infoDialog]) => {
    const pool = selectedPoolAddress.value
    const market = selectedMarketName.value
    const query = { ...route.query }

    const map: Record<string, boolean> = {
      supply,
      borrow,
      'market-info': infoDialog,
    }

    const active = Object.entries(map).find(([, v]) => v)?.[0]

    if (active) {
      query.dialog = active
      query.pool = pool
      router.replace({ query: { ...query, dialog: active, pool, market } })
    } else {
      delete query.dialog
      delete query.pool
      delete query.market
      delete query['collateral-only']
      delete query['leverage-market']
      router.replace({ query })
    }
  })

  const stop = watch(() => state.markets, async (markets) => {
    const pools = Object.values(markets).flatMap(m => m.marketState.pools_data)
    if (pools?.length > 0) {
      const q = route.query
      selectedPoolAddress.value = q?.pool
      selectedMarketName.value = q?.market

      if (!pools.some(p => p.pool.pool_address === q?.pool) || !selectedPoolAddress.value) {
        stop()
        return
      }
      if (q.dialog === 'supply') {
        dialogSupply.value = true
      }
      if (q.dialog === 'borrow') {
        dialogBorrow.value = true
      }
      if (q.dialog === 'leverage') {
        activeLeverageMarket.value = String(q['leverage-market']) || ''
        dialogLeverage.value = true
      }
      if (q.dialog === 'market-info') {
        marketInfoDialog.value = true
      }
      stop()
    }
  })

  return {
    activeLeverageMarket,
    activeMarket,
    marketClient,
    selectedMarketPools,
    assetDecimals,
    selectedMarketName,
    selectedPoolAddress,

    dialogSupply,
    dialogBorrow,
    dialogRepay,
    dialogWithdraw,
    dialogLeverage,
    dialogLeverageWithdraw,
    marketInfoDialog,

    activeActionPool,
  }
}
