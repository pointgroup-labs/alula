<script lang="ts" setup>
import { formatPrice } from '~/utils'

const GUIDE_LINK = 'https://docs.alula.finance/guides/multiply/'

const marketsStore = useMarketsStore()

const loading = computed(() => marketsStore.state.loading)

const multiplyStore = useMultiplyStore()
const vaults = computed(() => multiplyStore.vaults)

const marketsInfo = computed(() => {
  return vaults.value?.reduce((acc, el) => {
    const depositPool = el.depositPoolData
    const assetDecimals = depositPool.pool.token_decimals
    const price = Number(el.price || 0)
    const totalSupplied = depositPool.total_supply + depositPool.pool.total_collateral
    const supplied = Number(bigintToNumber(totalSupplied, assetDecimals)) * price
    acc.supplied += supplied
    acc.positions += 1
    return acc
  }, { supplied: 0, positions: 0 })
})
</script>

<template>
  <div class="markets-info">
    <div class="markets-info__content">
      <h1>Multiply</h1>

      <div class="markets-info__content__desc">
        Earn more with 'looping' strategies. Open and close multiplied positions with one click.
        <a
          :href="GUIDE_LINK"
          target="_blank"
          rel="noopener noreferrer nofollow"
        >How it works</a>
      </div>
    </div>
    <div class="d-flex gap-2">
      <template v-if="loading && marketsInfo.supplied === 0">
        <market-info-skeleton
          v-for="i in 2"
          :key="i"
        />
      </template>
      <template v-else>
        <total-card
          title="Total TVL"
          :body="`$${formatPrice(marketsInfo.supplied, 0, 0)}`"
          :loading="loading"
        />
        <total-card
          title="Active Strategies"
          :body="String(marketsInfo.positions)"
          :loading="loading"
        />
      </template>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.markets-info {
  a {
    color: $text-success;
  }
}
</style>
