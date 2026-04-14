<script lang="ts" setup>
import type { MultiplyVaultItem } from '~/types/table'
import { amountToUsdWithShort, formatCompactUSD, formatPrice, maxDecimalsForShortenNumber, shortenNumber, truncatePercent } from '~/utils'

const {
  vault,
  compact = false,
} = defineProps<{
  vault?: MultiplyVaultItem
  compact?: boolean
}>()

const marketActions = useMarketActions()

const {
  amount,
  balance,
  slippage,
  percentFromMax,
  selectedMultiplier,
  currentApy,
  blockedReason,
  unhealthyReason,
  maxInputAmount,
  availableBorrowLiquidity,
  flashLoanFeeBps,
  summary,
  loadingPreview,
  previewError,
  submit,
} = useMultiplyOpen(toRef(() => vault))

const slippageModel = computed<string | number>({
  get: () => slippage.value,
  set: (value) => {
    if (value === '' || value === null || value === undefined) {
      slippage.value = 0
      return
    }

    const nextValue = Number(value)
    slippage.value = Number.isFinite(nextValue) ? nextValue : 0
  },
})

const amountRules = computed(() => [
  (value: string | number) => {
    if (value === '' || value === null || value === undefined) {
      return true
    }

    const nextValue = Number(value)
    if (!Number.isFinite(nextValue) || nextValue <= 0) {
      return 'Enter an amount greater than 0'
    }

    if (nextValue > Number(balance.value || 0)) {
      return 'Amount exceeds wallet balance'
    }

    if (nextValue > Number(maxInputAmount.value || 0)) {
      return 'Amount exceeds safe borrow limit'
    }

    return true
  },
])

const slippageRules = [
  (value: string | number) => {
    if (value === '' || value === null || value === undefined) {
      return true
    }

    const nextValue = Number(value)
    if (!Number.isFinite(nextValue) || nextValue < 0) {
      return 'Slippage cannot be negative'
    }

    if (nextValue > 50) {
      return 'Slippage must be 50% or less'
    }

    return true
  },
]

async function openMultiply() {
  await submit()
}
</script>

<template>
  <div
    v-if="vault"
    class="multiply-trade-panel"
    :class="{ 'multiply-trade-panel--compact': compact }"
  >
    <div class="input-wrapper multiply-trade-panel__input-wrapper">
      <input-widget
        v-model="amount"
        :balance="balance"
        :limit="Number(maxInputAmount) || 0"
        :price="vault.borrowPoolPrice"
        :symbol="vault.borrowAsset.symbol"
        :icon="vault.borrowAsset.icon"
        label-left="Margin amount"
        :label-right="formatPrice(balance ?? 0, 0, 4)"
        class="multiply-trade-panel__amount-input"
        :rules="amountRules"
      />
    </div>

    <div class="multiply-trade-panel__toolbar">
      <div class="multiply-trade-panel__input-meta">
        <span>Borrow limit: {{ formatPrice(Number(maxInputAmount || 0), 2, 6) }} {{ vault.borrowAsset.symbol }}</span>
        <span>Pair: {{ vault.asset.symbol }}/{{ vault.borrowAsset.symbol }}</span>
      </div>

      <div class="multiply-trade-panel__slippage-inline">
        <span class="multiply-trade-panel__slippage-inline-label">Slippage</span>
        <j-input
          v-model="slippageModel"
          class="multiply-trade-panel__slippage-input"
          size="md"
          only-numbers
          :rules="slippageRules"
          placeholder="0.5"
        >
          <template #append>
            <span class="multiply-trade-panel__suffix">%</span>
          </template>
        </j-input>
      </div>
    </div>

    <multiply-select
      v-model="percentFromMax"
      :multiplier="selectedMultiplier"
      :max-multiply="vault.maxMultiplier"
    />

    <Transition name="summary-slide">
      <div
        v-if="(amount && amount > 0) || compact"
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
              <div class="label">Projected APY</div>
              <div class="value">
                <span :class="{ 'multiply-trade-panel__apy--positive': currentApy >= 0, 'multiply-trade-panel__apy--negative': currentApy < 0 }">
                  {{ truncatePercent(currentApy, 2) }}%
                </span>
              </div>
            </div>

            <div class="summary-list__item">
              <div class="label">Target multiplier</div>
              <div class="value">
                {{ truncatePercent(selectedMultiplier || 0, 2) }}x
              </div>
            </div>

            <div class="summary-list__item align-items-start mb-2">
              <div class="label">Borrow liquidity</div>
              <div class="value">
                <div class="text-end">
                  {{ shortenNumber(availableBorrowLiquidity || 0, 2, maxDecimalsForShortenNumber(availableBorrowLiquidity || 0)) }} {{ vault.borrowAsset.symbol }}
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="separator" />

        <div class="info-summary__item">
          <div class="info-summary__header">
            Batch Preview

            <j-loading-spinner
              v-if="loadingPreview"
              width="14px"
              style="margin-left: auto;"
            />
          </div>

          <div
            v-if="previewError"
            class="summary-list"
          >
            <div class="summary-list__item">
              <div class="label">Quote</div>
              <div class="value">
                {{ previewError }}
              </div>
            </div>
          </div>

          <div
            v-else-if="loadingPreview && !summary"
            class="summary-list"
          >
            <div class="summary-list__item">
              <div class="label">Quote</div>
              <div class="value">
                Updating batch preview...
              </div>
            </div>
          </div>

          <div
            v-else-if="summary"
            class="summary-list"
          >
            <div class="summary-list__item">
              <div class="label">Flash borrow</div>
              <div class="value">
                {{ shortenNumber(summary.flashBorrowAmount || 0, 2, maxDecimalsForShortenNumber(summary.flashBorrowAmount || 0)) }} {{ vault.borrowAsset.symbol }}
              </div>
            </div>

            <div class="summary-list__item">
              <div class="label">Total swap in</div>
              <div class="value">
                {{ shortenNumber(summary.swapAmountIn || 0, 2, maxDecimalsForShortenNumber(summary.swapAmountIn || 0)) }} {{ vault.borrowAsset.symbol }}
              </div>
            </div>

            <div class="summary-list__item">
              <div class="label">Expected out</div>
              <div class="value">
                {{ shortenNumber(summary.expectedAmountOut || 0, 2, maxDecimalsForShortenNumber(summary.expectedAmountOut || 0)) }} {{ vault.asset.symbol }}
              </div>
            </div>

            <div class="summary-list__item">
              <div class="label">Min deposit</div>
              <div class="value">
                {{ shortenNumber(summary.minAmountOut || 0, 2, maxDecimalsForShortenNumber(summary.minAmountOut || 0)) }} {{ vault.asset.symbol }}
              </div>
            </div>

            <div class="summary-list__item">
              <div class="label">Final borrow</div>
              <div class="value">
                {{ shortenNumber(summary.finalBorrowAmount || 0, 2, maxDecimalsForShortenNumber(summary.finalBorrowAmount || 0)) }} {{ vault.borrowAsset.symbol }}
              </div>
            </div>

            <div class="summary-list__item align-items-start mb-2">
              <div class="label">Est. collateral value</div>
              <div class="value">
                <div class="text-end">
                  ${{ amountToUsdWithShort(summary.minAmountOut, vault.price, false) }}
                </div>
              </div>
            </div>
          </div>

          <div
            v-else
            class="summary-list"
          >
            <div class="summary-list__item">
              <div class="label">Quote</div>
              <div class="value">
                Enter an amount to build the flash-borrow batch.
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
              <div class="label">Flash loan fee</div>
              <div class="value">
                {{ truncatePercent(flashLoanFeeBps / 100, 2) }}%
              </div>
            </div>

            <div class="summary-list__item">
              <div class="label">Collateral price</div>
              <div class="value">
                {{ formatCompactUSD(vault.price, 2, 4) }}
              </div>
            </div>
          </div>
        </j-accordion>
      </div>
    </Transition>

    <warning-block
      v-if="blockedReason"
      :text="blockedReason"
      class="mt-3"
    />

    <warning-block
      v-else-if="unhealthyReason"
      :text="unhealthyReason"
      class="mt-3"
      is-warning
    />

    <div class="supply-card__action mt-3">
      <market-dialog-action-btn
        variant="brand-secondary"
        class="market-action-btn"
        size="md"
        :loading="marketActions.isLoading(vault.pool_address, 'multiplyOpen', vault.market)"
        :pool="vault.depositPoolData.pool"
        :disabled="Boolean(blockedReason) || Boolean(unhealthyReason) || marketActions.isDisabled(vault.pool_address, 'multiplyOpen', vault.market) || !amount || amount <= 0 || amount > balance"
        @click-handler="openMultiply"
      >
        <i-metrics-complete class="complete-icon" /> Open Multiply
      </market-dialog-action-btn>
    </div>
  </div>
</template>

<style lang="scss">
.multiply-trade-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;

  &--compact {
    gap: 14px;
  }

  &__input-meta {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 16px;
    color: $text-tertiary;
    font-size: 12px;
    flex-wrap: wrap;

    @media (max-width: $breakpoint-md) {
      gap: 4px 12px;
    }
  }

  &__toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;

    @media (max-width: $breakpoint-md) {
      flex-direction: column;
      align-items: stretch;
      gap: 8px;
    }
  }

  &__slippage-inline {
    display: flex;
    align-items: center;
    flex-direction: column;
    gap: 8px;
    flex-shrink: 0;

    @media (max-width: $breakpoint-md) {
      justify-content: space-between;
    }
  }

  &__slippage-inline-label {
    font-size: 12px;
    font-weight: 700;
    color: $text-tertiary;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  &__slippage-input {
    width: 92px;
    height: 28px;

    &:focus-within {
      .input-group {
        border-color: $navi-200;
      }
    }

    .input-group {
      height: 28px;
      border-radius: 6px;
      border-color: $navi-400;

      input {
        font-size: 12px;
        margin-bottom: -2px;
      }
    }

    .j-input__append {
      display: flex;
      align-items: center;
    }
  }

  &__suffix {
    color: $text-brand;
    font-size: 10px;
    font-weight: 700;
  }

  &__apy {
    &--positive {
      color: $success;
    }

    &--negative {
      color: $danger;
    }
  }
}
</style>
