import type { RPCcluster } from '@alula/client-sdk'
import { StellarClient } from '@alula/client-sdk'
import { defineStore } from 'pinia'

export const useClientStore = defineStore('client', () => {
  const rpcStore = useRpcStore()

  const network = computed(() => rpcStore.network)

  const walletStore = useWallet()

  const publicKey = computed(() => walletStore.publicKey)

  const alulaClient = computed(() => initClient())

  function initClient(marketAddress?: string) {
    return import.meta.client && network.value
      ? StellarClient.fromAddress(publicKey.value, network.value as RPCcluster, marketAddress)
      : {} as StellarClient
  }
  return {
    alulaClient,

    initClient,
  }
})
