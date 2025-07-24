import Decimal from 'decimal.js'
import { defineStore } from 'pinia'
import { bigintToNumber } from '~/utils'

function toDec(value: bigint): Decimal {
  return new Decimal(value.toString())
}

type Op = 'add' | 'sub' | 'mul' | 'div'
function operate(a: bigint, b: bigint, op: Op): Decimal {
  const da = toDec(a)
  const db = toDec(b)
  switch (op) {
    case 'add': return da.plus(db)
    case 'sub': return da.minus(db)
    case 'mul': return da.times(db)
    case 'div': return da.dividedBy(db)
  }
}

function calcUserTotalShares(
  shares: bigint,
  totalShares: bigint,
  available: bigint,
  totalBorrowed: bigint,
  precision = 7,
): string {
  const fraction = operate(shares, totalShares, 'div')
  const totalLiq = operate(available, totalBorrowed, 'add')
  const raw = fraction.times(totalLiq)
  const scaled = raw.dividedBy(
    new Decimal(10).pow(precision),
  )
  return scaled.toFixed(precision)
}

export const useUserStore = defineStore('user', () => {
  const wallet = useWallet()
  const marketsStore = useMarketsStore()

  const clientStore = useClientStore()
  const jLendClient = computed(() => clientStore.jLendClient)

  const userObligation = ref()
  const loading = ref(false)

  async function loadUserObligation() {
    try {
      loading.value = true
      userObligation.value = await jLendClient.value.sdk.getUserObligation(wallet.publicKey)
      console.log('%c[User Obligation]', 'color: #FFB726', userObligation.value)
    } finally {
      loading.value = false
    }
  }

  const userTotalDepositInUsd = computed(() => {
    const deposits = userObligation.value?.deposits
    if (!deposits) {
      return 0
    }

    const assetDecimals = jLendClient.value.sdk.assetDecimals

    let userDepositsInUsd = 0

    for (const deposit of deposits) {
      const [depositedPoolAddress, data] = deposit
      const depositedPool = marketsStore.state.pools?.find(p => p.pool_address === depositedPoolAddress)

      const collateral = data?.collateral || 0
      userDepositsInUsd += Number(bigintToNumber(collateral, assetDecimals)) * Number(depositedPool?.pool_price)
      const userShares = data?.shares
      if (!depositedPool || !userShares) {
        userDepositsInUsd += 0
        continue
      }
      const userAvailable = calcUserTotalShares(userShares, depositedPool.total_shares, depositedPool?.available, depositedPool?.total_borrowed, assetDecimals)
      const availableInUsd = Number(userAvailable) * Number(depositedPool.pool_price)
      userDepositsInUsd += availableInUsd || 0
    }
    return userDepositsInUsd
  })

  const userTotalBorrowedInUsd = computed(() => {
    const borrows = userObligation.value?.borrows
    if (!borrows) {
      return 0
    }

    const assetDecimals = jLendClient.value.sdk.assetDecimals

    let userBorrowedInUsd = 0

    for (const borrow of borrows) {
      const [borrowedPoolAddress, data] = borrow
      const borrowedPool = marketsStore.state.pools?.find(p => p.pool_address === borrowedPoolAddress)

      const userBorrow = bigintToNumber(data?.borrowed, assetDecimals)
      if (!borrowedPool || !userBorrow) {
        userBorrowedInUsd += 0
        continue
      }
      const borrowedInUsd = Number(userBorrow) * Number(borrowedPool.pool_price)
      userBorrowedInUsd += borrowedInUsd || 0
    }
    return userBorrowedInUsd
  })

  watch(() => wallet.publicKey, async (p) => {
    if (!p) {
      userObligation.value = undefined
      return
    }
    await loadUserObligation()
  })

  return {
    loading,
    userObligation,
    userTotalDepositInUsd,
    userTotalBorrowedInUsd,

    loadUserObligation,

  }
})
