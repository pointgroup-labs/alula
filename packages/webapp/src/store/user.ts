import type { StellarClient } from '@alula/client-sdk'
import type { ObligationArray, ObligationUI } from '@alula/client-sdk/src/types'
import type { Obligation } from '@alula/market-sdk'
import { defineStore } from 'pinia'
import { buildMultiplyObligationKey, buildMultiplyPairKey } from '~/utils/obligation'

export const useUserStore = defineStore('user', () => {
  const state = reactive<UserState>({
    obligations: {},
    multiplyObligations: {},
  })

  const { publicKey } = useWalletComposable()

  const marketsStore = useMarketsStore()

  const loading = ref(false)

  async function loadUserObligation(market: string, client: StellarClient) {
    try {
      loading.value = true
      const oblKey = buildObligationKey({ pablicKey: publicKey.value })
      const obligations = await client.obligation.getUserObligation(oblKey)
      state.obligations[market] = adaptAbligation(obligations)
      console.log(`%c[${market} market User Obligation]`, 'color: #FFB726', state.obligations[market])
    } finally {
      loading.value = false
    }
  }

  async function loadUserMultiplyObligations(market: string, client: StellarClient, withLogs = true) {
    try {
      loading.value = true
      if (!client || !publicKey.value) {
        state.multiplyObligations[market] = {}
        return
      }

      const marketEntry = marketsStore.state.markets[market]
      const poolsData = marketEntry?.marketState?.pools_data ?? []
      const obligationsByPair: Record<string, ObligationArray | undefined> = {}

      const tasks = poolsData.flatMap((depositPoolData) => {
        return poolsData.map(async (borrowPoolData) => {
          if (!depositPoolData || !borrowPoolData) {
            return
          }
          if (depositPoolData.pool.pool_address === borrowPoolData.pool.pool_address) {
            return
          }
          if (depositPoolData.pool.token_address === borrowPoolData.pool.token_address) {
            return
          }

          const openLtvBps = Number(depositPoolData.pool.config.health_config.open_ltv_bps)
          if (openLtvBps <= 0) {
            return
          }

          const pairKey = buildMultiplyPairKey(
            depositPoolData.pool.pool_address,
            borrowPoolData.pool.pool_address,
          )

          try {
            const obligationKey = await buildMultiplyObligationKey({
              publicKey: publicKey.value,
              borrowTokenAddress: borrowPoolData.pool.token_address,
              depositTokenAddress: depositPoolData.pool.token_address,
            })
            const obligation = await client.obligation.getUserObligation(obligationKey)

            obligationsByPair[pairKey] = adaptAbligation(obligation)
          } catch {
            obligationsByPair[pairKey] = undefined
          }
        })
      })

      await Promise.allSettled(tasks)
      state.multiplyObligations[market] = obligationsByPair

      if (withLogs) {
        console.log(`%c[${market} market User Multiply Obligations]`, 'color: #FFB726', state.multiplyObligations[market])
      }
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
      const oblKey = buildObligationKey({ pablicKey: publicKey.value })
      const obligation = await client.obligation.getUserObligation(oblKey)
      state.obligations[market] = adaptAbligation(obligation)
      if (withLogs) {
        console.log(`%c[Update ${market} market Obligation]`, 'color: #FFB726', state.obligations[market])
      }
    } finally {
      loading.value = false
    }
  }

  async function updateUserMultiplyObligations(market: string, client: StellarClient, withLogs = true) {
    await loadUserMultiplyObligations(market, client, withLogs)
  }

  watchDebounced([
    () => publicKey.value,
    () => marketsStore.state.markets,
  ], async ([pubkey, markets]) => {
    if (!pubkey || Object.keys(markets).length === 0) {
      state.obligations = {}
      state.multiplyObligations = {}
      return
    }

    const tasks = Object.values(markets).flatMap(m => [
      loadUserObligation(m.marketName, m.client!),
      loadUserMultiplyObligations(m.marketName, m.client!, false),
    ])

    await Promise.allSettled(tasks)
  }, { debounce: 500 })

  return {
    state,

    loading,

    loadUserObligation,
    loadUserMultiplyObligations,
    updateUserObligation,
    updateUserMultiplyObligations,
  }
})

export type MultiplyObligationUI = Record<string, Record<string, ObligationArray | undefined>>

export type UserState = {
  obligations: ObligationUI
  multiplyObligations: MultiplyObligationUI
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
