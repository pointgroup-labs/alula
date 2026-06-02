import type { RPCcluster } from '@alula/client-sdk'
import type { MultiplyPositionItem, MultiplyVaultItem } from '~/types/table'
import { AQUA_PROVIDER_ADDRESS, bpsToNumber, calculateMultiplyMaxLeverage, SOROSWAP_PROVIDER_ADDRESS } from '@alula/client-sdk'
import { calculateBorrow, calculateTotalStake } from '@alula/client-sdk/src/utils'
import { calculateCurrentMultiplier } from '~/utils'
import { calcMultiplyObligationNetApy, getApyRangeForMultiplier } from '~/utils/multiply'
import { buildMultiplyPairKey } from '~/utils/obligation'

export const useMultiplyStore = defineStore('multiply', () => {
  const route = useRoute()
  const router = useRouter()
  const marketsStore = useMarketsStore()
  const { getFullTokenData } = useTokensStore()
  const rpcStore = useRpcStore()

  const swapProviderAddress = useLocalStorage('swapProviderAddress', '', { initOnMounted: true })

  watch(() => rpcStore.network as RPCcluster, (network) => {
    if (!network) { return }
    const validAddresses = [AQUA_PROVIDER_ADDRESS[network], SOROSWAP_PROVIDER_ADDRESS[network]].filter(Boolean)
    if (!validAddresses.includes(swapProviderAddress.value)) {
      swapProviderAddress.value = AQUA_PROVIDER_ADDRESS[network] || validAddresses[0] || ''
    }
  }, { immediate: true })

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

          const supplyApy = bpsToNumber(Number(depositPoolData.apy.supply_bps || 0)) * 100
          const borrowApy = bpsToNumber(Number(borrowPoolData.apy.borrow_bps || 0)) * 100
          const { maxApy } = getApyRangeForMultiplier({
            supplyApy,
            borrowApy,
            maxMultiplier,
          })
          const apyAtMaxMultiplier = maxApy

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
    const pairKey = route.params.pair as string | undefined
    if (!marketAddress || !pairKey) {
      return
    }

    return vaults.value.find(vault => vault.marketAddress === marketAddress && vault.pairKey === pairKey)
  })

  const positions = computed<MultiplyPositionItem[]>(() => {
    const userStore = useUserStore()
    const items: MultiplyPositionItem[] = []

    for (const vault of vaults.value) {
      const obligation = userStore.state.multiplyObligations[vault.market]?.[vault.pairKey]
      if (!obligation) {
        continue
      }

      const marketState = marketsStore.state.markets[vault.market]?.marketState
      const oraclePriceDecimals = marketState?.oracle_price_decimals ?? 0
      const depositPoolData = vault.depositPoolData
      const borrowPoolData = vault.borrowPoolData
      const depositObligation = obligation.deposits?.find(([address]) => address === depositPoolData.pool.pool_address)
      const borrowObligation = obligation.borrows?.find(([address]) => address === borrowPoolData.pool.pool_address)

      if (!depositObligation || !borrowObligation) {
        continue
      }

      const [, depositData] = depositObligation
      const [, borrowData] = borrowObligation

      const depositDecimals = depositPoolData.pool.token_decimals
      // V3 positions store the deposit as raw collateral (AddCollateral); V2 stores it as j_tokens (supply shares).
      // Sum both so legacy V2 obligations and new V3 obligations both display correctly.
      const jTokenStake = +calculateTotalStake(depositData.j_tokens, {
        total_j_tokens: depositPoolData.pool.total_j_tokens,
        total_borrowed: depositPoolData.pool.total_borrowed,
        total_available: depositPoolData.total_available_adjusted,
      }, depositDecimals) || 0
      const collateralAmount = Number(bigintToNumber(BigInt(depositData.collateral || 0n), depositDecimals)) || 0
      const deposited = jTokenStake + collateralAmount

      const borrowed = +calculateBorrow(borrowData.d_tokens, {
        total_borrowed: borrowPoolData.pool.total_borrowed,
        total_d_tokens: borrowPoolData.pool.total_d_tokens,
      }, borrowPoolData.pool.token_decimals) || 0

      const depositPrice = Number(bigintToNumber(depositPoolData.oracle_asset_price, oraclePriceDecimals)) || 0
      const borrowPrice = Number(bigintToNumber(borrowPoolData.oracle_asset_price, oraclePriceDecimals)) || 0
      const depositedUsd = deposited * depositPrice
      const borrowedUsd = borrowed * borrowPrice
      const netEquityUsd = depositedUsd - borrowedUsd
      const currentMultiplier = calculateCurrentMultiplier(deposited, depositPrice, borrowed, borrowPrice) || 0
      const supplyApy = bpsToNumber(Number(depositPoolData.apy.supply_bps || 0)) * 100
      const borrowApy = bpsToNumber(Number(borrowPoolData.apy.borrow_bps || 0)) * 100
      const currentApy = calcMultiplyObligationNetApy({
        suppliedUsd: depositedUsd,
        borrowedUsd,
        supplyApy,
        borrowApy })
      const closeLtvRate = bpsToNumber(Number(depositPoolData.pool.config.health_config.close_ltv_bps || 0))
      const liabilityFactorRate = bpsToNumber(Number(borrowPoolData.pool.config.health_config.liability_factor_bps || 0))
      const healthFactor = calculatePositionHealthFactor({
        deposited,
        depositPrice,
        closeLtvBps: Number(depositPoolData.pool.config.health_config.close_ltv_bps || 0),
        borrowed,
        borrowPrice,
        liabilityFactorBps: Number(borrowPoolData.pool.config.health_config.liability_factor_bps || 0),
      })
      const positionValueUsd = depositedUsd
      const currentLtv = depositedUsd > 0 ? (borrowedUsd / depositedUsd) * 100 : 0
      const openLtv = bpsToNumber(Number(depositPoolData.pool.config.health_config.open_ltv_bps || 0)) * 100
      const closeLtv = closeLtvRate * 100
      const liabilityFactor = liabilityFactorRate * 100
      const equityUsd = Math.max(depositedUsd - borrowedUsd, 0)
      const yearlyResultUsd = equityUsd * (currentApy / 100)
      const liquidationPriceRaw = deposited > 0 && closeLtvRate > 0
        ? (borrowedUsd * liabilityFactorRate) / (deposited * closeLtvRate)
        : null
      const liquidationPrice = liquidationPriceRaw && Number.isFinite(liquidationPriceRaw) && liquidationPriceRaw > 0
        ? liquidationPriceRaw
        : null
      const distanceToLiquidationPercent
        = liquidationPrice !== null && depositPrice > 0
          ? Math.max(((depositPrice - liquidationPrice) / depositPrice) * 100, 0)
          : null
      const liquidationBufferUsd
        = liquidationPrice === null
          ? 0
          : Math.max((depositPrice - liquidationPrice) * deposited, 0)

      items.push({
        market: vault.market,
        pairKey: vault.pairKey,
        deposited,
        borrowed,
        netEquityUsd,
        depositedUsd,
        borrowedUsd,
        positionValueUsd,
        currentMultiplier,
        supplyApy,
        borrowApy,
        currentApy,
        healthFactor,
        currentLtv,
        openLtv,
        closeLtv,
        liabilityFactor,
        yearlyResultUsd,
        liquidationBufferUsd,
        liquidationPrice,
        distanceToLiquidationPercent,
      })
    }

    return items
  })

  function getVaultRoute(vault: MultiplyVaultItem) {
    return `/multiply/${vault.marketAddress}/${vault.pairKey}/pool`
  }

  function openVault(vault: MultiplyVaultItem) {
    return router.push(getVaultRoute(vault))
  }

  return {
    vaults,
    positions,
    selectedVault,
    swapProviderAddress,
    getVaultRoute,
    openVault,
  }
})

function calculatePositionHealthFactor(params: {
  deposited: number
  depositPrice: number
  closeLtvBps: number
  borrowed: number
  borrowPrice: number
  liabilityFactorBps: number
}) {
  const weightedDepositUsd = params.deposited * params.depositPrice * bpsToNumber(params.closeLtvBps)
  const weightedBorrowUsd = params.borrowed * params.borrowPrice * bpsToNumber(params.liabilityFactorBps)

  if (weightedBorrowUsd <= 0) {
    return 10
  }

  return Math.min(weightedDepositUsd / weightedBorrowUsd, 10)
}
