import type { MultiplyMarginAsset, MultiplyPreview } from '@alula/client-sdk'
import type { MultiplyVaultItem } from '~/types/table'
import { bpsToNumber } from '@alula/client-sdk'
import Decimal from 'decimal.js'
import { destructurePoolAsset } from '~/utils'
import { buildMultiplyObligationKey } from '~/utils/obligation'

const DEFAULT_PERCENT_FROM_MAX = 85

export function useMultiplyOpen(vaultRef: MaybeRef<MultiplyVaultItem | undefined>) {
  const market = useMarketActions()
  const marketsStore = useMarketsStore()
  const userStore = useUserStore()
  const multiplyStore = useMultiplyStore()

  const {
    publicKey,
    nativeBalance,
    getAssetBalance,
  } = useWalletComposable()

  const vault = computed(() => unref(vaultRef))

  const amount = ref<number | undefined>()
  const slippage = ref(0.05)
  const percentFromMax = ref(DEFAULT_PERCENT_FROM_MAX)
  const swapProviderAddress = computed(() => multiplyStore.swapProviderAddress)
  const preview = ref<MultiplyPreview>()

  const previewRequest = useLatestRequest<MultiplyPreview>()
  const loadingPreview = previewRequest.loading
  const previewError = previewRequest.error

  const activeClient = computed(() => vault.value ? marketsStore.state.markets[vault.value.market]?.client : undefined)
  const marketState = computed(() => vault.value ? marketsStore.state.markets[vault.value.market]?.marketState : undefined)

  // Single source of truth for token-decimal lookups. All consumers below should reach
  // for these instead of redoing `vault.value.{deposit,borrow}PoolData.pool.token_decimals`.
  const vaultDecimals = computed(() => {
    if (!vault.value) {
      return undefined
    }
    return {
      borrowDecimals: vault.value.borrowPoolData.pool.token_decimals,
      depositDecimals: vault.value.depositPoolData.pool.token_decimals,
    }
  })

  // Oracle-priced view of the pair. Returns undefined until both vault and marketState load.
  const oraclePrices = computed(() => {
    if (!vault.value || !marketState.value) {
      return undefined
    }
    const oracleDecimals = Number(marketState.value.oracle_price_decimals || 0)
    return {
      oracleDecimals,
      depositPrice: Number(bigintToNumber(vault.value.depositPoolData.oracle_asset_price, oracleDecimals)),
      borrowPrice: Number(bigintToNumber(vault.value.borrowPoolData.oracle_asset_price, oracleDecimals)),
    }
  })

  const isMarginBorrow = ref(false)
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

  // Hard cap enforced on chain by the deposit pool's open_ltv (1 / (1 - open_ltv)).
  // Different from `vault.maxMultiplier`, which applies a 0.8 SAFETY_MULTIPLIER discount
  // to leave headroom for slippage/fees at open. Live positions can drift above the
  // suggested max but below the hard cap due to price movement after open.
  const hardMaxMultiplier = computed(() => {
    const openLtvBps = Number(vault.value?.depositPoolData.pool.config.health_config.open_ltv_bps || 0)
    if (openLtvBps <= 0 || openLtvBps >= 10_000) {
      return undefined
    }
    return 1 / (1 - openLtvBps / 10_000)
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

    // Use realized leverage from the preview when available so APY reflects what the user
    // actually opens — at high slippage in deposit-margin mode, realized leverage drops well
    // below the slider target (see projectedLeverage docstring) and APY drops with it.
    const leverage = projectedLeverage.value ?? selectedMultiplier.value

    return (supplyApy * leverage - borrowApy * Math.max(leverage - 1, 0)) * 100
  })

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

  const flashLoanFeeBps = computed(() => Number(marginPool.value?.pool.config.fee_config.flash_loan_fee_bps || 0))

  const availableBorrowLiquidity = computed(() => {
    if (!vault.value?.borrowPoolData || !vaultDecimals.value) {
      return 0
    }

    const pool = vault.value.borrowPoolData
    const decimals = vaultDecimals.value.borrowDecimals
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

    const prices = oraclePrices.value
    if (!prices || prices.depositPrice <= 0 || prices.borrowPrice <= 0) {
      return 0
    }

    return (maxFlashBorrow * prices.borrowPrice)
      / (prices.depositPrice * Math.max(selectedMultiplier.value - 1, 0))
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

  const summary = computed(() => {
    if (!preview.value || !vaultDecimals.value) {
      return
    }

    const { borrowDecimals, depositDecimals } = vaultDecimals.value

    return {
      // Flash always pulls from the borrow pool in V3, so flashBorrowAmount is in borrow-asset units.
      flashBorrowAmount: Number(bigintToNumber(preview.value.flashBorrowAmount, borrowDecimals)),
      swapAmountIn: Number(bigintToNumber(preview.value.swapAmountIn, borrowDecimals)),
      expectedAmountOut: Number(bigintToNumber(preview.value.expectedAmountOut, depositDecimals)),
      minAmountOut: Number(bigintToNumber(preview.value.minAmountOut, depositDecimals)),
      depositAmount: Number(bigintToNumber(preview.value.depositAmount, depositDecimals)),
      finalBorrowAmount: Number(bigintToNumber(preview.value.finalBorrowAmount, borrowDecimals)),
    }
  })

  const flashLoanFeeAmount = computed(() => {
    if (!preview.value || !vaultDecimals.value) {
      return 0
    }

    return Number(
      bigintToNumber(
        preview.value.flashRepaymentAmount - preview.value.flashBorrowAmount,
        vaultDecimals.value.borrowDecimals,
      ),
    )
  })

  // Health-check inputs derived once and reused by both unhealthyReason and
  // maxTolerableSlippagePercent. Returns undefined when preview/market data isn't ready.
  const healthCheckInputs = computed(() => {
    if (!vault.value || !preview.value || !marketState.value || !vaultDecimals.value || !oraclePrices.value) {
      return undefined
    }

    const { depositDecimals, borrowDecimals } = vaultDecimals.value
    const { depositPrice, borrowPrice } = oraclePrices.value

    const depositAmount = Number(bigintToNumber(preview.value.depositAmount, depositDecimals)) || 0
    const expectedDepositAmount = Number(bigintToNumber(preview.value.expectedAmountOut, depositDecimals)) || 0
    const borrowAmount = Number(bigintToNumber(preview.value.finalBorrowAmount, borrowDecimals)) || 0

    const openLtv = bpsToNumber(Number(vault.value.depositPoolData.pool.config.health_config.open_ltv_bps || 0))
    const liabilityFactor = bpsToNumber(Number(vault.value.borrowPoolData.pool.config.health_config.liability_factor_bps || 0))
    const hasBorrowBackingCollateral = Number(vault.value.depositPoolData.pool.config.health_config.close_ltv_bps || 0) > 0
    const minCollateralRequirementUsd = hasBorrowBackingCollateral
      ? Number(marketState.value.global_state.min_collateral_value_cents || 0) / 100
      : 0

    return {
      depositAmount,
      expectedDepositAmount,
      borrowAmount,
      depositPrice,
      borrowPrice,
      openLtv,
      liabilityFactor,
      minCollateralRequirementUsd,
    }
  })

  // Realized on-chain leverage projected from the preview, in oracle USD.
  //   realized = collateralUsd / equityUsd, where equity = collateralUsd − debtUsd.
  //
  // Can land above OR below the slider target depending on swap economics:
  //   * AMM fee + price impact: router quotes (`getAmountIn` / `getAmountOut`) include pool
  //     fees and depth-sensitive impact, so the swap consumes more borrow per unit collateral
  //     than the oracle suggests → debtUsd grows → realized > target.
  //   * Oracle/AMM divergence: when AMM is "expensive" vs oracle, same effect.
  //   * Deposit-margin slippage floor: under perfectly fair swaps both legs scale with
  //     `minAmountOut`, so realized ≈ 1 + (L − 1)(1 − slippage) ≤ target. Real opens drift
  //     above or below depending on which AMM effect dominates.
  // Returns undefined when preview/market data isn't ready.
  const projectedLeverage = computed<number | undefined>(() => {
    const inputs = healthCheckInputs.value
    if (!inputs) {
      return undefined
    }

    const collateralUsd = inputs.depositAmount * inputs.depositPrice
    const debtUsd = inputs.borrowAmount * inputs.borrowPrice
    const equityUsd = collateralUsd - debtUsd

    if (!Number.isFinite(collateralUsd) || !Number.isFinite(equityUsd) || equityUsd <= 0) {
      return undefined
    }

    return collateralUsd / equityUsd
  })

  // True depth-driven price impact of the swap leg, sourced directly from the
  // SDK preview's `priceImpactBps`. The SDK computes it provider-agnostically by
  // probing the same path with a small input and comparing rates — provider fee
  // cancels in the ratio, so what's left is pure depth impact. See
  // `MultiplyService.calculatePriceImpactBps` for the math.
  //
  // Distinct from "all-in cost vs oracle" (which would also fold in fee and
  // oracle/AMM divergence). This number answers the question the UI label has
  // always claimed: "how much is your trade size moving the pool?"
  const priceImpactPercent = computed<number | undefined>(() => {
    const bps = preview.value?.priceImpactBps
    return bps == null ? undefined : bps / 100
  })

  // Slippage cap above which the on-chain open_ltv check would fail. Only meaningful for
  // margin=borrow: in that flow `finalBorrowAmount` is sized from a slippage-independent
  // `flashBorrowAmount`, while `depositAmount = minAmountOut` shrinks with slippage —
  // so the two legs decouple and the cap is a real, monotonically-decreasing function
  // of slippage. In V3 deposit-margin both legs scale with `minAmountOut` together
  // (debt is grossed up from the same flash size that funded the swap), so high slippage
  // just opens at a lower effective leverage rather than underwater. The cap doesn't
  // apply there and is returned as undefined so the UI can hide the hint.
  // Returns undefined if inputs missing or mode N/A, 0 if even zero slippage wouldn't open.
  const maxTolerableSlippagePercent = computed<number | undefined>(() => {
    if (!isMarginBorrow.value) {
      return undefined
    }
    const inputs = healthCheckInputs.value
    if (!inputs) {
      return undefined
    }

    const debtValueUsd = inputs.borrowAmount * inputs.borrowPrice * inputs.liabilityFactor + inputs.minCollateralRequirementUsd

    // Borrow-asset margin: ALL collateral is the swap output, so zero-slippage baseline
    // is the router's expectedAmountOut (already net of swap fees).
    const collateralAtZeroSlippageUsd = inputs.expectedDepositAmount * inputs.depositPrice * inputs.openLtv

    if (!Number.isFinite(collateralAtZeroSlippageUsd) || collateralAtZeroSlippageUsd <= 0) {
      return undefined
    }

    const ratio = debtValueUsd / collateralAtZeroSlippageUsd
    if (ratio >= 1) {
      // Position is structurally unhealthy at any slippage — leverage is too high for this pair.
      return 0
    }

    return Math.max(0, (1 - ratio) * 100)
  })

  const unhealthyReason = computed(() => {
    const inputs = healthCheckInputs.value
    if (!inputs) {
      return ''
    }

    const collateralValueScaled = new Decimal(inputs.depositAmount).mul(inputs.depositPrice).mul(inputs.openLtv)
    const debtValueScaled = new Decimal(inputs.borrowAmount).mul(inputs.borrowPrice).mul(inputs.liabilityFactor)
    const healthBufferUsd = collateralValueScaled.minus(debtValueScaled).minus(inputs.minCollateralRequirementUsd)

    if (!healthBufferUsd.isFinite()) {
      return 'Multiply preview is invalid. Try a smaller amount or lower multiplier.'
    }

    if (healthBufferUsd.lte(0)) {
      const cap = maxTolerableSlippagePercent.value
      if (cap === 0) {
        return `Multiplier ${selectedMultiplier.value.toFixed(2)}× is too high for this pair — even zero slippage would open underwater. Lower the multiplier.`
      }
      if (cap != null && cap > 0) {
        return `Slippage ${slippage.value}% is too high at ${selectedMultiplier.value.toFixed(2)}×. Lower slippage to ≤ ${cap.toFixed(2)}% or reduce the multiplier.`
      }
      return 'This multiply setup would be unhealthy after swap slippage, flash-loan fee, and liability-factor checks. Lower the multiplier or amount.'
    }

    return ''
  })

  async function refreshPreview() {
    if (!vault.value || !activeClient.value || !amount.value || amount.value <= 0 || selectedMultiplier.value <= 1 || !publicKey.value) {
      previewRequest.cancel()
      preview.value = undefined
      previewError.value = undefined
      return
    }

    await previewRequest.run(
      () => activeClient.value!.multiply.getOpenPositionPreview({
        depositPoolAddress: vault.value!.depositPoolData.pool.pool_address,
        borrowPoolAddress: vault.value!.borrowPoolData.pool.pool_address,
        initialAmount: amount.value!,
        leverageMultiplier: selectedMultiplier.value,
        marginAsset: marginAssetType.value,
        slippagePercent: slippage.value,
        swapProviderAddress: swapProviderAddress.value,
        path: swapPath.value,
      }),
      (result) => { preview.value = result },
      () => { preview.value = undefined },
    )
  }

  async function openMultiply() {
    if (!vault.value || !activeClient.value || !amount.value || !publicKey.value || unhealthyReason.value) {
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
      swap_provider: swapProviderAddress.value,
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
    isMarginBorrow.value = false
    percentFromMax.value = Math.max(minPercent.value || 0, DEFAULT_PERCENT_FROM_MAX)
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
    swapProviderAddress,
  ], refreshPreview, { debounce: 250, maxWait: 1000 })

  onScopeDispose(() => {
    previewRequest.cancel()
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
    hardMaxMultiplier,
    currentApy,
    projectedLeverage,
    priceImpactPercent,
    unhealthyReason,
    maxTolerableSlippagePercent,
    maxInputAmount,
    availableBorrowLiquidity,
    flashLoanFeeBps,
    flashLoanFeeAmount,
    preview,
    summary,
    loadingPreview,
    previewError,
    openMultiply,
    reset,
  }
}
