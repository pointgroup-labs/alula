export const useRecentActivityStore = defineStore('recent-activity', () => {
  const wallet = useWallet()
  const rpcStore = useRpcStore()

  const state = reactive<RecentActivityState>({
    records: [],
    loading: false,
  })

  function fetchLink(limit = 100) {
    return `${rpcStore.horizonRPCUrl}/accounts/${wallet.publicKey}/operations?limit=${limit}&order=desc`
  }

  async function fetchTxs() {
    try {
      state.loading = true
      const resp = await fetch(fetchLink())

      if (!resp.ok) {
        return
      }

      const data = await resp.json()
      state.records = data._embedded.records
      console.log('%c[Recent Activity]', 'color: #FFB726', state.records)
    } catch (error) {
      console.error('Failed to load operations', error)
      state.records = []
    } finally {
      state.loading = false
    }
  }

  async function fetchAndUpdateLastTx() {
    try {
      state.loading = true
      const resp = await fetch(fetchLink(1))

      if (!resp.ok) {
        return
      }

      const data = await resp.json()
      state.records = [data._embedded.records[0], ...state.records]
      console.log('%c[Update Recent Activity]', 'color: #FFB726', data._embedded.records[0])
    } catch (error) {
      console.error('Failed to load operations', error)
      state.records = []
    } finally {
      state.loading = false
    }
  }

  watch(
    [() => wallet.publicKey, () => rpcStore.network],
    async ([pk]) => {
      state.records = []
      if (!pk) {
        return
      }

      await fetchTxs()
    },
  )

  return {
    state,
    fetchTxs,
    fetchAndUpdateLastTx,
  }
})

export type RecentActivityState = {
  records: OperationRecord[]
  loading: boolean
}

export type OperationRecord = {
  id: string
  type: string
  transaction_hash: string
  created_at: string
  transaction_successful: boolean
  amount?: string
  asset_code?: string
}

export function getTxActionLabel(op: OperationRecord) {
  switch (op.type) {
    case 'payment':
      return 'Payment'

    case 'path_payment_strict_send':
    case 'path_payment_strict_receive':
      return 'Swap'

    case 'create_account':
      return 'Create account'

    case 'account_merge':
      return 'Account merge'

    case 'change_trust':
      return 'Trustline'

    case 'manage_buy_offer':
    case 'manage_sell_offer':
      return 'Trade'

    case 'invoke_host_function':
      return 'Contract call'

    default:
      return 'Operation'
  }
}
