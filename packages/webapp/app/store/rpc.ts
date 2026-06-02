import { RPC_URLS, SOROBAN_PUBLIC_RPC_URL, SOROBAN_RPC_URLS, SOROBAN_TESTNET_RPC_URL } from '@alula/client-sdk'

export enum Network {
  Public = 'public',
  Testnet = 'testnet',
}

export const SOROBAN_PUBLIC_RPC_URLS = new Set([
  SOROBAN_PUBLIC_RPC_URL,
  'https://rpc.ankr.com/stellar_soroban',
  'https://stellar.api.onfinality.io/public',
  'https://soroban-rpc.mainnet.stellar.gateway.fm',
])

export const SOROBAN_TESTNET_RPC_URLS = new Set([
  SOROBAN_TESTNET_RPC_URL,
  'https://stellar-soroban-testnet-public.nodies.app',
  'https://rpc.ankr.com/stellar_testnet_soroban',
])

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

    const env = (config as { public?: { ALULA_CLIENT_NETWORK?: string } }).public?.ALULA_CLIENT_NETWORK as Network | undefined
    network.value
      = env === Network.Public || env === Network.Testnet
        ? env
        : Network.Testnet
  })

  const horizonRPCUrl = computed(() => customHorizonRpc.value || RPC_URLS[String(network.value)])
  const sorobanRPCUrl = computed(() => customSorobanRpc.value || SOROBAN_RPC_URLS[String(network.value)])

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
