import type { CloseMultiplyPreview, MultiplyMarginAsset } from '@alula/client-sdk/src/services/multiply'
import type { ComputedRef, Ref } from 'vue'
import type { MultiplyAccountTableItem, MultiplyTableItem, MultiplyVaultItem } from '~/types/table'
import { SOROSWAP_PROVIDER_ADDRESS } from '@alula/client-sdk'
import { calculateTotalStake } from '@alula/client-sdk/src/utils'
import { CLEAR_DIALOG_TIMEOUT, RELOAD_FEE_INTERVAL } from '~/config'
import { buildMultiplyObligationKey } from '~/utils/obligation'

type MultiplyWithdrawItem = MultiplyTableItem | MultiplyAccountTableItem | MultiplyVaultItem
type BooleanRef = Ref<boolean> | ComputedRef<boolean>
type MultiplyWithdrawItemRef = Ref<MultiplyWithdrawItem | undefined> | ComputedRef<MultiplyWithdrawItem | undefined>

function amountTolerance(decimals: number) {
  return 1 / 10 ** Math.min(decimals, 6)
}

function clampNumber(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value))
}

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
  const fullClosePreviews = ref<Partial<Record<MultiplyMarginAsset, CloseMultiplyPreview>>>({})
  const previewError = ref('')
  const previewLoading = ref(false)
  const txFee = ref(0)
  const loading = ref(false)
  const resolvedRepayAmount = ref<number>()
  const receivePreviewCache = new Map<string, { preview: CloseMultiplyPreview, repayAmount: number }>()
  let interval: string | number | NodeJS.Timeout | undefined

  const activeMarket = computed(() => marketsStore.state.markets[String(data.value?.market)])

  const isMarginBorrow = ref(true)
  const marginAssetType = computed<MultiplyMarginAsset>(() => isMarginBorrow.value ? 'borrow' : 'deposit')
  const marginAsset = computed(() => isMarginBorrow.value ? data.value?.borrowAsset : data.value?.asset)
  const notMarginAsset = computed(() => isMarginBorrow.value ? data.value?.asset : data.value?.borrowAsset)
  const marginPrice = computed(() => isMarginBorrow.value ? data.value?.borrowPoolPrice : data.value?.price)
  const borrowDecimals = computed(() => data.value?.borrowPoolData.pool.token_decimals || 7)
  const depositDecimals = computed(() => data.value?.depositPoolData.pool.token_decimals || 7)
  const inputLabel = computed(() => isMarginBorrow.value ? 'Repay amount' : 'Receive amount')
  const maxAmountLabel = computed(() => isMarginBorrow.value ? 'Max repay' : 'Max receive')
  const currentFullClosePreview = computed(() => fullClosePreviews.value[marginAssetType.value])

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
    if (!data.value) {
      return 0
    }

    const sourcePreview = (preview.value?.marginAsset === marginAssetType.value ? preview.value : undefined)
      || currentFullClosePreview.value
    if (!sourcePreview) {
      return 0
    }

    return isMarginBorrow.value
      ? Number(bigintToNumber(sourcePreview.maxRepayAmount, borrowDecimals.value)) || 0
      : Number(bigintToNumber(sourcePreview.maxReceivableAmount, depositDecimals.value)) || 0
  })

  const currentDeposited = computed(() => {
    if (!data.value) {
      return 0
    }

    if (preview.value) {
      return Number(bigintToNumber(preview.value.currentDepositAmount, depositDecimals.value)) || 0
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

    return Number(bigintToNumber(preview.value.requiredAmountIn, depositDecimals.value)) || 0
  })

  const estimatedReceiveAmount = computed(() => {
    if (!preview.value || !data.value) {
      return 0
    }

    return Number(bigintToNumber(preview.value.estimatedReceiveAmount, depositDecimals.value)) || 0
  })

  const debtRepaidAmount = computed(() => {
    if (!preview.value || !data.value) {
      return 0
    }

    return Number(bigintToNumber(preview.value.debtRepaidAmount, borrowDecimals.value)) || 0
  })

  const remainingBorrowAmount = computed(() => {
    if (!preview.value || !data.value) {
      return 0
    }

    return Number(bigintToNumber(preview.value.remainingBorrowAmount, borrowDecimals.value)) || 0
  })

  const remainingDepositAmount = computed(() => {
    if (!preview.value || !data.value) {
      return 0
    }

    return Number(bigintToNumber(preview.value.remainingDepositAmount, depositDecimals.value)) || 0
  })

  const marketFee = computed(() => {
    if (!preview.value || !data.value) {
      return 0
    }

    return Number(bigintToNumber(
      preview.value.flashRepaymentAmount - preview.value.flashBorrowAmount,
      borrowDecimals.value,
    )) || 0
  })

  function toBorrowAmount(value: bigint) {
    return Number(bigintToNumber(value, borrowDecimals.value)) || 0
  }

  function toDepositAmount(value: bigint) {
    return Number(bigintToNumber(value, depositDecimals.value)) || 0
  }

  // eslint-disable-next-line unicorn/consistent-function-scoping
  function amountsEqual(left: number | undefined, right: number | undefined, decimals: number) {
    if (left == null || right == null) {
      return false
    }

    return Math.abs(left - right) <= amountTolerance(decimals)
  }

  function getPreviewRepayAmount(sourcePreview: CloseMultiplyPreview) {
    return toBorrowAmount(sourcePreview.repayAmount)
  }

  function getPreviewReceiveAmount(sourcePreview: CloseMultiplyPreview) {
    return toDepositAmount(sourcePreview.estimatedReceiveAmount)
  }

  function getPreviewMinReceiveAmount(sourcePreview: CloseMultiplyPreview) {
    return sourcePreview.marginAsset === 'deposit'
      ? toDepositAmount(sourcePreview.estimatedReceiveAmount)
      : undefined
  }

  function getMaxReceivableAmount(sourcePreview: CloseMultiplyPreview) {
    return toDepositAmount(sourcePreview.maxReceivableAmount)
  }

  function makeReceiveCacheKey(targetReceiveAmount: number) {
    return `${marginAssetType.value}:${targetReceiveAmount.toFixed(Math.min(depositDecimals.value, 6))}`
  }

  function getClosestCachedReceivePreview(targetReceiveAmount: number) {
    let closest: { preview: CloseMultiplyPreview, repayAmount: number } | undefined
    let closestDistance = Number.POSITIVE_INFINITY

    for (const cachedPreview of receivePreviewCache.values()) {
      const cachedReceiveAmount = getPreviewReceiveAmount(cachedPreview.preview)
      const distance = Math.abs(cachedReceiveAmount - targetReceiveAmount)
      if (distance < closestDistance) {
        closest = cachedPreview
        closestDistance = distance
      }
    }

    return closest
  }

  function estimateRepayAmountFromReceive(targetReceiveAmount: number, maxPreview: CloseMultiplyPreview) {
    const maxRepayAmount = toBorrowAmount(maxPreview.maxRepayAmount)
    const maxReceiveAmount = getMaxReceivableAmount(maxPreview)

    let anchorReceiveAmount = maxReceiveAmount
    let anchorRepayAmount = maxRepayAmount

    const currentPreview = preview.value
    if (currentPreview) {
      const currentReceiveAmount = getPreviewReceiveAmount(currentPreview)
      if (currentReceiveAmount > 0 && Math.abs(currentReceiveAmount - targetReceiveAmount) < Math.abs(anchorReceiveAmount - targetReceiveAmount)) {
        anchorReceiveAmount = currentReceiveAmount
        anchorRepayAmount = getPreviewRepayAmount(currentPreview)
      }
    }

    const cachedPreview = getClosestCachedReceivePreview(targetReceiveAmount)
    if (cachedPreview) {
      const cachedReceiveAmount = getPreviewReceiveAmount(cachedPreview.preview)
      if (cachedReceiveAmount > 0 && Math.abs(cachedReceiveAmount - targetReceiveAmount) < Math.abs(anchorReceiveAmount - targetReceiveAmount)) {
        anchorReceiveAmount = cachedReceiveAmount
        anchorRepayAmount = cachedPreview.repayAmount
      }
    }

    if (anchorReceiveAmount <= 0 || anchorRepayAmount <= 0) {
      return maxRepayAmount
    }

    return clampNumber(
      (targetReceiveAmount / anchorReceiveAmount) * anchorRepayAmount,
      amountTolerance(borrowDecimals.value),
      maxRepayAmount,
    )
  }

  async function getClosePreview(repayAmount?: number) {
    if (!data.value || !activeMarket.value?.client || !obligationKey.value) {
      return
    }

    return await activeMarket.value.client.multiply.getClosePositionPreview({
      user: obligationKey.value,
      depositPoolAddress: data.value.depositPoolData.pool.pool_address,
      borrowPoolAddress: data.value.borrowPoolData.pool.pool_address,
      marginAsset: marginAssetType.value,
      repayAmount,
      swapProviderAddress: SOROSWAP_PROVIDER_ADDRESS,
      path: swapPath.value,
    })
  }

  async function loadFullClosePreview() {
    const cachedPreview = currentFullClosePreview.value
    if (cachedPreview) {
      return cachedPreview
    }

    const result = await getClosePreview()
    if (!result) {
      return
    }

    fullClosePreviews.value = {
      ...fullClosePreviews.value,
      [result.marginAsset]: result,
    }
    return result
  }

  async function resolvePreviewFromReceiveAmount(targetReceiveAmount: number) {
    const maxPreview = currentFullClosePreview.value || await loadFullClosePreview()
    if (!maxPreview) {
      return
    }

    const maxReceiveAmount = getMaxReceivableAmount(maxPreview)
    if (targetReceiveAmount > maxReceiveAmount) {
      throw new Error('Receive amount exceeds closeable collateral')
    }

    if (amountsEqual(targetReceiveAmount, maxReceiveAmount, depositDecimals.value)) {
      return {
        preview: maxPreview,
        repayAmount: getPreviewRepayAmount(maxPreview),
      }
    }

    if (preview.value && amountsEqual(targetReceiveAmount, getPreviewReceiveAmount(preview.value), depositDecimals.value)) {
      return {
        preview: preview.value,
        repayAmount: getPreviewRepayAmount(preview.value),
      }
    }

    const cacheKey = makeReceiveCacheKey(targetReceiveAmount)
    const cachedPreview = receivePreviewCache.get(cacheKey)
    if (cachedPreview) {
      return cachedPreview
    }

    const maxRepayAmount = toBorrowAmount(maxPreview.maxRepayAmount)
    const firstRepayAmount = estimateRepayAmountFromReceive(targetReceiveAmount, maxPreview)
    const firstPreview = await getClosePreview(firstRepayAmount)
    if (!firstPreview) {
      return
    }

    let bestPreview = firstPreview
    let bestRepayAmount = firstRepayAmount
    let bestDistance = Math.abs(getPreviewReceiveAmount(firstPreview) - targetReceiveAmount)

    const firstReceiveAmount = getPreviewReceiveAmount(firstPreview)
    if (!amountsEqual(firstReceiveAmount, targetReceiveAmount, depositDecimals.value) && firstReceiveAmount > 0) {
      const correctedRepayAmount = clampNumber(
        firstRepayAmount * (targetReceiveAmount / firstReceiveAmount),
        amountTolerance(borrowDecimals.value),
        maxRepayAmount,
      )

      if (!amountsEqual(correctedRepayAmount, firstRepayAmount, borrowDecimals.value)) {
        const correctedPreview = await getClosePreview(correctedRepayAmount)
        if (correctedPreview) {
          const correctedDistance = Math.abs(getPreviewReceiveAmount(correctedPreview) - targetReceiveAmount)
          if (correctedDistance < bestDistance) {
            bestPreview = correctedPreview
            bestRepayAmount = correctedRepayAmount
            bestDistance = correctedDistance
          }
        }
      }
    }

    const result = {
      preview: bestPreview,
      repayAmount: bestRepayAmount,
    }

    receivePreviewCache.set(cacheKey, result)

    return result
  }

  function clearState() {
    amount.value = undefined
    preview.value = undefined
    fullClosePreviews.value = {}
    previewError.value = ''
    resolvedRepayAmount.value = undefined
    receivePreviewCache.clear()
    isMarginBorrow.value = true
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
      const maxPreview = currentFullClosePreview.value || await loadFullClosePreview()
      if (!maxPreview) {
        preview.value = undefined
        return
      }

      if (!amount.value) {
        amount.value = isMarginBorrow.value
          ? Number(bigintToNumber(maxPreview.maxRepayAmount, borrowDecimals.value)) || 0
          : Number(bigintToNumber(maxPreview.maxReceivableAmount, depositDecimals.value)) || 0
      }

      if (!amount.value || amount.value <= 0) {
        preview.value = maxPreview
        resolvedRepayAmount.value = getPreviewRepayAmount(maxPreview)
        return
      }

      if (marginAssetType.value === 'borrow') {
        if (preview.value && amountsEqual(Number(amount.value), getPreviewRepayAmount(preview.value), borrowDecimals.value)) {
          resolvedRepayAmount.value = getPreviewRepayAmount(preview.value)
          return
        }

        const result = await getClosePreview(Number(amount.value))
        preview.value = result
        resolvedRepayAmount.value = Number(amount.value)
      } else {
        if (preview.value && amountsEqual(Number(amount.value), getPreviewReceiveAmount(preview.value), depositDecimals.value)) {
          resolvedRepayAmount.value = getPreviewRepayAmount(preview.value)
          return
        }

        const resolvedPreview = await resolvePreviewFromReceiveAmount(Number(amount.value))
        preview.value = resolvedPreview?.preview
        resolvedRepayAmount.value = resolvedPreview?.repayAmount
      }
    } catch (error: any) {
      preview.value = undefined
      resolvedRepayAmount.value = undefined
      previewError.value = String(error?.message || error)
      txFee.value = 0
    } finally {
      previewLoading.value = false
    }
  }

  async function loadTxFee() {
    if (!opened.value || !data.value || !activeMarket.value?.client || !obligationKey.value || !resolvedRepayAmount.value || resolvedRepayAmount.value <= 0) {
      txFee.value = 0
      return
    }

    if (!preview.value) {
      txFee.value = 0
      return
    }

    try {
      const multiplyService = activeMarket.value.client.multiply
      const { tx } = await multiplyService.buildClosePositionTx({
        user: obligationKey.value,
        depositPoolAddress: data.value.depositPoolData.pool.pool_address,
        borrowPoolAddress: data.value.borrowPoolData.pool.pool_address,
        marginAsset: marginAssetType.value,
        repayAmount: Number(resolvedRepayAmount.value),
        minReceiveAmount: getPreviewMinReceiveAmount(preview.value),
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
    if (!amount.value || amount.value <= 0 || !resolvedRepayAmount.value || resolvedRepayAmount.value <= 0 || !preview.value) {
      return
    }

    try {
      loading.value = true

      await market.withdrawMultiply({
        client: activeMarket.value.client,
        market: activeMarket.value.marketState.global_state.name,
        deposit_pool_address: data.value.depositPoolData.pool.pool_address,
        borrow_pool_address: data.value.borrowPoolData.pool.pool_address,
        margin_asset: marginAssetType.value,
        repay_amount: Number(resolvedRepayAmount.value),
        min_receive_amount: getPreviewMinReceiveAmount(preview.value),
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

    await loadFullClosePreview()
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
    () => isMarginBorrow.value,
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
  }, { immediate: true, debounce: 1000, maxWait: 1500 })

  watch(() => isMarginBorrow.value, (nextValue) => {
    const nextMarginAsset: MultiplyMarginAsset = nextValue ? 'borrow' : 'deposit'
    const nextPreview = fullClosePreviews.value[nextMarginAsset]

    preview.value = nextPreview
    previewError.value = ''
    resolvedRepayAmount.value = nextPreview ? getPreviewRepayAmount(nextPreview) : undefined
    amount.value = nextPreview
      ? (nextValue ? getPreviewRepayAmount(nextPreview) : getPreviewReceiveAmount(nextPreview))
      : undefined
  })

  onScopeDispose(() => {
    clearReloadInterval()
  })

  return {
    amount,
    balance,
    inputLabel,
    marginPrice,
    currentDeposited,
    maxAmountLabel,
    swapInputEstimate,
    estimatedReceiveAmount,
    debtRepaidAmount,
    remainingBorrowAmount,
    remainingDepositAmount,
    marketFee,
    marginAssetType,
    preview,
    previewError,
    previewLoading,
    txFee,
    loading,
    isMarginBorrow,
    marginAsset,
    notMarginAsset,
    withdraw,
  }
}
