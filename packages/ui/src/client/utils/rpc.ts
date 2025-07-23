import type { RPCcluster } from '../types'
import { Networks, TransactionBuilder } from '@stellar/stellar-sdk'
import { RPC_URLS, SOROBAN_RPC_URLS } from '../constants'
import { parseStellarError } from './errors'

export function getRPC(rpc: RPCcluster = 'public', rpcType: 'horizon' | 'soroban') {
    const rpcUrls: Record<string, string> = rpcType === 'horizon' ? RPC_URLS : SOROBAN_RPC_URLS
    const url = rpcUrls[rpc] ?? rpcUrls.public
    return url
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

    console.log('[Tx send responce]', sendResponse)

    if (sendResponse.status === 'ERROR') {
        const errorMessage = parseStellarError(tx.simulation?.error)
        throw new Error(errorMessage)
    }

    const result = await server.pollTransaction(sendResponse.hash, {
        sleepStrategy: (_iter: any) => 1000,
        attempts: 30,
    })

    console.log('✅ Transaction submitted!', result)

    return result
}
