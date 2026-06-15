import type { RPCcluster } from '@alula/client-sdk'
import { AlulaClient } from '@alula/client-sdk'
import { FarmsClient } from '@alula/farms-sdk'
import { defineStore } from 'pinia'

export const useClientStore = defineStore('client', () => {
  const toast = useToast()

  const rpcStore = useRpcStore()

  const network = computed(() => rpcStore.network)

  const isValidAccount = ref(false)

  const alulaClient = ref()

  const farmsClient = ref()

  async function initClient(marketAddress?: string) {
    try {
      const walletStore = useWallet()
      const pubkey = isValidAccount.value ? walletStore.publicKey : undefined

      return import.meta.client && network.value
        ? await AlulaClient.fromAddress(pubkey, marketAddress, {
            rpc: network.value as RPCcluster,
            horizonRpcUrl: rpcStore.horizonRPCUrl,
            sorobanRpcUrl: rpcStore.sorobanRPCUrl,
          })
        : {} as AlulaClient
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

  async function initFarmsClient(farmsContractAddress?: string | null) {
    try {
      if (!farmsContractAddress) {
        return
      }
      const walletStore = useWallet()
      const pubkey = isValidAccount.value ? walletStore.publicKey : undefined

      return import.meta.client && network.value
        ? await FarmsClient.fromAddress(pubkey, farmsContractAddress, {
            rpc: network.value as RPCcluster,
            horizonRpcUrl: rpcStore.horizonRPCUrl,
            sorobanRpcUrl: rpcStore.sorobanRPCUrl,
          })
        : {} as FarmsClient
    } catch (error: any) {
      console.error(error)
      toast.create({
        title: `Farms Client Error`,
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 5000,
      })
    }
  }

  watch([
    network,
    () => rpcStore.horizonRPCUrl,
    () => rpcStore.sorobanRPCUrl,
  ], async ([nextNetwork, nextHorizonRpcUrl, nextSorobanRpcUrl]) => {
    if (nextNetwork && nextHorizonRpcUrl && nextSorobanRpcUrl) {
      alulaClient.value = await initClient()
    }
  })
  return {
    alulaClient,
    farmsClient,

    initClient,
    isValidAccount,

    initFarmsClient,

  }
})
