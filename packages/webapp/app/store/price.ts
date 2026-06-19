// TODO: Replace with real data when pool data is available
const TEST_AQUA_PRICE = 0.000_377_8

export const usePriceStore = defineStore('price', () => {
  const marketsStore = useMarketsStore()
  const assetsPrices = ref<Record<string, number>>({})

  function getAssetPrice(symbol?: string): number {
    if (!symbol) {
      return 0
    }
    return assetsPrices.value[symbol === 'XLM' ? 'native' : symbol] ?? 0
  }

  watch(() => marketsStore.state.markets, (markets) => {
    if (!markets || !import.meta.client) {
      return
    }
    // TODO: Remove mock data price for aqua
    const prices: Record<string, number> = {
      AQUA: TEST_AQUA_PRICE,
    }
    for (const market of Object.values(markets)) {
      const oracle_price_decimals = market.marketState?.oracle_price_decimals ?? 14
      const pools_data = market.marketState?.pools_data ?? []
      for (const data of pools_data) {
        const priceUsd = Number(bigintToNumber(data.oracle_asset_price, oracle_price_decimals)) || 0
        const symbol = data.pool.token_symbol
        prices[symbol] = priceUsd
      }
    }
    assetsPrices.value = prices
  }, { immediate: true })
  return {
    assetsPrices,
    getAssetPrice,
  }
})
