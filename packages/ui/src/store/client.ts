import type { RPCcluster } from '@alula/client-sdk'
import { StellarClient } from '@alula/client-sdk'
// import { useRuntimeConfig } from 'nuxt/app'
import { defineStore } from 'pinia'

export const useClientStore = defineStore('client', () => {
  // const config = useRuntimeConfig()

  const rpcStore = useRpcStore()

  const network = computed(() => rpcStore.network)

  const walletStore = useWallet()

  const publicKey = computed(() => walletStore.publicKey)

  const alulaClient = computed(() => import.meta.client && network.value
    ? StellarClient.fromAddress(publicKey.value, network.value as RPCcluster)
    : {} as StellarClient)

  const assetDecimals = computed(() => alulaClient.value?.sdk?.assetDecimals || 7)
  return {
    alulaClient,
    assetDecimals,
  }
})
