import type { BorrowPosition, DepositPosition, Obligation, ObligationKey, PoolData, Request } from '@alula/market-sdk'
import type { DecimalsConfig } from '../config/decimals'
import type { RPCcluster } from '../types'
import { Client as MarketClient } from '@alula/market-sdk'
import Decimal from 'decimal.js'
import { Client as SoroswapRouterClient } from 'soroswap_router'
import { Client as SoroswapSwapProviderClient } from 'soroswap_swap_provider'
import { MAX_I128 } from '../constants'
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
  flashLoanFeeBps: number
  swapPath: string[]
  swapProviderAddress: string
  routerAddress: string
  initialAmount: bigint
  flashBorrowAmount: bigint
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
  routerAddress: string
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
  private routerCache = new Map<string, string>()
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
    const routerAddress = await this.getRouterAddress(params.swapProviderAddress)
    let flashBorrowAmount: bigint
    let swapAmountIn: bigint
    let depositAmount: bigint

    if (marginAsset === 'borrow') {
      flashBorrowAmount = this.decimalToBigInt(
        new Decimal(initialAmount.toString()).mul(leverageMultiplier.minus(1)),
        Decimal.ROUND_DOWN,
      )
      swapAmountIn = initialAmount + flashBorrowAmount
      depositAmount = 0n
    } else {
      const targetDepositFromSwap = this.decimalToBigInt(
        new Decimal(initialAmount.toString()).mul(leverageMultiplier.minus(1)),
        Decimal.ROUND_DOWN,
      )

      if (targetDepositFromSwap <= 0n) {
        throw new Error('Initial amount is too small for the selected multiplier')
      }

      const expectedAmountsIn = await this.getExpectedAmountsIn(routerAddress, targetDepositFromSwap, swapPath)
      const quotedBorrowAmount = expectedAmountsIn[0]
      if (quotedBorrowAmount == null || quotedBorrowAmount <= 0n) {
        throw new Error('Router did not return an input quote for this multiply swap path')
      }

      flashBorrowAmount = quotedBorrowAmount
      swapAmountIn = flashBorrowAmount
      depositAmount = initialAmount
    }

    const expectedAmountsOut = await this.getExpectedAmountsOut(routerAddress, swapAmountIn, swapPath)
    const expectedAmountOut = expectedAmountsOut[expectedAmountsOut.length - 1]
    if (expectedAmountOut == null) {
      throw new Error('Router did not return an output quote for this multiply swap path')
    }

    const slippageMultiplier = new Decimal(1).minus(new Decimal(slippagePercent).div(100))
    const minAmountOut = this.decimalToBigInt(
      new Decimal(expectedAmountOut.toString()).mul(slippageMultiplier),
      Decimal.ROUND_DOWN,
    )

    depositAmount = marginAsset === 'borrow' ? minAmountOut : depositAmount + minAmountOut

    const flashLoanFeeBps = Number(borrowPool.pool.config.fee_config.flash_loan_fee_bps || 0)
    const finalBorrowAmount = this.decimalToBigInt(
      new Decimal(flashBorrowAmount.toString()).mul(
        new Decimal(1).plus(new Decimal(flashLoanFeeBps).div(10_000)),
      ),
      Decimal.ROUND_UP,
    )

    return {
      depositPool,
      borrowPool,
      leverageMultiplier: leverageMultiplier.toNumber(),
      maxLeverageMultiplier,
      marginAsset,
      flashLoanFeeBps,
      swapPath,
      swapProviderAddress: params.swapProviderAddress,
      routerAddress,
      initialAmount,
      flashBorrowAmount,
      swapAmountIn,
      expectedAmountOut,
      minAmountOut,
      depositAmount,
      finalBorrowAmount,
    }
  }

  async buildOpenPositionTx(params: OpenMultiplyParams) {
    const preview = await this.getOpenPositionPreview(params)
    const requests: Request[] = []

    if (preview.marginAsset === 'deposit') {
      requests.push({
        tag: 'Deposit',
        values: [{
          amount: preview.initialAmount,
          pool_address: params.depositPoolAddress,
        }],
      })
    }

    requests.push(
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
        tag: 'Deposit',
        values: [{
          amount: preview.minAmountOut,
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
    )

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

    const currentDepositAmount = this.calculateCurrentDepositAmount(depositPool, depositPosition.j_tokens)
    const currentBorrowAmount = this.calculateCurrentBorrowAmount(borrowPool, borrowPosition.d_tokens)

    if (currentDepositAmount <= 0n) {
      throw new Error('No deposited collateral found for this multiply position')
    }

    if (currentBorrowAmount <= 0n) {
      throw new Error('No active borrow found for this multiply position')
    }

    const repayFeeBps = Number(borrowPool.pool.config.fee_config.repay_fee_bps || 0)
    const fullCloseRepayAmountWithoutBuffer = currentBorrowAmount + this.calculateFee(currentBorrowAmount, repayFeeBps)
    const repayBufferAmount = this.calculateCloseRepayBuffer(fullCloseRepayAmountWithoutBuffer)
    const maxRepayAmount = fullCloseRepayAmountWithoutBuffer + repayBufferAmount

    const requestedRepayAmount = params.repayAmount == null
      ? maxRepayAmount
      : this.amountToBigInt(params.repayAmount, borrowPool.pool.token_decimals)

    if (requestedRepayAmount <= 0n) {
      throw new Error('Repay amount must be greater than 0')
    }

    const isFullClose = requestedRepayAmount >= maxRepayAmount
    const repayAmount = isFullClose ? maxRepayAmount : requestedRepayAmount
    const debtRepaidAmount = isFullClose
      ? currentBorrowAmount
      : this.calculatePartialDebtRepaidAmount(repayAmount, repayFeeBps, currentBorrowAmount)

    const flashLoanFeeBps = Number(borrowPool.pool.config.fee_config.flash_loan_fee_bps || 0)
    const flashBorrowAmount = repayAmount
    const flashRepaymentAmount = flashBorrowAmount + this.calculateFee(flashBorrowAmount, flashLoanFeeBps)

    const swapPath = params.path?.length
      ? params.path
      : [depositPool.pool.token_address, borrowPool.pool.token_address]

    const routerAddress = await this.getRouterAddress(params.swapProviderAddress)
    const expectedAmountsIn = await this.getExpectedAmountsIn(routerAddress, flashRepaymentAmount, swapPath)
    const requiredAmountIn = expectedAmountsIn[0]
    if (requiredAmountIn == null) {
      throw new Error('Router did not return an input quote for this multiply close path')
    }

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
      estimatedReceiveAmount = await this.quoteSwapExactOut(routerAddress, depositReceiveAmount, swapPath)
      maxReceivableAmount = await this.quoteSwapExactOut(routerAddress, maxReceivableDepositAmount, swapPath)
    }

    return {
      depositPool,
      borrowPool,
      marginAsset,
      currentDepositAmount,
      currentBorrowAmount,
      maxRepayAmount,
      repayAmount,
      debtRepaidAmount,
      flashBorrowAmount,
      flashLoanFeeBps,
      flashRepaymentAmount,
      swapPath,
      swapProviderAddress: params.swapProviderAddress,
      routerAddress,
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
        tag: 'Withdraw',
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

  private async getRouterAddress(swapProviderAddress: string): Promise<string> {
    const cached = this.routerCache.get(swapProviderAddress)
    if (cached) {
      return cached
    }

    const providerClient = new SoroswapSwapProviderClient({
      publicKey: this.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId: swapProviderAddress,
      networkPassphrase: this.networkPassphrase,
    })

    const response = await providerClient.get_router()
    const router = String(response.result)

    this.routerCache.set(swapProviderAddress, router)

    return router
  }

  private async getExpectedAmountsOut(routerAddress: string, amountIn: bigint, path: string[]): Promise<bigint[]> {
    const routerClient = new SoroswapRouterClient({
      publicKey: this.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId: routerAddress,
      networkPassphrase: this.networkPassphrase,
    })

    const response = await routerClient.router_get_amounts_out({
      amount_in: amountIn,
      path,
    })

    return this.unwrapOk(response.result)
  }

  private async getExpectedAmountsIn(routerAddress: string, amountOut: bigint, path: string[]): Promise<bigint[]> {
    const routerClient = new SoroswapRouterClient({
      publicKey: this.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId: routerAddress,
      networkPassphrase: this.networkPassphrase,
    })

    const response = await routerClient.router_get_amounts_in({
      amount_out: amountOut,
      path,
    })

    return this.unwrapOk(response.result)
  }

  private async quoteSwapExactOut(routerAddress: string, amountIn: bigint, path: string[]): Promise<bigint> {
    if (amountIn <= 0n) {
      return 0n
    }

    const expectedAmountsOut = await this.getExpectedAmountsOut(routerAddress, amountIn, path)
    return expectedAmountsOut[expectedAmountsOut.length - 1] ?? 0n
  }

  private amountToBigInt(amount: string | number, decimals: number): bigint {
    return this.decimalToBigInt(
      new Decimal(amount).mul(new Decimal(10).pow(decimals)),
      Decimal.ROUND_DOWN,
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
