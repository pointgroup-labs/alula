import type { Pool } from 'sdk'
import { defineStore } from 'pinia'

export const useMarketsStore = defineStore('markets', () => {
  const state = reactive<MarketsState>({
    poolAddresses: [],
    pollsData: [],
    loading: false,
  })

  const connectionStore = useConnectionStore()

  const jLendClient = computed(() => connectionStore.jLendClient)
  const selectedMarketInfo = ref()

  async function loadPools() {
    try {
      state.loading = true
      const allPools = await jLendClient.value?.sdk.getAllPools()
      state.poolAddresses = allPools
      state.pollsData = await Promise.all(
        allPools.map(async (pool_address: string) => {
          const poolInfo = await jLendClient.value?.sdk.getPoolInfo(pool_address)
          const pool_price = await jLendClient.value?.sdk.getPoolAssetOraclePrice(pool_address)
          return {
            ...poolInfo,
            pool_price,
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

    selectedMarketInfo,
  }
})

export type MarketsState = {
  poolAddresses: string[]
  pollsData: PoolWithPrice[]
  loading: boolean
}

export type PoolWithPrice = {
  pool_price: number
} & Pool