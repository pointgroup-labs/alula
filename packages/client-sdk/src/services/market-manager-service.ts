import type { RPCcluster } from '../types'
import { Client } from '@alula/market-manager-sdk'
import { BaseClient } from '../core/base-client'
import { CONTRACT_ID, SOROBAN_CONTRACT_ID } from '../constants'
import { bindOwnMethods, hidePrivate } from '../utils'

/**
 * Market manager service configuration
 */
export interface MarketManagerServiceConfig {
  rpc: RPCcluster
  publicKey?: string
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

    hidePrivate(this, 'client')
    bindOwnMethods(this)
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
