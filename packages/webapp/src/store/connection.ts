import type { RPCcluster } from '@alula/client-sdk'
import { RPC_URLS } from '@alula/client-sdk'
import { defineStore } from 'pinia'

export const useConnectionStore = defineStore('connection', () => {
  const toast = useToast()

  const walletStore = useWallet()
  const clientStore = useClientStore()
  const rpcStore = useRpcStore()

  const network = computed(() => rpcStore.network)

  const alulaClient = computed(() => clientStore.alulaClient)

  const selectedWalletId = useLocalStorage('selectedWalletId', '', { initOnMounted: true })

  const { publicKey, balances } = toRefs(walletStore)

  const loading = ref(false)

  const kit = ref()

  async function createKit() {
    const {
      AlbedoModule,
      FreighterModule,
      StellarWalletsKit,
      WalletNetwork,
      XBULL_ID,
      xBullModule,
      RabetModule,
      LobstrModule,
      HanaModule,
      HotWalletModule,
    } = await import('@creit.tech/stellar-wallets-kit')
    const { WalletConnectAllowedMethods, WalletConnectModule } = await import('@creit.tech/stellar-wallets-kit/modules/walletconnect.module')

    const rpcNetwork = network.value === Network.Testnet ? WalletNetwork.TESTNET : WalletNetwork.PUBLIC

    kit.value = new StellarWalletsKit({
      network: rpcNetwork,
      selectedWalletId: selectedWalletId.value ?? XBULL_ID,
      modules: [
        new AlbedoModule(),
        new FreighterModule(),
        // eslint-disable-next-line new-cap
        new xBullModule(),
        new RabetModule(),
        new LobstrModule(),
        new HanaModule(),
        new HotWalletModule(),
        new WalletConnectModule({
          url: 'JLend',
          projectId: '3c5d0cb78534db1da6c199e29b775365',
          method: WalletConnectAllowedMethods.SIGN,
          description: ``,
          name: 'Alula',
          icons: [],
          network: rpcNetwork,
        }),
      ],
    })
  }

  async function initKit() {
    await createKit()
    if (import.meta.client && selectedWalletId.value) {
      try {
        await kit.value.setWallet(selectedWalletId.value)
        const { address } = await kit.value.getAddress()
        await validateAccount(address)
        await walletStore.initWallet(address)
      } catch {
        selectedWalletId.value = ''
      }
    }
  }

  async function connectWallet() {
    if (publicKey.value) {
      return
    }

    loading.value = true
    await kit.value.openModal({
      onWalletSelected: async (option: any) => {
        selectedWalletId.value = option.id

        if (option.id === 'wallet_connect') {
          loading.value = false
        }
        try {
          kit.value.setWallet(option.id)
          const { address } = await kit.value.getAddress()
          await validateAccount(address)
          await walletStore.initWallet(address)
        } finally {
          loading.value = false
        }
      },
      onClosed: () => {
        loading.value = false
      },
    })
  }

  async function validateAccount(address: string) {
    const rpcUrl = RPC_URLS[network.value as RPCcluster]
    try {
      const res = await fetch(`${rpcUrl}/accounts/${address}`)
      if (!res.ok) {
        throw new Error(`Account not found (${res.status})`)
      }
    } catch (error) {
      console.log('Account not found')
      toast.create({
        title: 'Account not found',
        body: 'Please create an account',
        variant: 'danger',
      })
      throw error
    }
  }

  function disconnect() {
    kit.value.disconnect()
    alulaClient.value?.reset()
    publicKey.value = undefined
    balances.value = undefined
    selectedWalletId.value = ''
  }

  watch(network, async () => {
    if (!kit.value) {
      return
    }
    disconnect()
    await initKit()
  })

  onMounted(async () => {
    if (import.meta.client) {
      await initKit()
    }
  })

  return {
    kit,
    loading,

    selectedWalletId,

    disconnect,
    connectWallet,

  }
})
