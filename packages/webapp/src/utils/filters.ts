import type { MarketTableItem, MultiplyVaultItem } from '~/types/table'

export function isValidItem(item: MultiplyVaultItem) {
  return isValidPair(item.asset.symbol, item.borrowAsset.symbol)
}

export function matchesSearch(item: MultiplyVaultItem | MarketTableItem, search: string) {
  if (!search) { return true }

  const s = search.toLowerCase()

  return (
    item.asset.symbol.toLowerCase().includes(s)
    || item.asset.name.toLowerCase().includes(s)
    || item.market?.toLowerCase().includes(s)
    || ('borrowAsset' in item && item.borrowAsset.name.toLowerCase().includes(s))
    || ('borrowAsset' in item && item.borrowAsset.symbol.toLowerCase().includes(s))
  )
}
