import type { ApiHistoryData, ApiPoolData, PoolHistoryBucket } from '~/services'
import { fetchPoolData, fetchPoolHistory } from '~/services'

export const useMarketStatisticsStore = defineStore('market-statistics', () => {
  const state = reactive<MarketStatisticsState>({
    pool: undefined,
    loading: false,
  })

  const historyMap = ref<Map<string, ApiHistoryData[]>>(new Map())

  const route = useRoute()

  const marketAddress = computed(() => route.params.market as string)
  const poolAddress = computed(() => route.params.pool as string)

  async function getPoolHistoryData(marketAddress: string, poolAddress: string, bucket?: PoolHistoryBucket) {
    try {
      state.loading = true
      const data = await fetchPoolHistory(marketAddress, poolAddress, bucket)
      historyMap.value.set(`${marketAddress}-${poolAddress}-${bucket ?? '1d'}`, data)
      console.log(`%c[Pool History: bucket=${bucket ?? 'empty'}]`, 'color: #1dc978', data)
    } catch (error) {
      console.error(error)
    } finally {
      state.loading = false
    }
  }

  async function getPoolData(marketAddress: string, poolAddress: string) {
    try {
      const data = await fetchPoolData(marketAddress, poolAddress)
      state.pool = data
      console.log(`%c[Pool Data]`, 'color: #1dc978', data)
    } catch (error) {
      console.error(error)
    }
  }

  watch(route, async () => {
    if (!import.meta.client) {
      return
    }
    const market = route.params?.market as string
    const pool = route.params?.pool as string
    if (!market || !pool) {
      historyMap.value.clear()
      state.pool = undefined
      return
    }
    await Promise.all([
      getPoolHistoryData(market, pool, '1d'),
      getPoolData(market, pool),
    ])
  }, { immediate: true })

  return {
    state,
    historyMap,

    marketAddress,
    poolAddress,

    getPoolHistoryData,
  }
})

type MarketStatisticsState = {
  loading: boolean
  pool: ApiPoolData | undefined
}
