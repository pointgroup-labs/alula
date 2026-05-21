export const usePriceStore = defineStore('price', () => {
  const marketsStore = useMarketsStore()
  const assetsPrices = ref<Record<string, number>>({})

  watch(() => marketsStore.state.markets, (markets) => {
    if (!markets || !import.meta.client) {
      return
    }
    for (const market of Object.values(markets)) {
      const oracle_price_decimals = market.marketState?.oracle_price_decimals ?? 14
      const pools_data = market.marketState?.pools_data ?? []
      for (const data of pools_data) {
        const priceUsd = Number(bigintToNumber(data.oracle_asset_price, oracle_price_decimals)) || 0
        const symbol = data.pool.token_symbol
        assetsPrices.value[symbol] = priceUsd
      }
    }
  }, { immediate: true })
  return {
    assetsPrices,
  }
})
