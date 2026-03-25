import type { MarketTableItem, TableAsset } from '~/types/table'
import { bpsToNumber, calculateBorrow, calculateTotalStake } from '@alula/client-sdk/src/utils'

export function useMarketTable() {
  const marketsStore = useMarketsStore()
  const userStore = useUserStore()
  const filterStore = useMarketFilterStore()

  const route = useRoute()

  const search = computed(() => route.query?.search)

  const loading = computed(() => marketsStore.state.loading)
  const activeMarket = computed(() => marketsStore.activeMarket)

  const collateralFilter = computed(() => filterStore.collateralFilter)
  const debtFilter = computed(() => filterStore.debtFilter)

  const marketWithTableItems = computed<MarketWithTableItems[]>(() => {
    const markets = Object.entries(marketsStore.state.markets)
    const preparedTableData = []
    for (const [marketName, data] of markets) {
      const assetDecimals = data?.marketState.asset_decimals ?? 0
      const oraclePriceDecimals = data?.marketState.oracle_price_decimals ?? 0
      const poolsData = data?.marketState?.pools_data ?? []
      const assets: TableAsset['asset'][] = []
      const marketSize = {
        supplied: 0,
        borrowed: 0,
      }
      const tableItems = poolsData?.map((d) => {
        const pool = d.pool
        const obligation = userStore.state.obligations[marketName]
        const depositPosition = obligation?.deposits?.find(([addr]) => addr === pool.pool_address)?.[1]
        const borrowPosition = obligation?.borrows?.find(([addr]) => addr === pool.pool_address)?.[1]

        const suppliedFromJTokens = depositPosition?.j_tokens
          ? Number(calculateTotalStake(
              depositPosition.j_tokens,
              { total_j_tokens: pool.total_j_tokens, total_borrowed: pool.total_borrowed, total_available: d.total_available_adjusted },
              assetDecimals,
            ))
          : 0
        const suppliedCollateral = depositPosition?.collateral
          ? Number(bigintToNumber(BigInt(depositPosition.collateral), assetDecimals))
          : 0
        const userSupplied = suppliedFromJTokens + suppliedCollateral

        const userBorrowed = borrowPosition?.d_tokens
          ? calculateBorrow(
              borrowPosition.d_tokens,
              { total_d_tokens: pool.total_d_tokens, total_borrowed: pool.total_borrowed },
              assetDecimals,
            )
          : 0
        const total_supply = Number(bigintToNumber(d.total_supply, assetDecimals)) || 0
        const total_borrowed = Number(bigintToNumber(pool.total_borrowed, assetDecimals)) || 0
        const total_collateral = Number(bigintToNumber(pool.total_collateral, assetDecimals)) || 0
        const depositApy = d.apy.supply_bps / 100
        const borrowApy = d.apy.borrow_bps / 100
        const utilRate = Number(pool.total_borrowed) / Number((pool.total_available + pool.total_borrowed)) * 100
        const openLtv = Number(pool.config.health_config.open_ltv_bps) / 100
        const supply_limit = Number(bigintToNumber(pool.config.health_config.supply_limit, assetDecimals)) || 0
        const utilization_rate_limit = bpsToNumber(Number(pool.config.health_config.utilization_ratio_limit_bps))
        const price = Number(bigintToNumber(d.oracle_asset_price, oraclePriceDecimals)) || 0
        const available = Number(bigintToNumber(d.total_available_adjusted, assetDecimals))
        const asset = getFullTokenData(pool.token_symbol)
        assets.push(asset)
        marketSize.supplied += (total_supply + total_collateral) * price
        marketSize.borrowed += total_borrowed * price
        return {
          raw: d,
          asset,
          total_supply,
          total_borrowed,
          deposit_apy: `${truncatePercent(depositApy || 0, 2)}%`,
          borrow_apy: `${truncatePercent(borrowApy || 0, 2)}%`,
          utilization_rate: `${truncatePercent(utilRate || 0, 2)}%`,
          utilization_rate_percent: utilRate,
          open_ltv: `${truncatePercent(openLtv || 0, 2)}%`,
          action: 'Supply',
          price,
          supply_limit,
          utilization_rate_limit,
          available,
          pool_address: pool.pool_address,
          market: marketName,
          assetDecimals,
          position: {
            supplied: truncatePercent(userSupplied, 5),
            borrowed: truncatePercent(userBorrowed, 5),
          },
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

  const filteredMarkets = computed(() => {
    const collateral = collateralFilter.value
    const debt = debtFilter.value

    const selected = new Set<string>()

    for (const key in collateral) {
      if (collateral[key]) {
        selected.add(key)
      }
    }

    for (const key in debt) {
      if (debt[key]) {
        selected.add(key)
      }
    }

    const hasFilter = selected.size > 0

    const searchValue
      = (typeof search.value === 'string' ? search.value : '').toLowerCase()

    return sortedMarkets.value
      .map((market) => {
        const tableItems = hasFilter
          ? market.tableItems.filter(item =>
              selected.has(item.asset.symbol),
            )
          : market.tableItems

        return {
          ...market,
          tableItems,
        }
      })
      .filter((market) => {
        if (!searchValue) {
          return true
        }

        return (
          market.marketName.toLowerCase().includes(searchValue)
          || market.assets.some(asset =>
            asset.symbol?.toLowerCase().includes(searchValue),
          )
        )
      })
  })

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
    dialogRepay: toRef(marketsStore, 'dialogRepay'),
    dialogWithdraw: toRef(marketsStore, 'dialogWithdraw'),
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
  marketSize: {
    supplied: number
    borrowed: number
  }
}
