import type { RPCcluster } from '../types'
import { rpc as SorobanRpc } from '@stellar/stellar-sdk'
import { getNetworkPassphrase, getRPC } from '../utils'

/**
 * Base configuration for all services
 */
export interface BaseClientConfig {
  rpc: RPCcluster
  publicKey?: string
  contractId?: string
}

/**
 * Base class for all services that interact with Soroban contracts
 */
export abstract class BaseClient {
  protected rpc: RPCcluster
  protected publicKey?: string
  protected contractId?: string
  protected sorobanServer: SorobanRpc.Server
  protected networkPassphrase: string

  constructor(config: BaseClientConfig) {
    this.rpc = config.rpc
    this.publicKey = config.publicKey
    this.contractId = config.contractId
    this.sorobanServer = new SorobanRpc.Server(getRPC(config.rpc, 'soroban'))
    this.networkPassphrase = getNetworkPassphrase(config.rpc)
  }

  /**
   * Unwrap Ok result from Soroban SDK
   */
  protected unwrapOk<T>(result: any): T {
    return result.value
  }

  /**
   * Get Soroban RPC URL
   */
  protected getSorobanRpcUrl(): string {
    return getRPC(this.rpc, 'soroban')
  }

  /**
   * Get Horizon RPC URL
   */
  protected getHorizonRpcUrl(): string {
    return getRPC(this.rpc, 'horizon')
  }
}
