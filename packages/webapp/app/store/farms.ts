import type { FarmState } from '@alula/farms-sdk'

export const useFarmsStore = defineStore('farms', () => {
  const state = reactive<FarmsStoreState>({
    loading: false,
    farms: [],
  })

  const clientStore = useClientStore()
  const marketsStore = useMarketsStore()
  const markets = computed(() => marketsStore.state.markets)

  watch(markets, async (m) => {
    if (Object.keys(m).length === 0 || state.farms.length > 0) {
      return
    }
    const values = Object.values(m)

    if (values.length > 0) {
      const farmsMap = new Map<string, FarmState>()
      for (const market of values) {
        const marketName = market.marketName
        const farmsContractAddress = await market.client?.market?.getFarmsContractAddress()
        if (!farmsContractAddress) {
          continue
        }
        const farmsClient = await clientStore.initFarmsClient(farmsContractAddress)
        const getMarketFarms = await farmsClient?.getMarketFarms()
        farmsMap.set(marketName, getMarketFarms)
      }
      console.log('%c[Farms]', 'color: #1dc978', farmsMap)
    //   const farmsData = await Promise.all(
    //     values.map(async (v) => {
    //       return {
    //         marketName: v.marketName,
    //         marketAddress: v.address,
    //         farms: await v.client?.farms?.getMarketFarms(),
    //       }
    //     }),
    //   )
    //   state.farms = farmsData.filter(Boolean)
      // console.log('%c[Farms]', 'color: #1dc978', state.farms)
    }
  })

  function getMarketFarms(address: string) {
    return state.farms.find(f => f?.marketAddress === address || f.marketName === address)?.farms
  }

  return {
    state,

    getMarketFarms,
  }
})

export type FarmsStoreState = {
  loading: boolean
  farms: {
    marketName: string
    marketAddress: string
    farms?: FarmState[]
  }[]
}
