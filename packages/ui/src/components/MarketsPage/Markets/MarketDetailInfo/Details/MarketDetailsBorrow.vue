<script lang="ts" setup>
import { bigintToNumber, generateExplorerLink, shortenNumber, truncatePercent } from '~/utils'

const { width } = useWindowSize()

const clientStore = useClientStore()
const marketsStore = useMarketsStore()
const market = computed(() => marketsStore.selectedMarketInfo)

const pool = computed(() => market.value?.raw)

const decimals = computed(() => clientStore.assetDecimals)

const totalBorrowed = computed(() => Number(bigintToNumber(pool.value?.total_borrowed, decimals.value)) || 0)
const totalSupplied = computed(() => {
  const supplied = Number(bigintToNumber(pool.value?.total_borrowed + pool.value?.available, decimals.value)) || 0
  const openLTV = Number(pool.value?.config.open_ltv_bps) / 10_000
  return (supplied * openLTV) || 0
})

const totalBorrowedUsd = computed(() => Number(totalBorrowed.value * pool.value?.pool_price).toFixed(2) || 0)
const totalSuppliedUsd = computed(() => Number(totalSupplied.value * pool.value?.pool_price).toFixed(2) || 0)

const borrowCap = computed(() => {
  if (!pool.value) {
    return 0
  }
  const cap = (totalBorrowed.value / totalSupplied.value) * 100
  return Number(cap || 0)
})

const reserve = computed(() => {
  if (!pool.value) {
    return 0
  }
  const reserve = Number(pool.value?.config.reserve_ratio_bps) / 100
  return Number(reserve || 0).toFixed(0)
})

const progress = computed(() => borrowCap.value.toFixed(2))
</script>

<template>
  <div class="market-details">
    <div class="market-details__title">
      Borrow Info
    </div>

    <div class="market-stats">
      <div class="market-stats__apy">
        <div class="stats-apy">
          Borrow APY
          <span>{{ market?.borrow_apy || '-' }}</span>
        </div>
        <div class="stats-params">
          <div class="stats-params__item">
            Borrow Cap:
            <span>{{ truncatePercent(borrowCap, 1) }}%</span>
          </div>
          <div class="stats-params__item">
            Reserve:
            <span>{{ reserve }}%</span>
          </div>
        </div>
      </div>

      <div class="separator-vert" />

      <market-progress
        is-progress
        :progress="progress"
        color="#FFD101"
        :cap="totalBorrowed"
        :limit="totalSupplied"
      >
        <div class="market-progress__info">
          <div class="market-progress__info__title">
            Total Borrowed
          </div>
          <div class="market-progress__info__data">
            {{ shortenNumber(totalBorrowed) }} / {{ shortenNumber(totalSupplied) }}

            <span>${{ shortenNumber(Number(totalBorrowedUsd)) }} / ${{ shortenNumber(Number(totalSuppliedUsd)) }}</span>
          </div>
        </div>
        <a
          v-if="width <= 650"
          :href="generateExplorerLink(pool?.pool_address, 'contract')"
          target="_blank"
          class="market-penalty"
        >
          View contract

          <i-app-export-icon color="#111" />
        </a>
      </market-progress>

      <div class="separator-vert hide-xs" />
      <a
        :href="generateExplorerLink(pool?.pool_address, 'contract')"
        target="_blank"
        class="market-penalty justify-content-center hide-xs"
      >
        View contract

        <i-app-export-icon color="#111" />
      </a>
    </div>

    <div class="separator" />

    <market-history-chart-borrow />
  </div>
</template>

<style lang="scss" scoped>
.market-penalty {
  gap: 10px;
  text-decoration: none;
}
</style>
