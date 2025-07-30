<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'
import { RELOAD_FEE_INTERVAL } from '~/config'
import { bigintToNumber, focusInput, formatPrice } from '~/utils'

const {
  data,
} = defineProps<{
  data?: MultiplyTableItem
}>()

const marketsStore = useMarketsStore()
const market = useMarket()

const userStore = useUserStore()
const obligation = computed(() => userStore.userObligation)

const amount = toRef(market, 'withdrawAmount')

const clientStore = useClientStore()
const jLendClient = computed(() => clientStore.jLendClient)

const decimals = computed(() => clientStore.assetDecimals)

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const balance = computed(() => {
  if (!data) {
    return 0
  }
  const depositPool = data.depositPool
  const depositAsset = obligation.value?.deposits.find((deposit: any) => deposit.includes(depositPool?.pool_address))
  if (!depositAsset) {
    return 0
  }
  const userShares = depositAsset[1].shares || 0
  const userPoolSharesInPercentage = Number(userShares) / Number(depositPool.total_shares || 0)
  const available = Number(bigintToNumber(depositPool.available, decimals.value))
  const totalBorrowed = Number(bigintToNumber(depositPool.total_borrowed, decimals.value))
  const totalSupplied = available + totalBorrowed
  const userSupplied = totalSupplied * userPoolSharesInPercentage
  return userSupplied || 0
})

const loading = computed(() => marketsStore.poolDepositAddr === data?.depositPool.pool_address)
const reloadFee = ref(false)

const txFee = ref(0)

watchDebounced([
  () => data,
  reloadFee,
], async ([d, _r]) => {
  if (!d || !publicKey.value) {
    return
  }
  const tx = await jLendClient.value?.sdk.withdrawLeverageTx(
    publicKey.value,
    d?.depositPool.pool_address || '',
    d?.borrowPool.pool_address || '',
    1,
  )
  txFee.value = jLendClient.value.sdk.getTransactionFee(tx)
}, { immediate: true, debounce: 300 })

const infoTableData = computed(() => {
  if (!data) {
    return []
  }

  return [
    // {
    //   name: 'liquidity',
    //   label: 'Liquidity Available',
    //   value: 0,
    // },
    {
      name: 'txFee',
      label: 'Transaction Fee',
      value: `${txFee.value || 0} ${data.asset.symbol}`,
    },
    // {
    //   name: 'maxApy',
    //   label: 'Max APY',
    //   value: `${truncatePercent(data.maxAPY || 0, 2)} %`,
    // },
    // {
    //   name: 'maxMultiply',
    //   label: 'Max Multiply',
    //   value: `${truncatePercent(supplyLimit.value || 0, 2)} ${data.asset.symbol}`,
    // },
    // {
    //   name: 'multiplier',
    //   label: 'Avg. Multiplier',
    //   value: '90.00 %',
    // },
    // {
    //   name: 'supplied',
    //   label: 'Total Supplied',
    //   value: `${formatPrice(data.supplied || 0, 2, 2)} ${data.asset.symbol}`,
    // },
  ]
})

const dialog = defineModel<boolean>({
  default: false,
})

async function withdrawLeverage() {
  if (!publicKey.value || !data?.depositPool.pool_address) {
    return
  }
  if (!amount.value || amount.value <= 0) {
    focusInput('.multiply-dialog')
    return
  }
  const deposit_pool_address = data?.depositPool.pool_address
  const borrow_pool_address = data?.borrowPool.pool_address
  const asset_code = data?.depositPool.token_ticker
  if (!deposit_pool_address || !borrow_pool_address) {
    return
  }
  await market.withdrawLeverage(
    deposit_pool_address,
    borrow_pool_address,
    Number(amount.value),
    asset_code,
  )
}

let interval: string | number | NodeJS.Timeout | undefined

watch(dialog, async (v) => {
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
    class-name="multiply-dialog"
  >
    <template #header>
      <div class="multiply-dialog__title">
        <span>Withdraw {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="multiply-dialog__body">
      <input-widget
        v-model="amount"
        :balance="balance"
        class="multiply-dialog__input"
        :icon="data?.asset.icon"
        label-left="You Deposit"
        :rules="[
          (v) => v && Number(v) < balance || 'Insufficient balance',
        ]"
      >
        <template #label-right>
          Multiplied: {{ formatPrice(balance, 0, 7) }} {{ data?.asset.symbol }}
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
          <span>{{ item?.value }}</span>
        </div>
      </div>

      <div class="multiply-dialog-action">
        <market-dialog-action-btn
          variant="primary"
          :loading="loading"
          :pool="data?.depositPool"
          @click-handler="withdrawLeverage"
        >
          Withdraw {{ data?.asset.symbol }}
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
