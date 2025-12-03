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

    activeActionPool,

    loadPoolData,
  } = useMarket(state)

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
    const map = await alulaClient.value?.marketManagerSdk.getMarketList()
    state.marketsList = [...map]?.map(([address]) => address)
    console.log('%c[Markets Adresses]', 'color: #FFB726', state.marketsList)
  }

  async function loadMarketsData() {
    try {
      state.loading = true
      state.markets = {}
      state.marketsList = []

      await getMarketsList()

      const marketsWithState = await Promise.all(
        state.marketsList.map(async (address) => {
          const client = clientStore.initClient(address)
          const marketState = await client?.marketSdk.getMarketData() as MarketData
          return {
            client,
            marketState,
            address,
            marketName: marketState?.global_state?.name,
          }
        }),
      )

      state.markets = Object.fromEntries(marketsWithState.map(c => [c.marketName, c]))
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

    poolActiveAddress,

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

    activeActionPool,

    updatePool,
  }
})

export type MarketsState = {
  loading: boolean
  loadingLeveragePools: boolean
  marketsList: string[]
  markets: Record<string, {
    marketState: MarketData
    marketName: string
    address: string
    client: StellarClient
  }>
}

export type PoolWithPrice = {
  pool_price: number | string
  pool_apy: any
  market?: string
} & Pool

export type TableActionType = 'deposit' | 'withdraw' | 'borrow' | 'repay' | 'leverage' | 'withdrawLeverage'
