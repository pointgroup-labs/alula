import type { RPCcluster } from '../types'
import { Client } from '@alula/market-sdk'
import { DecimalsConfig } from '../config/decimals'
import { BaseClient } from '../core/base-client'
import { TransactionHelper } from '../core/transaction-builder'
import { amountToBigInt, bindOwnMethods, hidePrivate } from '../utils'

/**
 * Lending service configuration
 */
export interface LendingServiceConfig {
  rpc: RPCcluster
  publicKey?: string
  contractId?: string
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

    hidePrivate(this, 'client')
    bindOwnMethods(this)
  }

  /**
   * Build deposit transaction
   */
  async buildDepositTx(user: string, poolAddress: string, amount: string | number) {
    const amountInBigInt = amountToBigInt(String(amount), this.decimals.getAssetDecimals())
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
  async buildWithdrawTx(user: string, poolAddress: string, amount: string | number) {
    const amountInBigInt = amountToBigInt(String(amount), this.decimals.getAssetDecimals())
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
  async buildAddCollateralTx(user: string, poolAddress: string, amount: string | number) {
    const amountInBigInt = amountToBigInt(String(amount), this.decimals.getAssetDecimals())
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
  async buildRemoveCollateralTx(user: string, poolAddress: string, amount: string | number) {
    const amountInBigInt = amountToBigInt(String(amount), this.decimals.getAssetDecimals())
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
  async deposit(user: string, poolAddress: string, amount: number, kit: any, options?: { debug?: boolean }) {
    const tx = await this.buildDepositTx(user, poolAddress, amount)

    if (options?.debug) {
      console.log('%c[Deposit Tx]', 'color: #00ff00', tx)
    }

    return await this.txHelper.signAndSend(tx, user, kit, options)
  }

  /**
   * Add collateral to pool
   */
  async addCollateral(user: string, poolAddress: string, amount: number, kit: any, options?: { debug?: boolean }) {
    const tx = await this.buildAddCollateralTx(user, poolAddress, amount)

    if (options?.debug) {
      console.log('%c[Add Collateral Tx]', 'color: #00ff00', tx)
    }

    return await this.txHelper.signAndSend(tx, user, kit, options)
  }

  /**
   * Remove collateral from pool
   */
  async removeCollateral(user: string, poolAddress: string, amount: number, kit: any, options?: { debug?: boolean }) {
    const tx = await this.buildRemoveCollateralTx(user, poolAddress, amount)

    if (options?.debug) {
      console.log('%c[Remove Collateral Tx]', 'color: #00ff00', tx)
    }

    return await this.txHelper.signAndSend(tx, user, kit, options)
  }

  /**
   * Withdraw from lending pool
   */
  async withdraw(user: string, poolAddress: string, amount: number, kit: any, options?: { debug?: boolean }) {
    const tx = await this.buildWithdrawTx(user, poolAddress, amount)

    if (options?.debug) {
      console.log('%c[Withdraw Tx]', 'color: #00ff00', tx)
    }

    return await this.txHelper.signAndSend(tx, user, kit, options)
  }

  /**
   * Get transaction fee
   */
  getTransactionFee(tx: any): number {
    return this.txHelper.getTransactionFee(tx, this.decimals.getAssetDecimals())
  }
}
