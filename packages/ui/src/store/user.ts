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
      const obligation = await client.marketSdk.getUserObligation(wallet.publicKey)
      if (obligation) {
        state.obligations[market] = obligation
        console.log(`%c[${market} market User Obligation]`, 'color: #FFB726', obligation)
      }
    } finally {
      loading.value = false
    }
  }

  async function loadUserMultilpyObligation(props: {
    market: string
    depositPoolAddress: string
    borrowPoolAddress: string
    client: StellarClient
  }) {
    try {
      loading.value = true
      const obligation = await props.client.marketSdk.getUserMultiplyObligation(wallet.publicKey, props.depositPoolAddress, props.borrowPoolAddress)
      if (obligation) {
        state.multiplyObligations[props.market] = obligation
        console.log(`%c[${props.market} market Multiply Obligation]`, 'color: #FFB726', obligation)
      }
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
      const obligation = await client.marketSdk.getUserObligation(wallet.publicKey)
      if (obligation) {
        state.obligations[market] = obligation
        console.log(`%c[Update ${market} market User Obligation]`, 'color: #FFB726', obligation)
      }
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
      const obligation = await props.client.marketSdk.getUserMultiplyObligation(wallet.publicKey, props.depositPoolAddress, props.borrowPoolAddress)
      if (obligation) {
        state.multiplyObligations[props.market] = obligation
        console.log(`%c[Update ${props.market} market multiply Obligation]`, 'color: #FFB726', obligation)
      }
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
      return
    }
    const marketClients = Object.values(markets).map(m => m)
    await Promise.all(
      marketClients.map(async (market) => {
        await loadUserObligation(market.marketState.name, market.client)
        market.leveragePools.map(async p =>
          await loadUserMultilpyObligation({
            market: market.marketState.name,
            depositPoolAddress: p.deposit_pool,
            borrowPoolAddress: p.borrow_pool,
            client: market.client,
          }),
        )
      }),
    )
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
