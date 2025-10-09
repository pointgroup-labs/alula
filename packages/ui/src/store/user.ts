import type { StellarClient } from '@alula/client-sdk'
import type { Obligation } from '@alula/market-sdk'
import { calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'
import { defineStore } from 'pinia'

export const useUserStore = defineStore('user', () => {
  const state = reactive<UserState>({
    obligations: {},
    multiplyObligations: {},
  })

  const wallet = useWallet()
  const marketsStore = useMarketsStore()

  const activeMarket = computed(() => marketsStore.activeMarket)

  const loading = ref(false)

  async function loadUserObligation(market: string, client: StellarClient) {
    try {
      loading.value = true
      state.obligations[market] = await client.marketSdk.getUserObligation(wallet.publicKey)
      console.log(`%c[${market} market User Obligation]`, 'color: #FFB726', state.obligations[market])
    } finally {
      loading.value = false
    }
  }

  async function loadUserMultilpyObligation(props: {
    client: StellarClient
    market: string
    depositPoolAddress: string
    borrowPoolAddress: string
  }) {
    try {
      loading.value = true
      const { client, market, depositPoolAddress, borrowPoolAddress } = props
      state.multiplyObligations[market] = await client.marketSdk.getUserMultiplyObligation(wallet.publicKey, depositPoolAddress, borrowPoolAddress)
      console.log(`%c[${market} market Multiply Obligation]`, 'color: #FFB726', state.multiplyObligations[market])
    } finally {
      loading.value = false
    }
  }

  async function updateUserObligation(market: string, client: StellarClient) {
    try {
      loading.value = true
      if (!client) {
        return
      }
      state.obligations[market] = await client.marketSdk.getUserObligation(wallet.publicKey)
    } finally {
      loading.value = false
    }
  }

  async function updateUserMultiplyObligation(props: {
    market: string
    client: StellarClient
    depositPoolAddress: string
    borrowPoolAddress: string
  }) {
    try {
      loading.value = true
      if (!props.client) {
        return
      }
      state.multiplyObligations[props.market] = await props.client.marketSdk.getUserMultiplyObligation(wallet.publicKey, props.depositPoolAddress, props.borrowPoolAddress)
    } finally {
      loading.value = false
    }
  }

  const userTotalDepositInUsd = computed(() => {
    const obligation = state.obligations[marketsStore.activeMarketFilter]
    const pools = activeMarket.value?.pools
    if (!obligation || !pools) {
      return 0
    }
    return calcUserTotalStakeInUsd(obligation, pools, marketsStore.assetDecimals) ?? 0
  })

  const userTotalBorrowedInUsd = computed(() => {
    const obligation = state.obligations[marketsStore.activeMarketFilter]
    const pools = activeMarket.value?.pools
    if (!obligation || !pools) {
      return 0
    }
    return calcUserTotalBorrowedInUsd(obligation, pools, marketsStore.assetDecimals) ?? 0
  })

  watch([
    () => wallet.publicKey,
    () => marketsStore.state.markets,
  ], async ([pubkey, markets]) => {
    if (!pubkey || Object.keys(markets).length === 0) {
      state.obligations = {}
      state.multiplyObligations = {}
      return
    }
    const marketClients = Object.values(markets).map(m => m)
    const promises: Promise<any>[] = []

    for (const market of marketClients) {
      promises.push(
        loadUserObligation(market.marketState.name, market.client),
      )

      for (const p of market.leveragePools) {
        promises.push(
          loadUserMultilpyObligation({
            market: market.marketState.name,
            depositPoolAddress: p.deposit_pool,
            borrowPoolAddress: p.borrow_pool,
            client: market.client,
          }),
        )
      }
    }

    await Promise.all(promises)
  })

  return {
    state,

    loading,
    userTotalDepositInUsd,
    userTotalBorrowedInUsd,

    loadUserObligation,
    updateUserObligation,
    updateUserMultiplyObligation,
  }
})

type UserState = {
  obligations: Record<string, Obligation>
  multiplyObligations: Record<string, Obligation>
}
