<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const emits = defineEmits(['dialogHandler'])

const marketActions = useMarketActions()

const poolData = inject<Ref<MarketTableItem>>('selectedPool')

const {
  availableToBorrow,
  isCanBorrow,
  attentionText,
} = useBorrowDialog(poolData, ref(false))

const {
  balance,
  isCanSupply,
  attentionText: supplyAttentionText,
} = useSupplyDialog(poolData, ref(false))

async function supplyDialogHandler(action: 'supply' | 'borrow') {
  const marketName = poolData?.value?.market ?? ''
  const poolAddress = poolData?.value?.pool_address ?? ''
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
          <info-tooltip
            v-if="!isCanSupply"
            :text="supplyAttentionText"
            icon-color="#fb4747"
          />
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
          <info-tooltip
            v-if="!isCanBorrow"
            :text="attentionText"
            icon-color="#fb4747"
          />
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
  display: flex;
  justify-content: flex-end;
  gap: 32px;
  padding-bottom: $spacing-lg;

  @media (max-width: $breakpoint-xs) {
    flex-direction: column;
    align-items: flex-end;
    gap: 16px;
  }

  .action-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    gap: 16px;

    &:first-child {
      &::after {
        content: '';
        width: 1px;
        height: 100%;
        background-color: $navi-50;
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
      color: $navi-200;
      display: flex;
      align-items: center;
      gap: 4px;

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

      .info-tooltip {
        margin-left: 8px;
      }
    }
  }
}

.theme-dark {
  .market-detail-actions {
    .action-stats__value:not(.action-stats__value--danger) {
      color: #fff;
    }

    .action-wrapper {
      &:first-child {
        &::after {
          background-color: $neutral-18;
        }
      }
    }
  }
}
</style>
