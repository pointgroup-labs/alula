<script lang="ts" setup>
import type { SuppliedCardTableItem } from '~/types/table'
import { calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'
import { CLEAR_DIALOG_TIMEOUT, RELOAD_FEE_INTERVAL } from '~/config'
import { focusInput, shortenNumber, truncatePercent } from '~/utils'

const {
  data,
  modelValue,
} = defineProps<{
  data?: SuppliedCardTableItem
  modelValue: boolean
}>()

const marketsStore = useMarketsStore()

const activeMarket = computed(() => marketsStore.state.markets[String(data?.market)])

const userStore = useUserStore()

const userTotalDepositByMarket = computed(() => {
  const obligation = userStore.state.obligations[String(activeMarket.value?.marketState.name)]
  const pools = activeMarket.value?.pools
  const assetDecimals = marketsStore.assetDecimals
  if (!obligation || !pools) {
    return 0
  }
  return calcUserTotalStakeInUsd(obligation, pools, assetDecimals) ?? 0
})

const userTotalBorrowedByMarket = computed(() => {
  const obligation = userStore.state.obligations[String(activeMarket.value?.marketState.name)]
  const pools = activeMarket.value?.pools
  const assetDecimals = marketsStore.assetDecimals
  if (!obligation || !pools) {
    return 0
  }
  return calcUserTotalBorrowedInUsd(obligation, pools, assetDecimals) ?? 0
})

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const market = useMarket()

const amount = toRef(market, 'withdrawAmount')
const collateralOnly = toRef(market, 'collateralOnly')

const loading = ref(false)
const reloadFee = ref(false)

const txFee = ref(0)

const collateralBalance = computed(() => Number(data?.collateral) || 0)
const supplyBalance = computed(() => Number(data?.balance || 0) - collateralBalance.value)
const totalSuppliedBalance = computed(() => Number(data?.balance) || 0)
const remainingBalance = computed(() => Number(collateralOnly.value ? collateralBalance.value : supplyBalance.value) - amount.value)

const closeLTV = computed(() => data?.raw.config.close_ltv_bps ? Number(data.raw.config.close_ltv_bps) / 10_000 : 0)
const openLtv = computed(() => data?.raw.config.open_ltv_bps ? Number(data.raw.config.open_ltv_bps) / 10_000 : 0)

const healthFactor = computed(() => {
  const price = Number(data?.price || 0)
  const withdrawUsd = Number(amount.value || 0) * price
  const depositedAfterWithdraw = Math.max(userTotalDepositByMarket.value - withdrawUsd, 0)
  const borrowed = userTotalBorrowedByMarket.value

  const result = borrowed === 0 ? 10 : Math.max((depositedAfterWithdraw * closeLTV.value) / borrowed, 0)
  return Math.min(result, 10)
})

const availableToWithdraw = computed(() => {
  const price = Number(data?.price || 1)
  const deposited = userTotalDepositByMarket.value
  const borrowed = userTotalBorrowedByMarket.value

  const targetDeposit = borrowed / openLtv.value
  const maxWithdrawUsd = Math.max(deposited - targetDeposit, 0)
  let maxWithdrawAmount = maxWithdrawUsd / price

  const balance = collateralOnly.value ? collateralBalance.value : supplyBalance.value
  maxWithdrawAmount = Math.min(balance, maxWithdrawAmount)

  return Math.max(maxWithdrawAmount, 0)
})

watchDebounced([
  () => data,
  reloadFee,
  publicKey,
], async ([d, _r]) => {
  if (!d?.pool_address || !publicKey.value) {
    return
  }

  const tx = await activeMarket.value?.client.marketSdk.withdrawTx(
    publicKey.value,
    d?.pool_address || '',
    0,
  )
  txFee.value = activeMarket.value?.client.marketSdk.getTransactionFee(tx) ?? 0
}, { immediate: true, debounce: 300 })

const infoTableData = computed(() => {
  if (!data) {
    return []
  }
  return [{
    name: 'healthFactor',
    label: 'Health Factor',
    value: truncatePercent(healthFactor.value || 0, 2),
  },
  {
    label: 'Total Supply',
    value: `${shortenNumber(totalSuppliedBalance.value || 0)} ${data?.asset.symbol}`,
  },
  {
    label: 'Deposited Balance',
    value: `${shortenNumber(supplyBalance.value || 0)} ${data?.asset.symbol}`,
  },
  {
    label: 'Collateral Balance',
    value: `${shortenNumber(collateralBalance.value || 0)} ${data?.asset.symbol}`,
  },
  {
    label: 'Remaining Supply',
    value: `${shortenNumber(Math.max(remainingBalance.value || 0, 0))} ${data?.asset.symbol}`,
  },
  {
    label: 'Available to Withdraw',
    value: `${shortenNumber(availableToWithdraw.value || 0)} ${data?.asset.symbol}`,
  },
  {
    label: 'Transaction Fee',
    value: `${txFee.value || 0} XLM`,
  }]
})

const dialog = defineModel({ default: false })

async function withdraw() {
  if (!data) {
    return
  }
  if (!amount.value || amount.value <= 0) {
    focusInput('.withdraw-dialog__input')
    return
  }
  try {
    loading.value = true

    const marketProps = {
      market: activeMarket.value!.marketState.name,
      client: activeMarket.value!.client,
      pool_address: data?.pool_address,
      amount: amount.value,
      asset_data: data?.raw?.name,
      limit: collateralBalance.value,
    }

    collateralOnly.value
      ? await market.removeCollateral(marketProps)
      : await market.withdraw({ ...marketProps, limit: supplyBalance.value })
  } finally {
    loading.value = false
  }
}

let interval: string | number | NodeJS.Timeout | undefined

watch(() => modelValue, async (v) => {
  clearInterval(interval)
  if (!v) {
    setTimeout(() => {
      amount.value = 0
    }, CLEAR_DIALOG_TIMEOUT)
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

watch(collateralBalance, (b) => {
  if (b <= 0) {
    collateralOnly.value = false
  }
}, { immediate: true })
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="account-dialog dialog-default"
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
          Amount: {{ availableToWithdraw.toFixed(5) }} {{ data?.asset.symbol }}
        </template>
      </input-widget>

      <div class="account-info-table">
        <div
          v-for="item in infoTableData"
          :key="item.label"
          class="account-info-table__item"
        >
          <span>{{ item?.label }}</span>
          <span>
            <template v-if="item?.name === 'healthFactor' && loading">
              <j-loading-spinner
                :color="isDark ? '#fff' : '#111'"
                width="14px"
                style="padding: 0; width: 14px; margin-left: auto"
              />
            </template>
            <template v-else>
              {{ item?.value }}
            </template>
          </span>
        </div>
      </div>

      <j-toggle
        v-if="collateralBalance > 0"
        v-model="collateralOnly"
      >
        <template #append>
          Collateral Balance
        </template>
      </j-toggle>

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

<style lang="scss">
.account-dialog {
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

body.body--dark {
  .account-dialog {
    .account-info-table__item:nth-child(even) {
      background-color: $neutral-18;
    }

    &-action {
      .btn-dark {
        background-color: $neutral-18;
      }
    }
  }
}
</style>
