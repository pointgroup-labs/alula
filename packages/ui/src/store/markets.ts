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

  const {
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
  } = useMarket(state)

  const clientStore = useClientStore()
  const alulaClient = computed(() => clientStore.alulaClient)

  const rpcStore = useRpcStore()
  const network = computed(() => rpcStore.network)

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

  async function updatePool(pool_address: string, market: string, client: StellarClient) {
    const preparedPool = await preparePool(pool_address, client)
    const updatedMarketPool = state.markets[market]?.pools.map(p => (p.pool_address === pool_address ? preparedPool : p)) as PoolWithPrice[]
    state.markets[market] = {
      ...state.markets[market]!,
      pools: updatedMarketPool,
    }
  }

  async function updateLeveragePool(props: {
    deposit_pool_address: string
    borrow_pool_address: string
    market: string
    client: StellarClient
  }) {
    const { client, market, deposit_pool_address, borrow_pool_address } = props
    const newPoolData = await client.marketSdk.getLeveragePool(deposit_pool_address, borrow_pool_address)
    const updatedMarketPools = state.markets[market]?.leveragePools.map((p) => {
      return (p.deposit_pool === deposit_pool_address && p.borrow_pool === borrow_pool_address ? (newPoolData || p) : p)
    }) as MultiplyPair[]
    state.markets[market] = {
      ...state.markets[market]!,
      leveragePools: updatedMarketPools,
    }
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
          const [pools = [], leveragePools = []] = await Promise.all([
            loadMarketPools(client, marketState.name).then(v => v ?? []).catch(() => []),
            loadLeveragePools(client).then(v => v ?? []).catch(() => []),
          ])
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

  watch(network, async () => {
    await loadMarketsData()
  })

  return {
    state,

    marketClient,
    assetDecimals,

    activeMarket,
    activeMarketFilter,
    activeLeverageMarket,

    selectedMarketPools,

    dialogSupply,
    dialogBorrow,
    dialogLeverage,
    marketInfoDialog,
    dialogLeverageWithdraw,

    selectedMarketAddress,

    poolActionType,
    poolActiveAddress,

    updatePool,
    updateLeveragePool,

  }
})

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
