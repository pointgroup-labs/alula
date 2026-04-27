import type { MultiplyVaultItem } from '~/types/table'

export function useMultiplyTable() {
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

    const selectedCollateral = new Set<string>()
    const selectedDebt = new Set<string>()

    for (const key in collateral) {
      if (collateral[key]) {
        selectedCollateral.add(key)
      }
    }

    for (const key in debt) {
      if (debt[key]) {
        selectedDebt.add(key)
      }
    }

    const hasFilter = selectedCollateral.size > 0 || selectedDebt.size > 0

    const searchValue
      = (typeof search.value === 'string' ? search.value : '').toLowerCase()

    return vaultsByMarket.value.map((vault) => {
      const items = hasFilter || searchValue
        ? vault.items.filter((item) => {
            if (searchValue) {
              return item.asset.symbol.toLowerCase().includes(searchValue)
                || item.asset.name.toLowerCase().includes(searchValue)
                || item.market?.toLowerCase().includes(searchValue)
                || item.borrowAsset.name.toLowerCase().includes(searchValue)
                || item.borrowAsset.symbol.toLowerCase().includes(searchValue)
            }
            return selectedCollateral.has(item.asset.symbol) || selectedDebt.has(item.borrowAsset.symbol)
          })
        : vault.items
      return {
        ...vault,
        items,
      }
    }).filter((vault) => {
      return vault.market.toLowerCase().includes(searchValue)
        || vault.items.some((item) => {
          return item.asset.symbol.toLowerCase().includes(searchValue)
            || item.asset.name.toLowerCase().includes(searchValue)
            || item.market?.toLowerCase().includes(searchValue)
            || item.borrowAsset.name.toLowerCase().includes(searchValue)
            || item.borrowAsset.symbol.toLowerCase().includes(searchValue)
        })
    }).toSorted((a, _b) => {
      if (a.market === MAIN_MARKET_NAME) {
        return -1
      }
      return 1
    })
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
}
