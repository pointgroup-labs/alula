import type { MultiplyPair, Obligation } from '@alula/market-sdk'
import type { RPCcluster } from '../types'
import { Client } from '@alula/market-sdk'
import { BaseClient } from '../core/base-client'
import { bindOwnMethods, hidePrivate } from '../utils'

/**
 * Market service configuration
 */
export interface ObligationServiceConfig {
  rpc: RPCcluster
  publicKey?: string
  contractId?: string
}

/**
 * Service for market data operations (pools, prices, obligations)
 */
export class ObligationService extends BaseClient {
  private client: Client

  constructor(config: ObligationServiceConfig) {
    super(config)

    this.client = new Client({
      publicKey: config.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId: config.contractId || '',
      networkPassphrase: this.networkPassphrase,
    })

    hidePrivate(this, 'client')
    bindOwnMethods(this)
  }

  /**
   * Get user obligation
   */
  async getUserObligation(user: string): Promise<Obligation> {
    const result = await this.client.get_user_obligation({ user })
    return this.unwrapOk(result.result)
  }

  /**
   * Get user multiply pair obligation
   */
  async getUserMultiplyObligation(
    user: string,
    depositPoolAddress: string,
    borrowPoolAddress: string,
  ): Promise<Obligation> {
    const result = await this.client.get_multiply_pair_obligation({
      user,
      deposit_pool_address: depositPoolAddress,
      borrow_pool_address: borrowPoolAddress,
    })
    return this.unwrapOk(result.result)
  }
}
