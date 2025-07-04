import { defineStore } from 'pinia'

export const useWallet = defineStore('wallet', () => {
    const connectionStore = useConnectionStore()

    const jLendClient = computed(() => connectionStore.jLendClient)
    const publicKey = ref()
    const balances = ref()

    const nativeBalance = computed(() => balances.value?.find((b: any) => b.asset_type === 'native')?.balance)

    async function initWallet(address: string) {
        publicKey.value = address
        balances.value = await jLendClient.value.getBalances()
        console.log('%c[Wallet Balances]', 'color: #FFB726', balances.value)
    }

    function getAssetBalance(asset_code?: string) {
        if (!asset_code) {
            return 0
        }
        return balances.value?.find((b: any) => b.asset_code?.toLowerCase() === asset_code?.toLowerCase())?.balance || 0
    }

    // async function findMatchingAsset(pool: Pool, sorobanClient: any) {
    //     const metadata = await sorobanClient.getTokenMetadata(pool.token_address)

    //     return balances.value?.find((asset: Horizon.HorizonApi.BalanceLineAsset) =>
    //         asset.asset_code === metadata.symbol
    //         && asset.asset_issuer === metadata.issuer,
    //     )
    // }

    return {
        publicKey,
        balances,

        nativeBalance,

        initWallet,
        getAssetBalance,
    }
})
