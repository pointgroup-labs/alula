import type { RPCcluster } from '../types'
import { Client } from 'sdk'
import { CONTRACT_ID, SOROBAN_CONTRACT_ID } from '../constants'
import { getRPC } from '../utils'

enum Network {
    Mainnet = 'mainnet',
    Testnet = 'testnet',
}

export const NetworkPassphrase = {
    [Network.Mainnet]: 'Public Global Stellar Network ; September 2015',
    [Network.Testnet]: 'Test SDF Network ; September 2015',
}

export class SorobanClient {
    sdk: Client

    constructor(rpc: RPCcluster) {
        this.sdk = new Client({
            rpcUrl: getRPC(rpc, 'soroban'),
            contractId: CONTRACT_ID[rpc] ?? SOROBAN_CONTRACT_ID,
            networkPassphrase: NetworkPassphrase[this.getNetworkPassphrase(rpc)],
        })
    }

    /**
     * Get network passphrase
     */
    private getNetworkPassphrase(rpc: RPCcluster = 'public') {
        return rpc === 'testnet' ? Network.Testnet : Network.Mainnet
    }

    /**
     * Get all pools
     */
    async getAllPools() {
        return (await this.sdk.get_all_pools()).result
    }

    /**
     * Get pool info
     */
    async getPoolInfo(pool_address: string) {
        console.log('POOL ADDRESS', pool_address)
        const poolResult = await this.sdk.get_pool({ pool_address })
        const result = poolResult
        console.log('RAW', result)
        console.log(poolResult.result)
        return this.unwrapOk2(result)
    }

    /**
     * Unwrap ok2
     */
    private unwrapOk2<T>(ok2: any): T {
        return ok2.value
    }

    /**
     * Get token metadata
     */
    // async getTokenMetadata(token_address: string) {
    //     const tokenResult = await this.sdk.get_token_metadata({ token_address })
    //     const result = tokenResult.result
    //     return this.unwrapOk2(result)
    // }
}
