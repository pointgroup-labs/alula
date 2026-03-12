<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
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
  poolBorrowLimit,
  isCanBorrow,
  attentionText,
  currentHealthFactor,
  dynamicHealthFactor,
  currentLtv,
  maxLtv,
  dynamicLtv,
  borrow: doBorrow,
} = useBorrowDialog(selectedPool, toRef(true))

async function borrow() {
  if (!amount.value || amount.value <= 0) {
    focusInput('.input-wrapper')
    return
  }
  await doBorrow()
}
</script>

<template>
  <div class="input-wrappe mt-4">
    <input-widget
      v-model="amount"
      :balance="availableToBorrow"
      :fee="POOL_REMAINING_BALANCE"
      :price="selectedPool?.price"
      label-left="Available to Borrow"
      :label-right="formatPrice(availableToBorrow ?? 0, 0, 4)"
      :symbol="selectedPool?.asset.symbol"
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

  <Transition name="summary-slide">
    <div
      v-if="amount && amount > 0 && selectedPool"
      class="info-card mt-3 info-summary"
    >
      <div class="info-summary__item">
        <div class="info-summary__header">
          Position Impact

          <reload-coundown :size="18" />
        </div>

        <div class="summary-list">
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
                <span class="positive">{{ truncatePercent(currentHealthFactor || 0, 2) }}</span>
                →
                <span class="negative">{{ truncatePercent(dynamicHealthFactor || 0, 2) }}</span>
              </template>
            </div>
          </div>

          <div class="summary-list__item align-items-start mb-2">
            <div class="label">
              Loan-to-Value (LTV)
            </div>
            <div
              class="value"
            >
              <div>
                <span class="positive">{{ truncatePercent(currentLtv || 0, 2) }}%</span>
                →
                <span class="negative">{{ truncatePercent(dynamicLtv || 0, 2) }}%</span>
              </div>
              <div class="max-ltv">
                Max LTV: {{ truncatePercent(maxLtv || 0, 2) }}%
              </div>
            </div>
          </div>
        </div>

      </div>

      <div class="separator" />

      <div class="info-summary__item">
        <div class="info-summary__header">
          Fees
        </div>

        <div class="summary-list">
          <div class="summary-list__item">
            <div class="label">
              Operation Fee
            </div>
            <div class="value">
              {{ formatPrice(marketFee, 0, 5) }} {{ selectedPool?.asset.symbol }}
            </div>
          </div>

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

      <div class="separator" />

      <div class="info-summary__item">
        <div class="info-summary__header">
          Market Details
        </div>

        <div class="summary-list">
          <div class="summary-list__item">
            <div class="label">
              Borrow Rate
            </div>
            <div class="value">
              {{ selectedPool?.borrow_apy }}
            </div>
          </div>

          <div class="summary-list__item">
            <div class="label">
              Pool Liquidity
            </div>
            <div class="value">
              {{ shortenNumber(poolBorrowLimit || 0) }} {{ selectedPool?.asset.symbol }}
            </div>
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
