<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { CLEAR_DIALOG_TIMEOUT, POOL_REMAINING_BALANCE } from '~/config'
import { formatPrice } from '~/utils'

const props = defineProps<{ data?: MarketTableItem }>()

const dialog = defineModel({ default: false })
const isOpen = ref(false)

const poolData = toRef(props, 'data')

const {
  collateralOnly,
  balance,
  txFee,
  isLoadingFee,
  supplyLimit,
  amount,
  reserveAmount,
  isLoading,
  isCanSupply,
  attentionText,
  infoPanelData,
  supply,
  stopSupplyWatchers,
} = useSupplyDialog(poolData, dialog)

watch(dialog, (v) => {
  setTimeout(() => isOpen.value = v, v ? 0 : 500)
  if (!v) {
    stopSupplyWatchers()
    setTimeout(() => { amount.value = 0 }, CLEAR_DIALOG_TIMEOUT)
    collateralOnly.value = false
  }
})
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="supply-dialog dialog-default"
  >
    <template #header>
      <div class="dialog-default__title">
        <img
          :src="poolData?.asset.icon"
          :alt="`${poolData?.asset.symbol} icon`"
        >
        <span>Supply {{ poolData?.asset.symbol }}</span>
      </div>
    </template>

    <div
      v-if="isOpen"
      class="dialog-default__body"
    >
      <input-widget
        v-model="amount"
        :balance="balance"
        :limit="Number(supplyLimit) || 0"
        :fee="POOL_REMAINING_BALANCE + txFee + reserveAmount"
        :price="poolData?.price"
        class="dialog-default__input mb-2"
        label-left="Balance"
        :label-right="`${formatPrice(balance ?? 0, 0, 4)} ${poolData?.asset.symbol}`"
        :reset="dialog"
        :rules="[
          (v) => {
            return Number(v) < balance || 'Insufficient balance'
          },
          (v) => {
            return (supplyLimit <= 0 || Number(v) <= supplyLimit) || 'Pool supply limit'
          },
        ]"
      />

      <template v-if="poolData">
        <!-- Pool Info -->
        <info-panel
          :data="infoPanelData.poolInfo!.data"
        />

        <!-- Fees -->
        <info-panel
          :data="infoPanelData.fees!.data"
        >
          <template #txFee>
            <j-loading-spinner
              v-if="isLoadingFee"
              width="14px"
              style="padding: 0; width: 20px; height: 20px; margin: 0 auto;"
            />
            <span v-else>{{ txFee }} XLM</span>
          </template>
        </info-panel>
      </template>

      <warning-block
        v-if="!isCanSupply"
        :text="attentionText"
        :is-warning="!isCanSupply"
      />

      <div class="extra-info mt-2">
        <div class="extra-info__label">Collateral Only</div>

        <j-toggle
          v-model="collateralOnly"
          size="small"
          :disabled="!isCanSupply"
          color="#22d3ee"
        />
      </div>

      <div class="extra-info">
        <div class="extra-info__label">Supply APY</div>
        <div class="extra-info__value text-num">{{ poolData?.deposit_apy }}</div>
      </div>

      <div class="dialog-default__action mt-2">
        <market-dialog-action-btn
          variant="cyan"
          pill
          size="lg"
          :loading="isLoading"
          :pool="poolData?.raw.pool"
          :disabled="!isCanSupply || amount >= balance"
          @click-handler="supply"
        >
          Supply {{ poolData?.asset.symbol }}
        </market-dialog-action-btn>
      </div>
    </div>
  </j-dialog>
</template>

<style lang="scss">
.supply-dialog {
  .modal-content {
    max-width: 442px;
  }

  .extra-info {
    display: flex;
    align-items: center;
    justify-content: space-between;

    &__label {
      font-size: 14px;
      font-style: normal;
      font-weight: 500;
      line-height: 16px;
    }

    &__value {
      font-family: $font-Inter;
      font-size: 24px;
      font-style: normal;
      font-weight: 700;
      line-height: 36px;
    }
  }

  .j-input__label {
    display: none;
  }
}
</style>
