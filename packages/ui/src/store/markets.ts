import type { CompoundRates, Pool } from 'sdk'
import { defineStore } from 'pinia'

export const useMarketsStore = defineStore('markets', () => {
  const state = reactive<MarketsState>({
    poolAddresses: [],
    pollsData: [],
    loading: false,
    markets: ['Main market', 'Assets'],
  })

  const connectionStore = useConnectionStore()

  const jLendClient = computed(() => connectionStore.jLendClient)
  const selectedMarketInfo = ref()

  const selectedMarket = ref('Main market')

  async function loadPools() {
    try {
      state.loading = true
      const allPools = await jLendClient.value?.sdk.getAllPools()
      state.poolAddresses = allPools
      state.pollsData = await Promise.all(
        allPools.map(async (pool_address: string) => {
          const poolInfo = await jLendClient.value?.sdk.getPoolInfo(pool_address)
          const pool_price = await jLendClient.value?.sdk.getPoolAssetOraclePrice(pool_address)
          const pool_apy = await jLendClient.value?.sdk.getPoolApy(pool_address)
          return {
            ...poolInfo,
            pool_price,
            pool_apy,
            market: 'main',
          }
        }),
      )
    } finally {
      state.loading = false
    }
  }

  onMounted(async () => {
    await loadPools()
  })

  return {
    state,

    selectedMarket,
    selectedMarketInfo,
  }
})

export type MarketsState = {
  poolAddresses: string[]
  pollsData: PoolWithPrice[]
  loading: boolean
  markets: string[]
}

export type PoolWithPrice = {
  pool_price: number
  pool_apy: CompoundRates
  market?: string
} & Pool
