import type { MultiplyVaultItem } from '~/types/table'
import { bpsToNumber, calculateMultiplyMaxLeverage } from '@alula/client-sdk'
import { buildMultiplyPairKey } from '~/utils/obligation'

export function useMultiplyCatalog() {
  const route = useRoute()
  const router = useRouter()
  const marketsStore = useMarketsStore()
  const { getFullTokenData } = useTokensStore()

  const vaults = computed<MultiplyVaultItem[]>(() => {
    const items: MultiplyVaultItem[] = []

    for (const [marketName, marketEntry] of Object.entries(marketsStore.state.markets)) {
      const marketState = marketEntry.marketState as any
      const poolsData = marketState?.pools_data ?? []
      const oraclePriceDecimals = marketState?.oracle_price_decimals ?? 0

      for (const depositPoolData of poolsData) {
        for (const borrowPoolData of poolsData) {
          if (!depositPoolData || !borrowPoolData) {
            continue
          }
          if (depositPoolData.pool.pool_address === borrowPoolData.pool.pool_address) {
            continue
          }
          if (depositPoolData.pool.token_address === borrowPoolData.pool.token_address) {
            continue
          }

          const openLtvBps = Number(depositPoolData.pool.config.health_config.open_ltv_bps)
          if (openLtvBps <= 0) {
            continue
          }

          const maxMultiplier = calculateMultiplyMaxLeverage(openLtvBps)
          if (!Number.isFinite(maxMultiplier) || maxMultiplier <= 1) {
            continue
          }

          const supplyBps = bpsToNumber(Number(depositPoolData.apy.supply_bps || 0))
          const borrowBps = bpsToNumber(Number(borrowPoolData.apy.borrow_bps || 0))
          const apyAtMaxMultiplier = (supplyBps * maxMultiplier - borrowBps * Math.max(maxMultiplier - 1, 0)) * 100

          items.push({
            pairKey: buildMultiplyPairKey(depositPoolData.pool.pool_address, borrowPoolData.pool.pool_address),
            market: marketName,
            marketAddress: marketEntry.address,
            depositPoolData,
            borrowPoolData,
            asset: getFullTokenData(depositPoolData.pool.token_symbol),
            borrowAsset: getFullTokenData(borrowPoolData.pool.token_symbol),
            maxMultiplier,
            apyAtMaxMultiplier,
            price: Number(bigintToNumber(depositPoolData.oracle_asset_price, oraclePriceDecimals)) || 0,
            borrowPoolPrice: Number(bigintToNumber(borrowPoolData.oracle_asset_price, oraclePriceDecimals)) || 0,
            supplied: Number(bigintToNumber(depositPoolData.pool.total_available, depositPoolData.pool.token_decimals)) || 0,
            liquidity: Number(bigintToNumber(borrowPoolData.total_available_adjusted, borrowPoolData.pool.token_decimals)) || 0,
            pool_address: depositPoolData.pool.pool_address,
          })
        }
      }
    }

    return items.toSorted((left, right) => right.apyAtMaxMultiplier - left.apyAtMaxMultiplier)
  })

  const selectedVault = computed(() => {
    const marketAddress = route.params.market as string | undefined
    const pairKey = route.params.pool as string | undefined
    if (!marketAddress || !pairKey) {
      return
    }

    return vaults.value.find(vault => vault.marketAddress === marketAddress && vault.pairKey === pairKey)
  })

  function getVaultRoute(vault: MultiplyVaultItem) {
    return `/multiply/${vault.marketAddress}/${vault.pairKey}`
  }

  function openVault(vault: MultiplyVaultItem) {
    return router.push(getVaultRoute(vault))
  }

  return {
    vaults,
    selectedVault,
    getVaultRoute,
    openVault,
  }
}
