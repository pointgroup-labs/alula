// import { TransactionBuilder } from '@stellar/stellar-sdk'
// import { parseStellarError } from '../utils/errors'

/**
 * Transaction builder and execution helper
 */
export class TransactionHelper {
  private sorobanServer: any

  constructor(/* rpc: any, sorobanServer: any */) {
    // this.rpc = rpc
    // this.sorobanServer = sorobanServer
  }

  /**
   * Sign and send a Soroban transaction
   */
  async signAndSend(): Promise<any> {
    return null
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
  async sendSorobanTx(/* tx: any, user: string, kit: any */): Promise<any> {
    // return sendSorobanTx(tx, user, this.rpc, this.sorobanServer, kit)
    return undefined
  }
}
