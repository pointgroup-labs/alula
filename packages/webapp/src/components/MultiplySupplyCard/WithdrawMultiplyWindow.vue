<script lang="ts" setup>
import type { MultiplyTableItem, MultiplyVaultItem } from '~/types/table'
import { useMultiplyWithdraw } from '~/hooks/multiply/withdraw'

const {
  opened = false,
  vault,
} = defineProps<{
  opened?: boolean
  vault?: MultiplyTableItem | MultiplyVaultItem
}>()

const multiplyStore = useMultiplyStore()

const isValidate = ref(true)

const swapProviderAddress = toRef(multiplyStore, 'swapProviderAddress')

const {
  amount,
  balance,
  slippage,
  inputLabel,
  marginPrice,
  currentDeposited,
  maxAmountLabel,
  swapInputEstimate,
  estimatedReceiveAmount,
  debtRepaidAmount,
  remainingBorrowAmount,
  remainingDepositAmount,
  marketFee,
  preview,
  previewError,
  previewLoading,
  txFee,
  loading: isLoading,
  isMarginBorrow,
  marginAsset,
  notMarginAsset,
  isClosePosition,
  withdraw,
} = useMultiplyWithdraw(toRef(() => opened), toRef(() => vault))

async function withdrawLeverage() {
  isValidate.value = false
  await withdraw()
  isValidate.value = true
}

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

const closeDetailsSteps = computed(() => {
  if (!preview.value || !vault) {
    return []
  }

  const depositSymbol = vault.asset.symbol
  const borrowSymbol = vault.borrowAsset.symbol
  const receiveSymbol = marginAsset.value?.symbol || (isMarginBorrow.value ? borrowSymbol : depositSymbol)
  const slippageLabel = `${formatPrice(slippage.value || 0, 0, 1)}%`

  return [
    {
      id: 1,
      title: 'Debt repaid',
      tooltip: 'Debt removed from the multiply position after repay fees are applied.',
      subtitle: `repaid in ${borrowSymbol}`,
      value: debtRepaidAmount.value,
      symbol: borrowSymbol,
    },
    {
      id: 2,
      title: 'Estimated receive',
      tooltip: 'Estimated amount returned to your wallet after the close batch finishes.',
      subtitle: `received in ${receiveSymbol}`,
      value: estimatedReceiveAmount.value,
      symbol: receiveSymbol,
    },
    {
      id: 3,
      title: 'Debt after repayment',
      tooltip: 'Borrow balance that remains open after this close operation.',
      subtitle: `still owed in ${borrowSymbol}`,
      value: remainingBorrowAmount.value,
      symbol: borrowSymbol,
    },
    {
      id: 4,
      title: 'Remaining collateral',
      tooltip: 'Collateral that remains supplied in the deposit pool after this close batch.',
      subtitle: `still locked in ${depositSymbol}`,
      value: remainingDepositAmount.value,
      symbol: depositSymbol,
      valueClass: 'text-cyan',
    },
    {
      id: 5,
      title: isMarginBorrow.value ? 'Collateral sold' : 'Swap estimate',
      tooltip: 'Deposit-asset amount routed through the swap to settle the flash-loan repayment.',
      subtitle: `swap at <= ${slippageLabel} slippage`,
      value: swapInputEstimate.value,
      symbol: depositSymbol,
      valueClass: 'text-indigo',
    },
  ]
})
</script>

<template>
  <div class="multiply-withdraw-panel">
    <div>
      <input-widget
        v-model="amount"
        :balance="balance"
        class="withdraw-dialog__input"
        :price="Number(marginPrice || 0)"
        :symbol="marginAsset?.symbol"
        :label-left="inputLabel"
        variant="accent"
        :label-right="formatPrice(balance ?? 0, 0, 4)"
        :rules="[
          (v) => !isValidate || (!!v && Number(v) > 0) || `Enter ${String(inputLabel).toLowerCase()}`,
          (v) => !isValidate || Number(v) <= balance || (isMarginBorrow ? 'Flash repay target exceeds closeable debt' : 'Receive amount exceeds closeable collateral'),
        ]"
      >
        <template #prepend>
          <j-popover
            position="bottom"
            :teleport-to-body="false"
            close-popup
            :disabled="isLoading || previewLoading"
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

      <div class="multiply-trade-panel__toolbar">
        <provider-select v-model="swapProviderAddress" />
        <slippage-select v-model="slippageInput" />
      </div>
    </div>

    <warning-block
      v-if="previewError"
      class="mt-3"
      title="Repay Multiply"
      :text="previewError"
    />

    <Transition name="summary-slide">
      <div
        v-if="(amount && amount > 0) || opened"
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
              <div class="label">Current deposited</div>
              <div class="value">
                <template v-if="previewLoading && !preview">
                  <j-loading-spinner
                    width="14px"
                    style="padding: 0; width: 14px; margin-left: auto"
                  />
                </template>
                <template v-else>
                  {{ shortenNumber(currentDeposited || 0, 2, maxDecimalsForShortenNumber(currentDeposited)) }} {{ vault?.asset.symbol }}
                </template>
              </div>
            </div>

            <div class="summary-list__item align-items-start">
              <div class="label">{{ maxAmountLabel }}</div>
              <div class="value">
                <div class="text-end">
                  {{ shortenNumber(balance || 0, 2, maxDecimalsForShortenNumber(balance)) }} {{ marginAsset?.symbol }}
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
              Close details

              <j-loading-spinner
                v-if="previewLoading"
                width="14px"
                style="margin-left: auto;"
              />
            </div>
          </template>
          <div class="summary-list">
            <div
              v-for="step in closeDetailsSteps"
              :key="step.id"
              class="summary-list__item"
              :class="{ 'pb-1': step.id === closeDetailsSteps.length }"
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
                {{ formatPrice(marketFee, 2, vault?.borrowPoolData.pool.token_decimals || 7) }}
                {{ isMarginBorrow ? vault?.borrowAsset.symbol : vault?.asset.symbol }}
              </div>
            </div>

            <div class="summary-list__item">
              <div class="label">Transaction Fee</div>
              <div class="value">
                <j-loading-spinner
                  v-if="previewLoading"
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
        :disabled="previewLoading || !!previewError"
        variant="accent"
        size="md"
        class="market-action-btn"
        @click="withdrawLeverage"
      >
        <i-metrics-complete class="complete-icon" />
        <template v-if="isClosePosition">
          Close Position
        </template>
        <template v-else>
          Withdraw
        </template>
      </j-btn>
    </div>
  </div>
</template>

<style lang="scss">
.multiply-withdraw-panel {
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
