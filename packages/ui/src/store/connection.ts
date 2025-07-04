import type {
  ISupportedWallet,
} from '@creit.tech/stellar-wallets-kit'
import {
  AlbedoModule,
  FreighterModule,
  StellarWalletsKit,
  WalletNetwork,
  XBULL_ID,
  xBullModule,
} from '@creit.tech/stellar-wallets-kit'
import { WalletConnectAllowedMethods, WalletConnectModule } from '@creit.tech/stellar-wallets-kit/modules/walletconnect.module'
import { StellarClient } from '@stellar-client'
import { defineStore } from 'pinia'

export const useConnectionStore = defineStore('connection', () => {
  const walletStore = useWallet()

  const { publicKey, balances } = toRefs(walletStore)
  const loading = ref(false)

  const jLendClient = computed(() => StellarClient.fromAddress(publicKey.value, 'testnet'))

  const kit: StellarWalletsKit = new StellarWalletsKit({
    network: WalletNetwork.TESTNET,
    selectedWalletId: XBULL_ID,
    modules: [
      new AlbedoModule(),
      new FreighterModule(),
      // eslint-disable-next-line new-cap
      new xBullModule(),
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

  async function connectWallet() {
    if (publicKey.value) {
      return
    }
    loading.value = true
    await kit.openModal({
      onWalletSelected: async (option: ISupportedWallet) => {
        try {
          kit.setWallet(option.id)
          const { address } = await kit.getAddress()
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
    kit.disconnect()
    jLendClient.value?.reset()
    publicKey.value = undefined
    balances.value = undefined
  }

  return {
    loading,

    jLendClient,

    disconnect,
    connectWallet,

  }
})
