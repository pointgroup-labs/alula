<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'
import { CLEAR_DIALOG_TIMEOUT, RELOAD_FEE_INTERVAL } from '~/config'
import { VAULT_INFO } from '~/config/vault'
import { formatPrice, truncatePercent } from '~/utils'

const {
  data,
} = defineProps<{
  data?: MultiplyTableItem
}>()

const dataRef = ref(data)

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
} = useLeverage(dataRef)

const dialog = defineModel<boolean>({ default: false })

const market = useMarketActions()

let interval: string | number | NodeJS.Timeout | undefined

watch(dialog, async (v) => {
  clearInterval(interval)
  if (!v) {
    setTimeout(() => {
      amount.value = 0
    }, CLEAR_DIALOG_TIMEOUT)
    return
  }

  interval = setInterval(() => {
    reloadFee.value = true
    nextTick(() => {
      reloadFee.value = false
    })
  }, RELOAD_FEE_INTERVAL)
})
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="multiply-dialog"
  >
    <template #header>
      <div class="multiply-dialog__title">
        <span>Multiply</span>
      </div>

    </template>

    <div class="multiply-dialog__body">
      <div class="multiply-dialog__data with-border">

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

        <div
          v-if="data"
          class="dialog-info-table"
        >

          <!-- Liquidation Available -->
          <div
            class="dialog-info-table__item"
          >
            <span>Liquidity Available</span>
            <span>{{ availableLiquidity }}</span>
          </div>

          <!-- Max APY -->
          <div class="dialog-info-table__item">
            <span>APY</span>
            <span>{{ truncatePercent(maxAPY, 2) }} %</span>
          </div>

          <!-- Max Multiplied Amount -->
          <div class="dialog-info-table__item">
            <span>Max Multiplied Amount</span>
            <span>{{ formatPrice(Number(supplyLimit || 0).toFixed(2), 2) }} {{ multiplySymbol }}</span>
          </div>

          <!-- Total Supply -->
          <div class="dialog-info-table__item">
            <span>Total Supply</span>
            <span>{{ formatPrice(Number(data!.supplied || 0), 2, 2) }} {{ data!.asset.symbol }}</span>
          </div>

          <!-- Market fee -->
          <div class="dialog-info-table__item">
            <span>Operation Fee</span>
            <span>{{ formatPrice(marketFee, 0, 5) }} {{ data?.borrowAsset.symbol }}</span>
          </div>

          <!-- Tx fee -->
          <div class="dialog-info-table__item">
            <span>Transaction Fee</span>
            <span>{{ txFee }} XLM</span>
          </div>

        </div>

        <multiply-select
          v-model="percentFromMaxMultiply"
          :multiplier="selectedMultiplier"
          :max-multiply="maxMultiply"
        />

        <div class="multiply-dialog-action">
          <market-dialog-action-btn
            variant="primary"
            :loading="market.isLoading(String(data?.pool_address), 'leverage', String(data?.market))"
            :pool="data?.depositPoolData.pool"
            :disabled="Number(selectedMultiplier) < 1"
            @click-handler="leverage"
          >
            Multiply {{ data?.asset.symbol }}
          </market-dialog-action-btn>
        </div>
      </div>

      <div class="d-flex flex-column multiply-chart-with-vault">
        <multiply-apy-chart />

        <div class="loop-multiply__vault hide-xs">
          <div class="loop-multiply__vault-title">
            {{ VAULT_INFO.title }}
          </div>

          <div class="loop-multiply__vault-info">
            {{ VAULT_INFO.shortDesciption }}
          </div>
        </div>
      </div>
    </div>
  </j-dialog>
</template>

<style lang="scss">
.multiply-dialog {
  .modal-dialog {
    width: max-content;
    max-width: 874px;

    @media (max-width: $breakpoint-sm) {
      width: 100%;
    }
  }

  .j-input__prepend {
    width: 40px;
    min-width: 40px;

    img {
      width: 32px;
      height: 32px;
      object-fit: contain;
      border-radius: 50%;
    }
  }

  &__title {
    color: $dark;
    font-size: 20px;
    font-style: normal;
    font-weight: 500;
    line-height: 20px;
  }

  &__body {
    padding-top: $spacing-16;
    display: flex;
    flex-direction: row;
    gap: 48px;

    @media (max-width: $breakpoint-xs) {
      flex-direction: column-reverse;
      gap: $spacing-16;
    }
  }

  &__data {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: $spacing-16;

    @media (max-width: $breakpoint-xs) {
      min-width: 100%;
      width: 100%;
    }

    &.with-border {
      &::after {
        content: '';
        width: 1px;
        height: 100%;
        background-color: $neutral-5;
        position: absolute;
        top: 0;
        right: -24px;

        @media (max-width: $breakpoint-xs) {
          display: none;
        }
      }
    }
  }

  .dialog-info-table {
    &__item {
      span {
        white-space: nowrap;
        &:nth-child(2) {
          font-family: sans-serif;
          font-variant-numeric: tabular-nums;

          @media (max-width: $breakpoint-sm) {
            width: initial;
          }
        }
      }
    }
  }

  .multiply-chart-with-vault {
    width: 500px;

    @media (max-width: $breakpoint-sm) {
      width: 100%;
    }
  }

  .loop-multiply__vault {
    display: flex;
    flex-direction: column;
    gap: $spacing-10;
    margin-top: auto;

    &-title {
      color: $dark;
      font-size: 12px;
      font-style: normal;
      font-weight: 700;
      line-height: 16px;
    }

    &-info {
      color: $neutral-16;
      font-size: 12px;
      font-style: normal;
      font-weight: 400;
      line-height: 16px;
    }
  }

  .multiply-dialog-action {
    display: flex;
    justify-content: space-between;
    gap: $spacing-32;

    .action-info {
      white-space: nowrap;
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 2px;

      span:first-child {
        color: $neutral-12;
        font-size: 12px;
        font-style: normal;
        font-weight: 500;
        line-height: 16px;
      }

      span:last-child {
        font-size: 20px;
        font-style: normal;
        font-weight: 700;
        line-height: 20px;
      }
    }

    .btn {
      width: 100%;
    }
  }
}

.theme-dark {
  .multiply-dialog {
    .j-input__prepend .popover {
      .popover-borrow-asset {
        color: #fff;
      }
    }

    &__data {
      &.with-border {
        &::after {
          background: $neutral-18;
        }
      }
    }
  }
}
</style>
