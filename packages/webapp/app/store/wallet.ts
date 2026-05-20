import { defineStore } from 'pinia'

export const useWallet = defineStore('wallet', () => {
  const clientStore = useClientStore()
  const alulaClient = computed(() => clientStore.alulaClient)

  const publicKey = ref()
  const balances = ref()

  const isloadingBalances = ref(false)

  const nativeBalance = computed(() => Number(balances.value?.find((b: any) => b.asset_type === 'native')?.balance) || 0)

  async function initWallet(address: string) {
    publicKey.value = address
    await loadBalances()
  }

  async function loadBalances() {
    try {
      isloadingBalances.value = true
      if (!publicKey.value || !alulaClient.value) {
        return
      }

      // Set publicKey in wallet service before getting balances
      alulaClient.value.wallet.setPublicKey(publicKey.value)
      balances.value = await alulaClient.value.wallet.getBalances()
      console.log('%c[Wallet Balances]', 'color: #5c6cff', balances.value)
    } finally {
      isloadingBalances.value = false
    }
  }

  function getAssetBalance(asset_issuer?: string) {
    if (!asset_issuer) {
      return 0
    }
    const balance = balances.value?.find((b: any) => b.asset_issuer?.toLowerCase() === asset_issuer?.toLowerCase())?.balance || 0
    return Number(balance)
  }

  return {
    publicKey,
    balances,

    nativeBalance,

    isloadingBalances,

    initWallet,
    loadBalances,
    getAssetBalance,
  }
})
