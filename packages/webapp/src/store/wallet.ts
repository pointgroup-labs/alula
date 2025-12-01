import { defineStore } from 'pinia'

export const useWallet = defineStore('wallet', () => {
  const clientStore = useClientStore()
  const alulaClient = computed(() => clientStore.alulaClient)

  const publicKey = ref()
  const balances = ref()

  const nativeBalance = computed(() => Number(balances.value?.find((b: any) => b.asset_type === 'native')?.balance) || 0)

  async function initWallet(address: string) {
    publicKey.value = address
    await loadBalances()
  }

  async function loadBalances() {
    balances.value = await alulaClient.value?.getBalances()
    console.log('%c[Wallet Balances]', 'color: #FFB726', balances.value)
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

    initWallet,
    loadBalances,
    getAssetBalance,
  }
})
