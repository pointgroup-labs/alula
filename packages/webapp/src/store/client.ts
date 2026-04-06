import type { RPCcluster } from '@alula/client-sdk'
import { StellarClient } from '@alula/client-sdk'
import { defineStore } from 'pinia'

export const useClientStore = defineStore('client', () => {
  const toast = useToast()

  const rpcStore = useRpcStore()

  const network = computed(() => rpcStore.network)

  const isValidAccount = ref(false)

  const alulaClient = computedAsync(async () => await initClient())

  async function initClient(marketAddress?: string) {
    try {
      const walletStore = useWallet()
      const pubkey = isValidAccount.value ? walletStore.publicKey : undefined

      return import.meta.client && network.value
        ? await StellarClient.fromAddress(pubkey, marketAddress, {
            rpc: network.value as RPCcluster,
            horizonRpcUrl: rpcStore.horizonRPCUrl,
            sorobanRpcUrl: rpcStore.sorobanRPCUrl,
          })
        : {} as StellarClient
    } catch (error: any) {
      console.error(error)
      toast.create({
        title: `Client Error`,
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 5000,
      })
    }
  }
  return {
    alulaClient,

    initClient,
    isValidAccount,
  }
})
