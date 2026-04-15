import type { ObligationArray } from '@alula/client-sdk'
import { bpsToNumber } from '@alula/client-sdk'
import { calculateBorrow, calculateTotalStake } from '@alula/client-sdk/src/utils'
import { calculateCurrentMultiplier, truncatePercent } from '~/utils'

type LeveragePosition = {
  deposited: number
  borrowed: number
  depositedUsd: number
  borrowedUsd: number
  equityUsd: number
  currentMultiplier: number
  supplyApy: number
  borrowApy: number
  currentApy: number
  healthFactor: number
  currentLtv: number
  openLtv: number
  closeLtv: number
  liabilityFactor: number
  yearlyResultUsd: number
  liquidationBufferUsd: number
}

type MyPositionState = {
  position: ComputedRef<LeveragePosition | undefined>
  isLoadingPosition: Ref<boolean>
  selectedVault: any
  obligation: ComputedRef<ObligationArray | undefined>
  apyDisplay: ComputedRef<string>
  healthIndicatorStyle: ComputedRef<{ '--indicator-width': string; '--indicator-color': string }>
  hasPosition: ComputedRef<boolean>
  openMultiply: () => void
  closeMultiply: () => void
}

const LEVERAGE_POSITION_KEY = 'leveragePosition'

export function createLeveragePorisionState(): MyPositionState {
  const marketsStore = useMarketsStore()
  const userStore = useUserStore()
  const multiplyStore = useMultiplyStore()

  const selectedVault = computed(() => multiplyStore.selectedVault)
  const dialogLeverage = toRef(marketsStore, 'dialogLeverage')
  const dialogLeverageWithdraw = toRef(marketsStore, 'dialogLeverageWithdraw')

  const isLoadingPosition = computed(() => userStore.loading || marketsStore.state.loading)

  const obligation = computed(() => {
    if (!selectedVault.value) {
      return
    }

    return userStore.state.multiplyObligations[selectedVault.value.market]?.[selectedVault.value.pairKey]
  })

  const hasPosition = computed(() => {
    if (!selectedVault.value || !obligation.value) {
      return false
    }

    const deposits: any[] = obligation.value.deposits ?? []
    const borrows: any[] = obligation.value.borrows ?? []

    if (deposits.length === 0 || borrows.length === 0) {
      return false
    }

    const hasDeposit = deposits.some(deposit => deposit.includes(selectedVault.value!.depositPoolData.pool.pool_address))
    const hasBorrow = borrows.some(borrow => borrow.includes(selectedVault.value!.borrowPoolData.pool.pool_address))

    return hasDeposit && hasBorrow
  })

  const position = computed(() => {
    if (!selectedVault.value || !hasPosition.value || !obligation.value) {
      return
    }

    const marketState = marketsStore.state.markets[selectedVault.value.market]?.marketState
    const oraclePriceDecimals = marketState?.oracle_price_decimals ?? 0
    const depositPoolData = selectedVault.value.depositPoolData
    const borrowPoolData = selectedVault.value.borrowPoolData
    const depositObligation = obligation.value.deposits?.find(([address]) => address === depositPoolData.pool.pool_address)
    const borrowObligation = obligation.value.borrows?.find(([address]) => address === borrowPoolData.pool.pool_address)

    if (!depositObligation || !borrowObligation) {
      return
    }

    const [, depositData] = depositObligation
    const [, borrowData] = borrowObligation

    const deposited = +calculateTotalStake(depositData.j_tokens, {
      total_j_tokens: depositPoolData.pool.total_j_tokens,
      total_borrowed: depositPoolData.pool.total_borrowed,
      total_available: depositPoolData.total_available_adjusted,
    }) || 0

    const borrowed = +calculateBorrow(borrowData.d_tokens, {
      total_borrowed: borrowPoolData.pool.total_borrowed,
      total_d_tokens: borrowPoolData.pool.total_d_tokens,
    }, borrowPoolData.pool.token_decimals) || 0

    const depositPrice = Number(bigintToNumber(depositPoolData.oracle_asset_price, oraclePriceDecimals)) || 0
    const borrowPrice = Number(bigintToNumber(borrowPoolData.oracle_asset_price, oraclePriceDecimals)) || 0
    const depositedUsd = deposited * depositPrice
    const borrowedUsd = borrowed * borrowPrice
    const equityUsd = Math.max(depositedUsd - borrowedUsd, 0)
    const currentMultiplier = calculateCurrentMultiplier(deposited, depositPrice, borrowed, borrowPrice) || 0
    const supplyApy = bpsToNumber(Number(depositPoolData.apy.supply_bps || 0)) * 100
    const borrowApy = bpsToNumber(Number(borrowPoolData.apy.borrow_bps || 0)) * 100
    const currentApy = supplyApy * currentMultiplier - borrowApy * Math.max(currentMultiplier - 1, 0)
    const healthFactor = calculatePositionHealthFactor({
      deposited,
      depositPrice,
      closeLtvBps: Number(depositPoolData.pool.config.health_config.close_ltv_bps || 0),
      borrowed,
      borrowPrice,
      liabilityFactorBps: Number(borrowPoolData.pool.config.health_config.liability_factor_bps || 0),
    })
    const currentLtv = depositedUsd > 0 ? (borrowedUsd / depositedUsd) * 100 : 0
    const openLtv = bpsToNumber(Number(depositPoolData.pool.config.health_config.open_ltv_bps || 0)) * 100
    const closeLtv = bpsToNumber(Number(depositPoolData.pool.config.health_config.close_ltv_bps || 0)) * 100
    const liabilityFactor = bpsToNumber(Number(borrowPoolData.pool.config.health_config.liability_factor_bps || 0)) * 100
    const yearlyResultUsd = depositedUsd * (supplyApy / 100) - borrowedUsd * (borrowApy / 100)
    const liquidationBufferUsd = Math.max(
      depositedUsd * bpsToNumber(Number(depositPoolData.pool.config.health_config.close_ltv_bps || 0))
      - borrowedUsd * bpsToNumber(Number(borrowPoolData.pool.config.health_config.liability_factor_bps || 0)),
      0,
    )

    return {
      deposited,
      borrowed,
      depositedUsd,
      borrowedUsd,
      equityUsd,
      currentMultiplier,
      supplyApy,
      borrowApy,
      currentApy,
      healthFactor,
      currentLtv,
      openLtv,
      closeLtv,
      liabilityFactor,
      yearlyResultUsd,
      liquidationBufferUsd,
    }
  })

  const apyDisplay = computed(() => truncatePercent(position.value?.currentApy || 0, 2))

  const healthIndicatorStyle = computed(() => ({
    '--indicator-width': `${Math.min(Math.max(((position.value?.healthFactor || 0) - 1) * 100, 0), 100)}%`,
    '--indicator-color': healthFactorColor(position.value?.healthFactor || 0),
  }))

  function openMultiply() {
    dialogLeverage.value = true
  }

  function closeMultiply() {
    dialogLeverageWithdraw.value = true
  }

  return {
    isLoadingPosition,
    selectedVault,
    position,
    obligation,
    apyDisplay,
    healthIndicatorStyle,
    hasPosition,
    openMultiply,
    closeMultiply,
  }
}

export function provideLeveragePosition() {
  const state = createLeveragePorisionState()
  provide(LEVERAGE_POSITION_KEY, state)
  return state
}

export function useLeveragePosition() {
  const state = inject<MyPositionState>(LEVERAGE_POSITION_KEY)
  if (!state) {
    throw new Error('My leverage position state was not provided')
  }
  return state
}

function calculatePositionHealthFactor(params: {
  deposited: number
  depositPrice: number
  closeLtvBps: number
  borrowed: number
  borrowPrice: number
  liabilityFactorBps: number
}) {
  const weightedDepositUsd = params.deposited * params.depositPrice * bpsToNumber(params.closeLtvBps)
  const weightedBorrowUsd = params.borrowed * params.borrowPrice * bpsToNumber(params.liabilityFactorBps)

  if (weightedBorrowUsd <= 0) {
    return 10
  }

  return Math.min(weightedDepositUsd / weightedBorrowUsd, 10)
}
