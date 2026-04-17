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
  borrowLimitUsedUsd,
  borrowLimitTotalUsd,
  loading: isLoading,
  withdraw: doWithdraw,
} = useWithdrawDialog(toRef(true))

const isHasCollateral = computed(() => collateralBalance.value > 0)
const isHasSupply = computed(() => supplyBalance.value > 0)
const supplyBalanceLabel = computed(() => `${normalizebalance(supplyBalance.value)} ${asset.value?.symbol}`)
const collateralBalanceLabel = computed(() => `${normalizebalance(collateralBalance.value)} ${asset.value?.symbol}`)

function normalizebalance(balance?: number) {
  return shortenNumber(balance ?? 0, maxDecimalsForShortenNumber(balance))
}

async function withdraw() {
  isValidate.value = false
  await doWithdraw()
  isValidate.value = true
}

const selected = computed({
  get() {
    return collateralOnly.value ? 'B' : 'A'
  },
  set(newValue) {
    collateralOnly.value = newValue === 'B'
  },
})

function selectBalance(type: 'A' | 'B') {
  if (type === selected.value || !isHasCollateral.value) {
    return
  }
  selected.value = type
}

watch([supplyBalance, isHasCollateral], ([b, c]) => {
  if (b === 0 && c) {
    selectBalance('B')
  }
}, { immediate: true })
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

  <div

    class="info-card mt-3 info-summary info-summary--collateral"
  >
    <div class="info-summary__item">
      <div class="info-summary__header">
        Balances
        <info-tooltip style="margin: 0 auto 0 6px">
          Total deposited balances, including funds available for withdrawal (Supply) and funds locked as collateral.
        </info-tooltip>
      </div>

      <div class="summary-list">
        <div
          v-if="isHasSupply"
          class="summary-list__item"
        >
          <div
            class="label"
            @click="selectBalance('A')"
          >
            <BFormRadio
              v-if="isHasCollateral"
              v-model="selected"
              name="some-radios"
              value="A"
              class="balance-radio"
            />

            Supply
            <info-tooltip>
              Available for borrowing by other users and earns yield.
            </info-tooltip>
          </div>
          <div class="value">
            {{ supplyBalanceLabel }}
          </div>
        </div>

        <div
          v-if="isHasCollateral"
          class="summary-list__item"
        >
          <div
            class="label"
            @click="selectBalance('B')"
          >
            <BFormRadio
              v-if="isHasSupply"
              v-model="selected"
              name="some-radios"
              value="B"
              class="balance-radio"
            />

            Collateral only
            <info-tooltip>
              Not available for borrowing. Does not participate in lending, but can be withdrawn anytime.
            </info-tooltip>
          </div>
          <div class="value">
            {{ collateralBalanceLabel }}
          </div>
        </div>
      </div>
    </div>

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
              Borrowed
              <info-tooltip>
                Shows how much of your total borrowing power you are currently using. This limit is based on the maximum
                Loan-to-Value (LTV) of your collaterals. At maximum % you cannot borrow more.
              </info-tooltip>
            </div>
            <div
              class="value"
            >
              <div class="text-end">
                <span>${{ formatPrice(borrowLimitUsedUsd || 0, 2, 2) }}</span>
                of
                <span>${{ formatPrice(borrowLimitTotalUsd || 0, 2, 2) }}</span>
              </div>
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
          Limits
        </div>

        <div class="summary-list">
          <div
            v-if="collateralOnly"
            class="summary-list__item"
          >
            <div class="label">Collateral balance</div>
            <div class="value">
              {{ shortenNumber(collateralBalance ?? 0, 2) }} {{ asset.symbol }}
            </div>
          </div>
          <div
            v-else
            class="summary-list__item"
          >
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
        </div>
      </div>

      <div class="separator" />

      <!-- <div class="info-summary__item">
        <div class="info-summary__header">
          Balance
        </div>

        <div class="summary-list">
          <div class="summary-list__item">
            <div class="label">Supply Balance</div>
            <div class="value">
              {{ supplyBalanceLabel }}
            </div>
          </div>

          <div class="summary-list__item">
            <div class="label">Collateral Balance</div>
            <div class="value">
              {{ collateralBalanceLabel }}
            </div>
          </div>
        </div>
      </div>

      <div class="separator" /> -->

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

  <div class="supply-card__action mt-3">
    <j-btn
      :loading="isLoading"
      variant="brand"
      size="md"
      class="market-action-btn"
      @click="withdraw"
    >
      <i-metrics-complete class="complete-icon" />  Withdraw {{ asset.symbol }}
    </j-btn>
  </div>
</template>

<style lang="scss">
.info-summary--collateral {
  .collateral-toggle {
    .form-check-input {
      height: 20px;
      background-size: 20px;

      &::before {
        width: 17px;
        height: 17px;
      }
    }
  }

  .summary-list__item {
    .label:has(.balance-radio) {
      cursor: pointer;
    }

    .balance-radio {
      box-shadow: none !important;
      margin-right: 4px;
      cursor: pointer;

      &:checked {
        background-color: $cyan;
        border: $cyan;
      }
    }
  }
}
</style>
