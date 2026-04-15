<script lang="ts" setup>
import type { MultiplyAccountTableItem, MultiplyTableItem } from '~/types/table'

const {
  items,
  showInAccounts = false,
} = defineProps<{
  items: MultiplyTableItem[] | MultiplyAccountTableItem[]
  showInAccounts?: boolean
}>()

const emits = defineEmits(['dialogHandler'])

const market = useMarketActions()

const userStore = useUserStore()

const labelsByPage = computed(() => ({
  supply: showInAccounts ? 'Borrowed' : 'Supplied',
  liquidity: showInAccounts ? 'Deposited' : 'Liquidity',
}))

function isUserHaveMultiply(poolAddress: string, market: string) {
  return checkIsHaveMultiply(
    userStore.state.multiplyObligations,
    items,
    poolAddress,
    market,
  )
}

function getLiquidity(data: MultiplyTableItem | MultiplyAccountTableItem) {
  let amount = 0
  if (showInAccounts && 'deposited' in data) {
    amount = data.deposited
  } else if (!showInAccounts && 'liquidity' in data) {
    amount = data.liquidity
  }
  return shortenNumber(amount || 0)
}

function getSupply(data: MultiplyTableItem | MultiplyAccountTableItem) {
  let amount = 0
  if (showInAccounts && 'borrowed' in data) {
    amount = data.borrowed
  } else if (!showInAccounts && 'supplied' in data) {
    amount = data.supplied
  }
  return shortenNumber(amount || 0)
}

function getApy(data: MultiplyTableItem | MultiplyAccountTableItem) {
  if ('apyAtMaxMultiplier' in data) {
    return data.apyAtMaxMultiplier || 0
  }

  return data.maxAPY || 0
}

function getHealthFactor(data: MultiplyTableItem | MultiplyAccountTableItem) {
  if (showInAccounts && 'healthFactor' in data) {
    return data.healthFactor || 0
  }

  return null
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
        <div class="info-wrapper with-pill">
          <div class="info-wrapper__title text-center">
            Multiplier
          </div>
          <div class="info-wrapper__value">
            <j-pill-label
              variant="success"
              size="sm"
            >
              {{ truncatePercent(item.multiplier || 0, 2) }}x
            </j-pill-label>
          </div>
        </div>
      </div>
    </div>

    <div class="mobile-card-body">
      <div class="info-wrapper">
        <div class="info-wrapper__title text-end">
          {{ labelsByPage.liquidity }}
        </div>
        <div class="info-wrapper__value ">
          <span>{{ getLiquidity(item) }}</span> {{ item.borrowAsset.symbol }}
        </div>
      </div>

      <div class="separator-vert" />

      <div class="info-wrapper">
        <div class="info-wrapper__title text-end">
          {{ labelsByPage.supply }}
        </div>
        <div class="info-wrapper__value ">
          <span>{{ getSupply(item) }}</span> {{ item.asset.symbol }}
        </div>
      </div>

      <div class="separator-vert" />

      <div class="info-wrapper">
        <div class="info-wrapper__title text-end">
          APY at Max
        </div>
        <div class="info-wrapper__value apy-success">
          {{ truncatePercent(getApy(item), 2) }}%
        </div>
      </div>

      <template v-if="showInAccounts && getHealthFactor(item) !== null">
        <div class="separator-vert" />

        <div class="info-wrapper">
          <div class="info-wrapper__title text-end">
            Health Factor
          </div>
          <div class="info-wrapper__value hf-value">
            <div
              class="hf-indicator"
              :style="{
                '--indicator-width': `${Math.min(Math.max(((getHealthFactor(item) || 0) - 1) * 100, 0), 100)}%`,
                '--indicator-color': healthFactorColor(getHealthFactor(item) || 0),
              }"
            />
            <span
              :style="{ color: healthFactorColor(getHealthFactor(item) || 0) }"
              class="text-num hf-percent"
            >
              {{ truncatePercent(getHealthFactor(item) || 0, 2) }}
            </span>
          </div>
        </div>
      </template>
    </div>

    <div class="mobile-card-footer">
      <j-btn
        size="sm"
        variant="cyan"
        :disabled="market.isDisabled(item.pool_address, 'leverage', item.market!)"
        :loading="market.isLoading(item.pool_address, 'leverage', item.market!)"
        @click="emits('dialogHandler', { item, action: 'supply' })"
      >
        Multiply
      </j-btn>
      <j-btn
        v-if="isUserHaveMultiply(item.pool_address, String(item.market))"
        size="sm"
        variant="accent"
        :disabled="market.isDisabled(item.pool_address, 'withdrawLeverage', item.market!)"
        :loading="market.isLoading(item.pool_address, 'withdrawLeverage', item.market!)"
        @click="emits('dialogHandler', { item, action: 'withdraw' })"
      >
        Withdraw
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

.apy-success {
  color: $success;
}

.hf-value {
  display: flex;
  align-items: center;
  gap: 4px;
}

.hf-indicator {
  position: relative;
  width: 50px;
  height: 4px;
  border-radius: $radius-lg;
  background-color: color-mix(in oklab, $border-primary 70%, transparent);
  overflow: hidden;
  flex-shrink: 0;

  &::after {
    content: '';
    position: absolute;
    right: 0;
    top: 0;
    height: 100%;
    width: var(--indicator-width, 0%);
    border-radius: $radius-lg;
    background-color: var(--indicator-color, #{$success});
    transition:
      width 0.3s ease,
      background-color 0.3s ease;
  }
}

.hf-percent {
  font-size: 12px;
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
