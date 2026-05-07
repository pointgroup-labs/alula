import type { MultiplyVaultItem } from '~/types/table'

export const useMultiplyTableStore = defineStore('multiply-table', () => {
  const marketsStore = useMarketsStore()
  const userStore = useUserStore()
  const filtersStore = useMarketFilterStore()

  const multiplyStore = useMultiplyStore()

  const route = useRoute()

  const search = computed(() => route.query?.searchMultiply)

  const vaults = computed(() => multiplyStore.vaults)

  const isLoading = computed(() => (marketsStore.state.loading || userStore.loading) && vaults.value.length === 0)

  const positions = computed(() => multiplyStore.positions)

  const dialogLeverage = toRef(marketsStore, 'dialogLeverage')
  const selectedVault = ref<MultiplyVaultItem>()

  const vaultsByMarket = computed(() => {
    const grouped = Object.values(
      vaults.value.reduce((acc, item) => {
        const key = item.market

        if (!acc[key]) {
          acc[key] = {
            market: key,
            items: [],
          }
        }

        const netEquityUsd = getNetEquity(item)
        item.netEquityUsd = netEquityUsd

        acc[key].items.push(item)

        return acc
      }, {} as Record<string, { market: string, items: MultiplyVaultItem[] }>),
    )
    return grouped
  })

  const filteredVaults = computed(() => {
    const collateral = filtersStore.filters.multiply.collateral
    const debt = filtersStore.filters.multiply.debt

    const selectedCollateral = new Set(
      Object.keys(collateral).filter(key => collateral[key]),
    )

    const selectedDebt = new Set(
      Object.keys(debt).filter(key => debt[key]),
    )

    const hasFilter = selectedCollateral.size > 0 || selectedDebt.size > 0

    const searchValue = (typeof search.value === 'string'
      ? search.value
      : ''
    ).toLowerCase()

    const matchesFilters = (item: MultiplyVaultItem) => {
      if (!hasFilter) { return true }
      return (
        selectedCollateral.has(item.asset.symbol)
        || selectedDebt.has(item.borrowAsset.symbol)
      )
    }

    return vaultsByMarket.value
      .map((vault) => {
        const items = vault.items
          .filter(item => isValidPairItem(item))
          .filter(item => matchesFilters(item))
          .filter(item => matchesSearch(item, searchValue))

        return {
          ...vault,
          items,
        }
      })
      .filter((vault) => {
        if (!searchValue) { return vault.items.length > 0 }

        return (
          vault.market.toLowerCase().includes(searchValue)
          || vault.items.some(item => matchesSearch(item, searchValue))
        )
      })
      .toSorted(a => (a.market === MAIN_MARKET_NAME ? -1 : 1))
  })

  function openDialog(vault: MultiplyVaultItem) {
    selectedVault.value = vault
    dialogLeverage.value = true
  }

  function onRowClicked(vault: MultiplyVaultItem) {
    multiplyStore.openVault(vault)
  }

  function getNetEquity(vault: MultiplyVaultItem): number {
    const position = positions.value.find(position => position.market === vault.market && position.pairKey === vault.pairKey)
    return position?.netEquityUsd ?? 0
  }

  function isUserHaveMultiply(vault: MultiplyVaultItem) {
    return checkIsHaveMultiply(userStore.state.multiplyObligations, [vault] as any, vault.depositPoolData.pool.pool_address, vault.market)
  }
  return {
    search,
    isLoading,
    vaultsByMarket,
    filteredVaults,
    selectedVault,
    dialogLeverage,
    openDialog,
    onRowClicked,
    getNetEquity,
    isUserHaveMultiply,
  }
})
