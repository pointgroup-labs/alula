import type { RPCcluster } from '../types'
import { Account, Asset, BASE_FEE, Horizon, Operation, TransactionBuilder } from '@stellar/stellar-sdk'
import { BaseClient } from '../core/base-client'

/**
 * Wallet service configuration
 */
export interface WalletServiceConfig {
  rpc: RPCcluster
  publicKey?: string
}

/**
 * Service for wallet operations (balances, trustlines)
 */
export class WalletService extends BaseClient {
  private horizonServer: Horizon.Server

  constructor(config: WalletServiceConfig) {
    super(config)
    this.horizonServer = new Horizon.Server(this.getHorizonRpcUrl())
  }

  /**
   * Get wallet balances
   */
  async getBalances(publicKey?: string): Promise<Horizon.HorizonApi.BalanceLine[] | undefined> {
    const address = publicKey || this.publicKey
    if (!address) {
      throw new Error('Public key is required to get balances')
    }

    const account = await this.horizonServer.loadAccount(address)
    return account.balances
  }

  /**
   * Build add trustline transaction
   */
  async buildAddTrustlineTx(publicKey: string, assetCode: string, assetIssuer: string) {
    const accountResponse = await this.horizonServer.loadAccount(publicKey)
    const account = new Account(accountResponse.accountId(), accountResponse.sequence.toString())
    const asset = new Asset(assetCode, assetIssuer)

    return new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(Operation.changeTrust({ asset }))
      .setTimeout(30)
      .build()
  }

  /**
   * Add trustline to wallet
   */
  async addTrustline(
    publicKey: string,
    assetCode: string,
    assetIssuer: string,
    kit: any,
    options?: { debug?: boolean },
  ) {
    const transaction = await this.buildAddTrustlineTx(publicKey, assetCode, assetIssuer)

    if (options?.debug) {
      console.log('%c[Add Trustline Tx]', 'color: #00ff00', transaction)
    }

    const { signedTxXdr } = await kit.signTransaction(transaction.toXDR(), {
      address: publicKey,
      networkPassphrase: this.networkPassphrase,
    })

    if (options?.debug) {
      console.log('[Signed Tx XDR]', signedTxXdr)
    }

    const signedTransaction = TransactionBuilder.fromXDR(signedTxXdr, this.networkPassphrase)
    const result = await this.horizonServer.submitTransaction(signedTransaction)

    if (options?.debug) {
      console.log('✅ Transaction submitted!', result)
    }

    return result
  }

  /**
   * Check if trustline exists for asset
   */
  async hasTrustline(publicKey: string, assetCode: string, assetIssuer: string): Promise<boolean> {
    const balances = await this.getBalances(publicKey)
    if (!balances) { return false }

    return balances.some((balance) => {
      if (balance.asset_type === 'native') { return false }
      const creditBalance = balance as Horizon.HorizonApi.BalanceLineAsset
      return creditBalance.asset_code === assetCode && creditBalance.asset_issuer === assetIssuer
    })
  }

  /**
   * Get balance for specific asset
   */
  async getAssetBalance(publicKey: string, assetCode: string, assetIssuer?: string): Promise<string | null> {
    const balances = await this.getBalances(publicKey)
    if (!balances) { return null }

    for (const balance of balances) {
      if (balance.asset_type === 'native' && assetCode === 'XLM') {
        return balance.balance
      }

      const creditBalance = balance as Horizon.HorizonApi.BalanceLineAsset
      if (creditBalance.asset_code === assetCode && (!assetIssuer || creditBalance.asset_issuer === assetIssuer)) {
        return balance.balance
      }
    }

    return null
  }
}
