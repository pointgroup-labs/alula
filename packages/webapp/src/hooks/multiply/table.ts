import type { MultiplyTableItem } from '~/types/table'

export function useMultiplyTable() {
  const marketsStore = useMarketsStore()
  const userStore = useUserStore()
  const { vaults } = useMultiplyCatalog()

  const activeLeverageMarket = toRef(marketsStore, 'activeLeverageMarket')
  const selectedPoolAddress = toRef(marketsStore, 'selectedPoolAddress')
  const dialogLeverage = toRef(marketsStore, 'dialogLeverage')
  const dialogLeverageWithdraw = toRef(marketsStore, 'dialogLeverageWithdraw')

  const markets = computed(() => Object.keys(marketsStore.state.markets) ?? [])
  const isLoading = computed(() => (marketsStore.state.loadingLeveragePools || marketsStore.state.loading) || userStore.loading)

  const tableItems = computed<MultiplyTableItem[]>(() =>
    vaults.value.map(vault => ({
      pairKey: vault.pairKey,
      market: vault.market,
      depositPoolData: vault.depositPoolData,
      borrowPoolData: vault.borrowPoolData,
      asset: vault.asset,
      borrowAsset: vault.borrowAsset,
      liquidity: vault.liquidity,
      multiplier: vault.maxMultiplier,
      apyAtMaxMultiplier: vault.apyAtMaxMultiplier,
      price: vault.price,
      borrowPoolPrice: vault.borrowPoolPrice,
      pool_address: vault.pool_address,
      supplied: vault.supplied,
      assetDecimals: vault.depositPoolData.pool.token_decimals,
    })),
  )

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
