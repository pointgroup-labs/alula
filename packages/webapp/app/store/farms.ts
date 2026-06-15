import type { FarmState } from '@alula/farms-sdk'

export const useFarmsStore = defineStore('farms', () => {
  const state = reactive<FarmsStoreState>({
    loading: false,
    farms: new Map(),
  })

  const clientStore = useClientStore()
  const marketsStore = useMarketsStore()
  const markets = computed(() => marketsStore.state.markets)

  watch(markets, async (m) => {
    if (Object.keys(m).length === 0 || state.farms.size > 0) {
      return
    }
    const values = Object.values(m)

    if (values.length > 0) {
      const farmsMap = new Map<string, FarmState[]>()
      for (const market of values) {
        const marketName = market.marketName
        const farmsContractAddress = await market.client?.market?.getFarmsContractAddress()
        if (!farmsContractAddress) {
          continue
        }
        const farmsClient = await clientStore.initFarmsClient(farmsContractAddress)
        const getMarketFarms = await farmsClient?.getMarketFarms()
        if (getMarketFarms) {
          farmsMap.set(marketName, getMarketFarms)
        }
      }
      state.farms = farmsMap
      console.log('%c[Farms]', 'color: #2ced53', state.farms)
    }
  }, {
    immediate: true,
  })

  function getMarketFarms(marketName: string) {
    return state.farms.get(marketName)
  }

  return {
    state,

    getMarketFarms,
  }
})

export type FarmsStoreState = {
  loading: boolean
  farms: Map<string, FarmState[]>
}
