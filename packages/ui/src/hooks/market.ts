export function useMarket(state: MarketsState) {
  const dialogSupply = ref(false)
  const dialogBorrow = ref(false)
  const dialogLeverage = ref(false)
  const dialogLeverageWithdraw = ref(false)

  const marketInfoDialog = ref(false)

  const poolActiveAddress = ref()
  const poolActionType = ref<TableActionType>()

  const route = useRoute()
  const router = useRouter()

  const walletStore = useWallet()
  const publicKey = computed(() => walletStore.publicKey)

  const clientStore = useClientStore()

  const activeMarketFilter = ref<string>('')
  const activeLeverageMarket = ref<string>('')

  const activeMarket = computed(() => state.markets[activeMarketFilter.value])
  const marketClient = computed(() => activeMarket.value?.client)

  const selectedMarketPools = computed(() => activeMarket.value?.pools ?? [])
  const assetDecimals = computed(() => marketClient.value?.marketSdk.assetDecimals || 7)

  const selectedMarketAddress = ref()

  function regenerateMarketClient() {
    const markets = Object.entries(state.markets)
    for (const [name, market] of markets) {
      market.client = clientStore.initClient(market.address)
      state.markets[name] = market
    }
  }

  watch([publicKey, () => state.markets], async ([, markets]) => {
    if (Object.keys(markets).length === 0) {
      return
    }
    await regenerateMarketClient()
    console.log('%c[Regenerated market clients]', 'color: #FFB726', state.markets)
  })

  watch([
    () => route.path,
    activeMarketFilter,
  ], ([_path, marketName]) => {
    if (_path !== '/') {
      return
    }
    const q = { ...route.query }

    q.market = marketName
    router.replace({ query: { ...q } })
  })

  watch([
    dialogSupply,
    dialogBorrow,
    marketInfoDialog,
    dialogLeverage,
  ], ([supply, borrow, infoDialog, leverage]) => {
    const pool = selectedMarketAddress.value
    const query = { ...route.query }

    const map: Record<string, boolean> = {
      supply,
      borrow,
      'market-info': infoDialog,
      leverage,
    }

    const active = Object.entries(map).find(([, v]) => v)?.[0]

    if (active) {
      query.dialog = active
      query.pool = pool
      if (leverage) {
        query['leverage-market'] = activeLeverageMarket.value
      }
      router.replace({ query: { ...query, dialog: active, pool } })
    } else {
      delete query.dialog
      delete query.pool
      delete query['collateral-only']
      delete query['leverage-market']
      router.replace({ query })
    }
  })

  const stop = watch(() => state.markets, async (markets) => {
    const pools = Object.values(markets).flatMap(m => m.pools)
    if (pools?.length > 0) {
      const q = route.query
      selectedMarketAddress.value = q?.pool

      if (!pools.some(p => p.pool_address === q?.pool) || !selectedMarketAddress.value) {
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

  onMounted(() => {
    const activeMarketQuery = route.query?.market
    if (activeMarketQuery) {
      activeMarketFilter.value = String(activeMarketQuery)
    }
  })
  return {
    activeMarketFilter,
    activeLeverageMarket,
    activeMarket,
    marketClient,
    selectedMarketPools,
    assetDecimals,
    selectedMarketAddress,

    dialogSupply,
    dialogBorrow,
    dialogLeverage,
    dialogLeverageWithdraw,
    marketInfoDialog,

    poolActiveAddress,
    poolActionType,

    preparePool,
    loadMarketPools,
  }
}

async function loadMarketPools(client?: any, marketName?: string) {
  if (!client) {
    return
  }
  try {
    const allPools = await client.marketSdk.getAllPools()
    console.log(`%c[${marketName} Pools]`, 'color: #FFB726', allPools)
    return await Promise.all(
      allPools.map(async (pool_address: string) => await preparePool(pool_address, client)),
    )
  } catch (error) {
    console.log(error)
  }
}

async function preparePool(pool_address: string, client?: any) {
  const [poolInfo, pool_price, pool_apy] = await Promise.all([
    client?.marketSdk.getPoolInfo(pool_address),
    client?.marketSdk.getPoolAssetOraclePrice(pool_address),
    client?.marketSdk.getPoolApy(pool_address),
  ])
  return {
    ...poolInfo,
    pool_price,
    pool_apy,
  }
}
