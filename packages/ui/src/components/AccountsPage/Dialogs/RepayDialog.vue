<script lang="ts" setup>
import type { BorrowCardTableItem } from '~/types/table'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { shortenNumber } from '~/utils'

const {
  data,
  modelValue,
} = defineProps<{
  data?: BorrowCardTableItem
  modelValue: boolean
}>()

const emits = defineEmits(['update:modelValue'])

const marketStore = useMarketsStore()
const market = useMarket()

const clientStore = useClientStore()
const jLendClient = computed(() => clientStore.jLendClient)

const wallet = useWallet()
const balance = computed(() => {
  if (!data) {
    return 0
  }
  if (data.asset.symbol === 'XLM') {
    return wallet.nativeBalance
  }
  return wallet.getAssetBalance(String(data.asset_issuer))
})

const loading = ref(false)

const amount = ref(0)

const txFee = ref(0)
const reloadFee = ref(false)

watchDebounced([
  () => data,
  reloadFee,
], async ([d, _r]) => {
  if (!d || !wallet.publicKey) {
    return
  }
  const tx = await jLendClient.value?.sdk.repayTx(
    wallet.publicKey,
    d?.pool_address || '',
    0,
  )
  txFee.value = jLendClient.value.sdk.getTransactionFee(tx)
}, { immediate: true, debounce: 300 })

const infoTableData = computed(() => {
  if (!data) {
    return []
  }
  const borrowBalanceAfterRepay = Math.max(Number(data?.debt) - amount.value || 0, 0)
  return [{
    label: 'Health Factor',
    value: 1.04,
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

const dialog = computed({
  get() {
    return modelValue
  },
  set(val) {
    emits('update:modelValue', val)
  },
})

async function repay() {
  if (!data) {
    return
  }
  try {
    loading.value = true
    marketStore.poolDepositAddr = data?.pool_address

    await market.repay(data?.pool_address, amount.value, balance.value, data?.asset.symbol)
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
        <span>Repay {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="account-dialog__body">
      <input-widget
        v-model="amount"
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
