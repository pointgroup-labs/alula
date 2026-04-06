import { RPC_URLS, SOROBAN_RPC_URLS } from '@alula/client-sdk'

export enum Network {
  Public = 'public',
  Testnet = 'testnet',
}

export const useRpcStore = defineStore('rpc', () => {
  const config = useRuntimeConfig()
  const customHorizonRpc = useLocalStorage('customHorizonRpc', '', { initOnMounted: true })
  const customSorobanRpc = useLocalStorage('customSorobanRpc', '', { initOnMounted: true })

  const network = useLocalStorage<Network | null>(
    'network',
    null,
    { initOnMounted: true, writeDefaults: false },
  )

  onMounted(() => {
    if (network.value) {
      return
    }

    const env = config.public.ALULA_CLIENT_NETWORK as Network | undefined
    network.value
      = env === Network.Public || env === Network.Testnet
        ? env
        : Network.Testnet
  })

  const horizonRPCUrl = computed(() => customHorizonRpc.value || RPC_URLS[String(network.value)] || '-')
  const sorobanRPCUrl = computed(() => customSorobanRpc.value || SOROBAN_RPC_URLS[String(network.value)] || '-')

  function setNetwork(newNetwork: Network) {
    network.value = newNetwork
  }

  return {
    network,
    setNetwork,
    horizonRPCUrl,
    sorobanRPCUrl,
    customHorizonRpc,
    customSorobanRpc,
  }
})
