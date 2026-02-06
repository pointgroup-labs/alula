import type { RPCcluster } from '../types'
import { TransactionBuilder } from '@stellar/stellar-sdk'
import { getNetworkPassphrase, sendSorobanTx } from '../utils'
import { parseStellarError } from '../utils/errors'

/**
 * Transaction builder and execution helper
 */
export class TransactionHelper {
  private rpc: RPCcluster
  private sorobanServer: any

  constructor(rpc: RPCcluster, sorobanServer: any) {
    this.rpc = rpc
    this.sorobanServer = sorobanServer
  }

  /**
   * Sign and send a Soroban transaction
   */
  async signAndSend(tx: any, user: string, kit: any, options?: { debug?: boolean }) {
    const networkPassphrase = getNetworkPassphrase(this.rpc)

    if (options?.debug) {
      console.log('%c[Transaction]', 'color: #00ff00', tx)
    }

    const { signedTxXdr } = await kit.signTransaction(tx.toXDR(), {
      address: user,
      networkPassphrase,
    })

    if (options?.debug) {
      console.log('[Signed Tx XDR]', signedTxXdr)
    }

    const txObject = TransactionBuilder.fromXDR(signedTxXdr, networkPassphrase)
    const sendResponse = await this.sorobanServer.sendTransaction(txObject)

    if (options?.debug) {
      console.log('[Tx Send Response]', sendResponse)
    }

    if (sendResponse.status === 'ERROR') {
      const errorMessage = parseStellarError(tx.simulation?.error)
      throw new Error(errorMessage)
    }

    const result = await this.sorobanServer.pollTransaction(sendResponse.hash, {
      sleepStrategy: (_iter: any) => 1000,
      attempts: 30,
    })

    if (result.status === 'FAILED') {
      const errorMessage = `Transaction failed! Tx Hash: ${result.txHash}`
      throw new Error(errorMessage)
    }

    if (options?.debug) {
      console.log('✅ Transaction submitted!', result)
    }

    return result
  }

  /**
   * Get transaction fee in XLM
   */
  getTransactionFee(tx: any, decimals: number = 7): number {
    const stroops: bigint = tx.simulation?.minResourceFee || 0
    return Number(stroops) / (10 ** decimals)
  }

  /**
   * Legacy method for backward compatibility
   * @deprecated Use signAndSend instead
   */
  async sendSorobanTx(tx: any, user: string, kit: any) {
    return sendSorobanTx(tx, user, this.rpc, this.sorobanServer, kit)
  }
}
