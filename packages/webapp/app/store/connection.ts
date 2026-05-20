import type { RPCcluster } from '@alula/client-sdk'
import { RPC_URLS } from '@alula/client-sdk'
import { defineStore } from 'pinia'

const VALIDATE_INTERVAL = 10_000

export const useConnectionStore = defineStore('connection', () => {
  const walletStore = useWallet()
  const clientStore = useClientStore()
  const rpcStore = useRpcStore()

  const network = computed(() => rpcStore.network)

  const selectedWalletId = useLocalStorage('selectedWalletId', '', { initOnMounted: true })

  const { publicKey, balances } = toRefs(walletStore)

  const loading = ref(false)
  const autoConnecting = ref(false)

  const kit = ref()
  const walletModules = ref<any[]>([])

  async function createKit() {
    const { Networks, ModuleType, StellarWalletsKit } = await import('@creit.tech/stellar-wallets-kit')
    const { AlbedoModule } = await import('@creit.tech/stellar-wallets-kit/modules/albedo')
    const { FreighterModule } = await import('@creit.tech/stellar-wallets-kit/modules/freighter')
    const { HanaModule } = await import('@creit.tech/stellar-wallets-kit/modules/hana')
    const { HotWalletModule } = await import('@creit.tech/stellar-wallets-kit/modules/hotwallet')
    const { LobstrModule } = await import('@creit.tech/stellar-wallets-kit/modules/lobstr')
    const { RabetModule } = await import('@creit.tech/stellar-wallets-kit/modules/rabet')
    const { WalletConnectModule, WALLET_CONNECT_ID, WalletConnectTargetChain } = await import('@creit.tech/stellar-wallets-kit/modules/wallet-connect')
    const { XBULL_ID, xBullModule } = await import('@creit.tech/stellar-wallets-kit/modules/xbull')

    const rpcNetwork = network.value === Network.Testnet ? Networks.TESTNET : Networks.PUBLIC
    const walletConnectChain = network.value === Network.Testnet ? WalletConnectTargetChain.TESTNET : WalletConnectTargetChain.PUBLIC

    walletModules.value = [
      new AlbedoModule(),
      new FreighterModule(),
      // eslint-disable-next-line new-cap
      new xBullModule(),
      new RabetModule(),
      new LobstrModule(),
      new HanaModule(),
      new HotWalletModule(),
      new WalletConnectModule({
        projectId: '3c5d0cb78534db1da6c199e29b775365',
        metadata: {
          name: 'Alula',
          description: '',
          url: import.meta.client ? globalThis.location.origin : 'https://alula.finance',
          icons: [],
        },
        allowedChains: [walletConnectChain],
      }),
    ]

    StellarWalletsKit.init({
      network: rpcNetwork,
      selectedWalletId: selectedWalletId.value ?? XBULL_ID,
      modules: walletModules.value,
      authModal: {
        hideUnsupportedWallets: false,
      },
    })

    kit.value = StellarWalletsKit

    return { ModuleType, WALLET_CONNECT_ID }
  }

  async function initKit() {
    const { ModuleType, WALLET_CONNECT_ID } = await createKit()

    if (import.meta.client && selectedWalletId.value) {
      if (!canAutoConnect(selectedWalletId.value, ModuleType, WALLET_CONNECT_ID)) {
        selectedWalletId.value = ''
        return
      }

      autoConnecting.value = true

      try {
        kit.value.setWallet(selectedWalletId.value)
        const { address } = await kit.value.fetchAddress()
        await validateAccount(address)
        await walletStore.initWallet(address)
      } catch {
        selectedWalletId.value = ''
      } finally {
        autoConnecting.value = false
      }
    }
  }

  async function connectWallet() {
    if (publicKey.value) {
      return
    }

    loading.value = true
    let selectedId = ''
    const unsubscribe = kit.value.on('WALLET_SELECTED', (event: any) => {
      selectedId = event.payload.id ?? ''
      if (selectedId) {
        selectedWalletId.value = selectedId
      }
    })

    try {
      const { address } = await kit.value.authModal()
      await validateAccount(address)
      await walletStore.initWallet(address)
    } finally {
      unsubscribe?.()
      loading.value = false
    }
  }

  function getModuleById(walletId: string) {
    return walletModules.value.find(module => module.productId === walletId)
  }

  function canAutoConnect(walletId: string, moduleType: any, walletConnectId: string) {
    const module = getModuleById(walletId)

    if (!module) {
      return false
    }

    if (module.moduleType !== moduleType.HOT_WALLET) {
      return false
    }

    if (['albedo', 'lobstr', 'hot-wallet', walletConnectId].includes(module.productId)) {
      return false
    }

    return true
  }

  async function validateAccount(address: string) {
    const rpcUrl = RPC_URLS[network.value as RPCcluster]
    const res = await fetch(`${rpcUrl}/accounts/${address}`)
    clientStore.isValidAccount = !!res.ok
  }

  async function disconnect() {
    await kit.value?.disconnect()
    // alulaClient.value?.reset()
    publicKey.value = undefined
    balances.value = undefined
    selectedWalletId.value = ''
  }

  watch([
    network,
  ], async () => {
    if (!kit.value) {
      return
    }
    await disconnect()
    await initKit()
  })

  let interval: any

  watch([
    publicKey,
    () => clientStore.isValidAccount,
  ], async ([pubkey, isValid], [, prevIsValid]) => {
    clearInterval(interval)
    // if user connect new wallet without balances, need to validate account
    if (pubkey && !isValid) {
      interval = setInterval(async () => {
        await validateAccount(pubkey)
      }, VALIDATE_INTERVAL)
    }
    // if user connected and topped up balance - init wallet again to load balances and regenerate market clients
    if (pubkey && !prevIsValid && isValid) {
      walletStore.publicKey = undefined
      await sleep(200)
      await walletStore.initWallet(pubkey)
    }
  }, { immediate: true })

  onMounted(async () => {
    if (import.meta.client) {
      await initKit()
    }
  })

  return {
    kit,
    loading,
    autoConnecting,

    selectedWalletId,

    disconnect,
    connectWallet,
  }
})
