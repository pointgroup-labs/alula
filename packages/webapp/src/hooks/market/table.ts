import type { MarketTableItem, TableAsset } from '~/types/table'

export function useMarketTable() {
  const marketsStore = useMarketsStore()

  const search = ref()

  const loading = computed(() => marketsStore.state.loading)
  const activeMarket = computed(() => marketsStore.activeMarket)

  const marketWithTableItems = computed<MarketWithTableItems[]>(() => {
    const markets = Object.entries(marketsStore.state.markets)
    const preparedTableData = []
    for (const [marketName, data] of markets) {
      const assetDecimals = data?.marketState.asset_decimals ?? 0
      const oraclePriceDecimals = data?.marketState.oracle_price_decimals ?? 0
      const poolsData = data?.marketState?.pools_data ?? []
      const assets: TableAsset['asset'][] = []
      let marketSize = 0
      const tableItems = poolsData?.map((d) => {
        const pool = d.pool
        const total_supply = Number(bigintToNumber(d.total_supply, assetDecimals)) || 0
        const total_borrowed = Number(bigintToNumber(pool.total_borrowed, assetDecimals)) || 0
        const total_collateral = Number(bigintToNumber(pool.total_collateral, assetDecimals)) || 0
        const depositApy = d.apy.supply_bps / 100
        const borrowApy = d.apy.borrow_bps / 100
        const utilRate = Number(pool.total_borrowed) / Number((pool.total_available + pool.total_borrowed)) * 100
        const openLtv = Number(pool.config.health_config.open_ltv_bps) / 100
        const supply_limit = Number(bigintToNumber(pool.config.health_config.supply_limit, assetDecimals)) || 0
        const price = Number(bigintToNumber(d.oracle_asset_price, oraclePriceDecimals)) || 0
        const available = Number(bigintToNumber(d.total_available_adjusted, assetDecimals))
        const asset = getFullTokenData(pool.token_symbol)
        assets.push(asset)
        marketSize += (total_supply + total_collateral) * price
        return {
          raw: d,
          asset,
          total_supply,
          total_borrowed,
          deposit_apy: `${truncatePercent(depositApy || 0, 2)}%`,
          borrow_apy: `${truncatePercent(borrowApy || 0, 2)}%`,
          utilization_rate: `${truncatePercent(utilRate || 0, 2)}%`,
          open_ltv: `${truncatePercent(openLtv || 0, 2)}%`,
          action: 'Supply',
          price,
          supply_limit,
          available,
          pool_address: pool.pool_address,
          market: marketName,
          assetDecimals,
        }
      })

      preparedTableData.push({
        marketName,
        assets,
        marketSize,
        tableItems,
      })
    }
    return preparedTableData
  })

  const sortedMarkets = computed(() => marketWithTableItems.value.toSorted((a, _b) => {
    if (a.marketName === 'main') {
      return -1
    }
    return 1
  }))

  const filteredMarkets = computed(() =>
    search.value
      ? sortedMarkets.value.filter(market => market.assets.some(asset => asset.symbol?.toLowerCase().includes(search.value?.toLowerCase())))
      : sortedMarkets.value)

  const selectedMarketName = toRef(marketsStore, 'selectedMarketName')
  const selectedPoolAddress = toRef(marketsStore, 'selectedPoolAddress')
  const selectedMarket = computed(() => marketWithTableItems.value.find(m => m.marketName === selectedMarketName.value))
  const selectedPool = computed(() => selectedMarket.value?.tableItems.find(p => p.pool_address === selectedPoolAddress.value))

  return {
    search,
    marketWithTableItems,
    filteredMarkets,
    loading,
    activeMarket,
    dialogSupply: toRef(marketsStore, 'dialogSupply'),
    dialogBorrow: toRef(marketsStore, 'dialogBorrow'),
    infoDialog: toRef(marketsStore, 'marketInfoDialog'),
    selectedMarketName,
    selectedPool,
    selectedPoolAddress,
  }
}

export type MarketWithTableItems = {
  marketName: string
  assets: TableAsset['asset'][]
  tableItems: MarketTableItem[]
  marketSize: number
}
