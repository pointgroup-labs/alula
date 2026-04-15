import type { /* MultiplyPair, */ ObligationKey, Pool, WithdrawResult } from '@alula/market-sdk'
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
  horizonRpcUrl?: string
  sorobanRpcUrl?: string
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
  }

  /**
   * ✅ async factory
   */
  static async create(config: MarketServiceConfig): Promise<MarketService> {
    const client = new Client({
      publicKey: config.publicKey,
      rpcUrl: config.sorobanRpcUrl || getRPC(config.rpc, 'soroban'),
      contractId: config.contractId,
      networkPassphrase: getNetworkPassphrase(config.rpc),
    })

    const decimals = await loadMarketDecimals(client, config.contractId)

    return new MarketService(config, client, decimals)
  }

  /**
   * Fetches market data from the contract
   *
   * @return A Promise of type MarketData containing the current market data
   */
  async getMarketData() {
    const result = await this.client.get_market_data()
    return this.unwrapOk(result.result)
  }

  /**
   * Fetches pool data from the contract
   *
   * @param poolAddress - pool address
   * @return A Promise of type Pool containing the current pool data
   */
  async getPoolData(poolAddress: string): Promise<Pool> {
    const result = await this.client.get_pool_data({ pool_address: poolAddress })
    return this.unwrapOk(result.result)
  }

  /**
   * Fetches the current asset price from the oracle contract for a given pool.
   *
   * @param poolAddress - pool address
   * @return A Promise of type number containing the current asset price from the oracle contract
   */
  async getPoolAssetOraclePrice(poolAddress: string): Promise<number> {
    const result = await this.client.get_pool_asset_oracle_price({
      pool_address: poolAddress,
    })

    const priceInBigInt: bigint = this.unwrapOk(result.result)
    return Number(
      bigintToNumber(priceInBigInt, this.decimals.oracleDecimals),
    ) || 0
  }

  /**
   * Retrieves all pools registered on the market contract
   *
   * @return A Promise of type Array<Pool> containing all registered pools
   */
  async getAllPools() {
    return (await this.client.get_all_pools()).result
  }

  /**
   * Retrieves all multiply pairs registered on the market contract
   *
   * @return A Promise of type Array<MultiplyPair> containing all registered multiply pairs
   */
  // async getAllMultiplyPairs() {
  //   return (await this.client.get_all_multiply_pairs()).result
  // }

  /**
   * Retrieves pool data from the contract for a given pool address.
   *
   * @param poolAddress - pool address
   * @return A Promise of type Pool containing the current pool data
   */
  async getPoolInfo(poolAddress: string): Promise<Pool> {
    const result = await this.client.get_pool({ pool_address: poolAddress })
    return this.unwrapOk(result.result)
  }

  /**
   * Retrieves a multiply pair configuration by its deposit and borrow pool addresses
   *
   * @param depositPoolAddress - deposit pool address
   * @param borrowPoolAddress - borrow pool address
   * @return A Promise of type MultiplyPair containing the multiply pair configuration
   */
  // async getMultiplyPair(
  //   depositPoolAddress: string,
  //   borrowPoolAddress: string,
  // ): Promise<MultiplyPair> {
  //   const result = await this.client.get_multiply_pair({
  //     deposit_pool_address: depositPoolAddress,
  //     borrow_pool_address: borrowPoolAddress,
  //   })

  //   return this.unwrapOk(result.result)
  // }

  /**
   * Simulates a withdrawal from a lending pool
   *
   * @param user - user address
   * @param poolAddress - pool address
   * @param amount - amount to withdraw (string or number)
   * @return A Promise of type WithdrawResult containing the simulated withdrawal result
   */
  async simulateWithdraw(
    user: ObligationKey,
    poolAddress: string,
    amount: string | number,
    assetDecimals: number,
  ): Promise<WithdrawResult> {
    const amountInBigInt = amountToBigInt(
      String(amount),
      assetDecimals,
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
