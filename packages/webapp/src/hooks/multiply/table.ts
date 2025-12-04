import type { MultiplyTableItem } from '~/types/table'

export function useMultiplyTable() {
  const marketsStore = useMarketsStore()
  const userStore = useUserStore()

  const selectedMarketAddress = toRef(marketsStore, 'selectedMarketAddress')
  const dialogLeverage = toRef(marketsStore, 'dialogLeverage')
  const dialogLeverageWithdraw = toRef(marketsStore, 'dialogLeverageWithdraw')

  const markets = computed(() => Object.keys(marketsStore.state.markets) ?? [])
  const isLoading = computed(() => (marketsStore.state.loadingLeveragePools || marketsStore.state.loading) || userStore.loading)

  const tableItems = computed<MultiplyTableItem[]>(() => {
    const res = []
    for (const market in marketsStore.state.markets) {
      const state = marketsStore.state.markets[market]?.marketState
      const poolsData = state?.pools_data ?? []
      const leveragePools = state?.multiply_pairs ?? []
      const oraclePriceDecimals = state?.oracle_price_decimals ?? 0
      const assetDecimals = state?.asset_decimals ?? 0
      for (const { borrow_pool, deposit_pool } of leveragePools) {
        const depositPoolData = poolsData.find(p => p.pool.pool_address === deposit_pool)!
        const borrowPoolData = poolsData.find(p => p.pool.pool_address === borrow_pool)!
        const depositTokenSymbol = getTokenSymbol(depositPoolData?.pool.token_symbol)
        const borrowTokenSymbol = getTokenSymbol(borrowPoolData?.pool.token_symbol)
        const depositTokenName = getTokenName(String(depositTokenSymbol))
        const depositTokenIcon = getTokenIcon(String(depositTokenSymbol)) || ''
        const borrowTokenName = getTokenName(String(borrowTokenSymbol))
        const borrowTokenIcon = getTokenIcon(String(borrowTokenSymbol)) || ''
        const ltv = Number(depositPoolData?.pool.config.health_config.open_ltv_bps) || 0
        const multiplier = calculateMaxMultiplierFromBps(ltv)
        const supplyBPS = Number(depositPoolData?.apy.supply_bps || 0) / 10_000
        const borrowBPS = Number(borrowPoolData?.apy.borrow_bps || 0) / 10_000
        const maxAPY = (supplyBPS * multiplier - borrowBPS * (multiplier - 1)) * 100
        const supplied = depositPoolData && depositPoolData.pool.total_available ? Number(bigintToNumber(depositPoolData.pool.total_available, assetDecimals)) : 0
        const liquidity
          = borrowPoolData && borrowPoolData.pool.total_available
            ? Number(bigintToNumber(borrowPoolData.pool.total_available/*  + borrowPoolData.total_borrowed + borrowPoolData.total_collateral */, assetDecimals))
            : 0
        const depositPoolPrice = Number(bigintToNumber(depositPoolData.oracle_asset_price, oraclePriceDecimals)) || 0
        const borrowPPoolPrice = Number(bigintToNumber(borrowPoolData.oracle_asset_price, oraclePriceDecimals)) || 0

        const data = {
          market,
          depositPoolData,
          borrowPoolData,
          asset: { name: depositTokenName, symbol: depositTokenSymbol, icon: depositTokenIcon },
          borrowAsset: { name: borrowTokenName, symbol: borrowTokenSymbol, icon: borrowTokenIcon },
          liquidity,
          multiplier,
          maxAPY,
          price: depositPoolPrice,
          borrowPoolPrice: borrowPPoolPrice,
          pool_address: depositPoolData?.pool.pool_address || '',
          supplied,
          assetDecimals,
        }

        res.push(data)
      }
    }

    return res
  })

  const activeLeverageMarket = toRef(marketsStore, 'activeLeverageMarket')
  const selectedPool = computed(() =>
    tableItems.value.find(item => item.pool_address === selectedMarketAddress.value
      && activeLeverageMarket.value === item.market))

  return {
    tableItems,
    selectedMarketAddress,
    dialogLeverage,
    dialogLeverageWithdraw,
    markets,
    isLoading,
    selectedPool,
    activeLeverageMarket,
  }
}
