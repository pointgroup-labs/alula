import type { MultiplyTableItem } from '~/types/table'
import { calcFee } from '@alula/client-sdk/src/utils'
import { bigintToNumber, destructurePoolAsset, focusInput, formatPrice } from '~/utils'

export function useLeverage(data: MaybeRef<MultiplyTableItem | undefined>) {
  const userStore = useUserStore()
  const marketsStore = useMarketsStore()
  const market = useMarketActions()

  const poolData = computed(() => unref(data))

  const isDepositMultiply = ref(true)

  const reloadFee = ref(false)

  const amount = toRef(market, 'depositAmount')

  const activeMarket = computed(() => marketsStore.state.markets[String(poolData?.value?.market)])

  const wallet = useWallet()
  const publicKey = computed(() => wallet.publicKey)

  const multiplyAssets = computed(() => [poolData?.value?.asset, poolData?.value?.borrowAsset])

  const depositAsset = computed(() => multiplyAssets.value[isDepositMultiply.value ? 0 : 1])
  const borrowAsset = computed(() => multiplyAssets.value[isDepositMultiply.value ? 1 : 0])

  const balance = computed(() => {
    if (!poolData?.value) {
      return 0
    }
    const currentPool = isDepositMultiply.value ? poolData.value.depositPoolData.pool : poolData.value.borrowPoolData.pool
    const poolAsset = currentPool.token_symbol
    if (poolAsset === 'native') {
      return wallet.nativeBalance
    }
    const [, asset_issuer] = destructurePoolAsset(currentPool.name)
    return wallet.getAssetBalance(String(asset_issuer))
  })

  const percentFromMaxMultiply = ref(90)

  const maxMultiply = computed(() => poolData?.value?.multiplier || 0)
  const selectedMultiplier = computed(() => {
    return Number((percentFromMaxMultiply.value / 100) * maxMultiply.value).toFixed(2)
  })

  const txFee = ref(0)

  const multiplySymbol = computed(() =>
    getTokenSymbol(String(isDepositMultiply.value ? poolData?.value?.depositPoolData.pool.token_symbol : poolData?.value?.borrowPoolData.pool.token_symbol)))

  const borrowPoolData = computed(() => poolData?.value?.borrowPoolData)
  const liquidityRemaining = computed(() => {
    if (!borrowPoolData.value) {
      return 0
    }
    const maxBorrowByUtil = Number(bigintToNumber(borrowPoolData.value.total_supply, poolData!.value!.assetDecimals))
      * (Number(borrowPoolData.value.pool.config.health_config.utilization_ratio_limit_bps) / 10_000)
      - Number(bigintToNumber(borrowPoolData.value.pool.total_borrowed, poolData!.value!.assetDecimals))
    return Math.max(0, Math.min(Number(bigintToNumber(borrowPoolData.value?.total_available_adjusted, poolData!.value!.assetDecimals)), maxBorrowByUtil))
  })

  const availableLiquidity = computed(() => {
    if (!borrowPoolData.value) {
      return 0
    }
    return `${formatPrice(liquidityRemaining.value || 0, 2, 2)} ${borrowPoolData.value.pool.token_symbol}`
  })

  const borrowAvailableInUsd = computed(() => Number(liquidityRemaining.value) * Number(poolData?.value?.borrowPoolPrice || 0))

  const marketFee = computed(() => {
    const marketFeeBps = borrowPoolData.value?.pool.config.fee_config.flash_loan_fee_bps
    return calcFee(Number(amount.value || 0), marketFeeBps || 0)
  })

  const maxAPY = computed(() => poolData?.value?.maxAPY || 0)

  const depositPoolPrice = computed(() => isDepositMultiply.value ? poolData?.value?.price : poolData?.value?.borrowPoolPrice)

  const supplyLimit = computed(() => {
    const flashLoanFeeBps = borrowPoolData.value?.pool.config.fee_config.flash_loan_fee_bps || 0
    const sum = calcRemainingMultiplyUSD(
      borrowAvailableInUsd.value,
      Number(depositPoolPrice.value || 0),
      Number(selectedMultiplier.value) || 0,
      flashLoanFeeBps,
    )
    return sum
  })

  function swapAsset() {
    isDepositMultiply.value = !isDepositMultiply.value
  }

  async function leverage() {
    if (!publicKey.value || !poolData?.value?.depositPoolData.pool.pool_address) {
      return
    }
    if (!amount.value || amount.value <= 0 || amount.value > balance.value) {
      focusInput('.multiply-dialog')
      return
    }

    const deposit_pool_address = poolData.value?.depositPoolData.pool.pool_address
    const borrow_pool_address = poolData.value?.borrowPoolData.pool.pool_address
    const asset_code = isDepositMultiply.value ? poolData.value?.asset.symbol : poolData.value?.borrowAsset.symbol
    if (!deposit_pool_address || !borrow_pool_address) {
      return
    }

    const marketProps = {
      client: activeMarket.value!.client,
      market: activeMarket.value!.marketState.global_state.name,
      deposit_pool_address,
      borrow_pool_address,
      deposit_as_margin: isDepositMultiply.value,
      amount: amount.value,
      leverage_multiplier: Number(selectedMultiplier.value),
      asset_code,
    }

    await market.leverage({
      ...marketProps,
      action: async () => {
        await Promise.allSettled([
          userStore.updateUserMultiplyObligation({
            market: activeMarket.value!.marketState.global_state.name,
            client: activeMarket.value!.client,
            depositPoolAddress: deposit_pool_address,
            borrowPoolAddress: borrow_pool_address,
          }),
          marketsStore.updatePool(borrow_pool_address, activeMarket.value!.marketState.global_state.name, activeMarket.value!.client),
        ])
      },
    })
  }

  watchDebounced([
    () => poolData?.value,
    reloadFee,
    publicKey,
  ], async ([data]) => {
    if (!data || !publicKey.value) {
      return
    }
    const tx = await activeMarket.value?.client.marketSdk.leverageTx(
      publicKey.value,
      data?.depositPoolData.pool.pool_address || '',
      data?.borrowPoolData.pool.pool_address || '',
      isDepositMultiply.value,
      1,
      2,
    )
    txFee.value = activeMarket.value?.client.marketSdk.getTransactionFee(tx) || 0
  }, { immediate: true, debounce: 300 })
  return {
    reloadFee,
    depositAsset,
    borrowAsset,
    amount,
    balance,
    selectedMultiplier,
    isDepositMultiply,

    txFee,
    availableLiquidity,
    borrowAvailableInUsd,
    supplyLimit,
    maxAPY,
    depositPoolPrice,

    maxMultiply,
    percentFromMaxMultiply,

    multiplySymbol,
    marketFee,

    swapAsset,
    leverage,
  }
}
