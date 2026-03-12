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
  reserveAmount,
  isLoading,
  isCanSupply,
  attentionText,
  marketFee,
  currentLtv,
  dynamicLtv,
  currentHealthFactor,
  dynamicHealthFactor,
  supply: doSupply,
} = useSupplyDialog(selectedPool, toRef(true))

async function supply() {
  if (!amount.value || amount.value <= 0) {
    focusInput('.input-wrapper')
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
  <div class="input-wrappe mt-4">
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
              <span class="positive">{{ truncatePercent(currentHealthFactor || 0, 2) }}</span>
              →
              <span class="negative">{{ truncatePercent(dynamicHealthFactor || 0, 2) }}</span>
            </div>
          </div>

          <div class="summary-list__item">
            <div class="label">
              Loan-to-Value (LTV)
            </div>
            <div class="value">
              <span class="positive">
                {{ truncatePercent(currentLtv || 0, 2) }}%
              </span>
              →
              <span class="negative">{{ truncatePercent(dynamicLtv || 0, 2) }}%</span>
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

      <div class="separator" />

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
            </div>
            <div class="value">
              {{ rewardsEarnings?.yearly ? `$${formatPrice(rewardsEarnings?.yearly)}` : '--' }}
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
    <market-dialog-action-btn
      variant="brand"
      size="md"
      :loading="isLoading"
      :pool="selectedPool?.raw.pool"
      :disabled="!isCanSupply || amount >= balance"
      @click-handler="supply"
    >
      <i-metrics-complete class="complete-icon" /> Supply {{ selectedPool?.asset.symbol }}
    </market-dialog-action-btn>
  </div>
</template>
