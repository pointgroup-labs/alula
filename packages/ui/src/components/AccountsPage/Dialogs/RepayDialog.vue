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
  return /* Number(assetBalance) ||  */0
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
    label: 'Borrow balance after repay',
    value: '5.00 XLM',
  },
  {
    label: 'Collateral balance after repay',
    value: '10.14 USDC',
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
      title: 'Repay Success',
      body: `You repaid ${amount.value} ${data?.asset.symbol}`,
      alertProps: {
        variant: 'success',
      },
    })
  } catch (error) {
    Toast.create({
      title: 'Repay Error',
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
        <span>Repay {{ data?.asset.symbol }}</span>
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
          <span>{{ item?.value }}</span>
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

<style lang="scss">
.account-dialog {
  .modal-dialog {
    min-width: 350px;
    width: 350px;
  }

  &__title {
    display: flex;
    align-items: center;
    gap: $spacing-8;
    font-size: 20px;
    font-style: normal;
    font-weight: 400;
    line-height: 20px;

    img {
      width: 40px;
      height: 40px;
      object-fit: contain;
      border-radius: 50%;
    }
  }

  &__body {
    padding-top: $spacing-16;
    display: flex;
    flex-direction: column;
    gap: $spacing-16;
  }

  .account-info-table {
    display: flex;
    flex-direction: column;

    &__item {
      display: grid;
      grid-template-columns: 1fr 1fr;
      justify-content: space-between;
      font-size: 11px;
      font-style: normal;
      font-weight: 500;
      line-height: 12px;

      &:nth-child(even) {
        background-color: $neutral-2;
      }

      span {
        padding: $spacing-6 $spacing-16;

        &:nth-child(2) {
          text-align: right;
        }
      }
    }
  }

  .account-dialog-action {
    display: flex;
    justify-content: space-between;
    gap: $spacing-32;

    .btn {
      width: 192px;
      margin-left: auto;
    }
  }
}
</style>
