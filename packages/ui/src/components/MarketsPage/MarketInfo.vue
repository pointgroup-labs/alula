<script lang="ts" setup>
import borrowingIcon from '~/assets/img/icons/percentage-square-icon.svg?raw'
import { bigintToNumber, formatPrice, shortenNumber } from '~/utils'

const marketsStore = useMarketsStore()
const clientStore = useClientStore()
const decimals = toRef(clientStore, 'assetDecimals')
const pools = computed(() => marketsStore.state.pollsData)
const poolsInfo = computed(() => {
  return pools.value?.reduce((acc, pool) => {
    const borrowed = Number(bigintToNumber(pool.total_borrowed, decimals.value))
    const collateral = Number(bigintToNumber(pool.total_collateral, decimals.value))
    acc.total_borrowed += borrowed * Number(pool.pool_price)
    acc.total_collateral += collateral * Number(pool.pool_price)
    return acc
  }, { total_collateral: 0, total_borrowed: 0 })
})

function normalizeAmount(price: number) {
  return price < 1_000_000 ? formatPrice(price, 2, 2) : shortenNumber(price)
}
</script>

<template>
  <div class="market-info">
    <total-card
      title="Total Collateral"
      :body="`$${normalizeAmount(poolsInfo.total_collateral)}`"
    />
    <total-card
      title="Total Borrowing"
      :body="`$${normalizeAmount(poolsInfo.total_borrowed)}`"
      color="#111"
      bg-color="#FFD101"
      :icon="borrowingIcon"
    />

    <div class="total-card market-size">
      <div class="total-card__info">
        <div class="total-card__title">
          Total market size
        </div>
        <div class="total-card__body">
          $23,123B
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.market-info {
  display: flex;
  align-items: center;
  gap: $spacing-24;

  .market-size {
    color: $dark;
    background-color: transparent;
    justify-content: flex-end;
    margin-left: auto;

    &::before {
      display: none;
    }
  }
}
</style>
