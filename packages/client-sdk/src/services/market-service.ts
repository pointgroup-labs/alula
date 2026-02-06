import type { MultiplyPair, Pool, WithdrawResult } from '@alula/market-sdk'
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
  private client!: Client
  private decimals: DecimalsConfig
  private decimalsReady: Promise<void> = Promise.resolve()

  private constructor(config: MarketServiceConfig) {
    super(config)

    this.client = new Client({
      publicKey: config.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId: config.contractId,
      networkPassphrase: this.networkPassphrase,
    })

    this.decimals = new DecimalsConfig(config.rpc, config.contractId)

    hidePrivate(this, 'client')
    bindOwnMethods(this)
  }

  /**
   * ✅ async factory
   */
  static async create(config: MarketServiceConfig): Promise<MarketService> {
    const service = new MarketService(config)

    service.decimalsReady = service.initializeDecimals()
    await service.decimalsReady

    return service
  }

  private async ensureReady() {
    await this.decimalsReady
  }

  getDecimalsConfig(): DecimalsConfig {
    return this.decimals
  }

  private async initializeDecimals(): Promise<void> {
    await this.decimals.fetchAll(
      async () => (await this.client.get_asset_decimals()).result,
      async () => (await this.client.get_oracle_price_decimals()).result,
    )
  }

  async getMarketData() {
    await this.ensureReady()
    const result = await this.client.get_market_data()
    return this.unwrapOk(result.result)
  }

  async getPoolData(poolAddress: string): Promise<Pool> {
    await this.ensureReady()
    const result = await this.client.get_pool_data({ pool_address: poolAddress })
    return this.unwrapOk(result.result)
  }

  async getPoolAssetOraclePrice(poolAddress: string): Promise<number> {
    await this.ensureReady()

    const result = await this.client.get_pool_asset_oracle_price({
      pool_address: poolAddress,
    })

    const priceInBigInt: bigint = this.unwrapOk(result.result)
    return Number(
      bigintToNumber(priceInBigInt, this.decimals.getOracleDecimals()),
    ) || 0
  }

  async getMultiplyPair(
    depositPoolAddress: string,
    borrowPoolAddress: string,
  ): Promise<MultiplyPair> {
    await this.ensureReady()

    const result = await this.client.get_multiply_pair({
      deposit_pool_address: depositPoolAddress,
      borrow_pool_address: borrowPoolAddress,
    })

    return this.unwrapOk(result.result)
  }

  async simulateWithdraw(
    user: string,
    poolAddress: string,
    amount: string | number,
  ): Promise<WithdrawResult> {
    await this.ensureReady()

    const amountInBigInt = amountToBigInt(
      String(amount),
      this.decimals.getAssetDecimals(),
    )

    const result = await this.client.simulate_withdraw({
      user,
      pool_address: poolAddress,
      amount: amountInBigInt,
      referrer: null,
    })

    return this.unwrapOk(result.result)
  }

  /** @deprecated */
  async getAllPools() {
    await this.ensureReady()
    return (await this.client.get_all_pools()).result
  }

  /** @deprecated */
  async getAllMultiplyPairs() {
    await this.ensureReady()
    return (await this.client.get_all_multiply_pairs()).result
  }

  /** @deprecated */
  async getPoolInfo(poolAddress: string): Promise<Pool> {
    await this.ensureReady()
    const result = await this.client.get_pool({ pool_address: poolAddress })
    return this.unwrapOk(result.result)
  }
}
