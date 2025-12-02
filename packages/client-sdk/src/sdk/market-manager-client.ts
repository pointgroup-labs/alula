import type { RPCcluster } from '../types'
import { Client } from '@alula/market-manager-sdk'
import { CONTRACT_ID, SOROBAN_CONTRACT_ID } from '../constants'
import { bindOwnMethods, getNetworkPassphrase, getRPC, hidePrivate } from '../utils'

export class MarketManagerClient {
  private base: Client

  constructor(rpc: RPCcluster, publicKey?: string) {
    this.base = new Client({
      publicKey,
      rpcUrl: getRPC(rpc, 'soroban'),
      contractId: CONTRACT_ID[rpc] ?? SOROBAN_CONTRACT_ID,
      networkPassphrase: getNetworkPassphrase(rpc),
    })
    hidePrivate(this, 'base')
    bindOwnMethods(this)
  }

  async getMarketList() {
    return (await this.base.get_markets()).result
  }
}
