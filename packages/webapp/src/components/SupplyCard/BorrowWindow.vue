<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { POOL_REMAINING_BALANCE } from '~/config'
import { focusInput, truncatePercent } from '~/utils'

const {
  opened = false,
  withSelectedPool = true,
} = defineProps<{
  opened?: boolean
  withSelectedPool?: boolean
}>()

const emits = defineEmits(['dialogHandler'])

const selectedPool = inject<Ref<MarketTableItem>>('selectedPool')

const { getFullTokenData } = useTokensStore()

const marketsStore = useMarketsStore()
const marketState = computed(() => marketsStore.state.markets[selectedPool?.value?.market ?? '']?.marketState)
const poolsData = computed(() => marketState.value?.pools_data ?? [])

const {
  amount,
  agree,
  isLoading,
  obligation,
  marketFee,
  txFee,
  isLoadingFee,
  availableToBorrow,
  poolBorrowLimit,
  isCanBorrow,
  attentionText,
  currentHealthFactor,
  dynamicHealthFactor,
  maxLtv,
  dynamicLtv,
  borrow: doBorrow,
} = useBorrowDialog(selectedPool, toRef(true))

async function borrow() {
  if (!amount.value || amount.value <= 0) {
    focusInput('.borrow-input-wrapper')
    return
  }
  await doBorrow()
}

const positions = computed(() => {
  if (!obligation.value) {
    return null
  }
  const borrows = obligation.value.borrows?.map(([address]) => {
    const symbol = poolsData.value?.find(p => p.pool.pool_address === address)?.pool?.token_symbol ?? ''
    const asset = getFullTokenData(symbol)
    return {
      address,
      ...asset,
    }
  }) ?? []
  const deposits = obligation.value.deposits?.map(([address]) => {
    const symbol = poolsData.value?.find(p => p.pool.pool_address === address)?.pool?.token_symbol ?? ''
    const asset = getFullTokenData(symbol)
    return {
      address,
      ...asset,
    }
  }) ?? []
  return {
    borrows,
    deposits,
  }
})
</script>

<template>
  <div class="input-wrapper borrow-input-wrapper">
    <input-widget
      v-model="amount"
      :balance="availableToBorrow"
      :fee="POOL_REMAINING_BALANCE"
      :price="selectedPool?.price"
      label-left="Available to Borrow"
      :label-right="formatPrice(Math.max((availableToBorrow ?? 0) - POOL_REMAINING_BALANCE, 0), 0, 4)"
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
                  <span :style="{ color: healthFactorColor(dynamicHealthFactor, '#fff') }">{{ truncatePercent(dynamicHealthFactor
                    || 0, 2) }}</span>
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
            <div class="value">
              <div>
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
          <div
            v-if="positions && positions?.deposits?.length > 0"
            class="summary-list__item"
          >
            <div class="label">
              Collateral assets
            </div>
            <div class="value collateral-assets">
              <img
                v-for="collateral in positions?.deposits"
                :key="collateral.name"
                :src="collateral?.icon"
                alt="asset icon"
              >
            </div>
          </div>

          <div class="summary-list__item">
            <div class="label">
              Total collateral value
            </div>
            <div class="value">
              {{ shortenNumber(poolBorrowLimit || 0) }} {{ selectedPool?.asset.symbol }}
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
      </j-accordion>
    </div>
  </Transition>

  <warning-block
    v-if="attentionText"
    :text="attentionText"
    class="mt-3"
  />

  <div class="extra-info mt-3">
    <j-checkbox
      v-model="agree"
      :disabled="!isCanBorrow || availableToBorrow <= 0"
    >
      <div class="extra-info__label">
        I acknowledge the risks involved.
      </div>
    </j-checkbox>
  </div>

  <div class="supply-card__action mt-3">
    <market-dialog-action-btn
      variant="brand-secondary"
      class="market-action-btn"
      size="md"
      :loading="isLoading"
      :pool="selectedPool?.raw.pool"
      :disabled="!agree || !isCanBorrow || amount > availableToBorrow || availableToBorrow <= 0"
      @click-handler="borrow"
    >
      <i-metrics-complete class="complete-icon" /> Borrow {{ selectedPool?.asset.symbol }}
    </market-dialog-action-btn>
  </div>
</template>
