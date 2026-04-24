import type { BorrowPosition, DepositPosition, Obligation, ObligationKey, PoolData, Request } from '@alula/market-sdk'
import type { DecimalsConfig } from '../config/decimals'
import type { RPCcluster } from '../types'
import { Client as MarketClient } from '@alula/market-sdk'
import { Client as AquaSwapProviderClient } from 'aqua_swap_provider'
import Decimal from 'decimal.js'
import { Client as SoroswapSwapProviderClient } from 'soroswap_swap_provider'
import { AQUA_PROVIDER_ADDRESS, MAX_I128 } from '../constants'
import { BaseClient } from '../core/base-client'
import { TransactionHelper } from '../core/transaction-builder'

export interface MultiplyServiceConfig {
  rpc: RPCcluster
  publicKey?: string
  contractId?: string
  horizonRpcUrl?: string
  sorobanRpcUrl?: string
  decimals: DecimalsConfig
}

export type MultiplyMarginAsset = 'borrow' | 'deposit'

export type MultiplyFlowVersion = 'v2' | 'v3'

// V3 single-anchor multiply (per docs/SecondTokenMarginGuideV3.md):
//   FlashBorrow(debt) → SwapExactTokens(debt→collateral) → AddCollateral(margin+Y) → Borrow(X+flash_fee)
// Preconditions strictly required for the determinism invariants to hold:
//   1. deposit pool add_collateral_fee_bps == 0  (else AddCollateral credits less than literal)
//   2. borrow pool  borrow_fee_bps == 0          (else Borrow pays less than literal → flash repay reverts
//                                                 OR silently milks the wallet)
//   3. referrer must be undefined/None           (defense-in-depth; with bps==0 it's already a no-op)
// If any precondition fails, the SDK falls back to the legacy V2-style flow (kept inline).
const SWAP_PRINCIPAL_SAFETY_BPS = 5

export interface MultiplyPreviewParams {
  depositPoolAddress: string
  borrowPoolAddress: string
  initialAmount: string | number
  leverageMultiplier: number
  marginAsset?: MultiplyMarginAsset
  slippagePercent?: number
  swapProviderAddress: string
  path?: string[]
}

export interface OpenMultiplyParams extends MultiplyPreviewParams {
  user: ObligationKey
  referrer?: string
}

export interface CloseMultiplyPreviewParams {
  user: ObligationKey
  depositPoolAddress: string
  borrowPoolAddress: string
  marginAsset?: MultiplyMarginAsset
  repayAmount?: string | number
  slippagePercent?: number
  swapProviderAddress: string
  path?: string[]
}

export interface CloseMultiplyParams extends CloseMultiplyPreviewParams {
  minReceiveAmount?: string | number
  referrer?: string
}

export interface MultiplyPreview {
  depositPool: PoolData
  borrowPool: PoolData
  leverageMultiplier: number
  maxLeverageMultiplier: number
  marginAsset: MultiplyMarginAsset
  flowVersion: MultiplyFlowVersion
  flashLoanFeeBps: number
  swapPath: string[]
  swapProviderAddress: string
  initialAmount: bigint
  flashBorrowAmount: bigint
  flashRepaymentAmount: bigint
  swapAmountIn: bigint
  expectedAmountOut: bigint
  minAmountOut: bigint
  depositAmount: bigint
  finalBorrowAmount: bigint
}

export interface CloseMultiplyPreview {
  depositPool: PoolData
  borrowPool: PoolData
  marginAsset: MultiplyMarginAsset
  flowVersion: MultiplyFlowVersion
  currentDepositAmount: bigint
  currentBorrowAmount: bigint
  maxRepayAmount: bigint
  repayAmount: bigint
  debtRepaidAmount: bigint
  flashBorrowAmount: bigint
  flashLoanFeeBps: number
  flashRepaymentAmount: bigint
  swapPath: string[]
  swapProviderAddress: string
  withdrawAmount: bigint
  depositReceiveAmount: bigint
  estimatedReceiveAmount: bigint
  remainingDepositAmount: bigint
  remainingBorrowAmount: bigint
  isFullClose: boolean
  requiredAmountIn: bigint
  maxReceivableAmount: bigint
}

const DEFAULT_SLIPPAGE_PERCENT = 0.5
const MAX_SLIPPAGE_PERCENT = 50
const SAFETY_MULTIPLIER = 0.8
const CLOSE_REPAY_BUFFER_BPS = 100

export function calculateMultiplyMaxLeverage(openLtvBps: number): number {
  if (!Number.isFinite(openLtvBps) || openLtvBps < 0 || openLtvBps >= 10_000) {
    throw new Error(`openLtvBps must be in [0, 10000), got ${openLtvBps}`)
  }

  const openLtv = new Decimal(openLtvBps).div(10_000)
  return new Decimal(SAFETY_MULTIPLIER)
    .div(new Decimal(1).minus(openLtv))
    .toDecimalPlaces(2, Decimal.ROUND_DOWN)
    .toNumber()
}

export class MultiplyService extends BaseClient {
  private marketClient: MarketClient
  private txHelper: TransactionHelper
  private CACHE_TTL = 10000
  private poolCache = new Map<string, { data: PoolData, ts: number }>()
  private obligationCache = new Map<string, { data: Obligation, ts: number }>()

  constructor(config: MultiplyServiceConfig) {
    super(config)

    this.marketClient = new MarketClient({
      publicKey: config.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId: config.contractId || '',
      networkPassphrase: this.networkPassphrase,
    })

    this.txHelper = new TransactionHelper(config.rpc, this.sorobanServer)
  }

  async getOpenPositionPreview(params: MultiplyPreviewParams): Promise<MultiplyPreview> {
    const depositPool = await this.getPoolData(params.depositPoolAddress)
    const borrowPool = await this.getPoolData(params.borrowPoolAddress)
    const marginAsset = params.marginAsset ?? 'borrow'

    const maxLeverageMultiplier = calculateMultiplyMaxLeverage(
      Number(depositPool.pool.config.health_config.open_ltv_bps),
    )

    const leverageMultiplier = new Decimal(params.leverageMultiplier)
    if (!leverageMultiplier.isFinite() || leverageMultiplier.lte(1)) {
      throw new Error('Leverage multiplier must be greater than 1')
    }
    if (leverageMultiplier.gt(maxLeverageMultiplier)) {
      throw new Error(`Leverage multiplier exceeds max ${maxLeverageMultiplier}`)
    }

    const slippagePercent = this.normalizeSlippage(params.slippagePercent)
    const initialAmount = this.amountToBigInt(
      params.initialAmount,
      marginAsset === 'borrow' ? borrowPool.pool.token_decimals : depositPool.pool.token_decimals,
    )
    if (initialAmount <= 0n) {
      throw new Error('Initial amount must be greater than 0')
    }

    const swapPath = params.path?.length ? params.path : [borrowPool.pool.token_address, depositPool.pool.token_address]
    const borrowFeeBps = Number(borrowPool.pool.config.fee_config.borrow_fee_bps || 0)
    const addCollateralFeeBps = Number(depositPool.pool.config.fee_config.add_collateral_fee_bps || 0)

    // V3 is the only supported open flow. Refuse early if the pool config violates the
    // determinism preconditions — silently falling back to V2 here would either re-introduce
    // phantom debt (borrow_fee_bps>0, see tests/src/multiply_v3.rs::v3_silently_milks_user_*)
    // or under-credit collateral (add_collateral_fee_bps>0). A clean preview-time error is
    // strictly safer than either failure mode.
    if (borrowFeeBps !== 0 || addCollateralFeeBps !== 0) {
      throw new Error(
        `Multiply requires borrow_fee_bps=0 and add_collateral_fee_bps=0 on the involved pools, `
        + `got borrow_fee_bps=${borrowFeeBps}, add_collateral_fee_bps=${addCollateralFeeBps}. `
        + `Reconfigure the pools or disable multiply for this market.`,
      )
    }

    let flashBorrowAmount: bigint
    let flashRepaymentAmount: bigint
    let swapAmountIn: bigint
    let depositAmount: bigint
    let expectedAmountOut: bigint
    let minAmountOut: bigint
    let finalBorrowAmount: bigint
    let flashLoanFeeBps: number
    const flowVersion: MultiplyFlowVersion = 'v3'

    if (marginAsset === 'borrow') {
      // V3 with margin in DEBT asset: user puts X_user USDC, we flash extra USDC,
      // swap all USDC→XLM, anchor as a single AddCollateral, borrow back exactly the flash repay.
      flashBorrowAmount = this.decimalToBigInt(
        new Decimal(initialAmount.toString()).mul(leverageMultiplier.minus(1)),
        Decimal.ROUND_DOWN,
      )
      swapAmountIn = initialAmount + flashBorrowAmount
      flashLoanFeeBps = Number(borrowPool.pool.config.fee_config.flash_loan_fee_bps || 0)

      const expectedAmountsOut = await this.getExpectedAmountsOut(params.swapProviderAddress, swapAmountIn, swapPath)
      expectedAmountOut = expectedAmountsOut[expectedAmountsOut.length - 1]
      if (expectedAmountOut == null) {
        throw new Error('Router did not return an output quote for this multiply swap path')
      }

      minAmountOut = this.applySlippageDown(expectedAmountOut, slippagePercent)
      depositAmount = minAmountOut
      flashRepaymentAmount = flashBorrowAmount + this.calculateFee(flashBorrowAmount, flashLoanFeeBps)
      // V3: exact, no gross-up (borrow_fee_bps == 0 enforced above).
      finalBorrowAmount = flashRepaymentAmount
    } else {
      // V3 canonical: margin in DEPOSIT (collateral) asset, flash-borrow DEBT asset, swap debt→collateral,
      // anchor everything in one AddCollateral(margin + Y), borrow exact (X + flash_fee).
      // Per docs/SecondTokenMarginGuideV3.md.
      flashLoanFeeBps = Number(borrowPool.pool.config.fee_config.flash_loan_fee_bps || 0)

      // Y = floor((L - 1) × margin × (1 - slippage))  — collateral-side floor in deposit asset units.
      const targetCollateralToAdd = new Decimal(initialAmount.toString()).mul(leverageMultiplier.minus(1))
      minAmountOut = this.decimalToBigInt(
        targetCollateralToAdd.mul(new Decimal(1).minus(new Decimal(slippagePercent).div(100))),
        Decimal.ROUND_DOWN,
      )
      if (minAmountOut <= 0n) {
        throw new Error('Initial amount is too small for the selected multiplier')
      }

      // Quote: how many DEBT tokens (X_quote) to receive Y collateral tokens?
      const expectedAmountsIn = await this.getExpectedAmountsIn(params.swapProviderAddress, minAmountOut, swapPath)
      const xQuote = expectedAmountsIn[0]
      if (xQuote == null || xQuote <= 0n) {
        throw new Error('Router did not return an input quote for this multiply swap path')
      }

      // X = ceil(X_quote × (1 + safety_bps)) — small buffer against router-internal rounding.
      swapAmountIn = this.decimalToBigInt(
        new Decimal(xQuote.toString()).mul(
          new Decimal(1).plus(new Decimal(SWAP_PRINCIPAL_SAFETY_BPS).div(10_000)),
        ),
        Decimal.ROUND_UP,
      )
      flashBorrowAmount = swapAmountIn
      flashRepaymentAmount = flashBorrowAmount + this.calculateFee(flashBorrowAmount, flashLoanFeeBps)

      // V3 invariant: Borrow.amount = X + flash_fee  — exact, NO gross-up (borrow_fee_bps == 0).
      finalBorrowAmount = flashRepaymentAmount
      // V3 invariant: AddCollateral.amount = margin + Y  — single deterministic anchor.
      depositAmount = initialAmount + minAmountOut
      expectedAmountOut = minAmountOut
    }

    return {
      depositPool,
      borrowPool,
      leverageMultiplier: leverageMultiplier.toNumber(),
      maxLeverageMultiplier,
      marginAsset,
      flowVersion,
      flashLoanFeeBps,
      swapPath,
      swapProviderAddress: params.swapProviderAddress,
      initialAmount,
      flashBorrowAmount,
      flashRepaymentAmount,
      swapAmountIn,
      expectedAmountOut,
      minAmountOut,
      depositAmount,
      finalBorrowAmount,
    }
  }

  async buildOpenPositionTx(params: OpenMultiplyParams) {
    const preview = await this.getOpenPositionPreview(params)

    if (params.referrer != null) {
      // V3 precondition #4: referrer must be None. With borrow_fee_bps=0 the referrer slice
      // is mathematically zero anyway, but we refuse explicitly to keep the invariant local.
      throw new Error('Multiply (V3) does not support a referrer; pass referrer=undefined')
    }

    // V3 batch (always): FlashBorrow(debt) → SwapExactTokens(debt→collateral)
    //                  → AddCollateral(anchor) → Borrow(exact flash repay).
    const requests: Request[] = [
      {
        tag: 'FlashBorrow',
        values: [{
          amount: preview.flashBorrowAmount,
          pool_address: params.borrowPoolAddress,
        }],
      },
      {
        tag: 'SwapExactTokens',
        values: [{
          amount_in: preview.swapAmountIn,
          min_amount_out: preview.minAmountOut,
          path: preview.swapPath,
          swap_provider: params.swapProviderAddress,
        }],
      },
      {
        tag: 'AddCollateral',
        values: [{
          amount: preview.depositAmount,
          pool_address: params.depositPoolAddress,
        }],
      },
      {
        tag: 'Borrow',
        values: [{
          amount: preview.finalBorrowAmount,
          pool_address: params.borrowPoolAddress,
        }],
      },
    ]

    const tx = await this.marketClient.submit_requests_batch({
      user: params.user,
      requests,
      referrer: params.referrer,
    })

    return { preview, tx }
  }

  async getClosePositionPreview(params: CloseMultiplyPreviewParams): Promise<CloseMultiplyPreview> {
    const [depositPool, borrowPool, obligation] = await Promise.all([
      this.getPoolData(params.depositPoolAddress),
      this.getPoolData(params.borrowPoolAddress),
      this.getUserObligation(params.user),
    ])
    const marginAsset = params.marginAsset ?? 'deposit'

    const depositPosition = this.findDepositPosition(obligation, params.depositPoolAddress)
    const borrowPosition = this.findBorrowPosition(obligation, params.borrowPoolAddress)

    // V3 stores margin+Y in `collateral`; V2 stores it as j_tokens (supply shares).
    // If `collateral > 0` the position was opened V3-style → close via RemoveCollateral.
    const collateralAmount = BigInt(depositPosition.collateral || 0n)
    const flowVersion: MultiplyFlowVersion = collateralAmount > 0n ? 'v3' : 'v2'
    const currentDepositAmount = flowVersion === 'v3'
      ? collateralAmount
      : this.calculateCurrentDepositAmount(depositPool, depositPosition.j_tokens)
    const currentBorrowAmount = this.calculateCurrentBorrowAmount(borrowPool, borrowPosition.d_tokens)

    if (currentDepositAmount <= 0n) {
      throw new Error('No deposited collateral found for this multiply position')
    }

    if (currentBorrowAmount <= 0n) {
      throw new Error('No active borrow found for this multiply position')
    }

    const slippagePercent = this.normalizeSlippage(params.slippagePercent)
    const repayFeeBps = Number(borrowPool.pool.config.fee_config.repay_fee_bps || 0)
    const fullCloseRepayAmount = currentBorrowAmount + this.calculateCloseRepayBuffer(currentBorrowAmount)

    const requestedRepayAmount = params.repayAmount == null
      ? fullCloseRepayAmount
      : this.amountToBigInt(params.repayAmount, borrowPool.pool.token_decimals)

    if (requestedRepayAmount <= 0n) {
      throw new Error('Repay amount must be greater than 0')
    }

    const isFullClose = requestedRepayAmount >= fullCloseRepayAmount
    const repayAmount = isFullClose ? fullCloseRepayAmount : requestedRepayAmount
    const debtRepaidAmount = isFullClose
      ? currentBorrowAmount
      : this.calculatePartialDebtRepaidAmount(repayAmount, repayFeeBps, currentBorrowAmount)

    const flashLoanFeeBps = Number(borrowPool.pool.config.fee_config.flash_loan_fee_bps || 0)
    const flashBorrowAmount = repayAmount
    const flashRepaymentAmount = flashBorrowAmount + this.calculateFee(flashBorrowAmount, flashLoanFeeBps)

    const swapPath = params.path?.length
      ? params.path
      : [depositPool.pool.token_address, borrowPool.pool.token_address]

    const expectedAmountsIn = await this.getExpectedAmountsIn(params.swapProviderAddress, flashRepaymentAmount, swapPath)
    const quotedRequiredAmountIn = expectedAmountsIn[0]
    if (quotedRequiredAmountIn == null) {
      throw new Error('Router did not return an input quote for this multiply close path')
    }

    const requiredAmountIn = this.applySlippageUp(quotedRequiredAmountIn, slippagePercent)

    if (requiredAmountIn > currentDepositAmount) {
      throw new Error('Not enough collateral to close this multiply position')
    }

    const withdrawAmount = isFullClose
      ? currentDepositAmount
      : this.calculatePartialWithdrawAmount(currentDepositAmount, debtRepaidAmount, currentBorrowAmount)

    if (!isFullClose && requiredAmountIn > withdrawAmount) {
      throw new Error('Repay amount is too small to cover flash-loan repayment for a partial close')
    }

    const depositReceiveAmount = isFullClose
      ? currentDepositAmount - requiredAmountIn
      : withdrawAmount - requiredAmountIn

    const maxReceivableDepositAmount = currentDepositAmount - requiredAmountIn

    let estimatedReceiveAmount = depositReceiveAmount
    let maxReceivableAmount = maxReceivableDepositAmount

    if (marginAsset === 'borrow') {
      estimatedReceiveAmount = await this.quoteSwapExactOut(params.swapProviderAddress, depositReceiveAmount, swapPath)
      maxReceivableAmount = await this.quoteSwapExactOut(params.swapProviderAddress, maxReceivableDepositAmount, swapPath)
    }

    return {
      depositPool,
      borrowPool,
      marginAsset,
      flowVersion,
      currentDepositAmount,
      currentBorrowAmount,
      maxRepayAmount: fullCloseRepayAmount,
      repayAmount,
      debtRepaidAmount,
      flashBorrowAmount,
      flashLoanFeeBps,
      flashRepaymentAmount,
      swapPath,
      swapProviderAddress: params.swapProviderAddress,
      withdrawAmount,
      depositReceiveAmount,
      estimatedReceiveAmount,
      remainingDepositAmount: currentDepositAmount - withdrawAmount,
      remainingBorrowAmount: currentBorrowAmount - debtRepaidAmount,
      isFullClose,
      requiredAmountIn,
      maxReceivableAmount,
    }
  }

  async buildClosePositionTx(params: CloseMultiplyParams) {
    const preview = await this.getClosePositionPreview(params)
    const receiveAssetDecimals = preview.marginAsset === 'borrow'
      ? preview.borrowPool.pool.token_decimals
      : preview.depositPool.pool.token_decimals

    const minReceiveAmount = params.minReceiveAmount == null
      ? (preview.marginAsset === 'deposit' && preview.isFullClose ? preview.maxReceivableAmount : 0n)
      : this.amountToBigInt(params.minReceiveAmount, receiveAssetDecimals)

    if (minReceiveAmount < 0n) {
      throw new Error('Minimum receive amount must be non-negative')
    }

    const maxReceiveAmount = preview.isFullClose ? preview.maxReceivableAmount : preview.estimatedReceiveAmount
    if (minReceiveAmount > maxReceiveAmount) {
      throw new Error('Requested receive amount exceeds what this multiply close can return')
    }

    const requestedWithdrawAmount = preview.isFullClose
      ? MAX_I128
      : preview.withdrawAmount

    // V3 positions store collateral in `obligation.collateral`; release with RemoveCollateral.
    // V2 positions store it as j_tokens (supply shares); release with Withdraw.
    const collateralReleaseTag = preview.flowVersion === 'v3' ? 'RemoveCollateral' : 'Withdraw'

    const requests: Request[] = [
      {
        tag: 'FlashBorrow',
        values: [{
          amount: preview.flashBorrowAmount,
          pool_address: params.borrowPoolAddress,
        }],
      },
      {
        tag: 'Repay',
        values: [{
          amount: preview.repayAmount,
          pool_address: params.borrowPoolAddress,
        }],
      },
      {
        tag: collateralReleaseTag,
        values: [{
          amount: requestedWithdrawAmount,
          pool_address: params.depositPoolAddress,
        }],
      },
      {
        tag: 'SwapForExactTokens',
        values: [{
          max_amount_in: preview.marginAsset === 'deposit'
            ? preview.withdrawAmount - minReceiveAmount
            : preview.requiredAmountIn,
          amount_out: preview.flashRepaymentAmount,
          path: preview.swapPath,
          swap_provider: params.swapProviderAddress,
        }],
      },
    ]

    if (preview.marginAsset === 'deposit') {
      const maxAmountIn = preview.withdrawAmount - minReceiveAmount
      if (preview.requiredAmountIn > maxAmountIn) {
        throw new Error('Requested receive amount is too high for this multiply close')
      }
    } else if (preview.depositReceiveAmount > 0n) {
      requests.push({
        tag: 'SwapExactTokens',
        values: [{
          amount_in: preview.depositReceiveAmount,
          min_amount_out: minReceiveAmount,
          path: preview.swapPath,
          swap_provider: params.swapProviderAddress,
        }],
      })
    }

    const tx = await this.marketClient.submit_requests_batch({
      user: params.user,
      requests,
      referrer: params.referrer,
    })

    return {
      preview,
      tx,
      minReceiveAmount,
      maxAmountIn: preview.marginAsset === 'deposit'
        ? preview.withdrawAmount - minReceiveAmount
        : preview.requiredAmountIn,
    }
  }

  async openPosition(params: OpenMultiplyParams, kit: any, options = { debug: true }) {
    const { tx } = await this.buildOpenPositionTx(params)

    if (options?.debug) {
      console.log('%c[Multiply Tx]', 'color: #00ff00', tx)
    }

    return await this.txHelper.signAndSend(tx, params.user, kit, options)
  }

  async closePosition(params: CloseMultiplyParams, kit: any, options = { debug: true }) {
    const { tx } = await this.buildClosePositionTx(params)

    if (options?.debug) {
      console.log('%c[Multiply Close Tx]', 'color: #00ff00', tx)
    }
    this.poolCache.delete(params.depositPoolAddress)
    this.poolCache.delete(params.borrowPoolAddress)

    return await this.txHelper.signAndSend(tx, params.user, kit, options)
  }

  getTransactionFee(tx: any): number {
    return this.txHelper.getTransactionFee(tx)
  }

  private async getPoolData(poolAddress: string): Promise<PoolData> {
    const cached = this.poolCache.get(poolAddress)

    if (cached && Date.now() - cached.ts < this.CACHE_TTL) {
      return cached.data
    }

    const response = await this.marketClient.get_pool_data({ pool_address: poolAddress })
    const data = this.unwrapOk<PoolData>(response.result)

    this.poolCache.set(poolAddress, {
      data,
      ts: Date.now(),
    })

    return data
  }

  private async getUserObligation(user: ObligationKey): Promise<Obligation> {
    const key = JSON.stringify(user)
    const cached = this.obligationCache.get(key)

    if (cached && Date.now() - cached.ts < this.CACHE_TTL) {
      return cached.data
    }

    const response = await this.marketClient.get_user_obligation({ user })
    const data = this.unwrapOk<Obligation>(response.result)

    this.obligationCache.set(key, {
      data,
      ts: Date.now(),
    })

    return data
  }

  private async getExpectedAmountsOut(swapProviderAddress: string, amountIn: bigint, path: string[]): Promise<bigint[]> {
    const providerClient = this.getSwapProviderClient(swapProviderAddress)

    const response = await providerClient.get_amount_out({
      amount_in: amountIn,
      path,
    })

    return [amountIn, response.result]
  }

  private async getExpectedAmountsIn(swapProviderAddress: string, amountOut: bigint, path: string[]): Promise<bigint[]> {
    const providerClient = this.getSwapProviderClient(swapProviderAddress)

    const response = await providerClient.get_amount_in({
      amount_out: amountOut,
      path,
    })

    return [response.result, amountOut]
  }

  private async quoteSwapExactOut(swapProviderAddress: string, amountIn: bigint, path: string[]): Promise<bigint> {
    if (amountIn <= 0n) {
      return 0n
    }

    const expectedAmountsOut = await this.getExpectedAmountsOut(swapProviderAddress, amountIn, path)
    return expectedAmountsOut[expectedAmountsOut.length - 1] ?? 0n
  }

  private getSwapProviderClient(swapProviderAddress: string) {
    const options = {
      publicKey: this.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId: swapProviderAddress,
      networkPassphrase: this.networkPassphrase,
    }

    if (swapProviderAddress === AQUA_PROVIDER_ADDRESS) {
      return new AquaSwapProviderClient(options)
    }

    return new SoroswapSwapProviderClient(options)
  }

  private amountToBigInt(amount: string | number, decimals: number): bigint {
    return this.decimalToBigInt(
      new Decimal(amount).mul(new Decimal(10).pow(decimals)),
      Decimal.ROUND_DOWN,
    )
  }

  private applySlippageDown(amount: bigint, slippagePercent: number): bigint {
    return this.decimalToBigInt(
      new Decimal(amount.toString()).mul(new Decimal(1).minus(new Decimal(slippagePercent).div(100))),
      Decimal.ROUND_DOWN,
    )
  }

  private applySlippageUp(amount: bigint, slippagePercent: number): bigint {
    return this.decimalToBigInt(
      new Decimal(amount.toString()).mul(new Decimal(1).plus(new Decimal(slippagePercent).div(100))),
      Decimal.ROUND_UP,
    )
  }

  private decimalToBigInt(value: Decimal, rounding: Decimal.Rounding): bigint {
    return BigInt(value.toFixed(0, rounding))
  }

  private calculateFee(amount: bigint, feeBps: number): bigint {
    if (amount <= 0n || feeBps <= 0) {
      return 0n
    }

    return (amount * BigInt(feeBps) + 9_999n) / 10_000n
  }

  private calculateCloseRepayBuffer(amount: bigint): bigint {
    if (amount <= 0n) {
      return 0n
    }

    const bpsBuffer = this.calculateFee(amount, CLOSE_REPAY_BUFFER_BPS)
    return bpsBuffer > 0n ? bpsBuffer : 1n
  }

  private calculatePartialDebtRepaidAmount(repayAmount: bigint, repayFeeBps: number, currentBorrowAmount: bigint): bigint {
    const repayFeeAmount = this.calculateFee(repayAmount, repayFeeBps)
    const debtRepaidAmount = repayAmount - repayFeeAmount

    if (debtRepaidAmount <= 0n) {
      throw new Error('Repay amount is too small after fees to reduce the multiply debt')
    }

    return debtRepaidAmount > currentBorrowAmount ? currentBorrowAmount : debtRepaidAmount
  }

  private calculatePartialWithdrawAmount(currentDepositAmount: bigint, debtRepaidAmount: bigint, currentBorrowAmount: bigint): bigint {
    if (currentDepositAmount <= 0n || debtRepaidAmount <= 0n || currentBorrowAmount <= 0n) {
      return 0n
    }

    const proportionalWithdrawAmount = (currentDepositAmount * debtRepaidAmount) / currentBorrowAmount
    return proportionalWithdrawAmount > 0n ? proportionalWithdrawAmount : 1n
  }

  private findDepositPosition(obligation: Obligation, poolAddress: string) {
    const depositPosition = this.getPositionFromCollection<DepositPosition>(obligation.deposits, poolAddress)

    if (!depositPosition) {
      throw new Error('Deposit position for this multiply obligation does not exist')
    }

    return depositPosition
  }

  private findBorrowPosition(obligation: Obligation, poolAddress: string) {
    const borrowPosition = this.getPositionFromCollection<BorrowPosition>(obligation.borrows, poolAddress)

    if (!borrowPosition) {
      throw new Error('Borrow position for this multiply obligation does not exist')
    }

    return borrowPosition
  }

  private getPositionFromCollection<T>(
    positions: Map<string, T> | Array<[string, T]> | Record<string, T> | undefined,
    poolAddress: string,
  ): T | undefined {
    if (!positions) {
      return undefined
    }

    if (positions instanceof Map) {
      return positions.get(poolAddress)
    }

    if (Array.isArray(positions)) {
      return positions.find(([address]) => address === poolAddress)?.[1]
    }

    if (typeof positions === 'object') {
      const mapLike = positions as {
        get?: (key: string) => T | undefined
        entries?: () => Iterable<[string, T]>
      }

      if (typeof mapLike.get === 'function') {
        const matchedPosition = mapLike.get(poolAddress)
        if (matchedPosition) {
          return matchedPosition
        }
      }

      if (typeof mapLike.entries === 'function') {
        for (const [address, position] of mapLike.entries()) {
          if (address === poolAddress) {
            return position
          }
        }
      }

      return (positions as Record<string, T | undefined>)[poolAddress]
    }

    return undefined
  }

  private calculateCurrentDepositAmount(pool: PoolData, jTokens: bigint): bigint {
    if (jTokens <= 0n || pool.pool.total_j_tokens <= 0n) {
      return 0n
    }

    const totalSupply = pool.total_supply
    return (jTokens * totalSupply) / pool.pool.total_j_tokens
  }

  private calculateCurrentBorrowAmount(pool: PoolData, dTokens: bigint): bigint {
    if (dTokens <= 0n || pool.pool.total_d_tokens <= 0n || pool.pool.total_borrowed <= 0n) {
      return 0n
    }

    return (dTokens * pool.pool.total_borrowed + pool.pool.total_d_tokens - 1n) / pool.pool.total_d_tokens
  }

  private normalizeSlippage(slippagePercent: number | undefined): number {
    const normalized = slippagePercent ?? DEFAULT_SLIPPAGE_PERCENT
    if (!Number.isFinite(normalized) || normalized < 0 || normalized > MAX_SLIPPAGE_PERCENT) {
      throw new Error(`Slippage percent must be in [0, ${MAX_SLIPPAGE_PERCENT}]`)
    }
    return normalized
  }
}
