<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { calcFee } from '@alula/client-sdk/src/utils'
import { CLEAR_DIALOG_TIMEOUT, POOL_REMAINING_BALANCE, RELOAD_FEE_INTERVAL } from '~/config'
import { /* bigintToNumber, destructurePoolAsset, */ focusInput, shortenNumber, truncatePercent } from '~/utils'

const {
  data,
} = defineProps<{
  data?: MarketTableItem
}>()

const marketsStore = useMarketsStore()
const market = useMarketActions()

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const userStore = useUserStore()

const marketClient = computed(() => marketsStore.marketClient)

const amount = toRef(market, 'borrowAmount')
const agree = ref(false)

const dialog = defineModel({ default: false })

const loading = computed(() => marketsStore.poolActiveAddress === data?.raw.pool.pool_address)

const reloadFee = ref(false)

const txFee = ref(0)

const poolBorrowLimit = computed(() => {
  if (!data) {
    return 0
  }
  const utilRatioLimit = Number(data?.raw.pool.config.health_config.utilization_ratio_limit_bps || 0) / 10_000
  const totalSupply = Number(bigintToNumber(data.raw.total_supply, data.assetDecimals))
  const totalBorrow = Number(bigintToNumber(data.raw.pool.total_borrowed, data.assetDecimals))
  const availableByRatioLimit = totalSupply * utilRatioLimit
  return Math.max(availableByRatioLimit - totalBorrow, 0)
})

const availableToBorrow = computed(() => {
  if (!data) {
    return 0
  }
  const userTotalDepositInUsd = userStore.userTotalDepositInUsd
  const userTotalBorrowedInUsd = Number(userStore.userTotalBorrowedInUsd) || 0
  const openLTV = Number(data?.raw.pool.config.health_config.open_ltv_bps || 0) / 10_000
  const marketAvailableInUsd = Number(poolBorrowLimit.value) * Number(data.price)
  const userAvailableByLTV = Number(userTotalDepositInUsd * openLTV) || 0
  const userAvailable = Math.max(userAvailableByLTV - userTotalBorrowedInUsd, 0)
  const maxAvailableUsd = Math.min(userAvailable, marketAvailableInUsd)
  const maxAvailableAssets = maxAvailableUsd / Number(data.price)

  return marketAvailableInUsd > userAvailable ? maxAvailableAssets : Math.floor(maxAvailableAssets)
})

const healthFactor = computed(() => {
  const depositUsd = userStore.userTotalDepositInUsd
  const borrowedUsd = userStore.userTotalBorrowedInUsd
  const price = data?.price || 0
  const closeLTV = Number(data?.raw.pool.config.health_config.close_ltv_bps || 0) / 10_000

  const extraBorrowUsd = (amount.value || 0) * price
  const totalBorrowUsd = borrowedUsd + extraBorrowUsd

  let hf = (depositUsd * closeLTV) / totalBorrowUsd

  if (!Number.isFinite(hf)) {
    hf = 0
  }

  return Math.min(hf, 10)
})

const maxLtv = computed(() => data?.max_ltv || 0)
const closeLTV = computed(() => Number(data?.raw.pool.config.health_config.close_ltv_bps || 0) / 100)

const liquidationPenalty = computed(() => Number(data?.raw.pool.config.health_config.liquidation_close_factor_bps || 0) / 100)

const marketFee = computed(() => {
  const marketFeeBps = data?.raw.pool.config.fee_config.borrow_fee_bps
  return calcFee(Number(amount.value || 0), marketFeeBps || 0)
})

const isCanBorrow = computed(() => {
  const depositObligations = userStore.state.obligations[String(data?.market)]?.deposits ?? []
  for (const [address] of depositObligations) {
    if (address === data?.raw.pool.pool_address) {
      return false
    }
  }
  return true
})

const attentionText = computed(() =>
  isCanBorrow.value
    ? 'Parameter changes via governance can alter your account health factor and risk of liquidation.'
    : 'You cannot open a loan in the same pool where you have a deposit.')

async function borrow() {
  if (!publicKey.value || !data?.raw.pool.pool_address) {
    return
  }
  if (!amount.value || amount.value <= 0) {
    focusInput('.borrow-dialog')
    return
  }

  try {
    marketsStore.poolActiveAddress = data?.raw.pool.pool_address

    const marketProps = {
      market: marketsStore.activeMarketFilter,
      client: marketClient.value!,
      pool_address: data?.raw.pool.pool_address,
      amount: amount.value,
      asset_data: data?.raw.pool.name,
      poolBorrowLimit: poolBorrowLimit.value,
    }

    await market.borrow(marketProps)
  } finally {
    marketsStore.poolActiveAddress = undefined
  }
}

let interval: string | number | NodeJS.Timeout | undefined

watch(dialog, async (v) => {
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
    d?.raw.pool.pool_address || '',
    0,
  )
  txFee.value = marketClient.value.marketSdk.getTransactionFee(tx)
}, { immediate: true, debounce: 300 })
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
          (v: any) => {
            return v && Number(v) < availableToBorrow || 'Borrow limit exceeded'
          },
        ]"
      >
        <template #label-right>
          Available: {{ formatPrice(availableToBorrow, 0, 7) }} {{ data?.asset.symbol }}
        </template>
      </input-widget>

      <div
        v-if="data"
        class="dialog-info-table"
      >
        <!-- Health Factor -->
        <div class="dialog-info-table__item">
          <span>Health Factor</span>
          <span>
            <template v-if="loading">
              <j-loading-spinner
                width="14px"
                style="padding: 0; width: 14px; margin-left: auto"
              />
            </template>
            <template v-else>
              {{ truncatePercent(healthFactor) }}
            </template>
          </span>
        </div>

        <!-- Pool available -->
        <div class="dialog-info-table__item">
          <span>Pool available amount to borrow</span>
          <span>
            {{ shortenNumber(poolBorrowLimit || 0) }}
          </span>
        </div>

        <!-- User available -->
        <div class="dialog-info-table__item">
          <span>User available amount to borrow</span>
          <span>
            {{ shortenNumber(availableToBorrow || 0) }}
          </span>
        </div>

        <!-- Max LTV -->
        <div class="dialog-info-table__item">
          <span>Max LTV</span>
          <span>
            {{ maxLtv }}
          </span>
        </div>

        <!-- Liquidation LTV -->
        <div class="dialog-info-table__item">
          <span>Liquidation LTV</span>
          <span>
            {{ truncatePercent(closeLTV || 0, 2) }}%
          </span>
        </div>

        <!-- Liquidation penalty -->
        <div class="dialog-info-table__item">
          <span>Liq. Penalty</span>
          <span>
            {{ truncatePercent(liquidationPenalty || 0, 2) }}%
          </span>
        </div>

        <!-- Market fee -->
        <div class="dialog-info-table__item">
          <span>Market Fee</span>
          <span>
            {{ formatPrice(marketFee, 0, 5) }} {{ data?.asset.symbol }}
          </span>
        </div>

        <!-- Tx fee -->
        <div class="dialog-info-table__item">
          <span>Transaction Fee</span>
          <span>
            {{ txFee }}
          </span>
        </div>
      </div>

      <warning-block
        :text="attentionText"
        :is-warning="!isCanBorrow"
      />

      <div class="supply-agree">
        <j-checkbox
          v-model="agree"
          :disabled="!isCanBorrow"
        >
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
          :pool="data?.raw.pool"
          :disabled="!agree || !isCanBorrow"
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
