import type { ApiHistoryData, ApiPoolData, PoolHistoryBucket } from '~/services'
import { fetchPoolData, fetchPoolHistory } from '~/services'

export const useMarketStatisticsStore = defineStore('market-statistics', () => {
  const state = reactive<MarketStatisticsState>({
    pool: undefined,
    pairPool: undefined,
    loading: false,
  })

  const historyMap = ref<Map<string, ApiHistoryData[]>>(new Map())

  const route = useRoute()

  const routePoolAddresses = computed(() => parseRoutePoolAddresses(route.params?.pool as string | undefined))

  const marketAddress = computed(() => route.params.market as string)
  const poolAddress = computed(() => routePoolAddresses.value.poolAddress)
  const pairPoolAddress = computed(() => routePoolAddresses.value.pairPoolAddress)

  async function getPoolHistoryData(marketAddress: string, poolAddress: string, bucket?: PoolHistoryBucket) {
    try {
      if (historyMap.value.has(`${marketAddress}-${poolAddress}-${bucket ?? '1d'}`)) {
        return
      }
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

  async function getPoolData(marketAddress: string, poolAddress: string, target: 'pool' | 'pairPool' = 'pool') {
    try {
      const data = await fetchPoolData(marketAddress, poolAddress)
      state[target] = data
      console.log(`%c[Pool Data]`, 'color: #1dc978', data)
    } catch (error) {
      console.error(error)
    }
  }

  watch(route, async () => {
    if (!import.meta.client) {
      historyMap.value.clear()
      state.pool = undefined
      state.pairPool = undefined
      return
    }
    const marketAddr = route.params?.market as string
    const { poolAddress: poolAddr, pairPoolAddress: pairPoolAddr } = parseRoutePoolAddresses(route.params?.pool as string | undefined)

    if (!marketAddr || !poolAddr) {
      historyMap.value.clear()
      state.pool = undefined
      state.pairPool = undefined
      return
    }
    if ('page' in route.params && route.params.page !== 'pool') {
      return
    }

    const hasPrimaryHistory = historyMap.value.has(`${marketAddr}-${poolAddr}-1d`)
    const hasPairHistory = !pairPoolAddr || historyMap.value.has(`${marketAddr}-${pairPoolAddr}-1d`)
    const hasPrimaryPoolData = state.pool?.pool === poolAddr
    const hasPairPoolData = !pairPoolAddr || state.pairPool?.pool === pairPoolAddr

    if (hasPrimaryHistory && hasPairHistory && hasPrimaryPoolData && hasPairPoolData) {
      return
    }

    const promises = [
      () => hasPrimaryHistory ? Promise.resolve() : getPoolHistoryData(marketAddr, poolAddr, '1d'),
      () => hasPrimaryPoolData ? Promise.resolve() : getPoolData(marketAddr, poolAddr),
    ]

    if (pairPoolAddr) {
      promises.push(() => hasPairHistory ? Promise.resolve() : getPoolHistoryData(marketAddr, pairPoolAddr, '1d'), () => hasPairPoolData ? Promise.resolve() : getPoolData(marketAddr, pairPoolAddr, 'pairPool'))
    } else {
      state.pairPool = undefined
    }

    await Promise.all(promises.map(cb => cb()))
  }, { immediate: true })

  return {
    state,
    historyMap,

    marketAddress,
    poolAddress,
    pairPoolAddress,

    getPoolHistoryData,
  }
})

type MarketStatisticsState = {
  loading: boolean
  pool: ApiPoolData | undefined
  pairPool: ApiPoolData | undefined
}

function parseRoutePoolAddresses(poolParam?: string) {
  const [poolAddress, pairPoolAddress] = poolParam?.split(':') ?? []

  return {
    poolAddress: poolAddress ?? '',
    pairPoolAddress: pairPoolAddress || undefined,
  }
}
