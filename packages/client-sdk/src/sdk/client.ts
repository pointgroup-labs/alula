import type { RPCcluster } from '../types'
import { Account, Asset, BASE_FEE, Horizon, Operation, TransactionBuilder } from '@stellar/stellar-sdk'
import { getNetworkPassphrase, getRPC } from '../utils'
import { MarketClient } from './market-client'
import { MarketManagerClient } from './market-manager-client'

export class StellarClient {
  server: Horizon.Server
  publicKey?: string
  marketManagerSdk: MarketManagerClient
  marketSdk: MarketClient

  constructor(publicKey: string, rpc: RPCcluster, market?: string) {
    this.publicKey = publicKey
    this.server = new Horizon.Server(getRPC(rpc, 'horizon')!)
    this.marketManagerSdk = new MarketManagerClient(rpc, publicKey)
    this.marketSdk = new MarketClient(rpc, publicKey, market)
  }

  static fromAddress(address: string, rpc: RPCcluster, market?: string) {
    return new this(address, rpc, market)
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
   * Add trust line
   */
  async addTrustlineTx(
    publicKey: string,
    assetCode: string,
    assetIssuer: string,
    kit: any) {
    const networkPassphrase = getNetworkPassphrase(this.marketSdk.rpc)
    const accountResponse = await this.server.loadAccount(publicKey)
    const account = new Account(accountResponse.accountId(), accountResponse.sequence.toString())

    const asset = new Asset(assetCode, assetIssuer)

    const transaction = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase,
    })
      .addOperation(Operation.changeTrust({
        asset,
      }))
      .setTimeout(30)
      .build()

    console.log('[TX]', transaction)

    const { signedTxXdr } = await kit.signTransaction(transaction.toXDR(), {
      address: publicKey,
      networkPassphrase,
    })

    console.log('[signedTxXdr]', signedTxXdr)

    const signedTransaction = TransactionBuilder.fromXDR(signedTxXdr, networkPassphrase)
    const result = await this.server.submitTransaction(signedTransaction)

    console.log('✅ Transaction submitted!', result)
    return result
  }

  /**
   * Reset client
   */
  reset() {
    this.publicKey = undefined
  }
}
