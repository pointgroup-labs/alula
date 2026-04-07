import type { RPCcluster } from '../types'
import { Client, ObligationKey } from '@alula/market-sdk'
import { DecimalsConfig } from '../config/decimals'
import { MAX_I128 } from '../constants'
import { BaseClient } from '../core/base-client'
import { TransactionHelper } from '../core/transaction-builder'
import { amountToBigInt } from '../utils'

/**
 * Lending service configuration
 */
export interface LendingServiceConfig {
  rpc: RPCcluster
  publicKey?: string
  contractId?: string
  horizonRpcUrl?: string
  sorobanRpcUrl?: string
  decimals: DecimalsConfig
}

/**
 * Service for lending operations (deposit, withdraw)
 */
export class LendingService extends BaseClient {
  private client: Client
  private txHelper: TransactionHelper
  private decimals: DecimalsConfig

  constructor(config: LendingServiceConfig) {
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
   * Build deposit transaction
   */
  async buildDepositTx(user: ObligationKey, poolAddress: string, amount: string | number, assetDecimals: number) {
    const amountInBigInt = amountToBigInt(String(amount), assetDecimals)
    return await this.client.deposit({
      user,
      pool_address: poolAddress,
      amount: amountInBigInt,
      referrer: null,
    })
  }

  /**
   * Build withdraw transaction
   */
  async buildWithdrawTx(user: ObligationKey, poolAddress: string, amount: string | number | bigint, assetDecimals: number) {
    const amountInBigInt = typeof amount === 'bigint' ? amount : amountToBigInt(String(amount), assetDecimals)
    return await this.client.withdraw({
      user,
      pool_address: poolAddress,
      amount: amountInBigInt,
      referrer: null,
    })
  }

  /**
   * Build add collateral transaction
   */
  async buildAddCollateralTx(user: ObligationKey, poolAddress: string, amount: string | number, assetDecimals: number) {
    const amountInBigInt = amountToBigInt(String(amount), assetDecimals)
    return await this.client.add_collateral({
      user,
      pool_address: poolAddress,
      amount: amountInBigInt,
      referrer: null,
    })
  }

  /**
   * Build remove collateral transaction
   */
  async buildRemoveCollateralTx(user: ObligationKey, poolAddress: string, amount: string | number | bigint, assetDecimals: number) {
    const amountInBigInt = typeof amount === 'bigint' ? amount : amountToBigInt(String(amount), assetDecimals)
    return await this.client.remove_collateral({
      user,
      pool_address: poolAddress,
      amount: amountInBigInt,
      referrer: null,
    })
  }

  /**
   * Deposit to lending pool
   */
  async deposit(user: ObligationKey, poolAddress: string, amount: number, assetDecimals: number, kit: any, options = { debug: true }) {
    const tx = await this.buildDepositTx(user, poolAddress, amount, assetDecimals)

    if (options?.debug) {
      console.log('%c[Deposit Tx]', 'color: #00ff00', tx)
    }

    return await this.txHelper.signAndSend(tx, user, kit, options)
  }

  /**
   * Add collateral to pool
   */
  async addCollateral(user: ObligationKey, poolAddress: string, amount: number, assetDecimals: number, kit: any, options = { debug: true }) {
    const tx = await this.buildAddCollateralTx(user, poolAddress, amount, assetDecimals)

    if (options?.debug) {
      console.log('%c[Add Collateral Tx]', 'color: #00ff00', tx)
    }

    return await this.txHelper.signAndSend(tx, user, kit, options)
  }

  /**
   * Remove collateral from pool
   */
  async removeCollateral(user: ObligationKey, poolAddress: string, amount: number, assetDecimals: number, kit: any, withBuffer: boolean, options = { debug: true }) {
    const resolvedAmount = withBuffer ? MAX_I128 : amount
    const tx = await this.buildRemoveCollateralTx(user, poolAddress, resolvedAmount, assetDecimals)

    if (options?.debug) {
      console.log('%c[Remove Collateral Tx]', 'color: #00ff00', tx)
    }

    return await this.txHelper.signAndSend(tx, user, kit, options)
  }

  /**
   * Withdraw from lending pool
   */
  async withdraw(user: ObligationKey, poolAddress: string, amount: number, assetDecimals: number, kit: any, withBuffer: boolean, options = { debug: true }) {
    const resolvedAmount = withBuffer ? MAX_I128 : amount
    const tx = await this.buildWithdrawTx(user, poolAddress, resolvedAmount, assetDecimals)

    if (options?.debug) {
      console.log('%c[Withdraw Tx]', 'color: #00ff00', tx)
    }

    return await this.txHelper.signAndSend(tx, user, kit, options)
  }

  /**
   * Get transaction fee
   */
  getTransactionFee(tx: any, assetDecimals: number): number {
    return this.txHelper.getTransactionFee(tx, assetDecimals)
  }
}
