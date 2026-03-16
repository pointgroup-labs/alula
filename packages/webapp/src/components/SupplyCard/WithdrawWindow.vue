<script lang="ts" setup>
const {
  opened = false,
} = defineProps<{
  opened?: boolean
}>()

const isValidate = ref(true)

const {
  poolLimit,
  asset,
  price,
  collateralBalance,
  remainingBalance,
  availableToWithdrawWithPoolLimit,
  isLoadingFee,
  supplyBalance,
  amount,
  collateralOnly,
  poolFee,
  txFee,
  currentHealthFactor,
  dynamicHealthFactor,
  currentLtv,
  dynamicLtv,
  maxLtv,
  loading: isLoading,
  withdraw: doWithdraw,
} = useWithdrawDialog(toRef(true))

async function withdraw() {
  isValidate.value = false
  await doWithdraw()
  isValidate.value = true
}
</script>

<template>
  <input-widget
    v-model="amount"
    :balance="availableToWithdrawWithPoolLimit"
    class="withdraw-dialog__input"
    :price="Number(price)"
    label-left="Amount"
    :label-right="formatPrice(availableToWithdrawWithPoolLimit ?? 0, 0, 4)"
    :symbol="asset.symbol"
    variant="cyan"
    :rules="[
      (v) => {
        return !isValidate || Number(v) <= availableToWithdrawWithPoolLimit || 'Withdraw limit exceeded'
      },
    ]"
  />

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
                <span :style="{ color: healthFactorColor(dynamicHealthFactor) }">{{ truncatePercent(dynamicHealthFactor || 0, 2) }}</span>
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
              <div class="text-end">
                <span class="positive">{{ truncatePercent(currentLtv || 0, 2) }}%</span>
                →
                <span :style="{ color: ltvColor(dynamicLtv, maxLtv) }">{{ truncatePercent(dynamicLtv || 0, 2) }}%</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="separator" />

      <div class="info-summary__item">
        <div class="info-summary__header">
          Limits
        </div>

        <div class="summary-list">
          <div class="summary-list__item">
            <div class="label">Pool liquidity</div>
            <div class="value">
              {{ shortenNumber(poolLimit ?? 0, 2) }} {{ asset.symbol }}
            </div>
          </div>

          <div class="summary-list__item">
            <div class="label">Available to withdraw</div>
            <div class="value">
              {{ shortenNumber(availableToWithdrawWithPoolLimit ?? 0, maxDecimalsForShortenNumber(availableToWithdrawWithPoolLimit)) }} {{ asset.symbol }}
            </div>
          </div>

          <div class="summary-list__item">
            <div class="label">Remaining supply</div>
            <div class="value">
              {{ shortenNumber(Math.max(remainingBalance, 0) ?? 0, maxDecimalsForShortenNumber(remainingBalance)) }} {{ asset.symbol }}
            </div>
          </div>
        </div>
      </div>

      <div class="separator" />

      <div class="info-summary__item">
        <div class="info-summary__header">
          Balance
        </div>

        <div class="summary-list">
          <div class="summary-list__item">
            <div class="label">Supply Balance</div>
            <div class="value">
              {{ shortenNumber(supplyBalance ?? 0, maxDecimalsForShortenNumber(supplyBalance)) }} {{ asset.symbol }}
            </div>
          </div>

          <div class="summary-list__item">
            <div class="label">Collateral Balance</div>
            <div class="value">
              {{ shortenNumber(collateralBalance ?? 0, maxDecimalsForShortenNumber(collateralBalance)) }} {{ asset.symbol }}
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
              Operation Fee
            </div>
            <div class="value">
              {{ formatPrice(poolFee) }} XLM
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
      </j-accordion>
    </div>
  </Transition>

  <div
    v-if="collateralBalance > 0"
    class="collateral mt-3"
  >
    <div class="collateral-label">Collateral Only</div>

    <j-toggle
      v-model="collateralOnly"
      size="small"
    />
  </div>

  <div class="dialog-default__action mt-2">
    <j-btn
      :loading="isLoading"
      variant="brand"
      size="lg"
      pill
      @click="withdraw"
    >
      Withdraw {{ asset.symbol }}
    </j-btn>
  </div>
</template>
