<script lang="ts" setup>
import type { SuppliedCardTableItem } from '~/types/table'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { shortenNumber } from '~/utils'

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

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const market = useMarket()

const loading = ref(false)

const amount = ref(0)

const txFee = ref(0)
const reloadFee = ref(false)

const availableToWithdraw = computed(() => Math.min(Number(data?.available) || 0, Number(data?.balance) || 0))
const supplyBalance = computed(() => Number(data?.balance) || 0)
const remainingBalance = computed(() => supplyBalance.value - amount.value)

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
    value: 1.04,
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
    await market.withdraw(data?.pool_address, amount.value, supplyBalance.value, data?.asset.symbol)
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
          Amount: {{ supplyBalance.toFixed(5) }} {{ data?.asset.symbol }}
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
