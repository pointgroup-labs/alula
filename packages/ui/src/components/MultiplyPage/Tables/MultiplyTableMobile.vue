<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'

const {
  items,
} = defineProps<{
  items: MultiplyTableItem[]
}>()

const emits = defineEmits(['dialogHandler'])

const market = useMarketActions()

const userStore = useUserStore()

function isUserHaveMultiply(poolAddress: string, market: string) {
  return checkIsHaveMultiply(
    userStore.state.multiplyObligations,
    items,
    poolAddress,
    market,
  )
}
</script>

<template>
  <div
    v-if="items?.length === 0"
    class="no-table-data"
  >
    No Pools
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
              color="#111"
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
          Liquidity
        </div>
        <div class="info-wrapper__value ">
          {{ shortenNumber(item.liquidity || 0) }} {{ item.borrowAsset.symbol }}
        </div>
      </div>

      <div class="separator-vert" />

      <div class="info-wrapper">
        <div class="info-wrapper__title text-end">
          Supplied
        </div>
        <div class="info-wrapper__value ">
          {{ shortenNumber(item.supplied.toFixed(2) || 0) }} {{ item.asset.symbol }}
        </div>
      </div>

      <div class="separator-vert" />

      <div class="info-wrapper">
        <div class="info-wrapper__title text-end">
          Max APY
        </div>
        <div class="info-wrapper__value apy-success">
          {{ truncatePercent(item.maxAPY || 0, 2) }}%
        </div>
      </div>
    </div>

    <div class="mobile-card-footer">
      <j-btn
        size="sm"
        pill
        icon-right
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
        pill
        icon-right
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
</style>
