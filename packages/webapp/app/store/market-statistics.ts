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

  const routePoolAddresses = computed(() => {
    const poolParam = route.params?.pool ?? route.params?.pair
    const poolParamString = Array.isArray(poolParam) ? poolParam[0] : poolParam
    return parseRoutePoolAddresses(poolParamString)
  })

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
    const marketAddrParam = route.params?.market as string
    const poolAddrParam = route.params?.pool ?? route.params?.pair
    const { poolAddress: poolAddr, pairPoolAddress: pairPoolAddr } = parseRoutePoolAddresses(poolAddrParam as string | undefined)

    if (!marketAddrParam || !poolAddrParam) {
      historyMap.value.clear()
      state.pool = undefined
      state.pairPool = undefined
      return
    }

    if ('page' in route.params && !['pool', 'statistics'].includes(route.params.page as string)) {
      return
    }

    const hasPrimaryHistory = historyMap.value.has(`${marketAddrParam}-${poolAddr}-1d`)
    const hasPairHistory = !pairPoolAddr || historyMap.value.has(`${marketAddrParam}-${pairPoolAddr}-1d`)
    const hasPrimaryPoolData = state.pool?.pool === poolAddr
    const hasPairPoolData = !pairPoolAddr || state.pairPool?.pool === pairPoolAddr

    if (hasPrimaryHistory && hasPairHistory && hasPrimaryPoolData && hasPairPoolData) {
      return
    }

    const promises = [
      () => hasPrimaryHistory ? Promise.resolve() : getPoolHistoryData(marketAddrParam, poolAddr, '1d'),
      () => hasPrimaryPoolData ? Promise.resolve() : getPoolData(marketAddrParam, poolAddr),
    ]

    if (pairPoolAddr) {
      promises.push(() => hasPairHistory
        ? Promise.resolve()
        : getPoolHistoryData(marketAddrParam, pairPoolAddr, '1d'), () => hasPairPoolData
        ? Promise.resolve()
        : getPoolData(marketAddrParam, pairPoolAddr, 'pairPool'))
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
