import type { MultiplyTableItem } from '~/types/table'

export function useMultiplyTable() {
  const marketsStore = useMarketsStore()
  const userStore = useUserStore()

  const selectedPoolAddress = toRef(marketsStore, 'selectedPoolAddress')
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
      for (const { borrow_pool, deposit_pool, max_leverage_multiplier } of leveragePools) {
        const depositPoolData = poolsData.find(p => p.pool.pool_address === deposit_pool)!
        const borrowPoolData = poolsData.find(p => p.pool.pool_address === borrow_pool)!
        const multiplier = max_leverage_multiplier / 100
        const supplyBPS = Number(depositPoolData?.apy.supply_bps || 0) / 10_000
        const borrowBPS = Number(borrowPoolData?.apy.borrow_bps || 0) / 10_000
        const maxAPY = (supplyBPS * multiplier - borrowBPS * (multiplier - 1)) * 100
        const supplied = depositPoolData && depositPoolData.pool.total_available ? Number(bigintToNumber(depositPoolData.pool.total_available, assetDecimals)) : 0
        const liquidity
          = borrowPoolData && borrowPoolData.total_available_adjusted
            ? Number(bigintToNumber(borrowPoolData.total_available_adjusted/*  + borrowPoolData.total_borrowed + borrowPoolData.total_collateral */, assetDecimals))
            : 0
        const depositPoolPrice = Number(bigintToNumber(depositPoolData.oracle_asset_price, oraclePriceDecimals)) || 0
        const borrowPoolPrice = Number(bigintToNumber(borrowPoolData.oracle_asset_price, oraclePriceDecimals)) || 0

        const data = {
          market,
          depositPoolData,
          borrowPoolData,
          asset: getFullTokenData(depositPoolData?.pool.token_symbol),
          borrowAsset: getFullTokenData(borrowPoolData?.pool.token_symbol),
          liquidity,
          multiplier,
          maxAPY,
          price: depositPoolPrice,
          borrowPoolPrice,
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
    tableItems.value.find(item => item.pool_address === selectedPoolAddress.value
      && activeLeverageMarket.value === item.market))

  return {
    tableItems,
    selectedPoolAddress,
    dialogLeverage,
    dialogLeverageWithdraw,
    markets,
    isLoading,
    selectedPool,
    activeLeverageMarket,
  }
}
