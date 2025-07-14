<script lang="ts" setup>
import type { SuppliedCardTableItem } from '~/types/table'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { shortenNumber, truncatePercent } from '~/utils'

const {
  data,
  modelValue,
} = defineProps<{
  data?: SuppliedCardTableItem
  modelValue: boolean
}>()

const emits = defineEmits(['update:modelValue'])

const clientStore = useClientStore()
const jLendClient = computed(() => clientStore.jLendClient)

const userStore = useUserStore()
const userTotalDepositInUsd = computed(() => userStore.userTotalDepositInUsd)
const userTotalBorrowedInUsd = computed(() => userStore.userTotalBorrowedInUsd)

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const market = useMarket()

const collateralOnly = toRef(market, 'collateralOnly')

const loading = ref(false)
const reloadFee = ref(false)

const amount = ref(0)
const txFee = ref(0)

const collateralBalance = computed(() => Number(data?.collateral) || 0)
const supplyBalance = computed(() => Number(data?.balance || 0) - collateralBalance.value)
const totalSuppliedBalance = computed(() => Number(data?.balance) || 0)
const remainingBalance = computed(() => Number(collateralOnly.value ? collateralBalance.value : supplyBalance.value) - amount.value)

const closeLTV = computed(() => data?.raw.config.close_ltv_bps ? Number(data.raw.config.close_ltv_bps) / 10_000 : 0)

const helthFactor = computed(() => {
  const A = Number(amount.value || 0) * Number(data?.price || 0)
  const D = (userTotalDepositInUsd.value * closeLTV.value) - A
  const B = userTotalBorrowedInUsd.value
  const result = B === 0 ? 0 : Math.max(D / B, 0)
  return Math.min(result, 10)
})

const availableToWithdraw = computed(() => {
  const balance = collateralOnly.value
    ? collateralBalance.value
    : supplyBalance.value

  const D = userTotalDepositInUsd.value
  const borrowed = userTotalBorrowedInUsd.value

  let limitUsd

  if (borrowed <= 0) {
    limitUsd = D
  } else {
    const minDeposit = borrowed / closeLTV.value
    limitUsd = D - minDeposit
  }

  const limitAsset = limitUsd / Number(data?.price || 1)

  return Math.max(Math.min(balance, limitAsset), 0)
})

watchDebounced([
  () => data,
  reloadFee,
], async ([d, _r]) => {
  if (!d || !publicKey.value) {
    return
  }
  const tx = await jLendClient.value?.sdk.withdrawTx(
    publicKey.value,
    d?.pool_address || '',
    0,
  )
  txFee.value = jLendClient.value.sdk.getTransactionFee(tx)
}, { immediate: true, debounce: 300 })

const infoTableData = computed(() => {
  if (!data) {
    return []
  }
  return [{
    label: 'Health Factor',
    value: truncatePercent(helthFactor.value, 2),
  },
  {
    label: 'Total supply',
    value: `${shortenNumber(totalSuppliedBalance.value)} ${data?.asset.symbol}`,
  },
  {
    label: 'Deposited balance',
    value: `${shortenNumber(supplyBalance.value)} ${data?.asset.symbol}`,
  },
  {
    label: 'Collateral balance',
    value: `${shortenNumber(collateralBalance.value)} ${data?.asset.symbol}`,
  },
  {
    label: 'Remaining supply',
    value: `${shortenNumber(Math.max(remainingBalance.value, 0))} ${data?.asset.symbol}`,
  },
  {
    label: 'Available to withdraw',
    value: `${shortenNumber(availableToWithdraw.value)} ${data?.asset.symbol}`,
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

async function withdraw() {
  if (!data) {
    return
  }
  try {
    loading.value = true
    collateralOnly.value
      ? await market.removeCollateral(data?.pool_address, amount.value, collateralBalance.value, data?.asset.symbol)
      : await market.withdraw(data?.pool_address, amount.value, supplyBalance.value, data?.asset.symbol)
    amount.value = 0
  } catch {
    if (!amount.value || amount.value <= 0) {
      const input = document.querySelector('.withdraw-dialog__input')?.querySelector('input') as HTMLInputElement
      input?.focus()
    }
  } finally {
    loading.value = false
  }
}

let interval: string | number | NodeJS.Timeout | undefined

watch(() => modelValue, async (v) => {
  clearInterval(interval)
  if (!v) {
    amount.value = 0
    collateralOnly.value = false
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
    class-name="account-dialog"
  >
    <template #header>
      <div class="account-dialog__title">
        <img
          :src="data?.asset.icon"
          :alt="`${data?.asset.symbol} icon`"
        >
        <span>Withdraw {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="account-dialog__body">
      <input-widget
        v-model="amount"
        :balance="availableToWithdraw"
        class="withdraw-dialog__input"
        :rules="[
          (v) => {
            return v && Number(v) <= availableToWithdraw || 'Withdraw limit exceeded'
          },
        ]"
      >
        <template #label-right>
          Amount: {{ availableToWithdraw.toFixed(5) }} {{ data?.asset.symbol }}
        </template>
      </input-widget>

      <div class="account-info-table">
        <div
          v-for="item in infoTableData"
          :key="item.label"
          class="account-info-table__item"
        >
          <span>{{ item?.label }}</span>
          <span>{{ item?.value }}</span>
        </div>
      </div>

      <j-toggle
        v-if="collateralBalance > 0"
        v-model="collateralOnly"
      >
        <template #append>
          Collateral Balance
        </template>
      </j-toggle>

      <div class="account-dialog-action">
        <j-btn
          :loading="loading"
          variant="dark"
          size="md"
          pill
          @click="withdraw"
        >
          Withdraw {{ data?.asset.symbol }}
        </j-btn>
      </div>
    </div>
  </j-dialog>
</template>
