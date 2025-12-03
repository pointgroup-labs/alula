<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'
import { calcFee, calculateTotalStake } from '@alula/client-sdk/src/utils'
import { CLEAR_DIALOG_TIMEOUT, RELOAD_FEE_INTERVAL } from '~/config'
import { focusInput, formatPrice } from '~/utils'

const {
  data,
} = defineProps<{
  data?: MultiplyTableItem
}>()

const marketsStore = useMarketsStore()
const market = useMarketActions()

const userStore = useUserStore()

const amount = toRef(market, 'withdrawAmount')

const dialog = defineModel({ default: false })

const isValidate = ref(true)

const reloadFee = ref(false)

const activeMarket = computed(() => marketsStore.state.markets[String(data?.market)])

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const balance = computed(() => {
  if (!data) {
    return 0
  }
  const deposits: any = userStore.state.multiplyObligations[String(data?.market)]?.deposits || []
  const { depositPoolData } = data
  const depositAsset = deposits.find(([deposit]: any) => deposit.includes(depositPoolData?.pool.pool_address))
  if (!depositAsset) {
    return 0
  }
  const userShares = depositAsset[1].j_tokens || 0
  const deposited = calculateTotalStake(userShares, {
    total_j_tokens: depositPoolData.pool.total_j_tokens,
    total_borrowed: depositPoolData.pool.total_borrowed,
    total_available: depositPoolData.pool.total_available,
  }).toString()
  return Number(deposited) || 0
})

const txFee = ref(0)

const marketFee = computed(() => {
  const marketFeeBps = data?.borrowPoolData.pool.config.fee_config.withdraw_fee_bps
  return calcFee(Number(amount.value || 0), marketFeeBps || 0)
})

async function withdrawLeverage() {
  if (!publicKey.value || !data?.depositPoolData.pool.pool_address) {
    return
  }
  if (!amount.value || amount.value <= 0) {
    focusInput('.multiply-dialog')
    return
  }
  const deposit_pool_address = data?.depositPoolData.pool.pool_address
  const borrow_pool_address = data?.borrowPoolData.pool.pool_address
  const asset_code = data?.depositPoolData.pool.token_symbol
  if (!deposit_pool_address || !borrow_pool_address) {
    return
  }

  const marketProps = {
    client: activeMarket.value!.client,
    market: activeMarket.value!.marketState.global_state.name,
    deposit_pool_address,
    borrow_pool_address,
    amount: amount.value,
    asset_code,
  }

  try {
    isValidate.value = false
    await market.withdrawLeverage({
      ...marketProps,
      action: async () => {
        await userStore.updateUserMultiplyObligation({
          market: activeMarket.value!.marketState.global_state.name,
          client: activeMarket.value!.client,
          depositPoolAddress: deposit_pool_address,
          borrowPoolAddress: borrow_pool_address,
        })
        await marketsStore.updateLeveragePool({
          deposit_pool_address,
          borrow_pool_address,
          market: activeMarket.value!.marketState.global_state.name,
          client: activeMarket.value!.client,
        })
        await marketsStore.updatePool(borrow_pool_address, activeMarket.value!.marketState.global_state.name, activeMarket.value!.client)
      },
    })
  } finally {
    isValidate.value = true
  }
}

let interval: string | number | NodeJS.Timeout | undefined

watch(dialog, async (v) => {
  clearInterval(interval)
  if (!v) {
    setTimeout(() => {
      amount.value = 0
    }, CLEAR_DIALOG_TIMEOUT)
    return
  }

  interval = setInterval(() => {
    reloadFee.value = true
    nextTick(() => {
      reloadFee.value = false
    })
  }, RELOAD_FEE_INTERVAL)
})

watchDebounced([
  () => data,
  reloadFee,
  publicKey,
], async ([data]) => {
  if (!data || !publicKey.value || !dialog.value) {
    return
  }
  const tx = await activeMarket.value!.client.marketSdk.withdrawLeverageTx(
    publicKey.value,
    data?.depositPoolData.pool.pool_address || '',
    data?.borrowPoolData.pool.pool_address || '',
    1,
  )

  txFee.value = activeMarket.value!.client.marketSdk.getTransactionFee(tx)
}, { immediate: true, debounce: 300 })
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="multiply-dialog dialog-default"
  >
    <template #header>
      <div class="multiply-dialog__title">
        <span>Withdraw {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="multiply-dialog__body">
      <div class="multiply-dialog__data">

        <input-widget
          v-model="amount"
          :balance="balance"
          class="multiply-dialog__input"
          :icon="data?.asset.icon"
          label-left="You Deposit"
          :rules="[
            (v: any) => !isValidate || (v && Number(v) < balance) || 'Insufficient balance',
          ]"
        >
          <template #label-right>
            Multiplied: {{ formatPrice(balance, 0, market.assetDecimals.value) }} {{ data?.asset.symbol }}
          </template>
        </input-widget>

        <div
          v-if="data"
          class="dialog-info-table"
        >
          <!-- Market fee -->
          <div class="dialog-info-table__item">
            <span>Market Fee</span>
            <span>{{ marketFee }} XLM</span>
          </div>

          <!-- Tx fee -->
          <div class="dialog-info-table__item">
            <span>Transaction Fee</span>
            <span>{{ txFee }} XLM</span>
          </div>
        </div>

        <div class="multiply-dialog-action">
          <market-dialog-action-btn
            variant="primary"
            :loading="market.isLoading(String(data?.pool_address), 'withdrawLeverage', String(data?.market))"
            :pool="data?.depositPoolData.pool"
            @click-handler="withdrawLeverage"
          >
            Withdraw {{ data?.asset.symbol }}
          </market-dialog-action-btn>
        </div>
      </div>
    </div>
  </j-dialog>
</template>

<style lang="scss">
.multiply-dialog {
  &__title {
    color: $dark;
    font-size: 20px;
    font-style: normal;
    font-weight: 500;
    line-height: 20px;
  }

  &__body {
    padding-top: $spacing-16;
    display: flex;
    flex-direction: column;
    gap: $spacing-16;
  }

  .multiply-dialog-action {
    display: flex;
    justify-content: space-between;
    gap: $spacing-32;

    .action-info {
      white-space: nowrap;
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 2px;

      span:first-child {
        color: $neutral-12;
        font-size: 12px;
        font-style: normal;
        font-weight: 500;
        line-height: 16px;
      }

      span:last-child {
        font-size: 20px;
        font-style: normal;
        font-weight: 700;
        line-height: 20px;
      }
    }

    .btn {
      width: 100%;
    }
  }
}
</style>
