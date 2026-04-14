<script lang="ts" setup>
import type { MultiplyAccountTableItem, MultiplyTableItem, MultiplyVaultItem } from '~/types/table'
import { useMultiplyWithdraw } from '~/hooks/multiply/withdraw'

const {
  opened = false,
  data,
} = defineProps<{
  opened?: boolean
  data?: MultiplyTableItem | MultiplyAccountTableItem | MultiplyVaultItem
}>()

const isValidate = ref(true)
const {
  amount,
  balance,
  currentDeposited,
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
  withdraw,
} = useMultiplyWithdraw(toRef(() => opened), toRef(() => data))

async function withdrawLeverage() {
  isValidate.value = false
  await withdraw()
  isValidate.value = true
}
</script>

<template>
  <input-widget
    v-model="amount"
    :balance="balance"
    class="withdraw-dialog__input"
    :price="Number(data?.borrowPoolPrice || 0)"
    :symbol="data?.borrowAsset.symbol"
    :icon="data?.borrowAsset.icon"
    label-left="Repay amount"
    variant="indigo"
    :label-right="formatPrice(balance ?? 0, 0, 4)"
    :rules="[
      (v) => !isValidate || (!!v && Number(v) > 0) || 'Enter repay amount',
      (v) => !isValidate || Number(v) <= balance || 'Repay amount exceeds closeable debt',
    ]"
  />

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
                {{ shortenNumber(currentDeposited || 0, 2, maxDecimalsForShortenNumber(currentDeposited)) }} {{ data?.asset.symbol }}
              </template>
            </div>
          </div>

          <div class="summary-list__item align-items-start">
            <div class="label">Max repay</div>
            <div class="value">
              <div class="text-end">
                {{ shortenNumber(balance || 0, 2, maxDecimalsForShortenNumber(balance)) }} {{ data?.borrowAsset.symbol }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="separator" />

      <div class="info-summary__item">
        <div class="info-summary__header">
          Close details

          <j-loading-spinner
            v-if="previewLoading"
            width="14px"
            style="margin-left: auto;"
          />
        </div>

        <div class="summary-list">
          <div class="summary-list__item">
            <div class="label">Debt repaid</div>
            <div class="value">
              {{ shortenNumber(debtRepaidAmount || 0, 2, maxDecimalsForShortenNumber(debtRepaidAmount)) }} {{ data?.borrowAsset.symbol }}
            </div>
          </div>

          <div class="summary-list__item">
            <div class="label">Estimated receive</div>
            <div class="value">
              {{ shortenNumber(estimatedReceiveAmount || 0, 2, maxDecimalsForShortenNumber(estimatedReceiveAmount)) }} {{ data?.asset.symbol }}
            </div>
          </div>

          <div class="summary-list__item align-items-start">
            <div class="label">Remaining debt</div>
            <div class="value">
              <div class="text-end">
                {{ shortenNumber(remainingBorrowAmount || 0, 2, maxDecimalsForShortenNumber(remainingBorrowAmount)) }} {{ data?.borrowAsset.symbol }}
              </div>
            </div>
          </div>

          <div class="summary-list__item align-items-start">
            <div class="label">Remaining supply</div>
            <div class="value">
              <div class="text-end">
                {{ shortenNumber(remainingDepositAmount || 0, 2, maxDecimalsForShortenNumber(remainingDepositAmount)) }} {{ data?.asset.symbol }}
              </div>
            </div>
          </div>

          <div class="summary-list__item">
            <div class="label">Swap estimate</div>
            <div class="value">
              {{ formatPrice(swapInputEstimate, 2, data?.depositPoolData.pool.token_decimals || 7) }} {{ data?.asset.symbol }}
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
              {{ formatPrice(marketFee, 2, data?.borrowPoolData.pool.token_decimals || 7) }} {{ data?.borrowAsset.symbol }}
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
      variant="brand-secondary-outlined"
      size="md"
      class="market-action-btn"
      @click="withdrawLeverage"
    >
      <i-metrics-complete class="complete-icon" /> Withdraw {{ data?.borrowAsset.symbol }}
    </j-btn>
  </div>
</template>
