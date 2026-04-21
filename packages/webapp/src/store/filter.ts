export type FilterScope = 'markets' | 'multiply'
type FilterType = 'collateral' | 'debt'

type AssetFilters = {
  collateral: Record<string, boolean>
  debt: Record<string, boolean>
}

type FilterState = {
  markets: AssetFilters
  multiply: AssetFilters
}

export const useMarketFilterStore = defineStore('market-filter', () => {
  const filters = ref<FilterState>({
    markets: {
      collateral: {},
      debt: {},
    },
    multiply: {
      collateral: {},
      debt: {},
    },
  })
  const marketsStore = useMarketsStore()
  const { getFullTokenData } = useTokensStore()

  const collateralFilter = ref<Record<string, boolean>>({})
  const debtFilter = ref<Record<string, boolean>>({})

  const ASSETS_MAP = new Map()
  const allMarkets = ref<MarketFullData>({})

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

  function toggle(scope: FilterScope, type: FilterType, symbol: string) {
    const targetGroup = filters.value[scope][type]
    targetGroup[symbol] = !targetGroup[symbol]
  }

  function clearFilter(scope: FilterScope, type: FilterType) {
    const targetGroup = filters.value[scope][type]
    for (const key in targetGroup) {
      targetGroup[key] = false
    }
  }

  const isActiveCollateralFilter = (scope: FilterScope) => Object.values(filters.value[scope].collateral).some(Boolean)
  const isActiveDebtFilter = (scope: FilterScope) => Object.values(filters.value[scope].debt).some(Boolean)

  watch(() => marketsStore.state.markets, (next, prev) => {
    const prevKeys = Object.keys(prev ?? {})
    const nextKeys = Object.keys(next)
    if (prevKeys.length === nextKeys.length) {
      return
    }
    allMarkets.value = next
  }, { immediate: true })

  watch(uniqueAssets, (assets) => {
    for (const asset of assets) {
      filters.value.markets.collateral[asset.symbol] = false
      filters.value.markets.debt[asset.symbol] = false
      filters.value.multiply.collateral[asset.symbol] = false
      filters.value.multiply.debt[asset.symbol] = false
    }
  }, { immediate: true })
  return {
    filters,
    uniqueAssets,
    collateralFilter,
    debtFilter,

    isActiveCollateralFilter,
    isActiveDebtFilter,

    toggle,
    clearFilter,
  }
})
