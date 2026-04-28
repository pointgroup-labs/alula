<script lang="ts" setup>
import type { MultiplyVaultItem } from '~/types/table'
import { formatPrice, maxDecimalsForShortenNumber, shortenNumber, truncatePercent } from '~/utils'

const {
  vault,
  compact = false,
} = defineProps<{
  vault?: MultiplyVaultItem
  compact?: boolean
}>()

const marketActions = useMarketActions()

const userStore = useUserStore()
const multiplyStore = useMultiplyStore()
const swapProviderAddress = toRef(multiplyStore, 'swapProviderAddress')

const {
  publicKey,
} = useWalletComposable()

const {
  amount,
  balance,
  slippage,
  percentFromMax,
  selectedMultiplier,
  hardMaxMultiplier,
  currentApy,
  projectedLeverage,
  priceImpactPercent,
  unhealthyReason,
  maxTolerableSlippagePercent,
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
  openMultiply,
} = useMultiplyOpen(toRef(() => vault))

// Color tiers for the price-impact readout. Green is the "deep pool, your
// trade barely moves it" signal users are scanning for; warning/danger flag
// trades large enough vs. depth that a smaller size or different pair would
// open at a meaningfully better rate. Three contiguous bands — every value
// gets a color, no in-between "default" gap that would leave moderate impact
// rendering in plain text.
const priceImpactClass = computed(() => {
  const v = priceImpactPercent.value
  if (v == null) {
    return ''
  }
  if (v > 3) {
    return 'text-danger'
  }
  if (v > 1) {
    return 'text-warning'
  }
  return 'text-success'
})

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

const batchPreviewSteps = computed(() => {
  if (!summary.value || !vault) {
    return []
  }

  const depositSymbol = vault.asset.symbol
  const borrowSymbol = vault.borrowAsset.symbol
  // Flash borrow is always taken from the borrow pool in V3, regardless of which side the user paid margin on.
  // The swap is always borrow -> deposit, so swap input is always the borrow asset.
  const slippageLabel = `${truncatePercent(slippage.value || 0, 1)}%`

  return [
    {
      id: 1,
      title: 'Flash borrow',
      tooltip: 'Temporary flash-loan amount taken from the borrow pool to assemble the leveraged collateral in one batch.',
      subtitle: `borrowed from the ${borrowSymbol} pool`,
      value: summary.value.flashBorrowAmount,
      symbol: borrowSymbol,
    },
    {
      id: 2,
      title: 'Total to swap',
      tooltip: 'Borrow-asset amount routed through the swap provider to mint the leveraged collateral leg.',
      subtitle: `swap at <= ${slippageLabel} slippage`,
      value: summary.value.swapAmountIn,
      symbol: borrowSymbol,
    },
    {
      id: 3,
      title: `${depositSymbol} received`,
      tooltip: 'Slippage-protected swap output. This exact amount is added as collateral; the flash loan is repaid separately by the final borrow.',
      subtitle: 'deposited as collateral',
      value: summary.value.minAmountOut,
      symbol: depositSymbol,
    },
    {
      id: 4,
      title: 'Total collateral',
      tooltip: 'Final collateral that will remain in the multiply position after the batch completes.',
      subtitle: 'locked in the deposit pool',
      value: summary.value.depositAmount,
      symbol: depositSymbol,
      valueClass: 'text-cyan',
    },
    {
      id: 5,
      title: 'Final debt',
      tooltip: 'Borrow opened at the end of the batch to repay the flash loan and leave the position leveraged.',
      subtitle: `owed in ${borrowSymbol}`,
      value: summary.value.finalBorrowAmount,
      symbol: borrowSymbol,
      valueClass: 'text-indigo',
    },
  ]
})

function isUserHaveMultiply(): boolean {
  return vault && checkIsHaveMultiply(userStore.state.multiplyObligations, [vault] as any, vault.depositPoolData.pool.pool_address, vault.market)
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
        :price="marginPrice"
        :symbol="marginAsset?.symbol"
        label-left="Wallet balance"
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
                class="select-pool-btn select-asset-btn"
              >
                <template v-if="isMarginBorrow">
                  <div class="asset-icons">
                    <img
                      :src="marginAsset?.icon"
                      alt="asset icon"
                    >
                    <img
                      :src="notMarginAsset?.icon"
                      alt="asset icon"
                    >
                  </div>
                  <div class="swap-asset-label">
                    <span class="text-tertiary">{{ marginAsset?.symbol }}</span> <i-app-line-arrow-right /> {{ notMarginAsset?.symbol }}
                  </div>
                  <i-app-chevron-down
                    class="arrow-icon"
                    :class="{ 'arrow-icon--active': active }"
                  />
                </template>
                <template v-else>
                  <img
                    :src="marginAsset?.icon"
                    alt="asset icon"
                  >
                  {{ marginAsset?.symbol }}
                  <i-app-chevron-down
                    class="arrow-icon"
                    :class="{ 'arrow-icon--active': active }"
                  />
                </template>
              </div>
            </template>

            <div class="select-pool-menu select-asset-menu">
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

      <div class="multiply-trade-panel__toolbar">
        <provider-select v-model="swapProviderAddress" />
        <slippage-select v-model="slippageInput" />
      </div>

      <div
        v-if="maxTolerableSlippagePercent !== undefined && amount && amount > 0"
        class="multiply-trade-panel__slippage-hint"
        :class="{ 'multiply-trade-panel__slippage-hint--breach': slippage > maxTolerableSlippagePercent }"
      >
        <template v-if="maxTolerableSlippagePercent === 0">
          Multiplier {{ truncatePercent(selectedMultiplier, 2) }}× is too high for this pair. Lower it to enable any slippage tolerance.
        </template>
        <template v-else-if="slippage > maxTolerableSlippagePercent">
          Slippage {{ slippage }}% is above the maximum safe value of {{ maxTolerableSlippagePercent.toFixed(2) }}% at {{ truncatePercent(selectedMultiplier, 2) }}×. Lower slippage or reduce multiplier.
        </template>
        <template v-else>
          Max safe slippage at {{ truncatePercent(selectedMultiplier, 2) }}×: {{ maxTolerableSlippagePercent.toFixed(2) }}%
        </template>
      </div>
    </div>

    <multiply-select
      v-model="percentFromMax"
      :multiplier="selectedMultiplier"
      :max-multiply="vault.maxMultiplier"
      :hard-max-multiply="hardMaxMultiplier"
      :net-apy="currentApy"
      :pool="vault.depositPoolData"
    />

    <warning-block
      v-if="previewError"
      class="mt-3"
      title="Multiply preview"
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

            <div
              v-if="priceImpactPercent != null"
              class="summary-list__item"
            >
              <div class="label">
                <div class="label-with-tip">
                  Price impact
                  <info-tooltip>
                    Pure depth-driven slippage of this swap against the current pool: how
                    much your trade size moves the rate. Computed by re-quoting the same
                    path with a small probe input — the provider's fee cancels in the
                    ratio, so this isolates depth impact from fee and from oracle/AMM
                    price divergence.
                  </info-tooltip>
                </div>
              </div>
              <div class="value">
                <span :class="priceImpactClass">
                  {{ priceImpactPercent.toFixed(2) }}%
                </span>
              </div>
            </div>

            <div
              v-if="projectedLeverage != null"
              class="summary-list__item"
            >
              <div class="label">
                <div class="label-with-tip">
                  Realized leverage
                  <info-tooltip>
                    Actual on-chain leverage after the swap. Differs from the slider target because
                    the AMM charges fees and price impact, and oracle prices may not match AMM rates —
                    so the USD value of collateral added vs. debt taken on is not 1:1 with the slider.
                  </info-tooltip>
                </div>
              </div>
              <div class="value">
                {{ truncatePercent(projectedLeverage, 2) }}×
              </div>
            </div>

            <div class="summary-list__item align-items-start mb-2">
              <div class="label">Borrow liquidity</div>
              <div class="value">
                <div class="text-end">
                  {{ shortenNumber(availableBorrowLiquidity || 0, 2, maxDecimalsForShortenNumber(availableBorrowLiquidity || 0)) }} {{ vault.borrowAsset?.symbol }}
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
              <div
                v-for="step in batchPreviewSteps"
                :key="step.id"
                class="summary-list__item"
                :class="{ 'pb-1': step.id === batchPreviewSteps.length }"
              >
                <div class="label">
                  <div class="label-with-tip">
                    <span class="step-id">{{ step.id }}</span> {{ step.title }}
                    <info-tooltip>
                      {{ step.tooltip }}
                    </info-tooltip>
                  </div>
                  <div class="sub-label">
                    <i-app-line-arrow-down class="line-arrow-icon" />
                    {{ step.subtitle }}
                  </div>
                </div>
                <div
                  class="value"
                  :class="step.valueClass"
                  style="opacity: 1;"
                >
                  {{ shortenNumber(step.value || 0, 2, maxDecimalsForShortenNumber(step.value || 0)) }} {{ step.symbol }}
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
                {{ shortenNumber(flashLoanFeeAmount || 0, 2, maxDecimalsForShortenNumber(flashLoanFeeAmount || 0)) }} {{ vault.borrowAsset?.symbol }}
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
      <market-action-btn
        variant="positive"
        class="market-action-btn"
        size="md"
        :loading="marketActions.isLoading(vault.pool_address, `multiplyOpen:${vault.pairKey}`, vault.market)"
        :pool="vault.depositPoolData.pool"
        :pool-secondary="vault.borrowPoolData.pool"
        :disabled="Boolean(unhealthyReason) || marketActions.isDisabled(vault.pool_address, `multiplyOpen:${vault.pairKey}`, vault.market) || !amount || amount <= 0 || amount > balance"
        @click-handler="openMultiply"
      >
        <i-metrics-complete class="complete-icon" />
        <template v-if="isUserHaveMultiply()">
          Add To Position
        </template>
        <template v-else>
          Open Position
        </template>
      </market-action-btn>
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
    justify-content: space-between;
    gap: 24px;
    margin: 12px 0 0;
  }

  &__slippage-hint {
    margin-top: 6px;
    font-size: 11px;
    line-height: 1.4;
    color: $text-tertiary;

    &--breach {
      color: $danger;
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

  .summary-list__item {
    padding-bottom: 4px;
    .label {
      display: flex;
      flex-direction: column;
      align-items: flex-start;

      .step-id {
        display: flex;
        align-items: center;
        justify-content: center;
        color: $navi-100;
        background-color: $navi-400;
        border-radius: 50%;
        width: 16px;
        height: 16px;
        font-size: 9px;
        font-weight: 700;
      }
    }
    .label-with-tip {
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .sub-label {
      font-size: 11px;
      color: rgb(79, 96, 128);
      margin-left: 18px;

      .line-arrow-icon {
        width: 8px;
        height: 12px;
        margin-right: 4px;
      }
    }

    &:last-child {
      .sub-label {
        margin-left: 24px;
        svg {
          display: none;
        }
      }
    }
  }

  .select-asset-btn {
    .asset-icons {
      position: relative;
      width: 24px;
      height: 24px;
      border-radius: 50%;

      img {
        position: absolute;
        width: 18px;
        height: 18px;

        &:nth-child(1) {
          left: 0;
          top: 0;
        }
        &:nth-child(2) {
          right: -2px;
          bottom: -2px;
        }
      }
    }

    .swap-asset-label {
      white-space: nowrap;
      display: flex;
      align-items: center;
      gap: 4px;

      svg {
        width: 12px;
        height: 12px;
        color: #fff;
      }

      span {
        font-size: 12px;
        font-weight: 500;
      }
    }
  }
}
</style>
