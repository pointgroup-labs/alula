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
    :key="item.pool_address"
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
          APY
        </div>
        <div class="info-wrapper__value apy-success">
          {{ truncatePercent(item.maxAPY || 0, 2) }}%
        </div>
      </div>
    </div>

    <div class="mobile-card-footer">
      <j-btn
        size="sm"
        variant="blue"
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
.apy-success {
  color: $success;
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
