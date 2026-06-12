import type { RPCcluster } from '../types'
import { Buffer } from 'node:buffer'
import { Client, FarmState } from '@alula/farms-sdk'
import { BaseClient } from '../core/base-client'
import { MarketService } from './market'

/**
 * Market manager service configuration
 */
export interface FarmsServiceConfig {
  rpc: RPCcluster
  publicKey?: string
  horizonRpcUrl?: string
  sorobanRpcUrl?: string
  contractId?: string
}

/**
 * Service for market manager operations
 */
export class FarmsService extends BaseClient {
  private client?: Client

  constructor(
    config: FarmsServiceConfig,
    private market: MarketService,
  ) {
    super(config)
  }

  private async getClient() {
    if (!this.client) {
      const contractId = await this.market.getFarmsContractAddress()

      if (!contractId) {
        throw new Error('Farms contract not found')
      }

      this.client = new Client({
        publicKey: this.publicKey,
        rpcUrl: this.getSorobanRpcUrl(),
        contractId,
        networkPassphrase: this.networkPassphrase,
      })
    }

    return this.client
  }

  async getMarketFarms(): Promise<FarmState[]> {
    const farmsAddresses = await this.getAllFarms()
    const farms = []
    for (const farm_id of farmsAddresses) {
      const farm = await this.getFarm(farm_id)
      farms.push(farm)
    }
    return farms
  }

  async getAllFarms(): Promise<Buffer[]> {
    const client = await this.getClient()
    const farmsAddresses = await client.get_all_farms()
    return farmsAddresses.result
  }

  async getFarm(farm_id: Buffer): Promise<FarmState> {
    const client = await this.getClient()
    const farm = await client.get_farm({ farm_id })
    return this.unwrapOk(farm.result)
  }
}
