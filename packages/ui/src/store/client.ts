import type { RPCcluster } from '@stellar-client'
import { StellarClient } from '@stellar-client'
import { useRuntimeConfig } from 'nuxt/app'
import { defineStore } from 'pinia'

export const useClientStore = defineStore('client', () => {
    const config = useRuntimeConfig()

    const walletStore = useWallet()

    const publicKey = computed(() => walletStore.publicKey)

    const clientNetwork = computed(() => config.public.JLEND_CLIENT_NETWORK || 'testnet')

    const jLendClient = computed(() => isClient ? StellarClient.fromAddress(publicKey.value, clientNetwork.value as RPCcluster) : {} as StellarClient)
    const assetDecimals = computed(() => jLendClient.value?.sdk?.assetDecimals || 7)
    return {
        jLendClient,
        assetDecimals,
    }
})
