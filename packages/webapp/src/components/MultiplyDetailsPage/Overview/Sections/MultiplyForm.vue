<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'
import { formatPrice, truncatePercent } from '~/utils'

const market = useMarketActions()

const poolData = inject<Ref<MultiplyTableItem>>('selectedPool')

const {
  reloadFee,
  depositAsset,
  borrowAsset,
  amount,
  balance,
  selectedMultiplier,

  txFee,
  availableLiquidity,
  supplyLimit,
  maxAPY,

  maxMultiply,
  percentFromMaxMultiply,

  multiplySymbol,
  marketFee,

  swapAsset,
  leverage,
} = useLeverage(poolData)

const debouncedAmount = refDebounced(amount, 1000)

const isHasAmount = computed(() => {
  return !!(debouncedAmount.value && debouncedAmount.value > 0)
})

watch(isHasAmount, async (v) => {
  if (!v) {
    txFee.value = 0
    return
  }
  reloadFee.value = true
  nextTick(() => {
    reloadFee.value = false
  })
})
</script>

<template>
  <section id="multiply-form">
    <div class="multiply-form-card">
      <div class="form-header">
        <h3 class="form-title">Multiply Position</h3>
        <div class="form-subtitle">Leverage your assets to multiply exposure</div>
      </div>

      <div class="form-content">
        <input-widget
          v-model="amount"
          :balance="balance"
          :limit="supplyLimit"
          class="multiply-dialog__input"
          label-left="You Deposit"
          :rules="[
            (v) => {
              return v && Number(v) < balance || 'Insufficient balance'
            },
            (v) => {
              return (supplyLimit <= 0 || Number(v) <= supplyLimit) || 'Pool leverage limit'
            },
          ]"
        >
          <template #label-right>
            Wallet: {{ balance }} {{ depositAsset?.name }}
          </template>
          <template #prepend>
            <j-select-popover>
              <template #menu>
                <div
                  class="popover-borrow-asset"
                  @click="swapAsset"
                >
                  <img
                    :src="borrowAsset?.icon"
                    :alt="`${borrowAsset?.name} icon`"
                  >
                  {{ borrowAsset?.name }}
                </div>
              </template>
              <template #target>
                <img
                  :src="depositAsset?.icon"
                  :alt="`${depositAsset?.name} icon`"
                >
              </template>
            </j-select-popover>
          </template>
        </input-widget>

        <div class="multiplier-section">
          <multiply-select
            v-model="percentFromMaxMultiply"
            :multiplier="selectedMultiplier"
            :max-multiply="maxMultiply"
          />
        </div>

        <market-dialog-action-btn
          class="severage-form-btn"
          variant="primary"
          :loading="market.isLoading(String(poolData?.pool_address), 'leverage', String(poolData?.market))"
          :pool="poolData?.depositPoolData.pool"
          :disabled="Number(selectedMultiplier) < 1"
          @click-handler="leverage"
        >
          Multiply {{ poolData?.asset.symbol }}
        </market-dialog-action-btn>
      </div>

      <transition name="slide-fade">
        <div
          v-if="amount > 0 && poolData"
          class="transaction-details"
        >
          <div class="details-header">
            <h4 class="details-title">Transaction Details</h4>
          </div>

          <div class="details-grid">
            <!-- Liquidation Available -->
            <div class="detail-item">
              <div class="detail-label">
                <span>Liquidity Available</span>
                <j-tooltip>
                  <i-app-info-circle class="info-icon" />
                  <template #content>
                    Total available liquidity in the pool
                  </template>
                </j-tooltip>
              </div>
              <div class="detail-value">{{ availableLiquidity }}</div>
            </div>

            <!-- Max APY -->
            <div class="detail-item highlight-apy">
              <div class="detail-label">
                <span>APY</span>
                <j-tooltip>
                  <i-app-info-circle class="info-icon" />
                  <template #content>
                    Annual percentage yield on your leveraged position
                  </template>
                </j-tooltip>
              </div>
              <div
                class="detail-value"
                :class="{ positive: maxAPY > 0, negative: maxAPY < 0 }"
              >
                {{ truncatePercent(maxAPY, 2) }} %
              </div>
            </div>

            <!-- Max Multiplied Amount -->
            <div class="detail-item">
              <div class="detail-label">
                <span>Max Multiplied Amount</span>
              </div>
              <div class="detail-value">{{ formatPrice(Number(supplyLimit || 0).toFixed(2), 2) }} {{ multiplySymbol }}</div>
            </div>

            <!-- Total Supply -->
            <div class="detail-item">
              <div class="detail-label">
                <span>Total Supply</span>
              </div>
              <div class="detail-value">{{ formatPrice(Number(poolData!.supplied || 0), 2, 2) }} {{ poolData!.asset.symbol }}</div>
            </div>

            <!-- Market fee -->
            <div class="detail-item fee-item">
              <div class="detail-label">
                <span>Operation Fee</span>
              </div>
              <div class="detail-value">{{ formatPrice(marketFee, 0, 5) }} {{ poolData?.borrowAsset.symbol }}</div>
            </div>

            <!-- Tx fee -->
            <div class="detail-item fee-item">
              <div class="detail-label">
                <span>Transaction Fee</span>
              </div>
              <div class="detail-value">{{ txFee }} XLM</div>
            </div>
          </div>
        </div>
      </transition>
    </div>
  </section>
</template>

<style lang="scss">
section#multiply-form {
  min-width: 378px;
  margin-top: 44px;

  .multiply-form-card {
    border-radius: 20px;
    padding: 0;
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.98) 0%, rgba(249, 250, 251, 0.99) 100%);
    border: 1px solid rgba(33, 44, 66, 0.08);
    box-shadow:
      0px 2px 8px rgba(33, 44, 66, 0.04),
      0px 8px 24px rgba(33, 44, 66, 0.06),
      inset 0px 1px 0px rgba(255, 255, 255, 0.7);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;

    &:hover {
      box-shadow:
        0px 4px 12px rgba(33, 44, 66, 0.08),
        0px 12px 32px rgba(33, 44, 66, 0.1),
        inset 0px 1px 0px rgba(255, 255, 255, 0.9);
      border-color: rgba(92, 108, 255, 0.2);
    }
  }

  .form-header {
    padding: 24px 28px 20px;
    background: linear-gradient(135deg, rgba(92, 108, 255, 0.05) 0%, rgba(92, 108, 255, 0.02) 100%);
    border-bottom: 1px solid rgba(33, 44, 66, 0.06);

    .form-title {
      font-size: 20px;
      font-weight: 600;
      margin: 0;
      color: #262729;
      letter-spacing: -0.02em;
    }

    .form-subtitle {
      font-size: 13px;
      color: #8a8b8d;
      margin-top: 4px;
      font-weight: 400;
    }
  }

  .form-content {
    padding: 24px 28px;
    display: flex;
    flex-direction: column;
    gap: $spacing-20;
  }

  .multiplier-section {
    padding: 16px;
    background: rgba(92, 108, 255, 0.02);
    border-radius: 12px;
    border: 1px solid rgba(92, 108, 255, 0.08);
    transition: all 0.2s ease;

    &:hover {
      background: rgba(92, 108, 255, 0.04);
      border-color: rgba(92, 108, 255, 0.15);
    }
  }

  .input-wrapper {
    margin-left: 6px;
  }

  .loop-multiply {
    width: 100%;
  }

  .severage-form-btn {
    width: 100%;
    height: 52px;
    font-size: 16px;
    font-weight: 600;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 108, 228, 0.2);
    transition: all 0.2s ease;
  }

  .transaction-details {
    border-top: 1px solid rgba(33, 44, 66, 0.06);
    background: linear-gradient(180deg, rgba(249, 250, 251, 0.5) 0%, rgba(255, 255, 255, 0.3) 100%);
    padding: 20px 28px 24px;

    .details-header {
      margin-bottom: 16px;

      .details-title {
        font-size: 15px;
        font-weight: 600;
        color: #4e4f51;
        margin: 0;
        letter-spacing: -0.01em;
      }
    }

    .details-grid {
      display: flex;
      flex-direction: column;
      gap: 12px;
    }

    .detail-item {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 16px;
      background: rgba(255, 255, 255, 0.7);
      border-radius: 10px;
      border: 1px solid rgba(33, 44, 66, 0.05);
      transition: all 0.2s ease;

      &:hover {
        background: rgba(255, 255, 255, 0.9);
        border-color: rgba(92, 108, 255, 0.15);
        transform: translateX(2px);
      }

      &.highlight-apy {
        background: linear-gradient(135deg, rgba(92, 108, 255, 0.08) 0%, rgba(92, 108, 255, 0.04) 100%);
        border-color: rgba(92, 108, 255, 0.2);

        &:hover {
          background: linear-gradient(135deg, rgba(92, 108, 255, 0.12) 0%, rgba(92, 108, 255, 0.06) 100%);
          border-color: rgba(92, 108, 255, 0.3);
        }

        .detail-value {
          font-weight: 600;
          font-size: 15px;
        }
      }

      &.fee-item {
        background: rgba(249, 250, 251, 0.5);

        .detail-value {
          font-size: 13px;
          color: #8a8b8d;
        }
      }

      .detail-label {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 14px;
        color: #4e4f51;
        font-weight: 500;

        .info-icon {
          width: 14px;
          height: 14px;
          opacity: 0.5;
          cursor: help;
          transition: opacity 0.2s ease;

          &:hover {
            opacity: 1;
          }
        }
      }

      .detail-value {
        font-size: 14px;
        font-weight: 600;
        color: #262729;
        font-variant-numeric: tabular-nums;

        &.positive {
          color: #08b576;
          font-weight: 700;
        }

        &.negative {
          color: #fb4747;
          font-weight: 700;
        }
      }
    }
  }

  // Animations
  .slide-fade-enter-active {
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .slide-fade-leave-active {
    transition: all 0.2s cubic-bezier(0.4, 0, 1, 1);
  }

  .slide-fade-enter-from {
    transform: translateY(-10px);
    opacity: 0;
  }

  .slide-fade-leave-to {
    transform: translateY(-5px);
    opacity: 0;
  }

  @media (max-width: $breakpoint-md) {
    min-width: 100%;
    margin-top: 32px;

    .multiply-form-card {
      border-radius: 16px;
    }

    .form-header,
    .form-content,
    .transaction-details {
      padding-left: 20px;
      padding-right: 20px;
    }
  }
}
</style>
