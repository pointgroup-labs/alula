<script lang="ts" setup>
import borrowingIcon from '~/assets/img/icons/percentage-square-icon.svg?raw'
import { bigintToNumber, formatPrice, shortenNumber } from '~/utils'

const marketsStore = useMarketsStore()
const clientStore = useClientStore()
const decimals = toRef(clientStore, 'assetDecimals')

const loading = computed(() => marketsStore.state.loading)
const pools = computed(() => marketsStore.state.pollsData)

const poolsInfo = computed(() => {
  return pools.value?.reduce((acc, pool) => {
    const borrowed = Number(bigintToNumber(pool.total_borrowed, decimals.value))
    const totalSupplied = pool.available + pool.total_borrowed + pool.total_collateral
    const supplied = Number(bigintToNumber(totalSupplied, decimals.value)) * Number(pool.pool_price)
    acc.total_borrowed += borrowed * Number(pool.pool_price)
    acc.total_collateral += supplied
    return acc
  }, { total_collateral: 0, total_borrowed: 0 })
})

const marketSize = computed(() => shortenNumber(poolsInfo.value.total_collateral + poolsInfo.value.total_borrowed))

function normalizeAmount(price: number) {
  return price < 1_000_000 ? formatPrice(price, 2, 2) : shortenNumber(price)
}
</script>

<template>
  <div class="market-info">
    <total-card
      title="Total Collateral"
      :body="`$${normalizeAmount(poolsInfo.total_collateral)}`"
      :loading="loading"
    />
    <total-card
      title="Total Borrowing"
      :body="`$${normalizeAmount(poolsInfo.total_borrowed)}`"
      color="#111"
      bg-color="#FFD101"
      :icon="borrowingIcon"
      :loading="loading"
    />

    <div class="total-card market-size">
      <div class="total-card__info">
        <div class="total-card__title">
          Total market size
        </div>
        <div class="total-card__body">
          <j-skeleton
            v-if="loading"
            height="28"
            full-width
          />
          <template v-else>
            {{ marketSize }}
          </template>
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
    text-align: right;

    &::before {
      display: none;
    }
  }
}
</style>
