<script lang="ts" setup>
import type { SuppliedCardTableItem } from '~/types/table'
import { bpsToNumber, calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk'
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
  return calcUserTotalStakeInUsd(obligation, pools, assetDecimals, oraclePriceDecimals, 'open') ?? 0
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

const openLtv = computed(() => data?.raw.pool.config.health_config.open_ltv_bps ?  bpsToNumber(Number(data.raw.pool.config.health_config.open_ltv_bps)) : 0)

const healthFactor = computed(() => {
  const price = Number(data?.price || 0)
  const withdrawUsd = Number(amount.value || 0) * price * openLtv.value
  const depositedAfterWithdraw = Math.max(userTotalDepositByMarket.value - withdrawUsd, 0)
  const borrowed = userTotalBorrowByMarket.value

  const result = borrowed === 0 ? 10 : Math.max(depositedAfterWithdraw / borrowed, 0)
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
  const depositWithOpenLtv = userTotalDepositByMarket.value
  const borrowed = userTotalBorrowByMarket.value
  const poolOpenLtv = openLtv.value

  const maxWithdrawUsd = poolOpenLtv > 0
    ? Math.max(depositWithOpenLtv - borrowed * 1.1, 0) / poolOpenLtv
    : 0
  const maxWithdrawAmount = maxWithdrawUsd / price
  const balance = collateralOnly.value ? collateralBalance.value : supplyBalance.value
  return Math.min(balance, maxWithdrawAmount)
})

const availableToWithdrawWithPoolLimit = computed(() => {
  return Math.min(Number(truncatePercent(availableToWithdraw.value, 7)), Number(poolLimit.value))
})

const infoPanelData = computed(() => {
  if (!data) {
    return {}
  }
  return {
    balances: {
      title: 'Balances',
      data: [
        {
          label: 'Total Supply',
          value: `${shortenNumber(totalSuppliedBalance.value || 0, 2, maxDecimalsForShortenNumber(totalSuppliedBalance.value))} ${data?.asset.symbol}`,
        },
        {
          label: 'Supply Balance',
          value: `${shortenNumber(supplyBalance.value || 0, 2, maxDecimalsForShortenNumber(supplyBalance.value))} ${data?.asset.symbol}`,
        },
        {
          label: 'Collateral Balance',
          value: `${shortenNumber(collateralBalance.value || 0, 2, maxDecimalsForShortenNumber(collateralBalance.value))} ${data?.asset.symbol}`,
        },
      ],
    },
    health: {
      title: 'Health',
      data: [
        {
          label: 'Health Factor',
          value: truncatePercent(healthFactor.value || 0, 2),
        },
        {
          label: 'Remaining Supply',
          value: `${shortenNumber(Math.max(remainingBalance.value || 0, 0), 2, maxDecimalsForShortenNumber(remainingBalance.value))} ${data?.asset.symbol}`,
        },
        {
          label: 'Available for Withdrawal',
          value: `${shortenNumber(availableToWithdraw.value || 0, 2, maxDecimalsForShortenNumber(availableToWithdraw.value))} ${data?.asset.symbol}`,
        },
      ],
    },
    poolInfo: {
      title: 'Info / Fee',
      data: [
        {
          label: 'Pool Withdrawal Limit',
          value: `${shortenNumber(poolLimit.value || 0, 2, maxDecimalsForShortenNumber(poolLimit.value))} ${data?.asset.symbol}`,
        },
        {
          label: 'Transaction Fee',
          value: `${txFee.value || 0} XLM`,
        },
      ],
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

      <template v-if="Object.keys(infoPanelData).length > 0">
        <!-- Balances -->
        <info-panel
          :title="infoPanelData.balances!.title"
          :data="infoPanelData.balances!.data"
          variant="success"
        />

        <!-- Health -->
        <info-panel
          :title="infoPanelData.health!.title"
          :data="infoPanelData.health!.data"
          variant="success"
        />

        <!-- Pool Info -->
        <info-panel
          :title="infoPanelData.poolInfo!.title"
          :data="infoPanelData.poolInfo!.data"
          variant="success"
        />
      </template>

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

      <div class="dialog-default__action mt-2">
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
