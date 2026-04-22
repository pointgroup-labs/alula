<script lang="ts" setup>
import type { MultiplyTableItem, MultiplyVaultItem } from '~/types/table'
import { useMultiplyWithdraw } from '~/hooks/multiply/withdraw'

const {
  opened = false,
  vault,
} = defineProps<{
  opened?: boolean
  vault?: MultiplyTableItem | MultiplyVaultItem
  teleportTarget?: HTMLElement
}>()

const isValidate = ref(true)
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
  marginAssetType,
  preview,
  previewError,
  previewLoading,
  txFee,
  loading: isLoading,
  isMarginBorrow,
  marginAsset,
  notMarginAsset,
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
</script>

<template>
  <div class="multiply-withdraw-panel">
    <teleport-content
      :to="teleportTarget"
    >
      <div class="multiply-trade-panel__toolbar">
        <provider-select />
        <slippage-select v-model="slippageInput" />
      </div>
    </teleport-content>
    <input-widget
      v-model="amount"
      :balance="balance"
      class="withdraw-dialog__input"
      :price="Number(marginPrice || 0)"
      :symbol="marginAsset?.symbol"
      :label-left="inputLabel"
      variant="danger"
      :label-right="formatPrice(balance ?? 0, 0, 4)"
      :rules="[
        (v) => !isValidate || (!!v && Number(v) > 0) || `Enter ${String(inputLabel).toLowerCase()}`,
        (v) => !isValidate || Number(v) <= balance || (isMarginBorrow ? 'Repay amount exceeds closeable debt' : 'Receive amount exceeds closeable collateral'),
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
            <div class="summary-list__item">
              <div class="label">Debt repaid</div>
              <div class="value">
                {{ shortenNumber(debtRepaidAmount || 0, 2, maxDecimalsForShortenNumber(debtRepaidAmount)) }} {{ vault?.borrowAsset.symbol }}
              </div>
            </div>

            <div class="summary-list__item">
              <div class="label">Estimated receive</div>
              <div class="value">
                {{ shortenNumber(estimatedReceiveAmount || 0, 2, maxDecimalsForShortenNumber(estimatedReceiveAmount)) }} {{ marginAsset?.symbol }}
              </div>
            </div>

            <div class="summary-list__item align-items-start">
              <div class="label">Debt after repayment</div>
              <div class="value">
                <div class="text-end">
                  {{ shortenNumber(remainingBorrowAmount || 0, 2, maxDecimalsForShortenNumber(remainingBorrowAmount)) }} {{ vault?.borrowAsset.symbol }}
                </div>
              </div>
            </div>

            <div class="summary-list__item align-items-start">
              <div class="label">Remaining supply</div>
              <div class="value">
                <div class="text-end">
                  {{ shortenNumber(remainingDepositAmount || 0, 2, maxDecimalsForShortenNumber(remainingDepositAmount)) }} {{ vault?.asset.symbol }}
                </div>
              </div>
            </div>

            <div class="summary-list__item">
              <div class="label">Swap estimate</div>
              <div class="value">
                {{ formatPrice(swapInputEstimate, 2, vault?.depositPoolData.pool.token_decimals || 7) }} {{ marginAssetType === 'borrow' ? vault?.asset.symbol : vault?.asset.symbol }}
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
        variant="negative"
        size="md"
        class="market-action-btn"
        @click="withdrawLeverage"
      >
        <i-metrics-complete class="complete-icon" /> Close Multiply
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
