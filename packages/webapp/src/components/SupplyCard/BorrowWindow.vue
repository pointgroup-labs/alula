<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { bpsToNumber } from '@alula/client-sdk'
import { POOL_REMAINING_BALANCE } from '~/config'
import { focusInput, truncatePercent } from '~/utils'

const emits = defineEmits(['dialogHandler'])

const selectedPool = inject<Ref<MarketTableItem>>('selectedPool')

const {
  amount,
  agree,
  isLoading,
  marketFee,
  txFee,
  isLoadingFee,
  availableToBorrow,
  closeLTV,
  isCanBorrow,
  attentionText,
  healthFactor,
  dynamicUtilizationRate,
  borrow: doBorrow,
} = useBorrowDialog(selectedPool, toRef(true))

async function borrow() {
  if (!amount.value || amount.value <= 0) {
    focusInput('.input-wrapper')
    return
  }
  await doBorrow()
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
      variant="indigo"
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
    :style="{ '--color': '#8a8df4', '--bg-color': 'rgba(99, 102, 241, 0.03)', '--border-color': 'rgba(99, 102, 241, 0.1)' }"
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

        <reload-coundown :size="18" />
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

        <!-- Utilization Rate -->
        <div class="summary-list__item">
          <div class="label">
            Utilization Rate
          </div>
          <div
            class="value"
            :style="{
              color:
                utilRateColor(Number(dynamicUtilizationRate.replace('%', '')),
                              bpsToNumber(Number(selectedPool.raw.pool.config.health_config.utilization_ratio_limit_bps) || 0) * 100),
              opacity: 1,
            }"
          >
            {{ dynamicUtilizationRate }}
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
            <j-loading-spinner
              v-if="isLoadingFee"
              width="14px"
              style="margin:0 20px 0 auto;"
            />
            <span v-else>{{ txFee }} XLM</span>
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
      variant="brand-secondary"
      size="md"
      :loading="isLoading"
      :pool="selectedPool?.raw.pool"
      :disabled="!agree || !isCanBorrow || amount > availableToBorrow"
      @click-handler="borrow"
    >
      <i-metrics-complete class="complete-icon" />  Borrow {{ selectedPool?.asset.symbol }}
    </market-dialog-action-btn>
  </div>
</template>
