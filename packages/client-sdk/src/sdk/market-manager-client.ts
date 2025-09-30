import type { RPCcluster } from '../types'
import { Client } from '@alula/market-manager-sdk'
import { CONTRACT_ID, SOROBAN_CONTRACT_ID } from '../constants'
import { getNetworkPassphrase, getRPC } from '../utils'

export class MarketManagerClient extends Client {
  constructor(rpc: RPCcluster, publicKey?: string) {
    super({
      publicKey,
      rpcUrl: getRPC(rpc, 'soroban'),
      contractId: CONTRACT_ID[rpc] ?? SOROBAN_CONTRACT_ID,
      networkPassphrase: getNetworkPassphrase(rpc),
    })
  }

  async getMarketList() {
    return (await this.get_market_list()).result
  }
}
