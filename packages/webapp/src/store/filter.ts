export const useMarketFilterStore = defineStore('market-filter', () => {
  const marketsStore = useMarketsStore()

  const collateralFilter = ref<Record<string, boolean>>({})
  const debtFilter = ref<Record<string, boolean>>({})

  const ASSETS_MAP = new Map()
  const allMarkets = ref<MarketFullData>({})

  const isActiveCollateralFilter = computed(() => Object.values(collateralFilter.value).some(Boolean))
  const isActiveDebtFilter = computed(() => Object.values(debtFilter.value).some(Boolean))

  const uniqueAssets = computed(() => {
    for (const market of Object.values(allMarkets.value)) {
      const poolsData = market.marketState.pools_data
      for (const pool of poolsData) {
        if (ASSETS_MAP.has(pool.pool.token_symbol)) {
          continue
        }
        const assetData = getFullTokenData(pool.pool.token_symbol)
        ASSETS_MAP.set(assetData.symbol, assetData)
      }
    }
    return [...ASSETS_MAP.values()]
  })

  function marketToggle(filters: Record<string, boolean>, symbol: string) {
    filters[symbol] = !filters[symbol]
  }

  watch(() => marketsStore.state.markets, (next, prev) => {
    const prevKeys = Object.keys(prev)
    const nextKeys = Object.keys(next)
    if (prevKeys.length === nextKeys.length) {
      return
    }
    allMarkets.value = next
  })

  watch(uniqueAssets, (assets) => {
    for (const asset of assets) {
      if (!(asset.symbol in collateralFilter.value)) {
        collateralFilter.value[asset.symbol] = false
        debtFilter.value[asset.symbol] = false
      }
    }
  }, { immediate: true })
  return {
    uniqueAssets,
    collateralFilter,
    debtFilter,

    isActiveCollateralFilter,
    isActiveDebtFilter,

    marketToggle,
  }
})
