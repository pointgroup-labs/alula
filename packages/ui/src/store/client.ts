import { StellarClient } from '@stellar-client'
import { defineStore } from 'pinia'

export const useClientStore = defineStore('client', () => {
    const walletStore = useWallet()

    const { publicKey } = toRefs(walletStore)

    const jLendClient = computed(() => StellarClient.fromAddress(publicKey.value, 'testnet'))
    const assetDecimals = computed(() => jLendClient.value?.sdk.assetDecimals || 7)
    return {
        jLendClient,
        assetDecimals,
    }
})
