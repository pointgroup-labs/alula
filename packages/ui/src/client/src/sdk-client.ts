import type { Pool } from 'sdk'
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
    assetDecimals: number = 7
    oracleDecimals: number = 14

    constructor(rpc: RPCcluster) {
        this.sdk = new Client({
            rpcUrl: getRPC(rpc, 'soroban'),
            contractId: CONTRACT_ID[rpc] ?? SOROBAN_CONTRACT_ID,
            networkPassphrase: NetworkPassphrase[this.getNetworkPassphrase(rpc)],
        })
        this.getDecimals()
    }

    /**
     * Get network passphrase
     */
    private getNetworkPassphrase(rpc: RPCcluster = 'public') {
        return rpc === 'testnet' ? Network.Testnet : Network.Mainnet
    }

    /**
     * Get asset decimals
     */
    async getAssetDecimals() {
        this.assetDecimals = (await this.sdk.get_asset_decimals()).result
    }

    /**
     * Get oracle decimals
     */
    async getOracleDecimals() {
        this.oracleDecimals = (await this.sdk.get_oracle_price_decimals()).result
    }

    /**
     * Get decimals
     */
    async getDecimals() {
        await this.getAssetDecimals()
        await this.getOracleDecimals()
    }

    /**
     * Get pool asset oracle price
     */
    async getPoolAssetOraclePrice(pool_address: string) {
        const poolPrice = await this.sdk.get_pool_asset_oracle_price({ pool_address })
        const poolPriceResult = this.unwrapOk2(poolPrice.result)
        const normalizedPrice = Number(poolPriceResult) / 10 ** this.oracleDecimals
        return normalizedPrice || 0
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
    async getPoolInfo(pool_address: string): Promise<Pool> {
        const poolResult = await this.sdk.get_pool({ pool_address })
        return this.unwrapOk2(poolResult.result)
    }

    /**
     * Unwrap ok2
     */
    private unwrapOk2<T>(ok2: any): T {
        return ok2.value
    }
}
