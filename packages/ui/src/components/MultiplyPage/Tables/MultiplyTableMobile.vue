<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'

const {
  items,
} = defineProps<{
  items: MultiplyTableItem[]
}>()

const emits = defineEmits(['dialogHandler'])

const market = useMarket()

const userStore = useUserStore()
const obligation = computed(() => userStore.userObligation)

function checkIsHaveMultiply(pool: MultiplyTableItem) {
  const deposits = obligation.value?.deposits || []
  const borrows = obligation.value?.borrows || []
  if (deposits.length === 0 || borrows.length === 0) {
    return false
  }
  const depositPoolAddress = pool.depositPool.pool_address
  const borrowPoolAddress = pool.borrowPool.pool_address

  const isDeposits = deposits.some((deposit: any) => deposit.includes(depositPoolAddress))
  const isBorrows = borrows.some((deposit: any) => deposit.includes(borrowPoolAddress))
  return isDeposits && isBorrows
}
</script>

<template>
  <div
    v-if="items?.length === 0"
    class="no-table-data"
  >
    No Markets
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
        :disabled="market.isDisabled(item.pool_address, 'leverage')"
        :loading="market.isLoading(item.pool_address, 'leverage')"
        @click="emits('dialogHandler', { item, action: 'supply' })"
      >
        Multiply
      </j-btn>
      <j-btn
        v-if="checkIsHaveMultiply(item)"
        size="sm"
        variant="accent"
        pill
        icon-right
        :disabled="market.isDisabled(item.pool_address, 'withdrawLeverage')"
        :loading="market.isLoading(item.pool_address, 'withdrawLeverage')"
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
