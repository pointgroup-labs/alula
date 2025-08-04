import type { ISupportedWallet } from '@creit.tech/stellar-wallets-kit'
import { defineStore } from 'pinia'

export const useConnectionStore = defineStore('connection', () => {
  const walletStore = useWallet()
  const clientStore = useClientStore()

  const jLendClient = computed(() => clientStore.jLendClient)

  const { publicKey, balances } = toRefs(walletStore)
  const loading = ref(false)

  const kit = ref()

  onMounted(async () => {
    if (isClient) {
      const savedWalletId = localStorage.getItem('selectedWalletId')

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

      kit.value = new StellarWalletsKit({
        network: WalletNetwork.TESTNET,
        selectedWalletId: savedWalletId ?? XBULL_ID,
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
            description: `A DESCRIPTION TO SHOW USERS`,
            name: 'THE NAME OF YOUR DAPP',
            icons: ['A LOGO/ICON TO SHOW TO YOUR USERS'],
            network: WalletNetwork.TESTNET,
          }),
        ],
      })

      if (savedWalletId) {
        try {
          await kit.value.setWallet(savedWalletId)
          const { address } = await kit.value.getAddress()
          await walletStore.initWallet(address)
          publicKey.value = address
        } catch {
          localStorage.removeItem('selectedWalletId')
        }
      }
    }
  })

  async function connectWallet() {
    if (publicKey.value) {
      return
    }

    loading.value = true
    await kit.value.openModal({
      onWalletSelected: async (option: ISupportedWallet) => {
        localStorage.setItem('selectedWalletId', option.id)

        if (option.id === 'wallet_connect') {
          loading.value = false
        }
        try {
          kit.value.setWallet(option.id)
          const { address } = await kit.value.getAddress()
          await walletStore.initWallet(address)
          publicKey.value = address
        } finally {
          loading.value = false
        }
      },
      onClosed: () => {
        loading.value = false
      },
    })
  }

  function disconnect() {
    kit.value.disconnect()
    jLendClient.value?.reset()
    publicKey.value = undefined
    balances.value = undefined
    localStorage.setItem('selectedWalletId', '')
  }

  return {
    kit,
    loading,

    disconnect,
    connectWallet,

  }
})
