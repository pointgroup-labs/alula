<script lang="ts" setup>
import type { SupplyTableItem } from '~/types/table'

const {
  data,
  modelValue,
} = defineProps<{
  data?: SupplyTableItem
  modelValue: boolean
}>()

const emits = defineEmits(['update:modelValue'])

const Toast = useToast()

const connection = useConnectionStore()
const balance = computed(() => {
  // const asset = data?.asset.symbol
  // const balances = connection.balances
  // const assetBalance = asset === 'XLM'
  //   ? balances?.native.balance
  //   : balances?.tokens.find((b: ParsedBalance) => b.asset === asset)?.balance
  // return Number(assetBalance) || 0
  return 0
})

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
    value: '5.00 XLM',
  },
  {
    label: 'Transaction Fee',
    value: '0.004 XLM',
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

const loading = ref(false)

const amount = ref(0)

async function repay() {
  try {
    loading.value = true
    Toast.create({
      modelValue: 50_000,
      title: 'Withdraw Success',
      body: `You withdraw ${amount.value} ${data?.asset.symbol}`,
      alertProps: {
        variant: 'success',
      },
    })
  } catch (error) {
    Toast.create({
      title: 'Withdraw Error',
      body: String(error),
      alertProps: {
        variant: 'error',
      },
    })
  } finally {
    loading.value = false
  }
}

watch(() => modelValue, (v) => {
  if (!v) {
    amount.value = 0
  }
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
        :balance="balance"
        :rules="[
          (v) => {
            return v && Number(v) < balance || 'Insufficient balance'
          },
        ]"
      >
        <template #label-right>
          Amount: {{ balance }} {{ data?.asset.symbol }}
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
          @click="repay"
        >
          Withdraw {{ data?.asset.symbol }}
        </j-btn>
      </div>
    </div>
  </j-dialog>
</template>
