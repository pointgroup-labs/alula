import type { CloseMultiplyPreview } from '@alula/client-sdk/src/services/multiply-service'
import type { ComputedRef, Ref } from 'vue'
import type { MultiplyAccountTableItem, MultiplyTableItem, MultiplyVaultItem } from '~/types/table'
import { SOROSWAP_PROVIDER_ADDRESS } from '@alula/client-sdk'
import { calculateTotalStake } from '@alula/client-sdk/src/utils'
import { CLEAR_DIALOG_TIMEOUT, RELOAD_FEE_INTERVAL } from '~/config'
import { buildMultiplyObligationKey } from '~/utils/obligation'

type MultiplyWithdrawItem = MultiplyTableItem | MultiplyAccountTableItem | MultiplyVaultItem
type BooleanRef = Ref<boolean> | ComputedRef<boolean>
type MultiplyWithdrawItemRef = Ref<MultiplyWithdrawItem | undefined> | ComputedRef<MultiplyWithdrawItem | undefined>

export function useMultiplyWithdraw(isOpen: BooleanRef, dataRef: MultiplyWithdrawItemRef) {
  const marketsStore = useMarketsStore()
  const market = useMarketActions()
  const userStore = useUserStore()
  const { publicKey } = useWalletComposable()

  const opened = computed(() => unref(isOpen))
  const data = computed(() => unref(dataRef))

  const amount = toRef(market, 'withdrawAmount')
  const reloadFee = ref(false)
  const preview = ref<CloseMultiplyPreview>()
  const previewError = ref('')
  const previewLoading = ref(false)
  const txFee = ref(0)
  const loading = ref(false)
  let interval: string | number | NodeJS.Timeout | undefined

  const activeMarket = computed(() => marketsStore.state.markets[String(data.value?.market)])

  const swapPath = computed(() => {
    if (!data.value) {
      return []
    }

    return [
      data.value.depositPoolData.pool.token_address,
      data.value.borrowPoolData.pool.token_address,
    ]
  })

  const obligationKey = computedAsync(async () => {
    if (!data.value || !publicKey.value) {
      return
    }

    return await buildMultiplyObligationKey({
      publicKey: publicKey.value,
      borrowTokenAddress: data.value.borrowPoolData.pool.token_address,
      depositTokenAddress: data.value.depositPoolData.pool.token_address,
    })
  })

  const balance = computed(() => {
    if (!preview.value || !data.value) {
      return 0
    }

    return Number(bigintToNumber(preview.value.maxRepayAmount, data.value.borrowPoolData.pool.token_decimals)) || 0
  })

  const currentDeposited = computed(() => {
    if (!data.value) {
      return 0
    }

    if (preview.value) {
      return Number(bigintToNumber(preview.value.currentDepositAmount, data.value.depositPoolData.pool.token_decimals)) || 0
    }

    const deposits: any = userStore.state.multiplyObligations[String(data.value.market)]?.[data.value.pairKey]?.deposits || []
    const depositAsset = deposits.find(([deposit]: any) => deposit === data.value?.depositPoolData.pool.pool_address)

    if (!depositAsset?.[1]?.j_tokens) {
      return 0
    }

    return Number(calculateTotalStake(depositAsset[1].j_tokens, {
      total_j_tokens: data.value.depositPoolData.pool.total_j_tokens,
      total_borrowed: data.value.depositPoolData.pool.total_borrowed,
      total_available: data.value.depositPoolData.total_available_adjusted,
    }).toString()) || 0
  })

  const swapInputEstimate = computed(() => {
    if (!preview.value || !data.value) {
      return 0
    }

    return Number(bigintToNumber(preview.value.requiredAmountIn, data.value.depositPoolData.pool.token_decimals)) || 0
  })

  const estimatedReceiveAmount = computed(() => {
    if (!preview.value || !data.value) {
      return 0
    }

    return Number(bigintToNumber(preview.value.estimatedReceiveAmount, data.value.depositPoolData.pool.token_decimals)) || 0
  })

  const debtRepaidAmount = computed(() => {
    if (!preview.value || !data.value) {
      return 0
    }

    return Number(bigintToNumber(preview.value.debtRepaidAmount, data.value.borrowPoolData.pool.token_decimals)) || 0
  })

  const remainingBorrowAmount = computed(() => {
    if (!preview.value || !data.value) {
      return 0
    }

    return Number(bigintToNumber(preview.value.remainingBorrowAmount, data.value.borrowPoolData.pool.token_decimals)) || 0
  })

  const remainingDepositAmount = computed(() => {
    if (!preview.value || !data.value) {
      return 0
    }

    return Number(bigintToNumber(preview.value.remainingDepositAmount, data.value.depositPoolData.pool.token_decimals)) || 0
  })

  const marketFee = computed(() => {
    if (!preview.value || !data.value) {
      return 0
    }

    return Number(bigintToNumber(
      preview.value.flashRepaymentAmount - preview.value.flashBorrowAmount,
      data.value.borrowPoolData.pool.token_decimals,
    )) || 0
  })

  function clearState() {
    amount.value = undefined
    preview.value = undefined
    previewError.value = ''
    txFee.value = 0
  }

  function clearReloadInterval() {
    clearInterval(interval)
    interval = undefined
  }

  async function loadPreview() {
    if (!opened.value || !data.value || !activeMarket.value?.client || !obligationKey.value) {
      preview.value = undefined
      previewError.value = ''
      txFee.value = 0
      return
    }

    previewLoading.value = true
    previewError.value = ''

    try {
      const multiplyService = activeMarket.value.client.multiply
      const result = await multiplyService.getClosePositionPreview({
        user: obligationKey.value,
        depositPoolAddress: data.value.depositPoolData.pool.pool_address,
        borrowPoolAddress: data.value.borrowPoolData.pool.pool_address,
        repayAmount: amount.value && amount.value > 0 ? Number(amount.value) : undefined,
        swapProviderAddress: SOROSWAP_PROVIDER_ADDRESS,
        path: swapPath.value,
      })

      preview.value = result

      if (!amount.value) {
        amount.value = Number(bigintToNumber(result.maxRepayAmount, data.value.borrowPoolData.pool.token_decimals)) || 0
      }
    } catch (error: any) {
      preview.value = undefined
      previewError.value = String(error?.message || error)
      txFee.value = 0
    } finally {
      previewLoading.value = false
    }
  }

  async function loadTxFee() {
    if (!opened.value || !data.value || !activeMarket.value?.client || !obligationKey.value || !amount.value || amount.value <= 0) {
      txFee.value = 0
      return
    }

    try {
      const multiplyService = activeMarket.value.client.multiply
      const { tx } = await multiplyService.buildClosePositionTx({
        user: obligationKey.value,
        depositPoolAddress: data.value.depositPoolData.pool.pool_address,
        borrowPoolAddress: data.value.borrowPoolData.pool.pool_address,
        repayAmount: Number(amount.value),
        swapProviderAddress: SOROSWAP_PROVIDER_ADDRESS,
        path: swapPath.value,
      })

      txFee.value = multiplyService.getTransactionFee(tx)
    } catch {
      txFee.value = 0
    }
  }

  function reset() {
    marketsStore.dialogLeverageWithdraw = false
  }

  async function withdraw() {
    if (!publicKey.value || !data.value?.depositPoolData.pool.pool_address || !activeMarket.value?.client || !obligationKey.value) {
      return
    }
    if (!amount.value || amount.value <= 0) {
      return
    }

    try {
      loading.value = true

      await market.withdrawMultiply({
        client: activeMarket.value.client,
        market: activeMarket.value.marketState.global_state.name,
        deposit_pool_address: data.value.depositPoolData.pool.pool_address,
        borrow_pool_address: data.value.borrowPoolData.pool.pool_address,
        repay_amount: Number(amount.value),
        swap_provider: SOROSWAP_PROVIDER_ADDRESS,
        obligation_key: obligationKey.value,
        path: swapPath.value,
        action: async () => {
          await Promise.allSettled([
            userStore.updateUserMultiplyObligations(activeMarket.value!.marketState.global_state.name, activeMarket.value!.client!, false),
            marketsStore.updatePool(data.value!.depositPoolData.pool.pool_address, activeMarket.value!.marketState.global_state.name, activeMarket.value!.client!, false),
            marketsStore.updatePool(data.value!.borrowPoolData.pool.pool_address, activeMarket.value!.marketState.global_state.name, activeMarket.value!.client!, false),
          ])
        },
        reset,
      })
    } finally {
      loading.value = false
    }
  }

  watch(() => opened.value, async (isDialogOpen) => {
    clearReloadInterval()

    if (!isDialogOpen) {
      setTimeout(() => {
        clearState()
      }, CLEAR_DIALOG_TIMEOUT)
      return
    }

    await loadPreview()

    interval = setInterval(() => {
      reloadFee.value = true
      nextTick(() => {
        reloadFee.value = false
      })
    }, RELOAD_FEE_INTERVAL)
  }, { immediate: true })

  watchDebounced([
    () => data.value?.pairKey,
    () => opened.value,
    () => publicKey.value,
    () => amount.value,
  ], async ([pairKey, isDialogOpen, wallet]) => {
    if (!pairKey || !isDialogOpen || !wallet) {
      return
    }

    await loadPreview()
  }, { immediate: true, debounce: 250 })

  watchDebounced([
    () => amount.value,
    reloadFee,
    () => opened.value,
    () => preview.value?.maxReceivableAmount?.toString(),
  ], async ([nextAmount, _reloadFee, isDialogOpen]) => {
    if (!isDialogOpen || !nextAmount || Number(nextAmount) <= 0 || previewError.value) {
      txFee.value = 0
      return
    }

    await loadTxFee()
  }, { immediate: true, debounce: 300 })

  onScopeDispose(() => {
    clearReloadInterval()
  })

  return {
    amount,
    balance,
    currentDeposited,
    swapInputEstimate,
    estimatedReceiveAmount,
    debtRepaidAmount,
    remainingBorrowAmount,
    remainingDepositAmount,
    marketFee,
    preview,
    previewError,
    previewLoading,
    txFee,
    loading,
    withdraw,
  }
}
