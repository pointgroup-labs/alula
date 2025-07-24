<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { POOL_REMAINING_BALANCE, RELOAD_FEE_INTERVAL } from '~/config'
import { destructurePoolAsset, focusInput, formatPrice, generateExplorerLink, shortenAddress } from '~/utils'

const {
  data,
  modelValue,
} = defineProps<{
  data?: MarketTableItem
  modelValue: boolean
}>()

const emits = defineEmits(['update:modelValue'])

const marketsStore = useMarketsStore()
const market = useMarket()

const amount = toRef(market, 'depositAmount')
const collateralOnly = toRef(market, 'collateralOnly')

const clientStore = useClientStore()
const jLendClient = computed(() => clientStore.jLendClient)

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const balance = computed(() => {
  if (!data) {
    return 0
  }
  if (data.raw.token_ticker === 'XLM') {
    return wallet.nativeBalance
  }
  const [, asset_issuer] = destructurePoolAsset(data?.raw.name)
  return wallet.getAssetBalance(String(asset_issuer))
})

const loading = computed(() => marketsStore.poolDepositAddr === data?.raw.pool_address)
const reloadFee = ref(false)

const txFee = ref(0)

watchDebounced([
  () => data,
  reloadFee,
], async ([d, _r]) => {
  if (!d || !publicKey.value) {
    return
  }
  const tx = await jLendClient.value?.sdk.depositTx(
    publicKey.value,
    d?.raw.pool_address || '',
    0,
  )
  txFee.value = jLendClient.value.sdk.getTransactionFee(tx)
}, { immediate: true, debounce: 300 })

const supplyLimit = ref(0)

const infoTableData = computed(() => {
  if (!data) {
    return []
  }

  const isSupplyLimited = data.supply_limit && data.supply_limit > 0
  // eslint-disable-next-line vue/no-side-effects-in-computed-properties
  supplyLimit.value = isSupplyLimited ? Math.max(Number(data.supply_limit) || 0 - Number(data.total_supply), 0) : 0
  return [
    {
      name: 'limit',
      label: 'Supply Limit',
      value: isSupplyLimited ? formatPrice(supplyLimit.value || 0, 2, 2) : '-',
    },
    {
      name: 'market',
      label: 'Market',
      value: 'Main',
    },
    {
      name: 'contract',
      label: 'Contract',
      value: data.raw.pool_address || '',
    },
    {
      name: 'fee',
      label: 'Transaction Fee',
      value: `${txFee.value} XLM`,
    },
  ]
})

const dialog = computed({
  get() {
    return modelValue
  },
  set(val) {
    emits('update:modelValue', val)
  },
})

async function supply() {
  if (!publicKey.value || !data?.raw.pool_address) {
    return
  }
  if (!amount.value || amount.value <= 0) {
    focusInput('.supply-dialog__input')
    return
  }
  collateralOnly.value
    ? await market.addCollateral(data?.raw.pool_address, amount.value, data?.raw.name)
    : await market.deposit(data?.raw.pool_address, amount.value, data?.raw.name)
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
        :limit="supplyLimit"
        :fee="POOL_REMAINING_BALANCE + txFee"
        class="supply-dialog__input"
        :rules="[
          (v) => {
            return v && Number(v) < balance || 'Insufficient balance'
          },
          (v) => {
            return (supplyLimit <= 0 || Number(v) <= supplyLimit) || 'Pool supply limit'
          },
        ]"
      >
        <template #label-right>
          Wallet: {{ balance }} {{ data?.asset.symbol }}
        </template>
      </input-widget>

      <div
        v-if="infoTableData.length > 0"
        class="dialog-info-table"
      >
        <div
          v-for="item in infoTableData"
          :key="item?.label"
          class="dialog-info-table__item"
        >
          <span>{{ item?.label }}</span>
          <template v-if="item?.name === 'contract'">
            <a
              :href="generateExplorerLink(String(item?.value), 'contract')"
              target="_blank"
            >{{ shortenAddress(item?.value, 5) }}
              <i-app-export-icon />
            </a>
          </template>
          <span v-else>{{ item?.value }}</span>
        </div>
      </div>

      <j-toggle
        v-model="collateralOnly"
        size="small"
      >
        <template #append>
          Collateral Only
        </template>
      </j-toggle>

      <div class="supply-dialog-action">
        <div class="action-info">
          <span>Supply APY</span>
          <span>{{ data?.deposit_apy }}</span>
        </div>

        <market-dialog-action-btn
          variant="primary"
          :loading="loading"
          :pool="data?.raw"
          @click-handler="supply"
        >
          Supply {{ data?.asset.symbol }}
        </market-dialog-action-btn>
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

  .j-toggle__label {
    font-size: 14px;
  }

  .supply-dialog-action {
    display: flex;
    justify-content: space-between;
    gap: $spacing-32;

    .action-info {
      white-space: nowrap;
      flex: 1;
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
