<script lang="ts" setup>
const {
  opened = false,
} = defineProps<{
  opened?: boolean
}>()

const isValidate = ref(true)

const {
  asset,
  price,
  debt,
  debtAfterRepay,
  balance,
  currentHealthFactor,
  dynamicHealthFactor,
  dynamicLtv,
  maxLtv,
  isLoadingFee,
  amount,
  txFee,
  loading: isLoading,
  repay: doRepay,
} = useRepayDialog(toRef(true))

async function repay() {
  isValidate.value = false
  await doRepay()
  isValidate.value = true
}
</script>

<template>
  <div class="input-wrapper repay-input-wrapper">
    <input-widget
      v-model="amount"
      class="repay-dialog__input"
      :balance="balance"
      :limit="debt"
      label-left="Balance"
      variant="indigo"
      :label-right="formatPrice(balance ?? 0, 0, 4)"
      :symbol="asset.symbol"
      :price="Number(price)"
      :rules="[
        (v) => {
          return !isValidate || Number(v) <= balance || 'Insufficient balance'
        },
      ]"
    />
  </div>

  <Transition name="summary-slide">
    <div
      v-if="amount && amount > 0 || opened"
      class="info-card mt-3 info-summary"
    >
      <div class="info-summary__item">
        <div class="info-summary__header">
          Position Impact

          <reload-coundown
            :size="16"
            color="#54627D"
            bg-color="#35476a"
          />
        </div>

        <div class="summary-list">
          <div class="summary-list__item">
            <div class="label">
              Health Factor
              <info-tooltip>
                An indicator of your position's safety. It compares your deposited collateral against your borrowed
                debt.
                If the Health Factor reaches 1.0, your assets will be liquidated.
              </info-tooltip>
            </div>
            <div class="value">
              <template v-if="isLoading">
                <j-loading-spinner
                  width="14px"
                  style="padding: 0; width: 14px; margin-left: auto"
                />
              </template>
              <template v-else>
                <span :style="{ color: healthFactorColor(currentHealthFactor, '#fff') }">{{ truncatePercent(currentHealthFactor || 0, 2) }}</span>
                <template v-if="amount && amount > 0">
                  →
                  <span :style="{ color: healthFactorColor(dynamicHealthFactor, '#fff') }">{{ truncatePercent(dynamicHealthFactor || 0, 2) }}</span>
                </template>
              </template>
            </div>
          </div>

          <div class="summary-list__item align-items-start mb-2">
            <div class="label">
              Borrow Limit Used
              <info-tooltip>
                Shows how much of your total borrowing power you are currently using. This limit is based on the maximum
                Loan-to-Value (LTV) of your collaterals. At maximum % you cannot borrow more.
              </info-tooltip>
            </div>
            <div
              class="value"
            >
              <div class="text-end">
                <span>{{ truncatePercent(dynamicLtv || 0, 2) }}%</span>
                of
                <span>{{ truncatePercent(maxLtv || 0, 2) }}%</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="separator" />

      <div class="info-summary__item">
        <div class="info-summary__header">
          Borrow details
        </div>

        <div class="summary-list">
          <div class="summary-list__item">
            <div class="label">
              Debt
            </div>
            <div class="value">
              {{ shortenNumber(debt, 2, maxDecimalsForShortenNumber(debt)) }} {{ asset.symbol }}
            </div>
          </div>

          <div class="summary-list__item align-items-start mb-2">
            <div class="label">
              Remaining debt
            </div>
            <div
              class="value"
            >
              {{ shortenNumber(debtAfterRepay, 2, maxDecimalsForShortenNumber(debtAfterRepay)) }} {{ asset.symbol }}
            </div>
          </div>
        </div>
      </div>

      <div class="separator" />

      <j-accordion
        class="info-summary__item accordion-summary"
        title="Fees"
      >
        <div class="summary-list">
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
      </j-accordion>
    </div>
  </Transition>

  <div class="supply-card__action mt-3">
    <j-btn
      :loading="isLoading"
      variant="brand-secondary"
      class="market-action-btn"
      size="md"
      @click="repay"
    >
      <i-metrics-complete class="complete-icon" /> Repay {{ asset.symbol }}
    </j-btn>
  </div>
</template>
