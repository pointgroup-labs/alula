import type { RPCcluster } from '../types'
import { Client } from '@alula/market-sdk'
import { DecimalsConfig } from '../config/decimals'
import { MAX_I128 } from '../constants'
import { BaseClient } from '../core/base-client'
import { TransactionHelper } from '../core/transaction-builder'
import { amountToBigInt } from '../utils'

/**
 * Borrowing service configuration
 */
export interface BorrowingServiceConfig {
  rpc: RPCcluster
  publicKey?: string
  contractId?: string
  decimals: DecimalsConfig
}

/**
 * Service for borrowing operations (borrow, repay, collateral)
 */
export class BorrowingService extends BaseClient {
  private client: Client
  private txHelper: TransactionHelper
  private decimals: DecimalsConfig

  constructor(config: BorrowingServiceConfig) {
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
   * Build borrow transaction
   */
  async buildBorrowTx(user: string, poolAddress: string, amount: string | number | bigint) {
    const amountInBigInt = typeof amount === 'bigint' ? amount : amountToBigInt(String(amount), this.decimals.assetDecimals)
    return await this.client.borrow({
      user,
      pool_address: poolAddress,
      amount: amountInBigInt,
      referrer: null,
    })
  }

  /**
   * Build repay transaction
   */
  async buildRepayTx(user: string, poolAddress: string, amount: string | number) {
    const amountInBigInt = amountToBigInt(String(amount), this.decimals.assetDecimals)
    return await this.client.repay({
      user,
      pool_address: poolAddress,
      amount: amountInBigInt,
      referrer: null,
    })
  }

  /**
   * Borrow from pool
   */
  async borrow(user: string, poolAddress: string, amount: number, kit: any, withBuffer: boolean, options = { debug: true }) {
    const resolvedAmount = withBuffer ? MAX_I128 : amount
    const tx = await this.buildBorrowTx(user, poolAddress, resolvedAmount)

    if (options?.debug) {
      console.log('%c[Borrow Tx]', 'color: #00ff00', tx)
    }

    return await this.txHelper.signAndSend(tx, user, kit, options)
  }

  /**
   * Repay borrowed amount
   */
  async repay(user: string, poolAddress: string, amount: number, kit: any, options = { debug: true }) {
    const tx = await this.buildRepayTx(user, poolAddress, amount)

    if (options?.debug) {
      console.log('%c[Repay Tx]', 'color: #00ff00', tx)
    }

    return await this.txHelper.signAndSend(tx, user, kit, options)
  }

  /**
   * Get transaction fee
   */
  getTransactionFee(tx: any): number {
    return this.txHelper.getTransactionFee(tx, this.decimals.assetDecimals)
  }
}
