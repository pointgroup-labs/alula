import type { CompoundRates, Pool } from '@jlend/sdk'
import { defineStore } from 'pinia'

const MAIN_MARKET_NAME = 'Main market'

export const useMarketsStore = defineStore('markets', () => {
  const state = reactive<MarketsState>({
    poolAddresses: [],
    pools: [],
    leveragePools: [],
    loading: false,
    loadingLeveragePools: false,
    markets: [MAIN_MARKET_NAME, 'Assets'],
  })

  const route = useRoute()
  const router = useRouter()

  const clientStore = useClientStore()
  const alulaClient = computed(() => clientStore.alulaClient)

  const rpcStore = useRpcStore()
  const network = computed(() => rpcStore.network)

  const poolActiveAddress = ref()
  const poolActionType = ref<TableActionType>()

  const dialogSupply = ref(false)
  const dialogBorrow = ref(false)
  const dialogLeverage = ref(false)
  const dialogWithdrawLeverage = ref(false)

  // selected pool address to show market info in supply/borrow dialogs
  const selectedMarketAddress = ref()

  const marketInfoDialog = ref(false)

  const activeMarket = ref<string>(MAIN_MARKET_NAME)

  const selectedMarketPools = computed(() => {
    return state.pools.filter(p => activeMarket.value.toLowerCase().includes(String(p.market?.toLowerCase())))
  })

  async function loadPools() {
    if (!isClient) {
      return
    }
    try {
      state.loading = true
      const allPools = await alulaClient.value?.sdk.getAllPools()
      state.poolAddresses = allPools
      state.pools = await Promise.all(
        allPools.map(async (pool_address: string) => await preparePool(pool_address)),
      )
      console.log('%c[Pools]', 'color: #FFB726', state.pools)
    } finally {
      state.loading = false
    }
  }

  async function loadLeveragePools() {
    if (!isClient) {
      return
    }
    try {
      state.loadingLeveragePools = true
      const allPools = await alulaClient.value?.sdk.getAllLeveragePools()
      state.leveragePools = allPools || []
      console.log('%c[Leverage Pools]', 'color: #FFB726', allPools)
    } finally {
      state.loadingLeveragePools = false
    }
  }

  async function preparePool(pool_address: string) {
    const [poolInfo, pool_price, pool_apy] = await Promise.all([
      alulaClient.value?.sdk.getPoolInfo(pool_address),
      alulaClient.value?.sdk.getPoolAssetOraclePrice(pool_address),
      alulaClient.value?.sdk.getPoolApy(pool_address),
    ])
    return {
      ...poolInfo,
      pool_price,
      pool_apy,
      market: 'main',
    }
  }

  async function updatePools(pool_address: string) {
    const preparedPool = await preparePool(pool_address)
    state.pools = state.pools.map(p => (p.pool_address === pool_address ? preparedPool : p))
  }

  async function loadPoolsData() {
    state.poolAddresses = []
    state.pools = []
    state.leveragePools = []

    await Promise.all([
      loadPools(),
      loadLeveragePools(),
    ])
  }

  watch(network, async () => {
    await loadPoolsData()
  })

  watch([
    () => route.path,
    activeMarket,
  ], ([path, marketName]) => {
    if (path !== '/') {
      return
    }
    const q = { ...route.query }

    if (marketName === MAIN_MARKET_NAME) {
      delete q['active-market']
    } else {
      q['active-market'] = marketName
    }
    router.replace({ query: { ...q } })
  })

  watch([
    dialogSupply,
    dialogBorrow,
    marketInfoDialog,
    dialogLeverage,
    dialogWithdrawLeverage,
  ], ([supply, borrow, infoDialog, leverage, withdrawLeverage]) => {
    const market = selectedMarketAddress.value
    const query = { ...route.query }

    const map: Record<string, boolean> = {
      supply,
      borrow,
      'market-info': infoDialog,
      leverage,
      'withdraw-leverage': withdrawLeverage,
    }

    const active = Object.entries(map).find(([, v]) => v)?.[0]

    if (active) {
      router.replace({ query: { ...query, dialog: active, market } })
    } else {
      delete query.dialog
      delete query.market
      delete query['collateral-only']
      router.replace({ query })
    }
  })

  const stop = watch(selectedMarketPools, (pools) => {
    if (pools?.length > 0) {
      const q = route.query
      selectedMarketAddress.value = q?.market

      if (!pools.some(p => p.pool_address === q?.market) || !selectedMarketAddress.value) {
        stop()
        return
      }
      if (q.dialog === 'supply') {
        dialogSupply.value = true
      }
      if (q.dialog === 'borrow') {
        dialogBorrow.value = true
      }
      if (q.dialog === 'leverage') {
        dialogLeverage.value = true
      }
      if (q.dialog === 'withdraw-leverage') {
        dialogWithdrawLeverage.value = true
      }
      if (q.dialog === 'market-info') {
        marketInfoDialog.value = true
      }
      stop()
    }
  })

  onMounted(() => {
    const activeMarketQuery = route.query?.['active-market']
    if (activeMarketQuery) {
      activeMarket.value = String(activeMarketQuery)
    }
  })

  return {
    state,

    activeMarket,

    selectedMarketPools,

    dialogSupply,
    dialogBorrow,
    dialogLeverage,
    marketInfoDialog,
    dialogWithdrawLeverage,

    selectedMarketAddress,

    poolActionType,
    poolActiveAddress,

    updatePools,

  }
})

export type MarketsState = {
  poolAddresses: string[]
  pools: PoolWithPrice[]
  leveragePools: LeveragePool[]
  loading: boolean
  loadingLeveragePools: boolean
  markets: string[]
}

export type PoolWithPrice = {
  pool_price: number | string
  pool_apy: CompoundRates
  market?: string
} & Pool

export type LeveragePool = {
  borrow_pool: string
  deposit_pool: string
}

export type TableActionType = 'deposit' | 'withdraw' | 'borrow' | 'repay' | 'leverage' | 'withdrawLeverage'
