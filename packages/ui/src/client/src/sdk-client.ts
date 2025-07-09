import type { CompoundRates, Pool } from 'sdk'
import type { RPCcluster } from '../types'
import { WalletNetwork } from '@creit.tech/stellar-wallets-kit'
import { Networks, rpc as SorobanRpc, TransactionBuilder } from '@stellar/stellar-sdk'
import { Client } from 'sdk'
import { CONTRACT_ID, SOROBAN_CONTRACT_ID } from '../constants'
import { amountToBigInt, getRPC, normalizeAssetAmount } from '../utils'

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
    sorobanServer: any
    assetDecimals: number = 7
    oracleDecimals: number = 14

    constructor(rpc: RPCcluster, publicKey?: string) {
        this.sdk = new Client({
            publicKey,
            rpcUrl: getRPC(rpc, 'soroban'),
            contractId: CONTRACT_ID[rpc] ?? SOROBAN_CONTRACT_ID,
            networkPassphrase: NetworkPassphrase[this.getNetworkPassphrase(rpc)],
        })

        this.sorobanServer = new SorobanRpc.Server(getRPC(rpc, 'soroban'))
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
        const poolPriceResult: bigint = this.unwrapOk2(poolPrice.result)
        const normalizedPrice = normalizeAssetAmount(poolPriceResult, this.oracleDecimals)
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
     * Get pool APY
     */
    async getPoolApy(pool_address: string): Promise<CompoundRates> {
        const poolApy = await this.sdk.get_apy({ pool_address })
        return this.unwrapOk2(poolApy.result)
    }

    /**
     * Get user obligation
     * @param {string} user
     */
    async getUserObligation(user: string) {
        const obligation = await this.sdk.get_user_obligation({ user })
        return this.unwrapOk2(obligation.result)
    }

    /**
     * Deposit Tx
     */
    async depositTx(user: string, pool_address: string, amount: string | number) {
        return await this.sdk.deposit({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
    }

    /**
     * Deposit
     */
    async deposit(
        user: string,
        pool_address: string,
        amount: number,
        kit: any) {
        const tx = await this.depositTx(user, pool_address, amount)

        console.log('[Deposit tx]', tx)

        const { signedTxXdr } = await kit.signTransaction(tx.toXDR(), {
            address: user,
            networkPassphrase: WalletNetwork.TESTNET,
        })

        console.log('[signedTxXdr]', signedTxXdr)

        const txObject = TransactionBuilder.fromXDR(signedTxXdr, Networks.TESTNET)

        const sendResponse = await this.sorobanServer.sendTransaction(txObject)

        console.log('[Tx send responce]', sendResponse)

        const result = await this.sorobanServer.pollTransaction(sendResponse.hash, {
            sleepStrategy: (_iter: any) => 1000,
            attempts: 30,
        })
        console.log('✅ Transaction submitted!', result)

        return result
    }

    /**
     * Borrow Tx
     */
    async borrowTx(user: string, pool_address: string, amount: string | number) {
        return await this.sdk.borrow({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
    }

    /**
     * Borrow
     */
    async borrow(
        user: string,
        pool_address: string,
        amount: number,
        kit: any) {
        const tx = await this.borrowTx(user, pool_address, String(amount))

        console.log('[Borrow tx]', tx)

        const { signedTxXdr } = await kit.signTransaction(tx.toXDR(), {
            address: user,
            networkPassphrase: WalletNetwork.TESTNET,
        })

        console.log('[signedTxXdr]', signedTxXdr)

        const txObject = TransactionBuilder.fromXDR(signedTxXdr, Networks.TESTNET)

        const sendResponse = await this.sorobanServer.sendTransaction(txObject)

        console.log('[Tx send responce]', sendResponse)
        if (sendResponse.status === 'ERROR') {
            throw new Error(sendResponse.error)
        }

        const result = await this.sorobanServer.pollTransaction(sendResponse.hash, {
            sleepStrategy: (_iter: any) => 1000,
            attempts: 30,
        })
        console.log('✅ Transaction submitted!', result)

        return result
    }

    /**
     * Get transaction fee
     */
    getTransactionFee(tx: any) {
        const stroops: bigint = tx.simulation.minResourceFee || 0
        return Number(normalizeAssetAmount(stroops, this.assetDecimals))
    }

    /**
     * Unwrap ok2
     */
    private unwrapOk2<T>(ok2: any): T {
        return ok2.value
    }
}
