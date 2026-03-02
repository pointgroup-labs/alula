<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { calcFee } from '@alula/client-sdk/src/utils'
import { POOL_REMAINING_BALANCE } from '~/config'
import { focusInput, truncatePercent } from '~/utils'

const emits = defineEmits(['dialogHandler'])

const selectedPool = inject<Ref<MarketTableItem>>('selectedPool')

const marketsStore = useMarketsStore()
const market = useMarketActions()

const userStore = useUserStore()

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const {
  marketClient,
  agree,
  isLoading,
  txFee,
  poolBorrowLimit,
  availableToBorrow,
  closeLTV,
  liquidationPenalty,
  isCanBorrow,
  attentionText,
} = useBorrowDialog(selectedPool)

const amount = toRef(market, 'borrowAmount')

const healthFactor = computed(() => {
  const depositUsd = userStore.userTotalDepositInUsd
  const borrowedUsd = userStore.userTotalBorrowedInUsd
  const price = selectedPool?.value?.price || 0
  const closeLTV = Number(selectedPool?.value?.raw.pool.config.health_config.close_ltv_bps || 0) / 10_000

  const extraBorrowUsd = (amount.value || 0) * price
  const totalBorrowUsd = borrowedUsd + extraBorrowUsd

  let hf = (depositUsd * closeLTV) / totalBorrowUsd

  if (!Number.isFinite(hf)) {
    hf = 0
  }

  return Math.min(hf, 10)
})

const marketFee = computed(() => {
  const marketFeeBps = selectedPool?.value?.raw.pool.config.fee_config.borrow_fee_bps
  return calcFee(Number(amount.value || 0), marketFeeBps || 0)
})

async function borrow() {
  if (!publicKey.value || !selectedPool?.value?.raw.pool.pool_address) {
    return
  }
  if (!amount.value || amount.value <= 0) {
    focusInput('.input-wrapper')
    return
  }

  try {
    marketsStore.poolActiveAddress = selectedPool?.value?.raw.pool.pool_address

    const marketProps = {
      market: marketsStore.selectedMarketName,
      client: marketClient.value!,
      pool_address: selectedPool?.value?.raw.pool.pool_address,
      amount: amount.value,
      asset_data: selectedPool?.value?.raw.pool.name,
      poolBorrowLimit: poolBorrowLimit.value,
    }

    await market.borrow(marketProps)

    marketsStore.dialogBorrow = false
  } finally {
    marketsStore.poolActiveAddress = undefined
  }
}

const debouncedFn = useDebounceFn(calculateDebtAccrual, 500)

function calculateDebtAccrual(
  deposit: number,
  apyPercent: number,
  price: number,
) {
  const apy = apyPercent / 100
  const dailyRate = (1 + apy) ** (1 / 365) - 1
  const daily = deposit * dailyRate * price
  const yearly = deposit * apy * price
  return {
    daily: daily.toFixed(daily > 1 ? 2 : 4),
    yearly: yearly.toFixed(yearly > 1 ? 2 : 4),
  }
}

const debtAccrual = computedAsync(async () => {
  if (!amount.value || amount.value === 0) {
    return {
      daily: 0,
      yearly: 0,
    }
  }
  const apyRaw = selectedPool?.value?.deposit_apy ?? '0'
  const apy = Number(apyRaw.replace('%', ''))
  const price = selectedPool?.value.price ?? 0
  return debouncedFn(
    Number(amount.value),
    apy,
    price)
})
</script>

<template>
  <div class="input-wrappe mt-4">
    <input-widget
      v-model="amount"
      :balance="availableToBorrow"
      :fee="POOL_REMAINING_BALANCE"
      :price="selectedPool?.price"
      label-left="Available to Borrow"
      :label-right="`${formatPrice(availableToBorrow ?? 0, 0, 4)} ${selectedPool?.asset.symbol}`"
      variant="borrow"
      :rules="[
        (v: any) => {
          return Number(v) < availableToBorrow || 'Borrow limit exceeded'
        },
      ]"
    >
      <template #prepend>
        <div
          class="select-pool-btn"
          @click="emits('dialogHandler')"
        >
          <img
            :src="selectedPool?.asset.icon"
            alt="asset icon"
          >
          {{ selectedPool?.asset.symbol }}
          <i-app-chevron-down />
        </div>
      </template>
    </input-widget>
  </div>

  <div
    class="info-card mt-3 info-supply"
    :style="{ '--color': '#6366F1', '--bg-color': 'rgba(245, 158, 11, 0.04)', '--border-color': 'rgba(252, 157, 16, 0.1)' }"
  >
    <div class="info-supply__header">
      <div class="info-title">
        Borrow APY
      </div>
      <div class="info-apy">
        {{ selectedPool?.borrow_apy }}
      </div>
    </div>
    <div class="info-supply__body">
      <div class="info-detail">
        <div class="info-detail__title">
          Daily
        </div>
        <div class="info-detail__value">
          {{ debtAccrual?.daily ? `$${formatPrice(debtAccrual?.daily)}` : '--' }}
        </div>
      </div>
      <div class="info-detail">
        <div class="info-detail__title">
          Est. Debt / yr
        </div>
        <div class="info-detail__value">
          {{ debtAccrual?.yearly ? `$${formatPrice(debtAccrual?.yearly)}` : '--' }}
        </div>
      </div>
    </div>
  </div>

  <Transition name="summary-slide">
    <div
      v-if="amount && amount > 0 && selectedPool"
      class="info-card mt-3 info-summary"
    >
      <div class="info-summary__header">
        Transaction Summary
      </div>

      <div class="summary-list">
        <!-- Health Factor -->
        <div class="summary-list__item">
          <div class="label">
            Health Factor
          </div>
          <div class="value">
            <template v-if="isLoading">
              <j-loading-spinner
                width="14px"
                style="padding: 0; width: 14px; margin-left: auto"
              />
            </template>
            <template v-else>
              {{ truncatePercent(healthFactor) }}
            </template>
          </div>
        </div>

        <!-- Borrowing Capacity -->
        <div class="summary-list__item">
          <div class="label">
            Borrowing Capacity
          </div>
          <div class="value">
            {{ shortenNumber(availableToBorrow || 0) }}
          </div>
        </div>

        <!-- Close LTV -->
        <div class="summary-list__item">
          <div class="label">
            Close LTV
          </div>
          <div class="value">
            {{ truncatePercent(closeLTV || 0, 2) }}%
          </div>
        </div>

        <!-- Liquidation penalty -->
        <div class="summary-list__item">
          <div class="label">
            Liquidation penalty
          </div>
          <div class="value">
            {{ truncatePercent(liquidationPenalty || 0, 2) }}%
          </div>
        </div>

        <!-- Market fee -->
        <div class="summary-list__item">
          <div class="label">
            Operation Fee
          </div>
          <div class="value">
            {{ formatPrice(marketFee, 0, 5) }} {{ selectedPool?.asset.symbol }}
          </div>
        </div>

        <!-- Tx fee -->
        <div class="summary-list__item">
          <div class="label">
            Transaction Fee
          </div>
          <div class="value">
            {{ txFee }}
          </div>
        </div>
      </div>
    </div>
  </Transition>

  <warning-block
    :text="attentionText"
    :is-warning="!isCanBorrow"
    class="mt-3"
  />

  <div class="extra-info mt-3">
    <j-checkbox
      v-model="agree"
      :disabled="!isCanBorrow"
    >
      <div class="extra-info__label">
        I acknowledge the risks involved.
      </div>
    </j-checkbox>
  </div>

  <div class="supply-card__action mt-4">
    <market-dialog-action-btn
      variant="accent"
      size="lg"
      :loading="isLoading"
      :pool="selectedPool?.raw.pool"
      :disabled="!agree || !isCanBorrow || amount > availableToBorrow"
      pill
      @click-handler="borrow"
    >
      Borrow {{ selectedPool?.asset.symbol }}
    </market-dialog-action-btn>
  </div>
</template>
