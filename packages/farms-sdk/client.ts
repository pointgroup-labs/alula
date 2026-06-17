import type { FarmReward, RPCcluster } from './types'
import { Buffer } from 'node:buffer'
import { Client, Delegatee, FarmState } from '@alula/farms-sdk'
import { TransactionHelper } from './core'
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
  private txHelper: TransactionHelper
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

    this.txHelper = new TransactionHelper(opts.rpc, this.sorobanServer)
  }

  static async create(config: FarmsClientConfig): Promise<FarmsClient> {
    return new FarmsClient(config)
  }

  /**
   * Build claim transaction
   */
  async buildClaimTx(publicKey: string, farmId: string, rewardIndex: number) {
    const delegatee = this.generateDelegatee(publicKey)
    const farm_id = Buffer.from(farmId, 'hex')
    const reward_index = rewardIndex
    return await this.farmsSdkClient.harvest({
      farm_id,
      delegatee,
      reward_index,
    })
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
    const delegatee = this.generateDelegatee(publicKey)
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

  async claimRewards(publicKey: string, farm_id: string, reward_index: number, kit: any, options = { debug: true }) {
    const tx = await this.buildClaimTx(publicKey, farm_id, reward_index)

    if (options?.debug) {
      console.log('%c[Claim Tx]', 'color: #00ff00', tx)
    }

     return await this.txHelper.signAndSend(tx, publicKey, kit, options)
  }

  generateDelegatee(publicKey: string): Delegatee {
    return {
      owner: publicKey,
      seed: undefined,
    }
  }
}
