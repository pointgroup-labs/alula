import type { AlulaClient } from '@alula/client-sdk'
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
  const multiplyStore = useMultiplyStore()

  const loading = ref(false)

  async function loadUserObligation(market: string, client: AlulaClient) {
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

  async function loadUserMultiplyObligations(market: string, client: AlulaClient, withLogs = true) {
    try {
      loading.value = true
      if (!client || !publicKey.value) {
        state.multiplyObligations[market] = {}
        return
      }

      const obligationsByPair: Record<string, ObligationArray | undefined> = {}

      const marketVaults = multiplyStore.vaults
        .filter(vault => vault.market === market)
        .filter((vault, index, vaults) => vaults.findIndex(item => item.pairKey === vault.pairKey) === index)

      await Promise.allSettled(marketVaults.map(async (vault) => {
        const pairKey = buildMultiplyPairKey(
          vault.depositPoolData.pool.pool_address,
          vault.borrowPoolData.pool.pool_address,
        )

        try {
          const obligationKey = await buildMultiplyObligationKey({
            publicKey: publicKey.value!,
            borrowTokenAddress: vault.borrowPoolData.pool.token_address,
            depositTokenAddress: vault.depositPoolData.pool.token_address,
          })

          const obligation = await client.obligation.getUserObligation(obligationKey)
          obligationsByPair[pairKey] = adaptAbligation(obligation)
        } catch {
          obligationsByPair[pairKey] = undefined
        }
      }))

      state.multiplyObligations[market] = obligationsByPair

      if (withLogs) {
        console.log(`%c[${market} market User Multiply Obligations]`, 'color: #FFB726', state.multiplyObligations[market])
      }
    } finally {
      loading.value = false
    }
  }

  async function updateUserObligation(market: string, client: AlulaClient, withLogs = true) {
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

  async function updateUserMultiplyObligations(market: string, client: AlulaClient, withLogs = true) {
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
      loadUserMultiplyObligations(m.marketName, m.client!),
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
