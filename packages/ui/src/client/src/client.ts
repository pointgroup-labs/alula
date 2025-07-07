import type { RPCcluster } from '../types'
import { Horizon, rpc as SorobanRpc } from 'stellar-sdk'
import { getRPC } from '../utils'
import { SorobanClient } from './sdk-client'

export class StellarClient {
  private server: Horizon.Server
  private publicKey?: string
  sdk: SorobanClient
  soroban: any

  constructor(address: string, rpc: RPCcluster) {
    this.publicKey = address
    this.server = new Horizon.Server(getRPC(rpc, 'horizon'))
    this.sdk = new SorobanClient(rpc)
    this.soroban = new SorobanRpc.Server(getRPC(rpc, 'soroban'), {
      allowHttp: true,
    })
  }

  static fromAddress(address: string, rpc: RPCcluster) {
    return new this(address, rpc)
  }

  /**
   * Get wallet balances
   */
  async getBalances(): Promise<Horizon.HorizonApi.BalanceLine[] | undefined> {
    if (!this.publicKey) {
      return
    }
    const account = await this.server.loadAccount(this.publicKey)
    return account.balances
  }

  /**
   * Reset client
   */
  reset() {
    this.publicKey = undefined
  }
}
