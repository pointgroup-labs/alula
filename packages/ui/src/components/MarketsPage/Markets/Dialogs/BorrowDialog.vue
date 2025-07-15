<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { bigintToNumber, shortenNumber, truncatePercent } from '~/utils'

const {
  data,
  modelValue,
} = defineProps<{
  data?: MarketTableItem
  modelValue: boolean
}>()

const emits = defineEmits(['update:modelValue'])

const clientStore = useClientStore()
const jLendClient = computed(() => clientStore.jLendClient)

const assetDecimals = computed(() => clientStore.assetDecimals)

const marketsStore = useMarketsStore()
const market = useMarket()

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const userStore = useUserStore()

const amount = toRef(market, 'borrowAmount')
const agree = ref(false)

const reloadFee = ref(false)
const txFee = ref(0)

watchDebounced([
  () => data,
  reloadFee,
], async ([d, _r]) => {
  if (!d || !publicKey.value) {
    return
  }
  const tx = await jLendClient.value?.sdk.borrowTx(
    publicKey.value,
    d?.raw.pool_address || '',
    0,
  )
  txFee.value = jLendClient.value.sdk.getTransactionFee(tx)
}, { immediate: true, debounce: 300 })

const balance = computed(() => {
  if (!data) {
    return 0
  }
  if (data.raw.token_ticker === 'XLM') {
    return wallet.nativeBalance
  }
  const asset_issuer = data.raw.name.split(':')[1]
  return wallet.getAssetBalance(String(asset_issuer))
})

const poolBorrowLimit = computed(() => {
  if (!data) {
    return 0
  }

  // market available
  const utilRatioLimit = Number(data?.raw.config.utilization_ratio_limit_bps || 0) / 10_000
  const marketAvailable = Number(bigintToNumber(data.raw.available, assetDecimals.value)) * utilRatioLimit
  return marketAvailable
})

const availableToBorrow = computed(() => {
  if (!data) {
    return 0
  }
  const userTotalDepositInUsd = userStore.userTotalDepositInUsd
  const userTotalBorrowedInUsd = Number(userStore.userTotalBorrowedInUsd) || 0
  const marketAvailableInUsd = Number(poolBorrowLimit.value) * Number(data.price)
  const openLTV = Number(data?.raw.config.open_ltv_bps || 0) / 10_000
  const userAvailableByLTV = Number(userTotalDepositInUsd * openLTV) || 0
  const maxAvailableUsd = Math.min(Math.max(userAvailableByLTV - userTotalBorrowedInUsd, 0), marketAvailableInUsd)
  const maxAvailableAssets = maxAvailableUsd / Number(data.price)
  return Math.max(maxAvailableAssets, 0)
})

const healthFactor = computed(() => {
  const depositUsd = userStore.userTotalDepositInUsd
  const borrowedUsd = userStore.userTotalBorrowedInUsd
  const price = data?.price || 0
  const closeLTV = Number(data?.raw.config.close_ltv_bps || 0) / 10_000

  const extraBorrowUsd = (amount.value || 0) * price
  const totalBorrowUsd = borrowedUsd + extraBorrowUsd

  let hf = (depositUsd * closeLTV) / totalBorrowUsd

  if (!Number.isFinite(hf)) {
    hf = 0
  }

  return Math.min(hf, 10)
})

const infoTableData = computed(() => {
  if (!data) {
    return []
  }
  const liquidation = Number(data.raw.config.liquidation_close_factor_bps) / 100
  const closeLTV = Number(data.raw.config.close_ltv_bps) / 100
  return [{
    label: 'Health Factor',
    value: truncatePercent(healthFactor.value, 2),
  },
  {
    label: 'Pool available amount to borrow',
    value: shortenNumber(poolBorrowLimit.value),
  },
  {
    label: 'User available amount to borrow',
    value: shortenNumber(availableToBorrow.value),
  },
  {
    label: 'Max LTV',
    value: data.max_ltv,
  },
  {
    label: 'Liquidation LTV',
    value: `${truncatePercent(closeLTV || 0, 2)}%`,
  },
  {
    label: 'Liq. Penalty',
    value: `${truncatePercent(liquidation || 0, 2)}%`,
  },
  {
    label: 'Transaction Fee',
    value: `${txFee.value} XLM`,
  }]
})

const dialog = computed({
  get() {
    return modelValue
  },
  set(val) {
    emits('update:modelValue', val)
  },
})

const loading = computed(() => marketsStore.poolDepositAddr === data?.raw.pool_address)

async function borrow() {
  try {
    if (!publicKey.value || !data?.raw.pool_address) {
      return
    }
    await market.borrow(data?.raw.pool_address, amount.value, data?.raw.name, poolBorrowLimit.value)
  } catch {
    if (!amount.value || amount.value <= 0) {
      const input = document.querySelector('.supply-dialog__input')?.querySelector('input') as HTMLInputElement
      input?.focus()
    }
  }
}
let interval: string | number | NodeJS.Timeout | undefined

watch(() => modelValue, async (v) => {
  clearInterval(interval)
  if (!v) {
    amount.value = 0
    agree.value = false
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
    class-name="supply-dialog borrow-dialog"
  >
    <template #header>
      <div class="supply-dialog__title">
        <img
          :src="data?.asset.icon"
          :alt="`${data?.asset.symbol} icon`"
        >
        <span>Borrow {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="supply-dialog__body">
      <input-widget
        v-model="amount"
        :balance="availableToBorrow"
        :rules="[
          (v) => {
            return v && Number(v) < availableToBorrow || 'Borrow limit exceeded'
          },
        ]"
      >
        <template #label-right>
          Wallet: {{ balance }} {{ data?.asset.symbol }}
        </template>
      </input-widget>

      <div class="supply-info-table">
        <div
          v-for="item in infoTableData"
          :key="item.label"
          class="supply-info-table__item"
        >
          <span>{{ item?.label }}</span>
          <span>{{ item?.value }}</span>
        </div>
      </div>

      <div class="supply-warning">
        <i-app-warning-color class="warning-icon" />
        <div>
          <span>Attention</span>: Parameter changes via governance can alter your account health factor and
          risk of
          liquidation.
        </div>
      </div>

      <div class="supply-agree">
        <j-checkbox v-model="agree">
          I acknowledge the risks involved.
        </j-checkbox>
      </div>

      <div class="supply-dialog-action">
        <div class="action-info">
          <span>Borrow APY</span>
          <span>{{ data?.borrow_apy }}</span>
        </div>

        <market-dialog-action-btn
          variant="accent"
          :loading="loading"
          :pool="data?.raw"
          :disabled="!agree"
          @click-handler="borrow"
        >
          Borrow {{ data?.asset.symbol }}
        </market-dialog-action-btn>
      </div>
    </div>
  </j-dialog>
</template>

<style lang="scss">
.borrow-dialog {
  .supply-warning {
    padding: $spacing-16;
    border-radius: $spacing-8;
    background-color: $neutral-2;
    display: flex;
    align-items: flex-start;
    gap: $spacing-8;
    color: $neutral-6;
    font-size: 11px;
    font-style: normal;
    font-weight: 500;
    line-height: 12px;

    span {
      font-weight: 700;
    }

    .warning-icon {
      min-width: 16px;
      width: 16px;
      height: 16px;
    }
  }

  .supply-agree {
    font-size: 11px;
    font-style: normal;
    font-weight: 500;
    line-height: 12px;
    display: flex;
    align-items: center;
    gap: $spacing-8;
  }
}
</style>
