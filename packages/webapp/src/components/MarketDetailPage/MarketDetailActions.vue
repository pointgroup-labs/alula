<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const props = defineProps<{
  poolData?: MarketTableItem
}>()

const emits = defineEmits(['dialogHandler'])

const marketActions = useMarketActions()

const poolData = toRef(props, 'poolData')

const {
  availableToBorrow,
  isCanBorrow,
  attentionText,
} = useBorrowDialog(poolData, false)

const {
  balance,
  isCanSupply,
  attentionText: supplyAttentionText,
} = useSupplyDialog(poolData, false)

async function supplyDialogHandler(action: 'supply' | 'borrow') {
  const marketName = poolData.value?.market ?? ''
  const poolAddress = poolData.value?.pool_address ?? ''
  emits('dialogHandler', marketName, poolAddress, action)
}
</script>

<template>
  <div class="market-detail-actions">
    <div class="action-wrapper">
      <div class="action-stats">
        <div class="action-stats__title">
          <i-app-wallet-icon /> Wallet Balance
        </div>
        <div
          class="action-stats__value"
          :class="{ 'action-stats__value--danger': !isCanSupply }"
        >
          {{ formatPrice(balance, 2, 5) }} {{ poolData?.asset.symbol }}
          <j-tooltip v-if="!isCanSupply">
            <i-app-info-circle />
            <template #content>
              {{ supplyAttentionText }}
            </template>
          </j-tooltip>
        </div>
      </div>

      <j-btn
        size="md"
        pill
        icon-right
        :disabled="marketActions.isDisabled(String(poolData?.pool_address), 'deposit', String(poolData?.market))"
        :loading="marketActions.isLoading(String(poolData?.pool_address), 'deposit', String(poolData?.market))"
        @click="supplyDialogHandler('supply')"
      >
        Supply
      </j-btn>
    </div>
    <div class="action-wrapper">
      <div class="action-stats">
        <div
          class="action-stats__title"
        >
          <i-app-percentage-square-icon /> Borrow Capacity
        </div>
        <div
          class="action-stats__value"
          :class="{ 'action-stats__value--danger': !isCanBorrow }"
        >
          {{ formatPrice(availableToBorrow, 2, 5) }} {{ poolData?.asset.symbol }}
          <j-tooltip v-if="!isCanBorrow">
            <i-app-info-circle />
            <template #content>
              {{ attentionText }}
            </template>
          </j-tooltip>
        </div>
      </div>

      <j-btn
        size="md"
        pill
        icon-right
        variant="accent"
        :disabled="marketActions.isDisabled(String(poolData?.pool_address), 'borrow', String(poolData?.market))"
        :loading="marketActions.isLoading(String(poolData?.pool_address), 'borrow', String(poolData?.market))"
        @click="supplyDialogHandler('borrow')"
      >
        Borrow
      </j-btn>
    </div>
  </div>
</template>

<style lang="scss">
.market-detail-actions {
  margin-left: auto;
  display: flex;
  gap: $spacing-32;

  .action-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    gap: $spacing-16;

    &:first-child {
      &::after {
        content: '';
        width: 1px;
        height: 100%;
        background-color: $neutral-5;
        position: absolute;
        top: 0;
        right: -16px;
      }
    }
  }

  .action-stats {
    display: flex;
    flex-direction: column;
    gap: 2px;
    line-height: normal;
    font-weight: 500;

    &__title {
      font-size: 14px;
      color: $neutral-12;
      display: flex;
      align-items: center;
      gap: $spacing-4;

      svg {
        width: 16px;
        height: 16px;
      }
    }

    &__value {
      font-size: 16px;
      color: $neutral-18;
      display: flex;
      align-items: center;

      &--danger {
        color: $danger;
      }

      [class*='tooltip'] {
        display: flex;
        align-items: center;
        margin-left: 8px;
      }
    }
  }
}
</style>
