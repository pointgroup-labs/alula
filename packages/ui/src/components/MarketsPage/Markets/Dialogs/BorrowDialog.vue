<script lang="ts" setup>
import type { BorrowTableItem } from '~/types/table'

const {
  data,
  modelValue,
} = defineProps<{
  data?: BorrowTableItem
  modelValue: boolean
}>()

const emits = defineEmits(['update:modelValue'])

const Toast = useToast()

const agree = ref(false)

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
  //   const { trust_ratio, risk_floor, price } = data
  return [{
    label: 'Health Factor',
    value: 1.04,
  },
  {
    label: 'Liquidation at',
    value: '< 1.0',
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

async function supply() {
  try {
    loading.value = true
    Toast.create({
      modelValue: 50_000,
      title: 'Supply Success',
      body: `You supplied ${amount.value} XLM`,
      alertProps: {
        variant: 'success',
      },
    })
  } catch (error) {
    Toast.create({
      title: 'Supply Error',
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
    class-name="supply-dialog borrow-dialog"
  >
    <template #header>
      <div class="supply-dialog__title">
        <img
          :src="data?.asset.icon"
          :alt="`${data?.asset.symbol} icon`"
        >
        <span>Supply {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="supply-dialog__body">
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
          <span>Supply APY</span>
          <span>{{ data?.borrow_apy }}</span>
        </div>

        <j-btn
          :disabled="!agree"
          :loading="loading"
          size="lg"
          variant="accent"
          pill
          @click="supply"
        >
          Borrow {{ data?.asset.symbol }}
        </j-btn>
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
