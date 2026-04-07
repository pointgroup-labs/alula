import type { RPCcluster } from '../types'
import { Client, ObligationKey } from '@alula/market-sdk'
import { DecimalsConfig } from '../config/decimals'
import { BaseClient } from '../core/base-client'
import { TransactionHelper } from '../core/transaction-builder'
// import { amountToBigInt } from '../utils'

/**
 * Leverage service configuration
 */
export interface LeverageServiceConfig {
  rpc: RPCcluster
  publicKey?: string
  contractId?: string
  decimals: DecimalsConfig
}

/**
 * Leverage operation parameters
 */
export interface LeverageParams {
  user: ObligationKey
  depositPoolAddress: string
  borrowPoolAddress: string
  depositAsMargin: boolean
  amount: number
  leverageMultiplier: number
}

/**
 * Withdraw leverage operation parameters
 */
export interface WithdrawLeverageParams {
  user: ObligationKey
  depositPoolAddress: string
  borrowPoolAddress: string
  amount: number
}

/**
 * Service for leverage operations (multiply pairs)
 */
export class LeverageService extends BaseClient {
  private client: Client
  private txHelper: TransactionHelper
  private decimals: DecimalsConfig

  constructor(config: LeverageServiceConfig) {
    super(config)

    this.client = new Client({
      publicKey: config.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId: config.contractId || '',
      networkPassphrase: this.networkPassphrase,
    })

    this.txHelper = new TransactionHelper(config.rpc, this.sorobanServer)
    this.decimals = config.decimals
  }

  /**
   * Build leverage deposit transaction
   */
  // async buildLeverageTx(params: LeverageParams) {
  //   const multiplier = Number(params.leverageMultiplier * 100).toFixed(0)
  //   const amountInBigInt = amountToBigInt(String(params.amount), this.decimals.assetDecimals)

  //   return await this.client.deposit_with_leverage({
  //     user: params.user,
  //     deposit_pool_address: params.depositPoolAddress,
  //     borrow_pool_address: params.borrowPoolAddress,
  //     deposit_as_margin: params.depositAsMargin,
  //     amount: amountInBigInt,
  //     leverage_multiplier: Number(multiplier),
  //     referrer: null,
  //   })
  // }

  /**
   * Build withdraw leverage transaction
   */
  // async buildWithdrawLeverageTx(params: WithdrawLeverageParams) {
  //   const amountInBigInt = amountToBigInt(String(params.amount), this.decimals.assetDecimals)

  //   return await this.client.withdraw_from_leveraged({
  //     user: params.user,
  //     deposit_pool_address: params.depositPoolAddress,
  //     borrow_pool_address: params.borrowPoolAddress,
  //     amount: amountInBigInt,
  //     referrer: null,
  //   })
  // }

  /**
   * Open leveraged position
   */
  // async openPosition(params: LeverageParams, kit: any, options = { debug: true }) {
  //   if (options?.debug) {
  //     console.log('%c[Leverage]', 'color: #00ff00', params)
  //   }

  //   const tx = await this.buildLeverageTx(params)

  //   if (options?.debug) {
  //     console.log('%c[Leverage Tx]', 'color: #00ff00', tx)
  //   }

  //   return await this.txHelper.signAndSend(tx, params.user, kit, options)
  // }

  /**
   * Close leveraged position
   */
  // async closePosition(params: WithdrawLeverageParams, kit: any, options = { debug: true }) {
  //   const tx = await this.buildWithdrawLeverageTx(params)

  //   if (options?.debug) {
  //     console.log('%c[Withdraw Leverage Tx]', 'color: #00ff00', tx)
  //   }

  //   return await this.txHelper.signAndSend(tx, params.user, kit, options)
  // }

  /**
   * Deposit with leverage (alias for openPosition)
   */
  // async depositWithLeverage(params: LeverageParams, kit: any, options = { debug: true }) {
  //   return this.openPosition(params, kit, options)
  // }

  /**
   * Withdraw from leveraged position (alias for closePosition)
   */
  // async withdrawFromLeveraged(params: WithdrawLeverageParams, kit: any, options = { debug: true }) {
  //   return this.closePosition(params, kit, options)
  // }

  /**
   * Get transaction fee
   */
  // getTransactionFee(tx: any): number {
  //   return this.txHelper.getTransactionFee(tx, this.decimals.assetDecimals)
  // }
}
