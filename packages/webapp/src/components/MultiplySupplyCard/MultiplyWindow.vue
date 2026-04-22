<script lang="ts" setup>
import type { MultiplyVaultItem } from '~/types/table'
import { amountToUsdWithShort, formatCompactUSD, formatPrice, maxDecimalsForShortenNumber, shortenNumber, truncatePercent } from '~/utils'

const {
  vault,
  compact = false,
} = defineProps<{
  vault?: MultiplyVaultItem
  compact?: boolean
  teleportTarget?: HTMLElement
}>()

const marketActions = useMarketActions()

const {
  amount,
  balance,
  slippage,
  percentFromMax,
  selectedMultiplier,
  currentApy,
  unhealthyReason,
  maxInputAmount,
  availableBorrowLiquidity,
  flashLoanFeeAmount,
  summary,
  loadingPreview,
  previewError,
  isMarginBorrow,
  marginAsset,
  marginPrice,
  notMarginAsset,
  swapProviderAddress,
  openMultiply,
} = useMultiplyOpen(toRef(() => vault))

const {
  publicKey,
} = useWalletComposable()

const slippageInput = computed<string | number>({
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
      return isMarginBorrow.value ? 'Amount exceeds safe borrow limit' : 'Amount exceeds available leverage capacity'
    }

    return true
  },
])
</script>

<template>
  <div
    v-if="vault"
    class="multiply-trade-panel"
    :class="{ 'multiply-trade-panel--compact': compact }"
  >
    <div class="input-wrapper multiply-trade-panel__input-wrapper">
      <teleport-content
        :to="teleportTarget"
      >
        <div class="multiply-trade-panel__toolbar">
          <provider-select v-model="swapProviderAddress" />
          <slippage-select v-model="slippageInput" />
        </div>
      </teleport-content>
      <input-widget
        v-model="amount"
        :balance="balance"
        :limit="Number(maxInputAmount) || 0"
        :price="marginPrice"
        :symbol="marginAsset?.symbol"
        label-left="Margin amount"
        :label-right="formatPrice(balance ?? 0, 0, 4)"
        class="multiply-trade-panel__amount-input"
        :rules="amountRules"
        variant="success"
      >
        <template #prepend>
          <j-popover
            position="bottom"
            :teleport-to-body="false"
            close-popup
            :disabled="loadingPreview"
          >
            <template #target="{ active }">
              <div
                class="select-pool-btn"
              >
                <img
                  :src="marginAsset?.icon"
                  alt="asset icon"
                >
                {{ marginAsset?.symbol }}
                <i-app-chevron-down
                  class="arrow-icon"
                  :class="{ 'arrow-icon--active': active }"
                />
              </div>
            </template>

            <div class="select-pool-menu">
              <div
                class="select-pool-menu__item"
                @click="isMarginBorrow = !isMarginBorrow"
              >
                <img
                  :src="notMarginAsset?.icon"
                  alt="asset icon"
                >
                <span>{{ notMarginAsset?.symbol }}</span>
              </div>
            </div>
          </j-popover>

        </template>
      </input-widget>
    </div>

    <div class="multiply-trade-panel__input-meta">
      <span>{{ isMarginBorrow ? 'Borrow limit' : 'Approx. margin limit' }}: {{ shortenNumber(Number(maxInputAmount || 0), 2, 2) }} {{ marginAsset?.symbol }}</span>
      <span>Pair: {{ vault.asset.symbol }}/{{ vault.borrowAsset.symbol }}</span>
    </div>

    <multiply-select
      v-model="percentFromMax"
      :multiplier="selectedMultiplier"
      :max-multiply="vault.maxMultiplier"
      :net-apy="currentApy"
      :pool="vault.depositPoolData"
    />

    <warning-block
      v-if="previewError"
      class="mt-3"
      title="Repay Multiply"
      :text="previewError"
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
                  {{ shortenNumber(availableBorrowLiquidity || 0, 2, maxDecimalsForShortenNumber(availableBorrowLiquidity || 0)) }} {{ marginAsset?.symbol }}
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="separator" />

        <j-accordion
          class="info-summary__item accordion-summary"
          title="Fees"
        >
          <template #title>
            <div
              class="info-summary__header"
              style="width: 100%;"
            >
              Batch Preview

              <j-loading-spinner
                v-if="loadingPreview"
                width="14px"
                style="margin-left: auto;"
              />
            </div>
          </template>

          <div class="summary-list">
            <div class="info-summary__item">
              <div
                v-if="loadingPreview && !summary"
                class="summary-list"
              >
                <div class="summary-list__item mb-2">
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
                    {{ shortenNumber(summary.flashBorrowAmount || 0, 2, maxDecimalsForShortenNumber(summary.flashBorrowAmount || 0)) }} {{ marginAsset?.symbol }}
                  </div>
                </div>

                <div class="summary-list__item">
                  <div class="label">Total swap in</div>
                  <div class="value">
                    {{ shortenNumber(summary.swapAmountIn || 0, 2, maxDecimalsForShortenNumber(summary.swapAmountIn || 0)) }} {{ isMarginBorrow ? marginAsset?.symbol : notMarginAsset?.symbol }}
                  </div>
                </div>

                <div class="summary-list__item">
                  <div class="label">{{ isMarginBorrow ? 'Expected out' : 'Flash repay target' }}</div>
                  <div class="value">
                    {{ shortenNumber(summary.expectedAmountOut || 0, 2, maxDecimalsForShortenNumber(summary.expectedAmountOut || 0)) }} {{ vault.asset.symbol }}
                  </div>
                </div>

                <div class="summary-list__item">
                  <div class="label">{{ isMarginBorrow ? 'Min deposit' : 'Swap out target' }}</div>
                  <div class="value">
                    {{ shortenNumber(summary.minAmountOut || 0, 2, maxDecimalsForShortenNumber(summary.minAmountOut || 0)) }} {{ vault.asset.symbol }}
                  </div>
                </div>

                <div class="summary-list__item">
                  <div class="label">Total collateral</div>
                  <div class="value">
                    {{ shortenNumber(summary.depositAmount || 0, 2, maxDecimalsForShortenNumber(summary.depositAmount || 0)) }} {{ vault.asset.symbol }}
                  </div>
                </div>

                <div class="summary-list__item">
                  <div class="label">Final borrow</div>
                  <div class="value">
                    {{ shortenNumber(summary.finalBorrowAmount || 0, 2, maxDecimalsForShortenNumber(summary.finalBorrowAmount || 0)) }} {{ isMarginBorrow ? marginAsset?.symbol : notMarginAsset?.symbol }}
                  </div>
                </div>

                <div class="summary-list__item align-items-start mb-2">
                  <div class="label">Est. collateral value</div>
                  <div class="value">
                    <div class="text-end">
                      ${{ amountToUsdWithShort(summary.depositAmount, vault.price, false) }}
                    </div>
                  </div>
                </div>
              </div>

              <div
                v-else
                class="summary-list"
              >
                <div class="summary-list__item mb-2">
                  <div class="label">Quote</div>
                  <div class="value">
                    <template v-if="publicKey">
                      Enter an amount to build the flash-borrow batch.
                    </template>
                    <template v-else>
                      Connect wallet to build the flash-borrow batch.
                    </template>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </j-accordion>

        <div class="separator" />

        <j-accordion
          class="info-summary__item accordion-summary"
          title="Fees"
        >
          <div class="summary-list">
            <div class="summary-list__item">
              <div class="label">Flash loan fee</div>
              <div class="value">
                {{ shortenNumber(flashLoanFeeAmount || 0, 2, maxDecimalsForShortenNumber(flashLoanFeeAmount || 0)) }} {{ marginAsset?.symbol }}
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
      v-if="unhealthyReason"
      :text="unhealthyReason"
      class="mt-3"
      is-warning
    />

    <div class="supply-card__action mt-3">
      <market-dialog-action-btn
        variant="positive"
        class="market-action-btn"
        size="md"
        :loading="marketActions.isLoading(vault.pool_address, 'multiplyOpen', vault.market)"
        :pool="vault.depositPoolData.pool"
        :disabled="Boolean(unhealthyReason) || marketActions.isDisabled(vault.pool_address, 'multiplyOpen', vault.market) || !amount || amount <= 0 || amount > balance"
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
    align-items: flex-start;
    gap: 24px;
    margin-bottom: 16px;
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

  .select-pool-menu {
    &__item {
      display: flex;
      align-items: center;
      gap: 8px;
      color: $text-secondary;
      cursor: pointer;

      img {
        width: 20px;
        height: 20px;
      }
    }
  }

  .arrow-icon {
    &--active {
      transform: rotate(180deg);
    }
  }
}
</style>
