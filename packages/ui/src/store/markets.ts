// import type { CompoundRates, Pool } from '@jlend/sdk'
import type { StellarClient } from '@alula/client-sdk'
import type { GlobalState, MultiplyPair, Pool } from '@alula/market-sdk'
import { defineStore } from 'pinia'

export const useMarketsStore = defineStore('markets', () => {
  const state = reactive<MarketsState>({
    loading: false,
    loadingLeveragePools: false,
    marketsList: [],
    markets: {},
  })

  const route = useRoute()
  const router = useRouter()

  const walletStore = useWallet()
  const publicKey = computed(() => walletStore.publicKey)

  const clientStore = useClientStore()
  const alulaClient = computed(() => clientStore.alulaClient)

  const rpcStore = useRpcStore()
  const network = computed(() => rpcStore.network)

  const poolActiveAddress = ref()
  const poolActionType = ref<TableActionType>()

  const dialogSupply = ref(false)
  const dialogBorrow = ref(false)
  const dialogLeverage = ref(false)
  const dialogLeverageWithdraw = ref(false)

  // selected pool address to show market info in supply/borrow dialogs
  const selectedMarketAddress = ref()

  const marketInfoDialog = ref(false)

  const activeMarketFilter = ref<string>('')

  const activeMarket = computed(() => state.markets[activeMarketFilter.value])
  const marketClient = computed(() => activeMarket.value?.client)

  const selectedMarketPools = computed(() => activeMarket.value?.pools ?? [])
  const assetDecimals = computed(() => marketClient.value?.marketSdk.assetDecimals || 7)

  async function loadLeveragePools(client?: any) {
    if (!isClient || !client) {
      return
    }
    try {
      state.loadingLeveragePools = true
      const allPools = await client.marketSdk.getAllLeveragePools()
      return allPools
    } finally {
      state.loadingLeveragePools = false
    }
  }

  async function updatePools(pool_address: string, market: string, client: StellarClient) {
    const preparedPool = await preparePool(pool_address, client)
    const updatedMarketPool = state.markets[market]?.pools.map(p => (p.pool_address === pool_address ? preparedPool : p)) as PoolWithPrice[]
    state.markets[market] = {
      ...state.markets[market]!,
      pools: updatedMarketPool,
    }
    console.log('%c[Updated pool]', 'color: #FFB726', preparedPool)
  }

  async function updateLeveragePools(props: {
    deposit_pool_address: string
    borrow_pool_address: string
    market: string
    client: StellarClient
  }) {
    const newPoolData = await props.client.marketSdk.getLeveragePool(props.deposit_pool_address, props.borrow_pool_address)
    const updatedMarketPools = state.markets[props.market]?.leveragePools.map((p) => {
      return (p.deposit_pool === props.deposit_pool_address && p.borrow_pool === props.borrow_pool_address ? (newPoolData || p) : p)
    }) as MultiplyPair[]
    state.markets[props.market] = {
      ...state.markets[props.market]!,
      leveragePools: updatedMarketPools,
    }
    console.log('%c[Updated leverage pool]', 'color: #FFB726', newPoolData)
  }

  async function getMarketsList() {
    state.marketsList = await alulaClient.value?.marketManagerSdk.getMarketList()
    console.log('%c[Markets list]', 'color: #FFB726', state.marketsList)
  }

  async function loadMarketsData() {
    try {
      state.loading = true
      state.markets = {}
      state.marketsList = []

      await getMarketsList()

      const results = await Promise.all(
        state.marketsList.map(async (market) => {
          const client = clientStore.initClient(market)
          const marketState = await client?.marketSdk.getMarketData()
          const pools = await loadMarketPools(client, marketState.name) ?? []
          const leveragePools = await loadLeveragePools(client) ?? []
          return {
            name: marketState.name,
            address: market,
            marketState,
            pools,
            leveragePools,
            client,
          }
        }),
      )

      state.markets = results.reduce((acc, { name, address, marketState, pools, leveragePools, client }) => {
        acc[name] = { marketState, pools, address, leveragePools, client }
        return acc
      }, {} as typeof state.markets)
      console.log('%c[Markets info]', 'color: #FFB726', state.markets)
    } catch (error) {
      console.log(error)
    } finally {
      state.loading = false
    }
  }

  function regenerateMarketClient() {
    const markets = Object.entries(state.markets)
    for (const [name, market] of markets) {
      market.client = clientStore.initClient(market.address)
      state.markets[name] = market
    }
  }

  watch(network, async () => {
    await loadMarketsData()
  })

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
  ], ([path, marketName]) => {
    if (path !== '/') {
      return
    }
    const q = { ...route.query }

    q['active-market'] = marketName
    router.replace({ query: { ...q } })
  })

  watch([
    dialogSupply,
    dialogBorrow,
    marketInfoDialog,
    dialogLeverage,
    dialogLeverageWithdraw,
  ], ([supply, borrow, infoDialog, leverage, withdrawLeverage]) => {
    const market = selectedMarketAddress.value
    const query = { ...route.query }

    const map: Record<string, boolean> = {
      supply,
      borrow,
      'market-info': infoDialog,
      leverage,
      'withdraw-leverage': withdrawLeverage,
    }

    const active = Object.entries(map).find(([, v]) => v)?.[0]

    if (active) {
      router.replace({ query: { ...query, dialog: active, market } })
    } else {
      delete query.dialog
      delete query.market
      delete query['collateral-only']
      router.replace({ query })
    }
  })

  const stop = watch(selectedMarketPools, (pools) => {
    if (pools?.length > 0) {
      const q = route.query
      selectedMarketAddress.value = q?.market

      if (!pools.some(p => p.pool_address === q?.market) || !selectedMarketAddress.value) {
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
        dialogLeverage.value = true
      }
      if (q.dialog === 'withdraw-leverage') {
        dialogLeverageWithdraw.value = true
      }
      if (q.dialog === 'market-info') {
        marketInfoDialog.value = true
      }
      stop()
    }
  })

  onMounted(() => {
    const activeMarketQuery = route.query?.['active-market']
    if (activeMarketQuery) {
      activeMarketFilter.value = String(activeMarketQuery)
    }
  })

  return {
    state,

    marketClient,
    assetDecimals,

    activeMarket,
    activeMarketFilter,

    selectedMarketPools,

    dialogSupply,
    dialogBorrow,
    dialogLeverage,
    marketInfoDialog,
    dialogLeverageWithdraw,

    selectedMarketAddress,

    poolActionType,
    poolActiveAddress,

    updatePools,
    updateLeveragePools,

  }
})

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

export type MarketsState = {
  loading: boolean
  loadingLeveragePools: boolean
  marketsList: string[]
  markets: Record<string, {
    marketState: GlobalState
    address: string
    pools: PoolWithPrice[]
    leveragePools: MultiplyPair[]
    client: StellarClient
  }>
}

export type PoolWithPrice = {
  pool_price: number | string
  pool_apy: any
  market?: string
} & Pool

export type TableActionType = 'deposit' | 'withdraw' | 'borrow' | 'repay' | 'leverage' | 'withdrawLeverage'
