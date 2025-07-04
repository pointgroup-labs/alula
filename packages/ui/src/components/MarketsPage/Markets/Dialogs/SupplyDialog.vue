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

const wallet = useWallet()
const balance = computed(() => {
  const asset = data?.asset.symbol
  const assetBalance = asset === 'XLM'
    ? wallet.nativeBalance
    : wallet.getAssetBalance(String(asset))
  return Number(assetBalance) || 0
})

const infoTableData = computed(() => {
  if (!data) {
    return []
  }
  const { trust_ratio, risk_floor, price } = data
  return [{
    label: 'Trust ratio',
    value: trust_ratio,
  },
  {
    label: 'Risk floor',
    value: risk_floor,
  },
  {
    label: 'Price',
    value: price,
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

watch(() => modelValue, async (v) => {
  if (!v) {
    amount.value = 0
  }
})
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="supply-dialog"
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

      <div class="supply-dialog-action">
        <div class="action-info">
          <span>Supply APY</span>
          <span>{{ data?.deposit_apy }}</span>
        </div>

        <j-btn
          :loading="loading"
          size="md"
          pill
          @click="supply"
        >
          Supply {{ data?.asset.symbol }}
        </j-btn>
      </div>
    </div>
  </j-dialog>
</template>

<style lang="scss">
.supply-dialog {
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

  .supply-info-table {
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

  .supply-dialog-action {
    display: flex;
    justify-content: space-between;
    gap: $spacing-32;

    .action-info {
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
