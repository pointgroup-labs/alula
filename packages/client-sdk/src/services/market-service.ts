import type { MultiplyPair, Pool, WithdrawResult } from '@alula/market-sdk'
import type { RPCcluster } from '../types'
import { Client } from '@alula/market-sdk'
import { DecimalsConfig, loadMarketDecimals } from '../config/decimals'
import { BaseClient } from '../core/base-client'
import { amountToBigInt, bigintToNumber, getNetworkPassphrase, getRPC } from '../utils'

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
  decimals: DecimalsConfig

  private constructor(
    config: MarketServiceConfig,
    client: Client,
    decimals: DecimalsConfig) {
    super(config)

    this.client = client

    this.decimals = decimals

    // console.log('%cdecimals', 'color: #00ff00', this.decimals)
  }

  /**
   * ✅ async factory
   */
  static async create(config: MarketServiceConfig): Promise<MarketService> {
    const client = new Client({
      publicKey: config.publicKey,
      rpcUrl: getRPC(config.rpc, 'soroban'),
      contractId: config.contractId,
      networkPassphrase: getNetworkPassphrase(config.rpc),
    })

    const decimals = await loadMarketDecimals(client, config.contractId)

    return new MarketService(config, client, decimals)
  }

  async getMarketData() {
    const result = await this.client.get_market_data()
    return this.unwrapOk(result.result)
  }

  async getPoolData(poolAddress: string): Promise<Pool> {
    const result = await this.client.get_pool_data({ pool_address: poolAddress })
    return this.unwrapOk(result.result)
  }

  async getPoolAssetOraclePrice(poolAddress: string): Promise<number> {
    const result = await this.client.get_pool_asset_oracle_price({
      pool_address: poolAddress,
    })

    const priceInBigInt: bigint = this.unwrapOk(result.result)
    return Number(
      bigintToNumber(priceInBigInt, this.decimals.oracleDecimals),
    ) || 0
  }

  async getAllPools() {
    return (await this.client.get_all_pools()).result
  }

  async getAllMultiplyPairs() {
    return (await this.client.get_all_multiply_pairs()).result
  }

  async getPoolInfo(poolAddress: string): Promise<Pool> {
    const result = await this.client.get_pool({ pool_address: poolAddress })
    return this.unwrapOk(result.result)
  }

  async getMultiplyPair(
    depositPoolAddress: string,
    borrowPoolAddress: string,
  ): Promise<MultiplyPair> {
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
    const amountInBigInt = amountToBigInt(
      String(amount),
      this.decimals.assetDecimals,
    )

    const result = await this.client.simulate_withdraw({
      user,
      pool_address: poolAddress,
      amount: amountInBigInt,
      referrer: null,
    })

    return this.unwrapOk(result.result)
  }
}
