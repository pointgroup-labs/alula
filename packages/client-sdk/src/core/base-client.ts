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
  horizonRpcUrl?: string
  sorobanRpcUrl?: string
}

/**
 * Base class for all services that interact with Soroban contracts
 */
export abstract class BaseClient {
  protected rpc: RPCcluster
  protected publicKey?: string
  protected contractId?: string
  protected horizonRpcUrl?: string
  protected sorobanRpcUrl?: string
  protected sorobanServer: SorobanRpc.Server
  protected networkPassphrase: string

  constructor(config: BaseClientConfig) {
    this.rpc = config.rpc
    this.publicKey = config.publicKey
    this.contractId = config.contractId
    this.horizonRpcUrl = config.horizonRpcUrl
    this.sorobanRpcUrl = config.sorobanRpcUrl
    this.sorobanServer = new SorobanRpc.Server(this.getSorobanRpcUrl())
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
    return this.sorobanRpcUrl || getRPC(this.rpc, 'soroban')
  }

  /**
   * Get Horizon RPC URL
   */
  protected getHorizonRpcUrl(): string {
    return this.horizonRpcUrl || getRPC(this.rpc, 'horizon')
  }
}
