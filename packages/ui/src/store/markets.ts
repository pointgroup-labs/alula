import type { CompoundRates, Pool } from 'sdk'
import { defineStore } from 'pinia'

export const useMarketsStore = defineStore('markets', () => {
  const state = reactive<MarketsState>({
    poolAddresses: [],
    pollsData: [],
    loading: false,
    markets: ['Main market', 'Assets'],
  })

  const Toast = useToast()

  const poolDepositAddr = ref()

  const connectionStore = useConnectionStore()
  const wallet = useWallet()

  const jLendClient = computed(() => connectionStore.jLendClient)
  const selectedMarketInfo = ref()

  const selectedMarket = ref('Main market')

  const selectedMarketPools = computed(() => {
    return state.pollsData.filter(p => selectedMarket.value.toLowerCase().includes(String(p.market?.toLowerCase())))
  })

  async function loadPools() {
    if (!isClient) {
      return
    }
    try {
      state.loading = true
      const allPools = await jLendClient.value?.sdk.getAllPools()
      state.poolAddresses = allPools
      state.pollsData = await Promise.all(
        allPools.map(async (pool_address: string) => await preparePool(pool_address)),
      )
    } finally {
      state.loading = false
    }
  }

  async function preparePool(pool_address: string) {
    const poolInfo = await jLendClient.value?.sdk.getPoolInfo(pool_address)
    const pool_price = await jLendClient.value?.sdk.getPoolAssetOraclePrice(pool_address)
    const pool_apy = await jLendClient.value?.sdk.getPoolApy(pool_address)
    return {
      ...poolInfo,
      pool_price,
      pool_apy,
      market: 'main',
    }
  }

  async function addTrustLine(asset: string, issuer: string) {
    try {
      const res = await jLendClient.value.addTrustlineTx(wallet.publicKey, asset, issuer, connectionStore.kit)
      await wallet.loadBalances()
      return res
    } catch (error) {
      console.log(error)
      throw error
    }
  }

  async function deposit(publicKey: string, pool_address: string, amount: number, asset_code: string) {
    let toast
    try {
      poolDepositAddr.value = pool_address

      if (!amount || amount <= 0) {
        throw new Error('Amount should be greater than 0')
      }
      const asset = asset_code === 'native' ? 'XLM' : asset_code
      toast = await Toast.create({
        title: 'Deposit',
        body: `Sending transaction to deposit ${amount} ${asset}`,
        modelValue: 30_000,
        variant: 'info',
        noProgress: false,
      })

      const res = await jLendClient.value.sdk.deposit(publicKey, pool_address, amount, connectionStore.kit)

      const poolInfo = await jLendClient.value.sdk.getPoolInfo(pool_address)
      const preparedPool = await preparePool(pool_address)
      state.pollsData = state.pollsData.map(p => (p.pool_address === pool_address ? preparedPool : p))
      Toast.create({
        title: 'Deposit Success',
        body: `You deposited ${amount} ${asset} successfully`,
        modelValue: 30_000,
        alertProps: {
          variant: 'success',
        },
        actions: [
          {
            label: 'View Transaction',
            href: `https://stellar.expert/explorer/testnet/tx/${res.txHash}`,
          },
        ],
      })
      console.log('POOL_INFO', poolInfo)
    } catch (error: any) {
      const message = error?.message || error
      Toast.create({
        title: 'Deposit Error',
        body: String(message),
        alertProps: {
          variant: 'error',
        },
      })
      throw error
    } finally {
      poolDepositAddr.value = undefined
      toast?.dismiss()
    }
  }

  onMounted(async () => {
    await loadPools()
  })

  return {
    state,

    selectedMarket,
    selectedMarketInfo,
    selectedMarketPools,

    addTrustLine,
    deposit,
    poolDepositAddr,
  }
})

export type MarketsState = {
  poolAddresses: string[]
  pollsData: PoolWithPrice[]
  loading: boolean
  markets: string[]
}

export type PoolWithPrice = {
  pool_price: number
  pool_apy: CompoundRates
  market?: string
} & Pool
