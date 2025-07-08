<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { normalizeAssetAmount } from '~/client/utils'
import { RELOAD_FEE_INTERVAL, TEST_PUBKEY } from '~/config'
import { getZeroCountAfterDecimal, truncatePercent } from '~/utils'

const {
  data,
  modelValue,
} = defineProps<{
  data?: MarketTableItem
  modelValue: boolean
}>()

const emits = defineEmits(['update:modelValue'])

const Toast = useToast()

const clientStore = useClientStore()
const jLendClient = computed(() => clientStore.jLendClient)

const agree = ref(false)

const reloadFee = ref(false)
const txFee = ref(0)

watchDebounced([
  () => data,
  reloadFee,
], async ([d, _r]) => {
  if (!d) {
    return
  }
  const tx = await jLendClient.value?.sdk.borrow(
    TEST_PUBKEY,
    d?.raw.pool_address || '',
    1,
  )

  txFee.value = jLendClient.value.sdk.getTransactionFee(tx)
}, { immediate: true, debounce: 300 })

const wallet = useWallet()
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

const infoTableData = computed(() => {
  if (!data) {
    return []
  }
  const available = normalizeAssetAmount(Number(data.available), jLendClient.value.sdk.assetDecimals)
  const availableDecimals = String(balance).includes('e') ? getZeroCountAfterDecimal(available) : null
  const availableString = availableDecimals ? available.toFixed(availableDecimals) : String(available)
  const liquidation = Number(data.raw.config.liquidation_close_factor_bps) / 100
  const closeLTV = Number(data.raw.config.close_ltv_bps) / 100
  return [{
    label: 'Health Factor',
    value: 1.04,
  },
  {
    label: 'Available amount to borrow',
    value: availableString,
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
          <span>Borrow APY</span>
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
