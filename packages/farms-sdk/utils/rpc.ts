import type { RPCcluster } from '../types'
import { Networks, TransactionBuilder } from '@stellar/stellar-sdk'
import { RPC_URLS, SOROBAN_RPC_URLS } from '../constants'

export function getRPC(rpc: RPCcluster = 'public', rpcType: 'horizon' | 'soroban'): string {
  const rpcUrls: Record<string, string> = rpcType === 'horizon' ? RPC_URLS : SOROBAN_RPC_URLS
  const url = rpcUrls[rpc] ?? rpcUrls.public
  return String(url)
}

export function getNetworkPassphrase(rpc: RPCcluster = 'public') {
  return rpc === 'testnet' ? Networks.TESTNET : Networks.PUBLIC
}

export async function sendSorobanTx(tx: any, user: string, network: RPCcluster, server: any, kit: any) {
  const networkPassphrase = getNetworkPassphrase(network)

  const { signedTxXdr } = await kit.signTransaction(tx.toXDR(), {
    address: user,
    networkPassphrase,
  })

  console.log('[signedTxXdr]', signedTxXdr)

  const txObject = TransactionBuilder.fromXDR(signedTxXdr, networkPassphrase)

  const sendResponse = await server.sendTransaction(txObject)

  console.log('[Tx send response]', sendResponse)

  if (sendResponse.status === 'ERROR') {
    throw new Error(tx.simulation?.error)
  }

  const result = await server.pollTransaction(sendResponse.hash, {
    sleepStrategy: (_iter: any) => 1000,
    attempts: 30,
  })

  if (result.status === 'FAILED') {
    const errorMessage = `Transaction failed! Tx Hash: ${result.txHash}`
    throw new Error(errorMessage)
  }

  console.log('✅ Transaction submitted!', result)

  return result
}
