<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { destructurePoolAsset, focusInput, formatPrice, generateExplorerLink, shortenAddress, truncatePercent } from '~/utils'

const {
  data,
} = defineProps<{
  data?: MultiplyTableItem
}>()

function getMaxDeposit(liquidity: number, multiplier: number): number {
  if (multiplier <= 1) { return liquidity } // no loop
  return liquidity / multiplier
}

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

const precentFromMaxMultiply = ref(90)

const maxMultiply = computed(() => data?.multiplier || 0)
const selectedMultiplier = computed(() => {
  return Number((precentFromMaxMultiply.value / 100) * maxMultiply.value).toFixed(2)
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

  const liquidity = data.liquidity / data.price
  const maxDeposit = getMaxDeposit(liquidity, Number(selectedMultiplier.value) || 0)

  return [
    {
      name: 'liquidity',
      label: 'Liquidity Available',
      value: `${formatPrice(maxDeposit || 0, 2, 2)} ${data.asset.symbol}`,
    },
    {
      name: 'maxApy',
      label: 'Max APY',
      value: `${truncatePercent(data.maxAPY || 0, 2)} %`,
    },
    {
      name: 'multiplier',
      label: 'Avg. Multiplier',
      value: '90.00 %',
    },
    {
      name: 'supplied',
      label: 'Total Supplied',
      value: `${formatPrice(data.supplied || 0, 2, 2)} ${data.asset.symbol}`,
    },
  ]
})

const dialog = defineModel<boolean>({
  default: false,
})

async function supply() {
  if (!publicKey.value || !data?.raw.pool_address) {
    return
  }
  if (!amount.value || amount.value <= 0) {
    focusInput('.multiply-dialog')
    return
  }
  console.log('MULTIPLIER', selectedMultiplier.value)
}

let interval: string | number | NodeJS.Timeout | undefined

watch(dialog, async (v) => {
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
    class-name="multiply-dialog"
  >
    <template #header>
      <div class="multiply-dialog__title">
        <span>Multiply {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="multiply-dialog__body">
      <input-widget
        v-model="amount"
        :balance="balance"
        :limit="supplyLimit"
        class="multiply-dialog__input"
        :icon="data?.asset.icon"
        label-left="You Deposit"
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

      <loop-multiply-select
        v-model="precentFromMaxMultiply"
        :multiplier="selectedMultiplier"
        :max-multiply="Number(data?.multiplier).toFixed(0) || 0"
      />

      <div class="multiply-dialog-action">
        <market-dialog-action-btn
          variant="primary"
          :loading="loading"
          :pool="data?.raw"
          @click-handler="supply"
        >
          Multiply {{ data?.asset.symbol }}
        </market-dialog-action-btn>
      </div>
    </div>
  </j-dialog>
</template>

<style lang="scss">
.multiply-dialog {
  .modal-dialog {
    min-width: 350px;
    width: 350px;
  }

  &__title {
    color: $dark;
    font-size: 20px;
    font-style: normal;
    font-weight: 500;
    line-height: 20px;
  }

  &__body {
    padding-top: $spacing-16;
    display: flex;
    flex-direction: column;
    gap: $spacing-16;
  }

  .multiply-dialog-action {
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
