// import type { CompoundRates, Pool } from '@jlend/sdk'
import type { StellarClient } from '@alula/client-sdk'
import type { MarketData, Pool, PoolData } from '@alula/market-sdk'
import { defineStore } from 'pinia'

export const useMarketsStore = defineStore('markets', () => {
  const state = reactive<MarketsState>({
    loading: false,
    loadingLeveragePools: false,
    marketsList: [],
    markets: {},
  })

  const {
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
  } = useMarket(state)

  const toast = useToast()

  const clientStore = useClientStore()
  const alulaClient = computed(() => clientStore.alulaClient)

  const rpcStore = useRpcStore()
  const network = computed(() => rpcStore.network)

  const poolActiveAddress = ref<string>()

  async function updatePool(pool_address: string, market: string, client: StellarClient, withLogs = true) {
    const poolData = await loadPoolData(pool_address, client)
    const updatedMarketPool = state.markets[market]?.marketState.pools_data.map(data => (data.pool.pool_address === pool_address ? poolData : data)) as PoolData[]
    state.markets[market] = {
      ...state.markets[market]!,
      marketState: {
        ...state.markets[market]!.marketState,
        pools_data: updatedMarketPool,
      },
    }

    if (withLogs) {
      console.log('%c[Updated pool]', 'color: #FFB726', poolData)
    }
  }

  async function getMarketsList() {
    const map = await alulaClient.value?.marketManager.getMarketList()
    state.marketsList = [...map]?.map(([address]) => address)
    console.log('%c[Markets Addresses]', 'color: #FFB726', state.marketsList)
  }

  const debouncedMarketFn = useDebounceFn(loadMarketsData, 100)

  async function loadMarketsData() {
    try {
      state.loading = true
      state.markets = {}
      state.marketsList = []

      await getMarketsList()

      const marketsWithState = await Promise.all(
        state.marketsList.map(async (address) => {
          const client = await clientStore.initClient(address)
          const marketState = await client?.market.getMarketData() as MarketData
          return {
            client,
            marketState,
            address,
            marketName: marketState?.global_state?.name,
          }
        }),
      )

      type Market = typeof marketsWithState[number]

      const marketsByName: Record<string, Market> = {}
      const counters: Record<string, number> = {}

      for (const market of marketsWithState) {
        const base = market.marketName

        const count = counters[base] ?? 0
        counters[base] = count + 1

        const key = count === 0
          ? base
          : `${base}_${count}`

        marketsByName[key] = market
      }

      // @ts-expect-error...
      state.markets = marketsByName
      console.log('%c[Markets info]', 'color: #FFB726', state.markets)
    } catch (error: any) {
      console.log(error)
      toast.create({
        title: 'Error',
        body: String(error?.message || error),
        variant: 'danger',
      })
    } finally {
      state.loading = false
    }
  }

  watch([
    network,
    alulaClient,
    () => rpcStore.horizonRPCUrl,
    () => rpcStore.sorobanRPCUrl],
  async ([nextNetwork, _nextClient, _nextHorizonRPCUrl, _nextSorobanRPCUrl],
    [prevNetwork, _prevClient, _prevHorizonRPCUrl, _prevSorobanRPCUrl]) => {
    if (import.meta.env.SSR) {
      return
    }
    if (nextNetwork !== prevNetwork || _nextHorizonRPCUrl !== _prevHorizonRPCUrl || _nextSorobanRPCUrl !== _prevSorobanRPCUrl) {
      state.markets = {}
      state.marketsList = []
    }
    if (Object.keys(state.markets).length === 0 && alulaClient.value?.market) {
      debouncedMarketFn()
    }
  }, {})

  return {
    state,

    marketClient,
    assetDecimals,

    poolActiveAddress,

    activeMarket,
    activeLeverageMarket,

    selectedMarketPools,

    dialogSupply,
    dialogBorrow,
    dialogRepay,
    dialogWithdraw,
    dialogLeverage,
    marketInfoDialog,
    dialogLeverageWithdraw,

    selectedMarketName,
    selectedPoolAddress,

    activeActionPool,

    updatePool,
  }
})

export type MarketsState = {
  loading: boolean
  loadingLeveragePools: boolean
  marketsList: string[]
  markets: MarketFullData
}

export type MarketFullData = Record<string, {
  marketState: MarketData
  marketName: string
  address: string
  client: StellarClient
}>

export type PoolWithPrice = {
  pool_price: number | string
  pool_apy: any
  market?: string
} & Pool

export type TableActionType = 'deposit' | 'withdraw' | 'borrow' | 'repay' | 'leverage' | 'withdrawLeverage'

async function loadPoolData(address: string, client: any) {
  try {
    const poolData = await client.market.getPoolData(address)
    return poolData
  } catch (error) {
    console.log(error)
  }
}
