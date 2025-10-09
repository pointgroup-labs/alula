<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { CLEAR_DIALOG_TIMEOUT, POOL_REMAINING_BALANCE, RELOAD_FEE_INTERVAL } from '~/config'
import { bigintToNumber, destructurePoolAsset, focusInput, shortenNumber, truncatePercent } from '~/utils'

const {
  data,
  modelValue,
} = defineProps<{
  data?: MarketTableItem
  modelValue: boolean
}>()

const marketsStore = useMarketsStore()
const market = useMarketActions()

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const userStore = useUserStore()

const marketClient = computed(() => marketsStore.marketClient)

const amount = toRef(market, 'borrowAmount')
const agree = ref(false)

const reloadFee = ref(false)
const txFee = ref(0)

watchDebounced([
  () => data,
  reloadFee,
  publicKey,
], async ([d, _r]) => {
  if (!d || !publicKey.value || !marketClient.value) {
    return
  }
  const tx = await marketClient.value?.marketSdk.borrowTx(
    publicKey.value,
    d?.raw.pool_address || '',
    0,
  )
  txFee.value = marketClient.value.marketSdk.getTransactionFee(tx)
}, { immediate: true, debounce: 300 })

const balance = computed(() => {
  if (!data) {
    return 0
  }
  if (data.raw.token_ticker === 'XLM') {
    return wallet.nativeBalance
  }
  const [, asset_issuer] = destructurePoolAsset(data.raw.name)
  return wallet.getAssetBalance(String(asset_issuer))
})

const poolBorrowLimit = computed(() => {
  if (!data) {
    return 0
  }
  const utilRatioLimit = Number(data?.raw.config.health_config.utilization_ratio_limit_bps || 0) / 10_000
  const totalSupply = Number(bigintToNumber(data.raw.total_available + data.raw.total_borrowed - data.raw.accumulated_reserve_fees, marketsStore.assetDecimals))
  const maxBorrow = totalSupply * utilRatioLimit
  const totalBorrow = Number(bigintToNumber(data.raw.total_borrowed, marketsStore.assetDecimals))
  return Math.max(maxBorrow - totalBorrow, 0)
})

const availableToBorrow = computed(() => {
  if (!data) {
    return 0
  }
  const userTotalDepositInUsd = userStore.userTotalDepositInUsd
  const userTotalBorrowedInUsd = Number(userStore.userTotalBorrowedInUsd) || 0
  const openLTV = Number(data?.raw.config.health_config.open_ltv_bps || 0) / 10_000
  const marketAvailableInUsd = Number(poolBorrowLimit.value) * Number(data.price)
  const userAvailableByLTV = Number(userTotalDepositInUsd * openLTV) || 0
  const maxAvailableUsd = Math.min(Math.max(userAvailableByLTV - userTotalBorrowedInUsd, 0), marketAvailableInUsd)
  const maxAvailableAssets = maxAvailableUsd / Number(data.price)

  return Math.max(maxAvailableAssets, 0)
})

const healthFactor = computed(() => {
  const depositUsd = userStore.userTotalDepositInUsd
  const borrowedUsd = userStore.userTotalBorrowedInUsd
  const price = data?.price || 0
  const closeLTV = Number(data?.raw.config.health_config.close_ltv_bps || 0) / 10_000

  const extraBorrowUsd = (amount.value || 0) * price
  const totalBorrowUsd = borrowedUsd + extraBorrowUsd

  let hf = (depositUsd * closeLTV) / totalBorrowUsd

  if (!Number.isFinite(hf)) {
    hf = 0
  }

  return Math.min(hf, 10)
})

const infoTableData = computed(() => {
  if (!data) {
    return []
  }
  const liquidation = Number(data.raw.config.health_config.liquidation_close_factor_bps) / 100
  const closeLTV = Number(data.raw.config.health_config.close_ltv_bps) / 100
  return [{
    name: 'healthFactor',
    label: 'Health Factor',
    value: truncatePercent(healthFactor.value, 2),
  },
  {
    label: 'Pool available amount to borrow',
    value: shortenNumber(poolBorrowLimit.value),
  },
  {
    label: 'User available amount to borrow',
    value: shortenNumber(availableToBorrow.value || 0),
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

const dialog = defineModel({ default: false })

const loading = computed(() => marketsStore.poolActiveAddress === data?.raw.pool_address)

async function borrow() {
  if (!publicKey.value || !data?.raw.pool_address) {
    return
  }
  if (!amount.value || amount.value <= 0) {
    focusInput('.borrow-dialog')
    return
  }

  const marketProps = {
    market: marketsStore.activeMarketFilter,
    client: marketClient.value!,
    pool_address: data?.raw.pool_address,
    amount: amount.value,
    asset_data: data?.raw.name,
    poolBorrowLimit: poolBorrowLimit.value,
  }

  await market.borrow(marketProps)
}

let interval: string | number | NodeJS.Timeout | undefined

watch(() => modelValue, async (v) => {
  clearInterval(interval)
  if (!v) {
    setTimeout(() => {
      amount.value = 0
    }, CLEAR_DIALOG_TIMEOUT)
    agree.value = false
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
    class-name="supply-dialog borrow-dialog dialog-default"
  >
    <template #header>
      <div class="supply-dialog__title">
        <img
          :src="data?.asset.icon"
          :alt="`${data?.asset.symbol} icon`"
        >
        <span>Borrow {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="supply-dialog__body">
      <input-widget
        v-model="amount"
        :balance="availableToBorrow"
        :fee="POOL_REMAINING_BALANCE"
        :rules="[
          (v) => {
            return v && Number(v) < availableToBorrow || 'Borrow limit exceeded'
          },
        ]"
      >
        <template #label-right>
          Wallet: {{ balance }} {{ data?.asset.symbol }}
        </template>
      </input-widget>

      <div class="dialog-info-table">
        <div
          v-for="item in infoTableData"
          :key="item.label"
          class="dialog-info-table__item"
        >
          <span>{{ item?.label }}</span>
          <span>
            <template v-if="item?.name === 'healthFactor' && loading">
              <j-loading-spinner
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

        <market-dialog-action-btn
          variant="accent"
          :loading="loading"
          :pool="data?.raw"
          :disabled="!agree"
          @click-handler="borrow"
        >
          Borrow {{ data?.asset.symbol }}
        </market-dialog-action-btn>
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

body.body--dark {
  .borrow-dialog {
    .supply-warning {
      background-color: $neutral-18;
      color: $neutral-12;
    }
  }
}
</style>
