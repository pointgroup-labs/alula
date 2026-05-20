import type { ObligationArray } from '@alula/client-sdk'
import type { MultiplyPositionItem } from '~/types/table'
import { truncatePercent } from '~/utils'

type MyPositionState = {
  position: ComputedRef<MultiplyPositionItem | undefined>
  isLoadingPosition: Ref<boolean>
  selectedVault: any
  obligation: ComputedRef<ObligationArray | undefined>
  apyDisplay: ComputedRef<string>
  healthIndicatorStyle: ComputedRef<{ '--indicator-width': string, '--indicator-color': string }>
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

  const selectedPosition = computed(() => {
    if (!selectedVault.value || !obligation.value) {
      return
    }

    return multiplyStore.positions.find(position => position.market === selectedVault.value!.market && position.pairKey === selectedVault.value!.pairKey)
  })

  const hasPosition = computed(() => {
    return !!selectedPosition.value
  })

  const position = computed(() => selectedPosition.value)

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
