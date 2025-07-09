import type { StellarWalletsKit } from '@creit.tech/stellar-wallets-kit'
import type { RPCcluster } from '../types'
import { WalletNetwork } from '@creit.tech/stellar-wallets-kit'
import { Account, Asset, BASE_FEE, Horizon, Networks, Operation, TransactionBuilder } from 'stellar-sdk'
import { getRPC } from '../utils'
import { SorobanClient } from './sdk-client'

export class StellarClient {
  private server: Horizon.Server
  private publicKey?: string
  sdk: SorobanClient

  constructor(address: string, rpc: RPCcluster) {
    this.publicKey = address
    this.server = new Horizon.Server(getRPC(rpc, 'horizon'))
    this.sdk = new SorobanClient(rpc, address)
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
   * Add trust line
   */
  async addTrustlineTx(
    publicKey: string,
    assetCode: string,
    assetIssuer: string,
    kit: StellarWalletsKit) {
    const accountResponse = await this.server.loadAccount(publicKey)
    const account = new Account(accountResponse.accountId(), accountResponse.sequence.toString())

    const asset = new Asset(assetCode, assetIssuer)

    const transaction = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: Networks.TESTNET,
    })
      .addOperation(Operation.changeTrust({
        asset,
      }))
      .setTimeout(30)
      .build()

    console.log('[TX]', transaction)

    const { signedTxXdr } = await kit.signTransaction(transaction.toXDR(), {
      address: publicKey,
      networkPassphrase: WalletNetwork.TESTNET,
    })

    console.log('[signedTxXdr]', signedTxXdr)

    const signedTransaction = TransactionBuilder.fromXDR(signedTxXdr, Networks.TESTNET)
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
