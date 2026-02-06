import type { MultiplyPair, Obligation, Pool, WithdrawResult } from '@alula/market-sdk'
import type { RPCcluster } from '../types'
import { Client } from '@alula/market-sdk'
import { DecimalsConfig } from '../config/decimals'
import { BaseClient } from '../core/base-client'
import { amountToBigInt, bigintToNumber, bindOwnMethods, hidePrivate } from '../utils'

/**
 * Market service configuration
 */
export interface MarketServiceConfig {
  rpc: RPCcluster
  publicKey?: string
  contractId?: string
}

/**
 * Service for market data operations (pools, prices, obligations)
 */
export class MarketService extends BaseClient {
  private client: Client
  private decimals: DecimalsConfig

  constructor(config: MarketServiceConfig) {
    super(config)

    this.client = new Client({
      publicKey: config.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId: config.contractId || '',
      networkPassphrase: this.networkPassphrase,
    })

    this.decimals = new DecimalsConfig(config.rpc, config.contractId)

    // Initialize decimals if contract is provided
    if (config.contractId) {
      this.initializeDecimals().catch(() => {})
    }

    hidePrivate(this, 'client')
    bindOwnMethods(this)
  }

  /**
   * Get decimals config
   */
  getDecimalsConfig(): DecimalsConfig {
    return this.decimals
  }

  /**
   * Initialize decimals from contract
   */
  private async initializeDecimals(): Promise<void> {
    await this.decimals.fetchAll(
      async () => (await this.client.get_asset_decimals()).result,
      async () => (await this.client.get_oracle_price_decimals()).result,
    )
  }

  /**
   * Get market data
   */
  async getMarketData() {
    const result = await this.client.get_market_data()
    return this.unwrapOk(result.result)
  }

  /**
   * Get pool data
   */
  async getPoolData(poolAddress: string): Promise<Pool> {
    const result = await this.client.get_pool_data({ pool_address: poolAddress })
    return this.unwrapOk(result.result)
  }

  /**
   * Get pool asset oracle price
   */
  async getPoolAssetOraclePrice(poolAddress: string): Promise<number> {
    const result = await this.client.get_pool_asset_oracle_price({ pool_address: poolAddress })
    const priceInBigInt: bigint = this.unwrapOk(result.result)
    const normalizedPrice = bigintToNumber(priceInBigInt, this.decimals.getOracleDecimals())
    return Number(normalizedPrice) || 0
  }

  /**
   * Get user obligation
   */
  async getUserObligation(user: string): Promise<Obligation> {
    const result = await this.client.get_user_obligation({ user })
    return this.unwrapOk(result.result)
  }

  /**
   * Get multiply pair (leverage pair)
   */
  async getMultiplyPair(depositPoolAddress: string, borrowPoolAddress: string): Promise<MultiplyPair> {
    const result = await this.client.get_multiply_pair({
      deposit_pool_address: depositPoolAddress,
      borrow_pool_address: borrowPoolAddress,
    })
    return this.unwrapOk(result.result)
  }

  /**
   * Get user multiply pair obligation
   */
  async getUserMultiplyObligation(
    user: string,
    depositPoolAddress: string,
    borrowPoolAddress: string,
  ): Promise<Obligation> {
    const result = await this.client.get_multiply_pair_obligation({
      user,
      deposit_pool_address: depositPoolAddress,
      borrow_pool_address: borrowPoolAddress,
    })
    return this.unwrapOk(result.result)
  }

  /**
   * Simulate withdraw operation
   */
  async simulateWithdraw(user: string, poolAddress: string, amount: string | number): Promise<WithdrawResult> {
    const amountInBigInt = amountToBigInt(String(amount), this.decimals.getAssetDecimals())
    const result = await this.client.simulate_withdraw({
      user,
      pool_address: poolAddress,
      amount: amountInBigInt,
      referrer: null,
    })
    return this.unwrapOk(result.result)
  }

  /**
   * Get all pools
   * @deprecated This method will be removed in future versions
   */
  async getAllPools() {
    return (await this.client.get_all_pools()).result
  }

  /**
   * Get all multiply pairs (leverage pools)
   * @deprecated This method will be removed in future versions
   */
  async getAllMultiplyPairs() {
    return (await this.client.get_all_multiply_pairs()).result
  }

  /**
   * Get pool info
   * @deprecated Use getPoolData instead
   */
  async getPoolInfo(poolAddress: string): Promise<Pool> {
    const result = await this.client.get_pool({ pool_address: poolAddress })
    return this.unwrapOk(result.result)
  }
}
