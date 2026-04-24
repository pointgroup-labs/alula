<script lang="ts" setup>
import type { MultiplyVaultItem } from '~/types/table'

const {
  items,
} = defineProps<{
  items: MultiplyVaultItem[]
  showInAccounts?: boolean
}>()

const emits = defineEmits(['dialogHandler'])

const market = useMarketActions()
const multiplyStore = useMultiplyStore()

const userStore = useUserStore()

function isUserHaveMultiply(vault: MultiplyVaultItem) {
  return checkIsHaveMultiply(userStore.state.multiplyObligations, [vault] as any, vault.depositPoolData.pool.pool_address, vault.market)
}

function getApy(data: MultiplyVaultItem) {
  return data.apyAtMaxMultiplier || 0
}

function handleDetails(vault: MultiplyVaultItem) {
  multiplyStore.openVault(vault)
}
</script>

<template>
  <div
    v-if="items?.length === 0"
    class="no-data"
  >
    No Multiply vaults
  </div>
  <table-mobile-card
    v-for="item in items"
    v-else
    :key="item.pairKey || item.pool_address"
  >
    <div class="mobile-card-top">
      <div class="card-asset">
        <img
          :src="item.asset.icon"
          alt=""
        >
        <div class="card-asset__info">
          <div class="card-asset__info__name">
            {{ item.asset.symbol }}
          </div>
          <div class="card-asset__info__symbol">
            {{ item.asset.name }}
          </div>
        </div>
      </div>

      <div class="card-top-info">
        <div class="info-wrapper with-pill align-items-center">
          <div class="info-wrapper__title text-center">
            Multiplier
          </div>
          <div class="info-wrapper__value">

            {{ truncatePercent(item.maxMultiplier || 0, 2) }}x
          </div>
        </div>

        <div class="info-wrapper with-pill align-items-center">
          <div class="info-wrapper__title text-center">
            Details
          </div>
          <div class="info-wrapper__value">
            <i-app-info-circle @click="handleDetails(item)" />
          </div>
        </div>
      </div>
    </div>

    <div class="mobile-card-body">
      <div class="info-wrapper">
        <div class="info-wrapper__title text-end">
          APY at Multiplier
        </div>
        <div
          class="info-wrapper__value"
        >
          <j-pill-label
            :variant="item.apyAtMaxMultiplier > 0 ? 'success' : 'danger'"
            size="sm"
          >
            {{ truncatePercent(getApy(item), 2) }}%
          </j-pill-label>
        </div>
      </div>

      <div class="separator-vert" />

      <div class="info-wrapper">
        <div class="info-wrapper__title text-end">
          Net Equity
        </div>
        <div
          class="info-wrapper__value text-end"
          :class="[`multiply-table__netEquity--${item?.netEquityUsd ? (item?.netEquityUsd < 0 ? 'negative' : 'positive') : 'neutral'}`]"
        >
          <template v-if="item.netEquityUsd">
            ${{ formatPrice(item.netEquityUsd ?? 0, 2, 2) }}
          </template>
          <template v-else>
            —
          </template>
        </div>
      </div>

    </div>

    <div class="mobile-card-footer">

      <j-btn
        v-if="isUserHaveMultiply(item)"
        size="sm"
        variant="positive-outlined"
        :disabled="market.isDisabled(item.pool_address, 'withdrawLeverage', item.market!)"
        :loading="market.isLoading(item.pool_address, 'withdrawLeverage', item.market!)"
        @click="emits('dialogHandler', { item, action: 'Manage' })"
      >
        Manage
      </j-btn>
      <j-btn
        v-else
        size="sm"
        variant="positive-outlined"
        :disabled="market.isDisabled(item.pool_address, 'leverage', item.market!)"
        :loading="market.isLoading(item.pool_address, 'leverage', item.market!)"
        @click="emits('dialogHandler', { item, action: 'supply' })"
      >
        Multiply
      </j-btn>
    </div>
  </table-mobile-card>
</template>

<style lang="scss" scoped>
:deep(.table-mobile-card) {
  padding: 18px;
  border-radius: 24px;
  background:
    radial-gradient(circle at top left, rgba(24, 185, 119, 0.1), transparent 36%),
    linear-gradient(180deg, rgba(17, 24, 39, 0.96) 0%, rgba(13, 18, 31, 0.96) 100%);
  border: 1px solid $border-primary;
  box-shadow: 0 14px 38px rgba(0, 0, 0, 0.2);

  &:not(:last-child) {
    margin-bottom: 14px;
    border-bottom: 1px solid $border-primary;
    padding: 18px;
  }

  .separator-vert {
    background-color: $border-primary;
  }

  .info-wrapper__value {
    color: $text-primary;
  }

  .mobile-card-footer {
    margin-top: 4px;
  }
}

:deep(.table-mobile-card .card-asset__info__name) {
  color: $text-primary;
}

:deep(.table-mobile-card .card-asset__info__symbol) {
  color: $text-tertiary;
}

:deep(.table-mobile-card .info-wrapper__title) {
  color: $text-tertiary;
}

.apy {
  &--positive {
    color: $success;
  }
  &--negative {
    color: $danger;
  }
}

.no-data {
  color: $text-secondary;
  font-size: 12px;
  font-style: normal;
  font-weight: 400;
  line-height: 16px;
  text-align: center;
}
</style>
