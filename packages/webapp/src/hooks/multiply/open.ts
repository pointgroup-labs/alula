import type { MultiplyMarginAsset, MultiplyPreview } from '@alula/client-sdk'
import type { MultiplyVaultItem } from '~/types/table'
import { bpsToNumber, SOROSWAP_PROVIDER_ADDRESS } from '@alula/client-sdk'
import Decimal from 'decimal.js'
import { destructurePoolAsset } from '~/utils'
import { buildMultiplyObligationKey } from '~/utils/obligation'

export function useMultiplyOpen(vaultRef: MaybeRef<MultiplyVaultItem | undefined>) {
  const market = useMarketActions()
  const marketsStore = useMarketsStore()
  const userStore = useUserStore()

  const {
    publicKey,
    nativeBalance,
    getAssetBalance,
  } = useWalletComposable()

  const vault = computed(() => unref(vaultRef))

  const amount = ref<number | undefined>()
  const slippage = ref(0.5)
  const percentFromMax = ref(85)
  const preview = ref<MultiplyPreview>()
  const loadingPreview = ref(false)
  const previewError = ref<string>()
  let interval: ReturnType<typeof setInterval> | undefined

  const activeClient = computed(() => vault.value ? marketsStore.state.markets[vault.value.market]?.client : undefined)
  const marketState = computed(() => vault.value ? marketsStore.state.markets[vault.value.market]?.marketState : undefined)

  const isMarginBorrow = ref(true)
  const marginAssetType = computed<MultiplyMarginAsset>(() => isMarginBorrow.value ? 'borrow' : 'deposit')
  const marginAsset = computed(() => isMarginBorrow.value ? vault.value?.borrowAsset : vault.value?.asset)
  const notMarginAsset = computed(() => isMarginBorrow.value ? vault.value?.asset : vault.value?.borrowAsset)

  const minPercent = computed(() => {
    const max = Number(vault.value?.maxMultiplier || 0)
    if (!max) {
      return 0
    }
    return Math.min(100, Math.ceil((1.1 / max) * 100))
  })

  const selectedMultiplier = computed(() => {
    const max = Number(vault.value?.maxMultiplier || 0)
    if (!max) {
      return 0
    }

    return Number(
      new Decimal(max)
        .mul(new Decimal(percentFromMax.value).div(100))
        .toDecimalPlaces(2, Decimal.ROUND_DOWN),
    )
  })

  const currentApy = computed(() => {
    if (!vault.value || selectedMultiplier.value <= 0) {
      return 0
    }

    const supplyApy = bpsToNumber(Number(vault.value.depositPoolData.apy.supply_bps || 0))
    const borrowApy = bpsToNumber(Number(vault.value.borrowPoolData.apy.borrow_bps || 0))

    return (supplyApy * selectedMultiplier.value - borrowApy * Math.max(selectedMultiplier.value - 1, 0)) * 100
  })

  const borrowPool = computed(() => vault.value?.borrowPoolData)
  const marginPool = computed(() => isMarginBorrow.value ? vault.value?.borrowPoolData : vault.value?.depositPoolData)
  const marginPrice = computed(() => isMarginBorrow.value ? vault.value?.borrowPoolPrice : vault.value?.price)

  const balance = computed(() => {
    const pool = marginPool.value?.pool
    if (!pool) {
      return 0
    }
    if (pool.token_symbol === 'native') {
      return nativeBalance.value
    }
    const [, issuer] = destructurePoolAsset(pool.name)
    return getAssetBalance(String(issuer))
  })

  const flashLoanFeeBps = computed(() => Number(borrowPool.value?.pool.config.fee_config.flash_loan_fee_bps || 0))

  const availableBorrowLiquidity = computed(() => {
    if (!borrowPool.value) {
      return 0
    }

    const pool = borrowPool.value
    const decimals = pool.pool.token_decimals
    const totalSupply = Number(bigintToNumber(pool.total_supply, decimals))
    const totalBorrowed = Number(bigintToNumber(pool.pool.total_borrowed, decimals))
    const availableAdjusted = Number(bigintToNumber(pool.total_available_adjusted, decimals))
    const utilizationLimit = bpsToNumber(Number(pool.pool.config.health_config.utilization_ratio_limit_bps || 0))
    const borrowCapByUtilization = totalSupply * utilizationLimit - totalBorrowed

    return Math.max(0, Math.min(availableAdjusted, borrowCapByUtilization))
  })

  const maxInputAmount = computed(() => {
    if (selectedMultiplier.value <= 1) {
      return 0
    }

    const maxFlashBorrow = availableBorrowLiquidity.value / (1 + bpsToNumber(flashLoanFeeBps.value))

    if (isMarginBorrow.value) {
      return maxFlashBorrow / (selectedMultiplier.value - 1)
    }

    if (!vault.value || !marketState.value) {
      return 0
    }

    const oracleDecimals = Number(marketState.value.oracle_price_decimals || 0)
    const depositPrice = Number(bigintToNumber(vault.value.depositPoolData.oracle_asset_price, oracleDecimals))
    const borrowPrice = Number(bigintToNumber(vault.value.borrowPoolData.oracle_asset_price, oracleDecimals))

    if (depositPrice <= 0 || borrowPrice <= 0) {
      return 0
    }

    return (maxFlashBorrow * borrowPrice)
      / (depositPrice * Math.max(selectedMultiplier.value - 1, 0))
  })

  const swapPath = computed(() => {
    if (!vault.value) {
      return []
    }
    return [
      vault.value.borrowPoolData.pool.token_address,
      vault.value.depositPoolData.pool.token_address,
    ]
  })

  const existingBorrowInPool = computed(() => {
    if (!vault.value) {
      return false
    }

    const borrows = userStore.state.obligations[String(vault.value.market)]?.borrows ?? []
    return borrows.some(([poolAddress, borrowPosition]) => poolAddress === vault.value?.borrowPoolData.pool.pool_address && Boolean(borrowPosition))
  })

  const blockedReason = computed(() => {
    if (!vault.value || !existingBorrowInPool.value) {
      return ''
    }

    return `Multiply is unavailable because you already have an active borrow position in ${vault.value.borrowAsset.symbol}. Repay that borrow first, then open the multiply position.`
  })

  const summary = computed(() => {
    if (!preview.value || !vault.value) {
      return
    }

    const borrowDecimals = vault.value.borrowPoolData.pool.token_decimals
    const depositDecimals = vault.value.depositPoolData.pool.token_decimals

    return {
      flashBorrowAmount: Number(bigintToNumber(preview.value.flashBorrowAmount, borrowDecimals)),
      swapAmountIn: Number(bigintToNumber(preview.value.swapAmountIn, borrowDecimals)),
      expectedAmountOut: Number(bigintToNumber(preview.value.expectedAmountOut, depositDecimals)),
      minAmountOut: Number(bigintToNumber(preview.value.minAmountOut, depositDecimals)),
      depositAmount: Number(bigintToNumber(preview.value.depositAmount, depositDecimals)),
      finalBorrowAmount: Number(bigintToNumber(preview.value.finalBorrowAmount, borrowDecimals)),
      routerAddress: preview.value.routerAddress,
    }
  })

  const unhealthyReason = computed(() => {
    if (!vault.value || !preview.value || !marketState.value) {
      return ''
    }

    const oracleDecimals = Number(marketState.value.oracle_price_decimals || 0)
    const depositDecimals = vault.value.depositPoolData.pool.token_decimals
    const borrowDecimals = vault.value.borrowPoolData.pool.token_decimals

    const depositAmount = new Decimal(bigintToNumber(preview.value.depositAmount, depositDecimals) || 0)
    const borrowAmount = new Decimal(bigintToNumber(preview.value.finalBorrowAmount, borrowDecimals) || 0)
    const depositPrice = new Decimal(bigintToNumber(vault.value.depositPoolData.oracle_asset_price, oracleDecimals) || 0)
    const borrowPrice = new Decimal(bigintToNumber(vault.value.borrowPoolData.oracle_asset_price, oracleDecimals) || 0)

    const openLtv = bpsToNumber(Number(vault.value.depositPoolData.pool.config.health_config.open_ltv_bps || 0))
    const liabilityFactor = bpsToNumber(Number(vault.value.borrowPoolData.pool.config.health_config.liability_factor_bps || 0))
    const hasBorrowBackingCollateral = Number(vault.value.depositPoolData.pool.config.health_config.close_ltv_bps || 0) > 0
    const minCollateralRequirementUsd = hasBorrowBackingCollateral
      ? Number(marketState.value.global_state.min_collateral_value_cents || 0) / 100
      : 0

    const collateralValueScaled = depositAmount.mul(depositPrice).mul(openLtv)
    const debtValueScaled = borrowAmount.mul(borrowPrice).mul(liabilityFactor)
    const healthBufferUsd = collateralValueScaled.minus(debtValueScaled).minus(minCollateralRequirementUsd)

    if (!healthBufferUsd.isFinite()) {
      return 'Multiply preview is invalid. Try a smaller amount or lower multiplier.'
    }

    if (healthBufferUsd.lte(0)) {
      return 'This multiply setup would be unhealthy after swap slippage, flash-loan fee, and liability-factor checks. Lower the multiplier or amount.'
    }

    return ''
  })

  async function refreshPreview() {
    if (!vault.value || !activeClient.value || !amount.value || amount.value <= 0 || selectedMultiplier.value <= 1 || blockedReason.value) {
      preview.value = undefined
      previewError.value = undefined
      return
    }

    loadingPreview.value = true
    previewError.value = undefined

    try {
      preview.value = await activeClient.value.multiply.getOpenPositionPreview({
        depositPoolAddress: vault.value.depositPoolData.pool.pool_address,
        borrowPoolAddress: vault.value.borrowPoolData.pool.pool_address,
        initialAmount: amount.value,
        leverageMultiplier: selectedMultiplier.value,
        marginAsset: marginAssetType.value,
        slippagePercent: slippage.value,
        swapProviderAddress: SOROSWAP_PROVIDER_ADDRESS,
        path: swapPath.value,
      })
    } catch (error: any) {
      preview.value = undefined
      previewError.value = String(error?.message || error)
    } finally {
      loadingPreview.value = false
    }
  }

  async function openMultiply() {
    if (!vault.value || !activeClient.value || !amount.value || !publicKey.value || blockedReason.value || unhealthyReason.value) {
      return false
    }

    const obligationKey = await buildMultiplyObligationKey({
      publicKey: publicKey.value,
      borrowTokenAddress: vault.value.borrowPoolData.pool.token_address,
      depositTokenAddress: vault.value.depositPoolData.pool.token_address,
    })

    await market.openMultiply({
      client: activeClient.value,
      market: vault.value.market,
      deposit_pool_address: vault.value.depositPoolData.pool.pool_address,
      borrow_pool_address: vault.value.borrowPoolData.pool.pool_address,
      obligation_key: obligationKey,
      initial_amount: amount.value,
      leverage_multiplier: selectedMultiplier.value,
      margin_asset: marginAssetType.value,
      slippage: slippage.value,
      swap_provider: SOROSWAP_PROVIDER_ADDRESS,
      path: swapPath.value,
      action: async () => {
        await Promise.allSettled([
          userStore.updateUserMultiplyObligations(vault.value!.market, activeClient.value!, false),
          marketsStore.updatePool(vault.value!.depositPoolData.pool.pool_address, vault.value!.market, activeClient.value!),
          marketsStore.updatePool(vault.value!.borrowPoolData.pool.pool_address, vault.value!.market, activeClient.value!),
        ])
      },
      reset,
    })

    return true
  }

  function reset() {
    amount.value = undefined
    preview.value = undefined
    previewError.value = undefined
    isMarginBorrow.value = true
    percentFromMax.value = Math.max(minPercent.value || 0, 85)
    marketsStore.dialogLeverage = false
  }

  watch(() => vault.value?.pairKey, () => {
    reset()
  }, { immediate: true })

  watch(() => minPercent.value, (value) => {
    if (percentFromMax.value < value) {
      percentFromMax.value = value
    }
  }, { immediate: true })

  watchDebounced([
    amount,
    slippage,
    isMarginBorrow,
    selectedMultiplier,
    vault,
    activeClient,
  ], refreshPreview, { debounce: 250, maxWait: 1000 })

  onScopeDispose(() => {
    clearInterval(interval)
  })

  return {
    amount,
    balance,
    marginPrice,
    slippage,
    isMarginBorrow,
    marginAsset,
    notMarginAsset,
    percentFromMax,
    minPercent,
    selectedMultiplier,
    currentApy,
    blockedReason,
    unhealthyReason,
    maxInputAmount,
    availableBorrowLiquidity,
    flashLoanFeeBps,
    preview,
    summary,
    loadingPreview,
    previewError,
    openMultiply,
    reset,
    swapProviderAddress: SOROSWAP_PROVIDER_ADDRESS,
  }
}
