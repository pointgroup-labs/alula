<script lang="ts" setup>
import borrowingIcon from '~/assets/img/icons/percentage-square-icon.svg?raw'
import { formatPrice, shortenNumber } from '~/utils'

const marketsStore = useMarketsStore()

const loading = computed(() => marketsStore.state.loading)

const poolsInfo = computed(() => {
  return Object.values(marketsStore.state.markets)?.reduce((acc, { marketState }) => {
    const assetDecimals = marketsStore.assetDecimals
    const oraclePriceDecimale = marketState.oracle_price_decimals

    for (const data of marketState.pools_data) {
      const price = Number(bigintToNumber(data.oracle_asset_price, oraclePriceDecimale))
      const totalSupplied = data.pool.total_available + data.pool.total_borrowed + data.pool.total_collateral
      const supplied = Number(bigintToNumber(totalSupplied, assetDecimals)) * price
      const borrowed = Number(bigintToNumber(data.pool.total_borrowed, assetDecimals)) * price
      acc.total_borrowed += borrowed
      acc.total_collateral += supplied
    }
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
    <template v-if="loading">
      <market-info-skeleton
        v-for="i in 2"
        :key="i"
      />
    </template>
    <template v-else>
      <total-card
        title="Total Collateral"
        :body="`$${normalizeAmount(poolsInfo.total_collateral)}`"
      />
      <total-card
        title="Total Borrow"
        :body="`$${normalizeAmount(poolsInfo.total_borrowed)}`"
        color="#111"
        bg-color="#FFD101"
        :icon="borrowingIcon"
      />
    </template>

    <div class="total-card market-size">
      <div class="total-card__info">
        <div class="total-card__title">
          Total Market Size
        </div>
        <div class="total-card__body">
          <j-skeleton
            v-if="loading"
            height="28"
            style="border-radius: 6px;"
            full-width
          />
          <template v-else>
            ${{ marketSize }}
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

body.body--dark {
  .market-size {
    .total-card__info {
      color: $neutral-5;
    }
  }
}
</style>
