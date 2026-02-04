<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'
import { calcFee } from '@alula/client-sdk/src/utils'
import { bigintToNumber, destructurePoolAsset, focusInput, formatPrice, truncatePercent } from '~/utils'

const userStore = useUserStore()
const marketsStore = useMarketsStore()
const market = useMarketActions()

const poolData = inject<Ref<MultiplyTableItem>>('selectedPool')

const isDepositMultiply = ref(true)

const reloadFee = ref(false)

const amount = toRef(market, 'depositAmount')

const debouncedAmount = refDebounced(amount, 1000)

const isHasAmount = computed(() => {
  return !!(debouncedAmount.value && debouncedAmount.value > 0)
})

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
  const currentPool = isDepositMultiply.value ? poolData?.value.depositPoolData.pool : poolData.value.borrowPoolData.pool
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
  const maxBorrowByUtil = Number(bigintToNumber(borrowPoolData.value.total_supply, poolData!.value.assetDecimals))
    * (Number(borrowPoolData.value.pool.config.health_config.utilization_ratio_limit_bps) / 10_000)
    - Number(bigintToNumber(borrowPoolData.value.pool.total_borrowed, poolData!.value.assetDecimals))
  return Math.max(0, Math.min(Number(bigintToNumber(borrowPoolData.value?.total_available_adjusted, poolData!.value.assetDecimals)), maxBorrowByUtil))
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
      await Promise.any([
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

watch(isHasAmount, async (v) => {
  if (!v) {
    txFee.value = 0
    return
  }
  reloadFee.value = true
  nextTick(() => {
    reloadFee.value = false
  })
})

watchDebounced([
  () => poolData?.value,
  reloadFee,
  publicKey,
], async ([data]) => {
  if (!data || !publicKey.value || !amount.value || amount.value <= 0) {
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
</script>

<template>
  <section id="multiply-form">
    <div class="stat-card">
      <input-widget
        v-model="amount"
        :balance="balance"
        :limit="supplyLimit"
        class="multiply-dialog__input"
        label-left="You Deposit"
        :rules="[
          (v) => {
            return v && Number(v) < balance || 'Insufficient balance'
          },
          (v) => {
            return (supplyLimit <= 0 || Number(v) <= supplyLimit) || 'Pool leverage limit'
          },
        ]"
      >
        <template #label-right>
          Wallet: {{ balance }} {{ depositAsset?.name }}
        </template>
        <template #prepend>
          <j-select-popover>
            <template #menu>
              <div
                class="popover-borrow-asset"
                @click="swapAsset"
              >
                <img
                  :src="borrowAsset?.icon"
                  :alt="`${borrowAsset?.name} icon`"
                >
                {{ borrowAsset?.name }}
              </div>
            </template>
            <template #target>
              <img
                :src="depositAsset?.icon"
                :alt="`${depositAsset?.name} icon`"
              >
            </template>
          </j-select-popover>
        </template>
      </input-widget>

      <multiply-select
        v-model="percentFromMaxMultiply"
        :multiplier="selectedMultiplier"
        :max-multiply="maxMultiply"
      />

      <market-dialog-action-btn
        class="severage-form-btn"
        variant="primary"
        :loading="market.isLoading(String(poolData?.pool_address), 'leverage', String(poolData?.market))"
        :pool="poolData?.depositPoolData.pool"
        :disabled="Number(selectedMultiplier) < 1"
        @click-handler="leverage"
      >
        Multiply {{ poolData?.asset.symbol }}
      </market-dialog-action-btn>

      <div
        v-if="amount > 0 && poolData"
        class="dialog-info-table"
      >

        <!-- Liquidation Available -->
        <div
          class="dialog-info-table__item"
        >
          <span>Liquidity Available</span>
          <span>{{ availableLiquidity }}</span>
        </div>

        <!-- Max APY -->
        <div class="dialog-info-table__item">
          <span>APY</span>
          <span>{{ truncatePercent(maxAPY, 2) }} %</span>
        </div>

        <!-- Max Multiplied Amount -->
        <div class="dialog-info-table__item">
          <span>Max Multiplied Amount</span>
          <span>{{ formatPrice(Number(supplyLimit || 0).toFixed(2), 2) }} {{ multiplySymbol }}</span>
        </div>

        <!-- Total Supply -->
        <div class="dialog-info-table__item">
          <span>Total Supply</span>
          <span>{{ formatPrice(Number(poolData!.supplied || 0), 2, 2) }} {{ poolData!.asset.symbol }}</span>
        </div>

        <!-- Market fee -->
        <div class="dialog-info-table__item">
          <span>Operation Fee</span>
          <span>{{ formatPrice(marketFee, 0, 5) }} {{ poolData?.borrowAsset.symbol }}</span>
        </div>

        <!-- Tx fee -->
        <div class="dialog-info-table__item">
          <span>Transaction Fee</span>
          <span>{{ txFee }} XLM</span>
        </div>

      </div>
    </div>
  </section>
</template>

<style lang="scss">
section#multiply-form {
  min-width: 378px;
  margin-top: 44px;

  .stat-card {
    display: flex;
    flex-direction: column;
    gap: $spacing-16;
  }

  .input-wrapper {
    margin-left: 6px;
  }

  .loop-multiply {
    width: 100%;
  }

  .severage-form-btn {
    width: 100%;
  }
}
</style>
