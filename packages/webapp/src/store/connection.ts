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
    requestAnimationFrame(() => {
      styleWalletModal()
    })
  }

  async function validateAccount(address: string) {
    const rpcUrl = RPC_URLS[network.value as RPCcluster]
    const res = await fetch(`${rpcUrl}/accounts/${address}`)
    clientStore.isValidAccount = !!res.ok
  }

  function disconnect() {
    kit.value.disconnect()
    // alulaClient.value?.reset()
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

    selectedWalletId,

    disconnect,
    connectWallet,

  }
})

function styleWalletModal() {
  const modal = document.querySelector('stellar-wallets-modal')
  const root = modal?.shadowRoot
  if (!root) { return }

  const style = document.createElement('style')
  style.textContent = `
    .backdrop {
      background: rgba(0, 0, 0, 0.1) !important;
      backdrop-filter: blur(6.4px);
    }
    .dialog-modal  {
      background: transparent !important
    }
      .dialog-modal-body {
      background-color: transparent !important;
      }
    .dialog-modal-body__help {
      background-color: rgba(255, 255, 255, 0.04) !important;
      backdrop-filter: blur(30px);
    }
    .dialog-modal-body__wallets {
      background-color: rgba(255, 255, 255, 0.10) !important;
      backdrop-filter: blur(30px);
    }
    .wallets-header__button svg {
      fill: #fff !important;
      width: 16px;
      height: 16px
    }
    small.not-available {
      border-radius: 16px !important;
      border: 1px solid rgba(255, 255, 255, 0.08) !important;
      background: rgba(255, 255, 255, 0.04) !important;
    }
    .dialog-text,
    .dialog-text-solid {
      color: #f5f5f5 !important;
    }
  `

  root.append(style)
}
