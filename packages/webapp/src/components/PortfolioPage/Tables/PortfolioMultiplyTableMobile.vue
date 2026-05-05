<script lang="ts" setup>
import type { MultiplyPortfolioTableItem } from '~/types/table'

const {
  items,
  showInAccounts = false,
} = defineProps<{
  items: MultiplyPortfolioTableItem[]
  showInAccounts?: boolean
}>()

const emits = defineEmits(['dialogHandler'])

const market = useMarketActions()

const labelsByPage = computed(() => ({
  supply: showInAccounts ? 'Borrowed' : 'Supplied',
  liquidity: showInAccounts ? 'Supplied' : 'Liquidity',
}))

function getLiquidity(data: MultiplyPortfolioTableItem) {
  const amount = data.deposited
  return shortenNumber(amount || 0)
}

function getSupply(data: MultiplyPortfolioTableItem) {
  const amount = data.borrowed
  return shortenNumber(amount || 0)
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
          Health Factor
        </div>
        <div
          class="info-wrapper__value text-end"
          :style="{ color: healthFactorColor(item.healthFactor) }"
        >
          {{ truncatePercent(item.healthFactor || 0, 2) }}%
        </div>
      </div>
    </div>

    <div class="mobile-card-footer">
      <j-btn
        size="sm"
        variant="outlined-accent"
        :disabled="market.isDisabled(item.pool_address, 'withdrawLeverage', item.market!)"
        :loading="market.isLoading(item.pool_address, 'withdrawLeverage', item.market!)"
        @click="emits('dialogHandler', { item, action: 'withdraw' })"
      >
        Close
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

.no-data {
  color: $text-tertiary;
  font-size: 12px;
  font-style: normal;
  font-weight: 400;
  line-height: 16px;
  text-align: center;

  @media (max-width: $breakpoint-sm) {
    min-height: 116px;
    max-height: 200px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
  }
}
</style>
