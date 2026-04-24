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
  openMultiply,
} = useMultiplyOpen(toRef(() => vault))

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
  const marginSymbol = marginAsset.value?.symbol || borrowSymbol
  const swapInSymbol = isMarginBorrow.value ? marginSymbol : (notMarginAsset.value?.symbol || borrowSymbol)
  const slippageLabel = `${truncatePercent(slippage.value || 0, 1)}%`

  return [
    {
      id: 1,
      title: 'Flash borrow',
      tooltip: 'Temporary flash-loan amount used to open the leveraged position in one batch.',
      subtitle: `borrowed from the ${marginSymbol} pool`,
      value: summary.value.flashBorrowAmount,
      symbol: marginSymbol,
    },
    {
      id: 2,
      title: 'Total to swap',
      tooltip: 'Amount routed through the swap provider to assemble or repay the leveraged collateral leg.',
      subtitle: `swap at <= ${slippageLabel} slippage`,
      value: summary.value.swapAmountIn,
      symbol: swapInSymbol,
    },
    {
      id: 3,
      title: isMarginBorrow.value ? `${depositSymbol} received` : 'Flash repay target',
      tooltip: isMarginBorrow.value
        ? 'Estimated deposit asset received after slippage protection and deposited as collateral.'
        : 'Exact deposit-asset amount the swap must produce to repay the flash loan plus fee.',
      subtitle: isMarginBorrow.value ? 'deposited as collateral' : 'required to settle the flash loan',
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
                {{ shortenNumber(flashLoanFeeAmount || 0, 2, maxDecimalsForShortenNumber(flashLoanFeeAmount || 0)) }} {{ marginAsset?.symbol }}
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
        :pool-secondary="vault.borrowPoolData.pool"
        :disabled="Boolean(unhealthyReason) || marketActions.isDisabled(vault.pool_address, 'multiplyOpen', vault.market) || !amount || amount <= 0 || amount > balance"
        @click-handler="openMultiply"
      >
        <i-metrics-complete class="complete-icon" />
        <template v-if="isUserHaveMultiply()">
          Add To Position
        </template>
        <template v-else>
          Open Position
        </template>
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
    justify-content: space-between;
    gap: 24px;
    margin: 12px 0 0;
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
