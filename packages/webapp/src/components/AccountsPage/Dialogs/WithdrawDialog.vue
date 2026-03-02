<script lang="ts" setup>
import type { SuppliedCardTableItem } from '~/types/table'
import { calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk'
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
  const obligation = userStore.state.obligations[String(activeMarket.value?.marketName)]
  const pools = activeMarket.value?.marketState.pools_data
  const assetDecimals = activeMarket.value?.marketState.asset_decimals ?? 7
  const oraclePriceDecimals = activeMarket.value?.marketState.oracle_price_decimals ?? 0
  if (!obligation || !pools) {
    return 0
  }
  return calcUserTotalStakeInUsd(obligation, pools, assetDecimals, oraclePriceDecimals) ?? 0
})

const userTotalBorrowByMarket = computed(() => {
  const obligation = userStore.state.obligations[String(activeMarket.value?.marketName)]
  const pools = activeMarket.value?.marketState.pools_data
  const assetDecimals = activeMarket.value?.marketState.asset_decimals ?? 7
  const oraclePriceDecimals = activeMarket.value?.marketState.oracle_price_decimals ?? 0
  if (!obligation || !pools) {
    return 0
  }
  return calcUserTotalBorrowedInUsd(obligation, pools, assetDecimals, oraclePriceDecimals) ?? 0
})

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const market = useMarketActions()

const amount = toRef(market, 'withdrawAmount')
const collateralOnly = toRef(market, 'collateralOnly')

const dialog = defineModel({ default: false })

const loading = ref(false)
const reloadFee = ref(false)

const isValidate = ref(true)

const txFee = ref(0)
const poolFee = ref(0)

const collateralBalance = computed(() => Number(data?.collateral) || 0)
const supplyBalance = computed(() => Number(data?.balance || 0) - collateralBalance.value)
const totalSuppliedBalance = computed(() => Number(data?.balance) || 0)
const remainingBalance = computed(() => Number(collateralOnly.value ? collateralBalance.value : supplyBalance.value) - amount.value)

const closeLTV = computed(() => data?.raw.pool.config.health_config.close_ltv_bps ? Number(data.raw.pool.config.health_config.close_ltv_bps) / 10_000 : 0)
const openLtv = computed(() => data?.raw.pool.config.health_config.open_ltv_bps ? Number(data.raw.pool.config.health_config.open_ltv_bps) / 10_000 : 0)

const healthFactor = computed(() => {
  const price = Number(data?.price || 0)
  const withdrawUsd = Number(amount.value || 0) * price
  const depositedAfterWithdraw = Math.max(userTotalDepositByMarket.value - withdrawUsd, 0)
  const borrowed = userTotalBorrowByMarket.value

  const result = borrowed === 0 ? 10 : Math.max((depositedAfterWithdraw * closeLTV.value) / borrowed, 0)
  return Math.min(result, 10)
})

const poolLimit = computed(() => {
  if (!data) {
    return 0
  }
  const limit = collateralOnly.value ? data.raw.pool.total_collateral : data.raw.total_available_adjusted
  return Math.max(Number(bigintToNumber(limit, data.assetDecimals)), 0)
})

const availableToWithdraw = computed(() => {
  const price = Number(data?.price || 1)
  const deposited = userTotalDepositByMarket.value
  const borrowed = userTotalBorrowByMarket.value

  const targetDeposit = borrowed / openLtv.value
  const maxWithdrawUsd = Math.max(deposited - targetDeposit, 0)
  const maxWithdrawAmount = maxWithdrawUsd / price
  const balance = collateralOnly.value ? collateralBalance.value : supplyBalance.value
  return Math.min(balance, maxWithdrawAmount)
})

const availableToWithdrawWithPoolLimit = computed(() => {
  return Math.min(Number(truncatePercent(availableToWithdraw.value, 7)), Number(poolLimit.value))
})

const infoTableData = computed(() => {
  if (!data) {
    return {}
  }
  return {
    healthFactor: {
      label: 'Health Factor',
      value: truncatePercent(healthFactor.value || 0, 2),
    },
    totalTupply: {
      label: 'Total Supply',
      value: `${shortenNumber(totalSuppliedBalance.value || 0, 2, maxDecimalsForShortenNumber(totalSuppliedBalance.value))} ${data?.asset.symbol}`,
    },
    supplyBalance: {
      label: 'Supply Balance',
      value: `${shortenNumber(supplyBalance.value || 0, 2, maxDecimalsForShortenNumber(supplyBalance.value))} ${data?.asset.symbol}`,
    },
    collateralBalance: {
      label: 'Collateral Balance',
      value: `${shortenNumber(collateralBalance.value || 0, 2, maxDecimalsForShortenNumber(collateralBalance.value))} ${data?.asset.symbol}`,
    },
    remaining: {
      label: 'Remaining Supply',
      value: `${shortenNumber(Math.max(remainingBalance.value || 0, 0), 2, maxDecimalsForShortenNumber(remainingBalance.value))} ${data?.asset.symbol}`,
    },
    available: {
      label: 'Available for Withdrawal',
      value: `${shortenNumber(availableToWithdraw.value || 0, 2, maxDecimalsForShortenNumber(availableToWithdraw.value))} ${data?.asset.symbol}`,
    },
    poolLimit: {
      label: 'Pool Withdrawal Limit',
      value: `${shortenNumber(poolLimit.value || 0, 2, maxDecimalsForShortenNumber(poolLimit.value))} ${data?.asset.symbol}`,
    },
    poolFee: {
      label: 'Pool Withdrawal Limit',
      value: `${shortenNumber(poolLimit.value || 0, 2, maxDecimalsForShortenNumber(poolLimit.value))} ${data?.asset.symbol}`,
    },
    txFee: {
      label: 'Transaction Fee',
      value: `${txFee.value || 0} XLM`,
    },
  }
})

async function withdraw() {
  if (!data) {
    return
  }
  if (!amount.value || amount.value <= 0 || amount.value > availableToWithdrawWithPoolLimit.value) {
    focusInput('.withdraw-dialog__input')
    return
  }
  try {
    loading.value = true
    isValidate.value = false

    const marketProps = {
      market: activeMarket.value!.marketName,
      client: activeMarket.value!.client,
      pool_address: data?.pool_address,
      amount: amount.value,
      asset_data: data?.raw?.pool.name,
      limit: collateralBalance.value,
      withBuffer: Number(availableToWithdraw.value) === Number(amount.value),
    }

    collateralOnly.value
      ? await market.removeCollateral(marketProps)
      : await market.withdraw({ ...marketProps, limit: supplyBalance.value })

    dialog.value = false
  } finally {
    loading.value = false
    isValidate.value = true
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

watchDebounced(amount, async (a) => {
  if (!a || Number(a) <= 0) {
    poolFee.value = 0
    return
  }
  const feeData = await activeMarket.value?.client.market.simulateWithdraw(publicKey.value, data!.pool_address, a)
  const feeSum = feeData?.operation_fees?.fee_sum
  poolFee.value = feeSum && data?.assetDecimals ? Number(bigintToNumber(feeSum, data.assetDecimals)) : 0
}, { debounce: 500 })

watch(() => data, (d) => {
  if (!d) {
    dialog.value = false
  }
})

watchDebounced([
  () => data,
  reloadFee,
  publicKey,
], async ([d, _r]) => {
  if (!d?.pool_address || !publicKey.value) {
    return
  }

  const tx = await activeMarket.value?.client.lending.buildWithdrawTx(
    publicKey.value,
    d?.pool_address || '',
    0,
  )
  txFee.value = activeMarket.value?.client.lending.getTransactionFee(tx) ?? 0
}, { immediate: true, debounce: 300 })

watch(collateralBalance, (b) => {
  if (b <= 0) {
    collateralOnly.value = false
  }
}, { immediate: true })
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name=" dialog-default"
  >
    <template #header>
      <div class="dialog-default__title">
        <img
          :src="data?.asset.icon"
          :alt="`${data?.asset.symbol} icon`"
        >
        <span>Withdraw {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="dialog-default__body">
      <input-widget
        v-model="amount"
        :balance="availableToWithdrawWithPoolLimit"
        class="withdraw-dialog__input mb-2"
        :price="data?.price"
        label-left="Amount"
        :label-right="`${formatPrice(availableToWithdrawWithPoolLimit ?? 0, 0, 4)} ${data?.asset.symbol}`"
        :reset="dialog"
        variant="success"
        :rules="[
          (v) => {
            return !isValidate || Number(v) <= availableToWithdrawWithPoolLimit || 'Withdraw limit exceeded'
          },
        ]"
      />

      <template v-if="Object.keys(infoTableData).length > 0">
        <!-- Balances -->
        <div
          class="dialog-info-card dialog-info-card--success"
        >
          <div class="dialog-info-card__title">
            Balances
          </div>

          <div class="dialog-info-card__body">
            <!-- Total -->
            <div class="dialog-info-card__item">
              <span class="label">{{ infoTableData.totalTupply?.label }}</span>
              <span class="value">
                {{ infoTableData.totalTupply?.value }}
              </span>
            </div>

            <!-- Supply -->
            <div class="dialog-info-card__item">
              <span class="label">{{ infoTableData.supplyBalance?.label }}</span>
              <span class="value">
                {{ infoTableData.supplyBalance?.value }}
              </span>
            </div>

            <!-- Collateral -->
            <div class="dialog-info-card__item">
              <span class="label">{{ infoTableData.collateralBalance?.label }}</span>
              <span class="value">
                {{ infoTableData.collateralBalance?.value }}
              </span>
            </div>
          </div>
        </div>

        <!-- Health -->
        <div
          class="dialog-info-card dialog-info-card--success"
        >
          <div class="dialog-info-card__title">
            Health
          </div>

          <div class="dialog-info-card__body">
            <!-- Health -->
            <div class="dialog-info-card__item">
              <span class="label">{{ infoTableData.healthFactor?.label }}</span>
              <span class="value">
                {{ infoTableData.healthFactor?.value }}
              </span>
            </div>

            <!-- Remaining -->
            <div class="dialog-info-card__item">
              <span class="label">{{ infoTableData.remaining?.label }}</span>
              <span class="value">
                {{ infoTableData.remaining?.value }}
              </span>
            </div>

            <!-- Available -->
            <div class="dialog-info-card__item">
              <span class="label">{{ infoTableData.available?.label }}</span>
              <span class="value">
                {{ infoTableData.available?.value }}
              </span>
            </div>
          </div>
        </div>

        <!-- Pool Info -->
        <div
          class="dialog-info-card dialog-info-card--success"
        >
          <div class="dialog-info-card__title">
            Pool Info
          </div>

          <div class="dialog-info-card__body">
            <!-- pool limit -->
            <div class="dialog-info-card__item">
              <span class="label">{{ infoTableData.poolLimit?.label }}</span>
              <span class="value">
                {{ infoTableData.poolLimit?.value }}
              </span>
            </div>
          </div>
        </div>

        <!-- Fees -->
        <div
          class="dialog-info-card dialog-info-card--success"
        >
          <div class="dialog-info-card__title">
            Fees
          </div>

          <div class="dialog-info-card__body">
            <!-- pool fee -->
            <div class="dialog-info-card__item">
              <span class="label">{{ infoTableData.poolFee?.label }}</span>
              <span class="value">
                {{ infoTableData.poolFee?.value }}
              </span>
            </div>

            <!-- tx fee -->
            <div class="dialog-info-card__item">
              <span class="label">{{ infoTableData.txFee?.label }}</span>
              <span class="value">
                {{ infoTableData.txFee?.value }}
              </span>
            </div>
          </div>
        </div>
      </template>

      <!-- <div class="dialog-info-table">
        <div
          v-for="item in infoTableData"
          :key="item.label"
          class="dialog-info-table__item"
        >
          <span>{{ item?.label }}</span>
          <span>
            <template v-if="item?.name === 'healthFactor' && loading">
              <j-loading-spinner
                width="10px"
                style="padding: 0; width: 14px; margin-left: auto"
              />
            </template>
            <template v-else>
              {{ item?.value }}
            </template>
          </span>
        </div>

        <div class="separator" />
      </div> -->

      <j-toggle
        v-if="collateralBalance > 0"
        v-model="collateralOnly"
        color="#00c950"
        class="my-2"
      >
        <template #append>
          Collateral Balance
        </template>
      </j-toggle>

      <div class="dialog-default__action">
        <j-btn
          :loading="loading"
          variant="success"
          size="lg"
          pill
          @click="withdraw"
        >
          Withdraw {{ data?.asset.symbol }}
        </j-btn>
      </div>
    </div>
  </j-dialog>
</template>
