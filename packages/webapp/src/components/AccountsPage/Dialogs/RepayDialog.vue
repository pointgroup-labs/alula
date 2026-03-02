<script lang="ts" setup>
import type { BorrowCardTableItem } from '~/types/table'
import { calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk'
import { CLEAR_DIALOG_TIMEOUT, RELOAD_FEE_INTERVAL } from '~/config'
import { focusInput, shortenNumber, truncatePercent } from '~/utils'

const {
  data,
  modelValue,
} = defineProps<{
  data?: BorrowCardTableItem
  modelValue: boolean
}>()

const dialog = defineModel({ default: false })

const marketsStore = useMarketsStore()
const market = useMarketActions()

const isValidate = ref(true)

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const activeMarket = computed(() => marketsStore.state.markets[String(data?.market)])

const userStore = useUserStore()

const userTotalDepositByMarket = computed(() => {
  const obligation = userStore.state.obligations[String(activeMarket.value?.marketName)]
  const pools = activeMarket.value?.marketState.pools_data
  const assetDecimals = activeMarket.value?.marketState.asset_decimals ?? 7
  const oraclePriceDecimals = activeMarket.value?.marketState.oracle_price_decimals ?? 0
  if (!obligation || !pools) {
    return 0
  }
  return calcUserTotalStakeInUsd(obligation, pools, assetDecimals, oraclePriceDecimals, 'close') ?? 0
})

const userTotalBorrowByMarket = computed(() => {
  const obligation = userStore.state.obligations[String(activeMarket.value?.marketName)]
  const pools = activeMarket.value?.marketState.pools_data
  const assetDecimals = activeMarket.value?.marketState.asset_decimals ?? 7
  const oraclePriceDecimals = activeMarket.value?.marketState.oracle_price_decimals ?? 0
  if (!obligation || !pools) {
    return 0
  }
  return calcUserTotalBorrowedInUsd(obligation, pools, assetDecimals, oraclePriceDecimals) ?? 0
})

const loading = ref(false)
const reloadFee = ref(false)

const amount = toRef(market, 'repayAmount')
const txFee = ref(0)

const balance = computed(() => {
  if (!data) {
    return 0
  }
  if (data.asset.symbol === 'XLM') {
    return wallet.nativeBalance
  }
  return wallet.getAssetBalance(String(data.asset_issuer))
})

const healthFactor = computed(() => {
  const amountInUsd = Number(amount.value || 0) * Number(data?.price || 0)
  const deposited = userTotalDepositByMarket.value
  const borrowed = Math.max(Number(userTotalBorrowByMarket.value) - amountInUsd, 0)
  const result = Math.max(deposited / borrowed, 0)
  return Math.min(result, 10)
})

const infoTableData = computed(() => {
  if (!data) {
    return []
  }
  const debt = Number(data?.debt ?? 0)
  const borrowBalanceAfterRepay = Math.max(Number(debt) - amount.value || 0, 0)
  return [{
    name: 'healthFactor',
    label: 'Health Factor',
    value: truncatePercent(healthFactor.value, 2),
  },
  {
    name: 'debt',
    label: 'Debt',
    value: `${shortenNumber(data?.debt || 0, 2, maxDecimalsForShortenNumber(debt))} ${data?.asset.symbol}`,
  },
  {
    label: 'Debt Balance After Repayment',
    value: `${shortenNumber(borrowBalanceAfterRepay, 2, maxDecimalsForShortenNumber(borrowBalanceAfterRepay))} ${data.asset.symbol}`,
  },
  {
    label: 'Transaction Fee',
    value: `${txFee.value} XLM`,
  }]
})

async function repay() {
  if (!data) {
    return
  }
  if (!amount.value || amount.value <= 0 || amount.value > Number(balance.value)) {
    focusInput('.repay-dialog__input')
    return
  }
  try {
    loading.value = true
    isValidate.value = false
    const withBuffer = Number(data.debt) === Number(amount.value) && Number(balance.value) !== Number(amount.value)

    const marketProps = {
      market: activeMarket.value!.marketName,
      client: activeMarket.value!.client,
      pool_address: data?.pool_address,
      amount: amount.value,
      asset_data: data?.raw?.pool.name,
      limit: balance.value,
      withBuffer,
    }

    await market.repay(marketProps)

    dialog.value = false
  } finally {
    loading.value = false
    isValidate.value = true
  }
}

let interval: string | number | NodeJS.Timeout | undefined

watch(() => data, (d) => {
  if (!d) {
    dialog.value = false
  }
})

watchDebounced([
  () => data,
  reloadFee,
  publicKey,
], async ([d, _r]) => {
  if (!d?.pool_address || !publicKey.value) {
    return
  }

  const tx = await activeMarket.value?.client.borrowing.buildRepayTx(
    publicKey.value,
    d?.pool_address || '',
    0,
  )
  txFee.value = activeMarket.value?.client.borrowing.getTransactionFee(tx) ?? 0
}, { immediate: true, debounce: 300 })

watch(() => modelValue, async (v) => {
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
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="dialog-default"
  >
    <template #header>
      <div class="dialog-default__title">
        <img
          :src="data?.asset.icon"
          :alt="`${data?.asset.symbol} icon`"
        >
        <span>Repay {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="dialog-default__body">
      <input-widget
        v-model="amount"
        class="repay-dialog__input"
        :balance="balance"
        :limit="Number(data?.debt) || 0"
        label-left="Balance"
        variant="borrow"
        :label-right="`${formatPrice(balance ?? 0, 0, 4)} ${data?.asset.symbol}`"
        :reset="dialog"
        :price="Number(data?.price ?? 0)"
        :rules="[
          (v) => {
            return !isValidate || Number(v) <= balance || 'Insufficient balance'
          },
        ]"
      />

      <div class="dialog-info-table">
        <div
          v-for="item in infoTableData"
          :key="item.label"
          class="dialog-info-table__item"
        >
          <span>{{ item?.label }}</span>
          <span>
            <template v-if="item?.name === 'healthFactor' && loading">
              <j-loading-spinner
                width="14px"
                style="padding: 0; width: 14px; margin-left: auto"
              />
            </template>
            <template v-else>
              {{ item?.value }}
            </template>
          </span>
        </div>

        <div class="separator" />
      </div>

      <div class="dialog-default__action">
        <j-btn
          :loading="loading"
          variant="accent"
          size="lg"
          pill
          @click="repay"
        >
          Repay {{ data?.asset.symbol }}
        </j-btn>
      </div>
    </div>
  </j-dialog>
</template>
