import type { RPCcluster } from '@alula/client-sdk'
import { StellarClient } from '@alula/client-sdk'
import { defineStore } from 'pinia'

export const useClientStore = defineStore('client', () => {
  const rpcStore = useRpcStore()

  const network = computed(() => rpcStore.network)

  const walletStore = useWallet()

  const publicKey = computed(() => walletStore.publicKey)

  const isValidAccount = ref(false)

  const alulaClient = computedAsync(async () => await initClient())

  async function initClient(marketAddress?: string) {
    const pubkey = isValidAccount.value ? publicKey.value : undefined
    return import.meta.client && network.value
      ? await StellarClient.fromAddress(pubkey, network.value as RPCcluster, marketAddress)
      : {} as StellarClient
  }
  return {
    alulaClient,

    initClient,
    isValidAccount,
  }
})
