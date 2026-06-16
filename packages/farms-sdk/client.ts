import type { FarmReward, RPCcluster } from './types'
import { Buffer } from 'node:buffer'
import { Client, Delegatee, FarmState } from '@alula/farms-sdk'
import { BaseClient } from './core/base-client'

export interface FarmsClientConfig {
  publicKey?: string
  farmsContractAddress: string
  opts: FarmsClientOptions
}

export type FarmsClientOptions = {
  rpc: RPCcluster
  horizonRpcUrl?: string
  sorobanRpcUrl?: string
}

export class FarmsClient extends BaseClient {
  protected readonly farmsSdkClient: Client

  public readonly farmsContractAddress: string
  public readonly horizonRpcUrl?: string
  public readonly sorobanRpcUrl?: string
  public farmIds: Buffer<ArrayBufferLike>[] = []

  private constructor(config: FarmsClientConfig) {
    const { opts, publicKey, farmsContractAddress } = config

    super({
      rpc: opts.rpc,
      publicKey,
      contractId: farmsContractAddress,
      horizonRpcUrl: opts.horizonRpcUrl,
      sorobanRpcUrl: opts.sorobanRpcUrl,
    })

    this.farmsContractAddress = farmsContractAddress
    this.horizonRpcUrl = opts.horizonRpcUrl
    this.sorobanRpcUrl = opts.sorobanRpcUrl

    this.farmsSdkClient = new Client({
      publicKey: this.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId: farmsContractAddress,
      networkPassphrase: this.networkPassphrase,
    })
  }

  static async create(config: FarmsClientConfig): Promise<FarmsClient> {
    return new FarmsClient(config)
  }

  static async fromAddress(
    publicKey: string | undefined,
    farmsContractAddress: string,
    opts: FarmsClientOptions = { rpc: 'testnet' },
  ): Promise<FarmsClient> {
    return FarmsClient.create({
      publicKey,
      farmsContractAddress,
      opts,
    })
  }

  async getFarmIds(): Promise<Buffer[]> {
    if (this.farmIds.length === 0) {
      const response = await this.farmsSdkClient.get_all_farms()
      this.farmIds = response.result ?? []
    }

    return this.farmIds
  }

  async getFarm(farmId: Buffer): Promise<FarmState> {
    const response = await this.farmsSdkClient.get_farm({
      farm_id: farmId,
    })

    return this.unwrapOk(response.result)
  }

  async getMarketFarms(): Promise<FarmState[]> {
    const farmIds = await this.getFarmIds()

    return Promise.all(
      farmIds.map(farmId => this.getFarm(farmId)),
    )
  }

  async getUserRewards(publicKey: string): Promise<Array<FarmReward>> {
    const farmIds = await this.getFarmIds()
    const delegatee: Delegatee = {
      owner: publicKey,
      seed: undefined,
    }
    const farms = await Promise.all(
      farmIds.map(async (farm_id) => {
        const response = await this.farmsSdkClient.get_pending_rewards({ farm_id, delegatee })
        return {
          farm_id: farm_id.toString('hex'),
          reward: this.unwrapOk(response.result),
        }
      }),
    )

    return farms.filter((farm): farm is FarmReward => Array.isArray(farm.reward))
  }
}
