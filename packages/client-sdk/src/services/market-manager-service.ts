import type { RPCcluster } from '../types'
import { Client } from '@alula/market-manager-sdk'
import { CONTRACT_ID, SOROBAN_CONTRACT_ID } from '../constants'
import { BaseClient } from '../core/base-client'

/**
 * Market manager service configuration
 */
export interface MarketManagerServiceConfig {
  rpc: RPCcluster
  publicKey?: string
  horizonRpcUrl?: string
  sorobanRpcUrl?: string
}

/**
 * Service for market manager operations
 */
export class MarketManagerService extends BaseClient {
  private client: Client

  constructor(config: MarketManagerServiceConfig) {
    super(config)

    const contractId = CONTRACT_ID[config.rpc] ?? SOROBAN_CONTRACT_ID

    this.client = new Client({
      publicKey: config.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId,
      networkPassphrase: this.networkPassphrase,
    })
  }

  /**
   * Get list of all available markets
   */
  async getMarketList() {
    return (await this.client.get_markets()).result
  }

  /**
   * Get markets (alias for getMarketList)
   */
  async getMarkets() {
    return this.getMarketList()
  }
}
