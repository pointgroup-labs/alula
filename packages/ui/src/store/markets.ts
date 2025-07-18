import type { CompoundRates, Pool } from 'sdk'
import { defineStore } from 'pinia'

export const useMarketsStore = defineStore('markets', () => {
  const state = reactive<MarketsState>({
    poolAddresses: [],
    pollsData: [],
    loading: false,
    markets: ['Main market', 'Assets'],
  })

  const clientStore = useClientStore()
  const jLendClient = computed(() => clientStore.jLendClient)

  const poolDepositAddr = ref()
  const poolActionType = ref<TableActionType>()

  const selectedMarketInfo = ref()

  const selectedMarket = ref('Main market')

  const selectedMarketPools = computed(() => {
    return state.pollsData.filter(p => selectedMarket.value.toLowerCase().includes(String(p.market?.toLowerCase())))
  })

  async function loadPools() {
    if (!isClient) {
      return
    }
    try {
      state.loading = true
      const allPools = await jLendClient.value?.sdk.getAllPools()
      state.poolAddresses = allPools
      state.pollsData = await Promise.all(
        allPools.map(async (pool_address: string) => await preparePool(pool_address)),
      )
      console.log('%c[Pools]', 'color: #FFB726', state.pollsData)
    } finally {
      state.loading = false
    }
  }

  async function preparePool(pool_address: string) {
    const poolInfo = await jLendClient.value?.sdk.getPoolInfo(pool_address)
    const pool_price = await jLendClient.value?.sdk.getPoolAssetOraclePrice(pool_address)
    const pool_apy = await jLendClient.value?.sdk.getPoolApy(pool_address)
    return {
      ...poolInfo,
      pool_price,
      pool_apy,
      market: 'main',
    }
  }

  async function updatePools(pool_address: string) {
    const preparedPool = await preparePool(pool_address)
    state.pollsData = state.pollsData.map(p => (p.pool_address === pool_address ? preparedPool : p))
  }

  onMounted(async () => {
    await loadPools()
  })

  return {
    state,

    selectedMarket,
    selectedMarketInfo,
    selectedMarketPools,

    poolActionType,
    poolDepositAddr,

    updatePools,

  }
})

export type MarketsState = {
  poolAddresses: string[]
  pollsData: PoolWithPrice[]
  loading: boolean
  markets: string[]
}

export type PoolWithPrice = {
  pool_price: number | string
  pool_apy: CompoundRates
  market?: string
} & Pool

export type TableActionType = 'deposit' | 'withdraw' | 'borrow' | 'repay'
