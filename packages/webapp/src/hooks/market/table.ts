import type { MarketTableItem } from '~/types/table'

export function useMarketTable() {
  const marketsStore = useMarketsStore()

  const loading = computed(() => marketsStore.state.loading)
  const activeMarket = computed(() => marketsStore.activeMarket)

  const tableItems = computed<MarketTableItem[]>(() => {
    const assetDecimals = activeMarket.value?.marketState.asset_decimals ?? 0
    const oraclePriceDecimals = activeMarket.value?.marketState.oracle_price_decimals ?? 0
    const poolsData = activeMarket.value?.marketState?.pools_data ?? []
    return poolsData?.map((data) => {
      const pool = data.pool
      const tokenSymbol = getTokenSymbol(pool.token_symbol)
      const tokenName = getTokenName(pool.token_symbol)
      const icon = getTokenIcon(pool.token_symbol) || ''
      const total_supply = Number(bigintToNumber(data.total_supply, assetDecimals)) || 0
      const total_borrowed = Number(bigintToNumber(pool.total_borrowed, assetDecimals)) || 0
      const depositApy = data.apy.supply_bps / 100
      const borrowApy = data.apy.borrow_bps / 100
      const utilRate = Number(pool.total_borrowed) / Number((pool.total_available + pool.total_borrowed)) * 100
      const maxLTV = Number(pool.config.health_config.open_ltv_bps) / 100
      const supply_limit = Number(bigintToNumber(pool.config.health_config.supply_limit, assetDecimals)) || 0
      const price = Number(bigintToNumber(data.oracle_asset_price, oraclePriceDecimals)) || 0
      const available = Number(bigintToNumber(data.total_available_adjusted, assetDecimals))
      return {
        raw: data,
        asset: { name: tokenName, symbol: tokenSymbol, icon },
        total_supply,
        total_borrowed,
        deposit_apy: `${truncatePercent(depositApy || 0, 2)}%`,
        borrow_apy: `${truncatePercent(borrowApy || 0, 2)}%`,
        utilization_rate: `${truncatePercent(utilRate || 0, 2)}%`,
        max_ltv: `${truncatePercent(maxLTV || 0, 2)}%`,
        action: 'Supply',
        price,
        supply_limit,
        available,
        pool_address: pool.pool_address,
        market: marketsStore.activeMarketFilter,
        assetDecimals,
      }
    })
  })

  const selectedMarketAddress = toRef(marketsStore, 'selectedMarketAddress')
  const selectedPool = computed(() => tableItems.value.find(item => item.pool_address === selectedMarketAddress.value))
  const selectedMarketDetails = computed(() => tableItems.value.find(item => item.pool_address === selectedMarketAddress.value))

  return {
    tableItems,
    loading,
    activeMarket,
    dialogSupply: toRef(marketsStore, 'dialogSupply'),
    dialogBorrow: toRef(marketsStore, 'dialogBorrow'),
    infoDialog: toRef(marketsStore, 'marketInfoDialog'),
    selectedMarketAddress,
    selectedPool,
    selectedMarketDetails,
  }
}
