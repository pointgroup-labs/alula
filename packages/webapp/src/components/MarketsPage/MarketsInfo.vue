<script lang="ts" setup>
import { formatPrice } from '~/utils'

const GUIDE_LINK = 'https://docs.alula.finance/guides/'

const marketsStore = useMarketsStore()

const loading = computed(() => marketsStore.state.loading)

const marketsInfo = computed(() => {
  return Object.values(marketsStore.state.markets)?.reduce((acc, { marketState }) => {
    const oraclePriceDecimale = marketState?.oracle_price_decimals ?? 0

    for (const data of marketState?.pools_data) {
      const assetDecimals = data.pool.token_decimals
      const price = Number(bigintToNumber(data.oracle_asset_price, oraclePriceDecimale))
      const totalSupplied = data.total_supply + data.pool.total_collateral
      const supplied = Number(bigintToNumber(totalSupplied, assetDecimals)) * price
      const borrowed = Number(bigintToNumber(data.pool.total_borrowed, assetDecimals)) * price
      acc.total_borrowed += borrowed
      acc.total_collateral += supplied
    }
    return acc
  }, { total_collateral: 0, total_borrowed: 0 })
})
</script>

<template>
  <div class="markets-info">
    <div class="markets-info__content">
      <h1>Markets</h1>

      <div class="markets-info__content__desc">
        Earn yield or borrow against your assets.
        <a
          :href="GUIDE_LINK"
          target="_blank"
          rel="noopener noreferrer nofollow"
        >How it works</a>
      </div>
    </div>
    <div class="d-flex gap-2">
      <template v-if="loading && marketsInfo.total_collateral === 0">
        <market-info-skeleton
          v-for="i in 2"
          :key="i"
        />
      </template>
      <template v-else>
        <total-card
          title="Total Supply"
          :body="`$${formatPrice(marketsInfo.total_collateral, 0, 0)}`"
          :loading="loading"
        />
        <total-card
          title="Total Borrow"
          :body="`$${formatPrice(marketsInfo.total_borrowed, 0, 0)}`"
          :loading="loading"
        />
      </template>
    </div>
  </div>
</template>

<style lang="scss">
.markets-info {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 8px;

  &__content {
    display: flex;
    flex-direction: column;
    gap: 4px;

    @media (max-width: $breakpoint-sm) {
      display: none;
    }

    h1 {
      color: $text-primary;
      font-size: 32px;
      margin: 0;
    }

    &__desc {
      color: $text-tertiary;
      font-size: 14px;
    }

    a {
      text-decoration: none;
      color: $text-brand;
    }
  }
}
</style>
