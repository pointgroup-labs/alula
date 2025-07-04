import type { RPCcluster } from '../types'
import { RPC_URLS, SOROBAN_RPC_URLS } from '../constants'

export function getRPC(rpc: RPCcluster = 'public', rpcType: 'horizon' | 'soroban') {
    const rpcUrls: Record<string, string> = rpcType === 'horizon' ? RPC_URLS : SOROBAN_RPC_URLS
    const url = rpcUrls[rpc] ?? rpcUrls.public
    return url
}
