import type { RPCcluster } from '@alula/client-sdk'
import { StellarClient } from '@alula/client-sdk'
import { defineStore } from 'pinia'

export const useClientStore = defineStore('client', () => {
  const rpcStore = useRpcStore()

  const network = computed(() => rpcStore.network)

  const isValidAccount = ref(false)

  const alulaClient = computedAsync(async () => await initClient())

  async function initClient(marketAddress?: string) {
    const walletStore = useWallet()
    const pubkey = isValidAccount.value ? walletStore.publicKey : undefined

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
