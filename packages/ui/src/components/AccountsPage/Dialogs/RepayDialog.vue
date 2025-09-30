<script lang="ts" setup>
import type { BorrowCardTableItem } from '~/types/table'
import { calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'
import { CLEAR_DIALOG_TIMEOUT, RELOAD_FEE_INTERVAL } from '~/config'
import { focusInput, shortenNumber, truncatePercent } from '~/utils'

const {
  data,
  modelValue,
} = defineProps<{
  data?: BorrowCardTableItem
  modelValue: boolean
}>()

const marketsStore = useMarketsStore()
const market = useMarketActions()

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const activeMarket = computed(() => marketsStore.state.markets[String(data?.market)])

const userStore = useUserStore()

const userTotalDepositByMarket = computed(() => {
  const obligation = userStore.state.obligations[String(activeMarket.value?.marketState.name)]
  const pools = activeMarket.value?.pools
  const assetDecimals = marketsStore.assetDecimals
  if (!obligation || !pools) {
    return 0
  }
  return calcUserTotalStakeInUsd(obligation, pools, assetDecimals) ?? 0
})

const userTotalBorrowedByMarket = computed(() => {
  const obligation = userStore.state.obligations[String(activeMarket.value?.marketState.name)]
  const pools = activeMarket.value?.pools
  const assetDecimals = marketsStore.assetDecimals
  if (!obligation || !pools) {
    return 0
  }
  return calcUserTotalBorrowedInUsd(obligation, pools, assetDecimals) ?? 0
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

const closeLTV = computed(() => data?.raw?.config?.close_ltv_bps ? Number(data.raw.config.close_ltv_bps) / 10_000 : 0)

const healthFactor = computed(() => {
  const amountInUsd = Number(amount.value || 0) * Number(data?.raw?.pool_price || 0)
  const deposited = (userTotalDepositByMarket.value * closeLTV.value)
  const borrowed = Math.max(userTotalBorrowedByMarket.value - amountInUsd, 0)
  const result = Math.max(deposited / borrowed, 0)
  return Math.min(result, 10)
})

watchDebounced([
  () => data,
  reloadFee,
  publicKey,
], async ([d, _r]) => {
  if (!d?.pool_address || !publicKey.value) {
    return
  }

  const tx = await activeMarket.value?.client.marketSdk.repayTx(
    publicKey.value,
    d?.pool_address || '',
    0,
  )
  txFee.value = activeMarket.value?.client.marketSdk.getTransactionFee(tx) ?? 0
}, { immediate: true, debounce: 300 })

const infoTableData = computed(() => {
  if (!data) {
    return []
  }
  const borrowBalanceAfterRepay = Math.max(Number(data?.debt) - amount.value || 0, 0)
  return [{
    name: 'healthFactor',
    label: 'Health Factor',
    value: truncatePercent(healthFactor.value, 2),
  },
  {
    label: 'Borrow balance after repay',
    value: `${shortenNumber(borrowBalanceAfterRepay)} ${data.asset.symbol}`,
  },
  {
    label: 'Transaction Fee',
    value: `${txFee.value} XLM`,
  }]
})

const dialog = defineModel({ default: false })

async function repay() {
  if (!data) {
    return
  }
  if (!amount.value || amount.value <= 0) {
    focusInput('.repay-dialog__input')
    return
  }
  try {
    loading.value = true
    marketsStore.poolActiveAddress = data?.pool_address
    marketsStore.activeMarketFilter = String(activeMarket.value?.marketState.name)

    const marketProps = {
      market: activeMarket.value!.marketState.name,
      client: activeMarket.value!.client,
      pool_address: data?.pool_address,
      amount: amount.value,
      asset_data: data?.raw.name,
      limit: balance.value,
    }

    await market.repay(marketProps)
  } finally {
    loading.value = false
  }
}

let interval: string | number | NodeJS.Timeout | undefined

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
    class-name="account-dialog dialog-default"
  >
    <template #header>
      <div class="account-dialog__title">
        <img
          :src="data?.asset.icon"
          :alt="`${data?.asset.symbol} icon`"
        >
        <span>Repay {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="account-dialog__body">
      <input-widget
        v-model="amount"
        class="repay-dialog__input"
        :balance="balance"
        :limit="Number(data?.debt) || 0"
        :rules="[
          (v) => {
            return v && Number(v) < balance || 'Insufficient balance'
          },
        ]"
      >
        <template #label-right>
          Repay with Wallet: {{ balance }} {{ data?.asset.symbol }}
        </template>
      </input-widget>

      <div class="account-info-table">
        <div
          v-for="item in infoTableData"
          :key="item.label"
          class="account-info-table__item"
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
      </div>

      <div class="account-dialog-action">
        <j-btn
          :loading="loading"
          variant="success"
          size="md"
          pill
          @click="repay"
        >
          Repay {{ data?.asset.symbol }}
        </j-btn>
      </div>
    </div>
  </j-dialog>
</template>
