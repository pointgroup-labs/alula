import type { StellarClient } from '@alula/client-sdk'
import type { ObligationArray, ObligationUI } from '@alula/client-sdk/src/types'
import type { Obligation } from '@alula/market-sdk'
import { calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk'
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
      const obligations = await client.marketSdk.getUserObligation(wallet.publicKey)
      state.obligations[market] = adaptAbligation(obligations)
      console.log(`%c[${market} market User Obligation]`, 'color: #FFB726', state.obligations[market])
    } finally {
      loading.value = false
    }
  }

  async function loadUserMultiplyObligation(props: {
    client: StellarClient
    market: string
    depositPoolAddress: string
    borrowPoolAddress: string
  }) {
    try {
      loading.value = true
      const { client, market, depositPoolAddress, borrowPoolAddress } = props
      const obligations = await client.marketSdk.getUserMultiplyObligation(wallet.publicKey, depositPoolAddress, borrowPoolAddress)
      state.multiplyObligations[market] = adaptAbligation(obligations)
      console.log(`%c[${market} market Multiply Obligation]`, 'color: #FFB726', state.multiplyObligations[market])
    } finally {
      loading.value = false
    }
  }

  async function updateUserObligation(market: string, client: StellarClient, withLogs = true) {
    try {
      loading.value = true
      if (!client) {
        return
      }
      const obligation = await client.marketSdk.getUserObligation(wallet.publicKey)
      state.obligations[market] = adaptAbligation(obligation)
      if (withLogs) {
        console.log(`%c[Update ${market} market Obligation]`, 'color: #FFB726', state.obligations[market])
      }
    } finally {
      loading.value = false
    }
  }

  async function updateUserMultiplyObligation({
    market,
    depositPoolAddress,
    borrowPoolAddress,
    client,
    withLogs = true,
  }: {
    market: string
    depositPoolAddress: string
    borrowPoolAddress: string
    client?: StellarClient
    withLogs?: boolean
  }) {
    try {
      loading.value = true
      if (!client) {
        return
      }
      const obligation = await client.marketSdk.getUserMultiplyObligation(wallet.publicKey, depositPoolAddress, borrowPoolAddress)
      state.multiplyObligations[market] = adaptAbligation(obligation)
      if (withLogs) {
        console.log(`%c[Update ${market} market Leverage Obligation]`, 'color: #FFB726', state.multiplyObligations[market])
      }
    } finally {
      loading.value = false
    }
  }

  const userTotalDepositInUsd = computed(() => {
    const obligation = state.obligations[marketsStore.selectedMarketName]
    const marketState = activeMarket.value?.marketState

    if (!obligation || !marketState) {
      return 0
    }
    const assetDecimals = marketState.asset_decimals
    const oraclePriceDecimals = marketState.oracle_price_decimals
    const poolsData = marketState.pools_data

    return calcUserTotalStakeInUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals) ?? 0
  })

  const userTotalBorrowedInUsd = computed(() => {
    const obligation = state.obligations[marketsStore.selectedMarketName]
    const marketState = activeMarket.value?.marketState

    if (!obligation || !marketState) {
      return 0
    }
    const assetDecimals = marketState.asset_decimals
    const oraclePriceDecimals = marketState.oracle_price_decimals
    const poolsData = marketState.pools_data

    return calcUserTotalBorrowedInUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals) ?? 0
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

    const tasks = [
      ...Object.values(markets).map(m =>
        loadUserObligation(m.marketName, m.client),
      ),
      ...Object.values(markets).flatMap(m =>
        m.marketState.multiply_pairs.map(p =>
          loadUserMultiplyObligation({
            // TODO: remove if market name is unique
            market: m.marketName.split('_')[0]!,
            depositPoolAddress: p.deposit_pool,
            borrowPoolAddress: p.borrow_pool,
            client: m.client,
          }),
        ),
      ),
    ]

    await Promise.allSettled(tasks)
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

export type UserState = {
  obligations: ObligationUI
  multiplyObligations: ObligationUI
}

function adaptAbligation(ob?: Obligation): ObligationArray | undefined {
  if (!ob) {
    return undefined
  }
  return {
    positions_count: ob?.positions_count,
    insurance_fund_requests_ids: ob?.insurance_fund_requests_ids,
    borrows: Array.isArray(ob?.borrows)
      ? ob?.borrows as ObligationArray['borrows']
      : [...ob.borrows.entries()],

    deposits: Array.isArray(ob?.deposits)
      ? ob?.deposits as ObligationArray['deposits']
      : [...ob?.deposits.entries()],
  }
}
