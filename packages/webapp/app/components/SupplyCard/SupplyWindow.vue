<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { POOL_REMAINING_BALANCE } from '~/config'
import { focusInput, formatPrice } from '~/utils'

const {
  opened = false,
  withSelectedPool = true,
} = defineProps<{
  opened?: boolean
  withSelectedPool?: boolean
}>()

const emits = defineEmits(['dialogHandler'])

const selectedPool = inject<Ref<MarketTableItem>>('selectedPool')

const {
  amount,
  collateralOnly,
  balance,
  txFee,
  isLoadingFee,
  supplyLimit,
  limitLabel,
  isHasBorrows,
  reserveAmount,
  isLoading,
  isCanSupply,
  attentionText,
  marketFee,
  borrowLimitUsedUsd,
  borrowLimitTotalUsd,
  currentHealthFactor,
  dynamicHealthFactor,
  supply: doSupply,
} = useSupplyDialog(selectedPool, toRef(true))

async function supply() {
  if (!amount.value || amount.value <= 0) {
    focusInput('.supply-input-wrapper')
    return
  }
  await doSupply()
}

const debouncedFn = useDebounceFn(calculateRewardsEarnings, 500)

function calculateRewardsEarnings(
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

const rewardsEarnings = computedAsync(async () => {
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
  <div class="input-wrapper supply-input-wrapper">
    <input-widget
      v-model="amount"
      :balance="balance"
      :limit="Number(supplyLimit) || 0"
      :fee="POOL_REMAINING_BALANCE + txFee + reserveAmount"
      :price="selectedPool?.price"
      label-left="You Supply"
      :label-right="formatPrice(balance ?? 0, 0, 4)"
      :symbol="selectedPool?.asset.symbol"
      :rules="[
        (v) => {
          return Number(v) < balance || 'Insufficient balance'
        },
        (v) => {
          return (supplyLimit <= 0 || Number(v) <= supplyLimit) || 'Pool supply limit'
        },
      ]"
    >
      <template #prepend>
        <div
          v-if="withSelectedPool"
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
      v-if="amount && amount > 0 && selectedPool || opened"
      class="info-card mt-3 info-summary"
    >
      <template v-if="isHasBorrows">
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
                <span :style="{ color: healthFactorColor(currentHealthFactor, '#fff') }">{{ truncatePercent(currentHealthFactor || 0, 2) }}</span>
                <template v-if="amount && amount > 0">
                  →
                  <span :style="{ color: healthFactorColor(dynamicHealthFactor, '#fff') }">{{ truncatePercent(dynamicHealthFactor
                    || 0, 2) }}</span>
                </template>
              </div>
            </div>

            <div class="summary-list__item">
              <div class="label">
                Borrowed
                <info-tooltip>
                  Shows how much of your total borrowing power you are currently using. This limit is based on the maximum
                  Loan-to-Value (LTV) of your collaterals. At maximum % you cannot borrow more.
                </info-tooltip>
              </div>
              <div class="value">
                <span>${{ formatPrice(borrowLimitUsedUsd || 0, 2, 2) }}</span>
                of
                <span>${{ formatPrice(borrowLimitTotalUsd || 0, 2, 2) }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="separator" />
      </template>

      <div class="info-summary__item">
        <div class="info-summary__header">
          Market Details
        </div>

        <div class="summary-list">
          <div class="summary-list__item">
            <div class="label">
              Supply APY
            </div>
            <div class="value">
              {{ selectedPool?.deposit_apy }}
            </div>
          </div>

          <div class="summary-list__item">
            <div class="label">
              Est. yearly income
              <info-tooltip>
                Estimated earnings from your deposit based on the current APY.
              </info-tooltip>
            </div>
            <div class="value">
              {{ rewardsEarnings?.yearly ? `≈ $${formatPrice(rewardsEarnings?.yearly)}` : '--' }}
            </div>
          </div>

          <div class="summary-list__item">
            <div class="label">
              Supply Limit
            </div>
            <div class="value">
              {{ limitLabel }} {{ limitLabel !== '-' ? selectedPool?.asset.symbol : '' }}
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
              {{ formatPrice(marketFee) }} XLM
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
    v-if="amount && amount > 0 && selectedPool || opened"
    class="collateral mt-3"
  >
    <div class="collateral-label">Collateral Only</div>

    <j-toggle
      v-model="collateralOnly"
      size="small"
      :disabled="!isCanSupply"
    />
  </div>

  <warning-block
    v-if="!isCanSupply"
    :text="attentionText"
    :is-warning="!isCanSupply"
    class="mt-3"
  />

  <div class="supply-card__action mt-3">
    <market-action-btn
      variant="brand"
      size="md"
      :loading="isLoading"
      :pool="selectedPool?.raw.pool"
      :disabled="!isCanSupply || amount >= balance"
      class="market-action-btn"
      @click-handler="supply"
    >
      <i-metrics-complete class="complete-icon" /> Supply {{ selectedPool?.asset.symbol }}
    </market-action-btn>
  </div>
</template>
