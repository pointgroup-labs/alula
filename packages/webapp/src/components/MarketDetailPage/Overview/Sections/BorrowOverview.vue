<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { bigintToNumber } from '~/utils'

const { generateExplorerLink } = useExplorerLink()

const selectedPool = inject('selectedPool') as Ref<MarketTableItem>

const pool = computed(() => selectedPool.value?.raw?.pool)

const detailCardsData = computed(() => {
  if (!pool.value) {
    return {
      borrowAPY: '0%',
      utilRate: 0,
      utilRatioLimit: 0,
      withdrawFee: 0,
    }
  }

  const borrowAPY = selectedPool.value?.borrow_apy ?? '0%'
  const utilRatioLimit = Number(pool.value?.config.health_config.utilization_ratio_limit_bps || 0) / 100
  const withdrawFee = Number(pool.value?.config.fee_config.withdraw_fee_bps) / 100
  return {
    utilRate: selectedPool.value?.utilization_rate,
    utilRatioLimit: utilRatioLimit.toFixed(0),
    withdrawFee: truncatePercent(withdrawFee, 2),
    borrowAPY,
  }
})

const totalBorrowed = computed(() => Number(bigintToNumber(pool.value?.total_borrowed, selectedPool.value?.assetDecimals)) || 0)
const totalSupplied = computed(() => {
  const supplied = Number(bigintToNumber(selectedPool.value?.raw?.total_supply, selectedPool.value?.assetDecimals)) || 0
  const utilRatio = Number(pool.value?.config.health_config.utilization_ratio_limit_bps) / 10_000
  return (supplied * utilRatio) || 0
})

const totalBorrowedUsd = computed(() => Number(totalBorrowed.value * Number(selectedPool.value?.price || 0)).toFixed(2) || 0)
const totalSuppliedUsd = computed(() => Number(totalSupplied.value * Number(selectedPool.value?.price || 0)).toFixed(2) || 0)

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
  const reserve = Number(pool.value?.config.fee_config.take_rate_bps) / 100
  return Number(reserve || 0).toFixed(0)
})
const progress = computed(() => borrowCap.value.toFixed(2))
</script>

<template>
  <section
    id="borrow"
  >
    <div class="stat-card">
      <market-history-chart-borrow />
    </div>
    <div class="market-stats-cards">
      <div class="stat-card">
        <div class="stat-title">
          <svg
            class="icon-supply"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
          >
            <path
              d="M12 16l-6-6h4V2h4v8h4l-6 6z"
              fill="#ffb726"
            />
            <rect
              x="4"
              y="20"
              width="16"
              height="2"
              fill="#ffb726"
            />
          </svg>
          Borrow
        </div>
        <market-progress
          is-progress
          :progress="progress"
          color="#ffb726"
          details-color="#ffb726"
          :cap="totalBorrowed"
          :symbol="selectedPool?.asset?.symbol"
          :limit="totalSupplied"
        >
          <div class="market-progress__info">
            <div class="market-progress__info__title">
              Total Borrow
            </div>
            <div class="market-progress__info__data">
              {{ shortenNumber(totalBorrowed) }} / {{ shortenNumber(totalSupplied) }}

              <span>${{ shortenNumber(Number(totalBorrowedUsd)) }} / ${{ shortenNumber(Number(totalSuppliedUsd)) }}</span>
            </div>
          </div>
        </market-progress>
      </div>
      <div class="cards-list">
        <market-info-card>
          <div class="info-value">
            Util. Rate
          </div>
          <div class="info-label">
            {{ detailCardsData.utilRate }}
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Take Rate
          </div>
          <div class="info-label">
            {{ reserve }}%
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Borrow APY
          </div>
          <div class="info-label warning">
            {{ detailCardsData.borrowAPY }}
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Util. Rate Limit
          </div>
          <div class="info-label">
            {{ detailCardsData.utilRatioLimit }}%
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Withdraw Fee
          </div>
          <div class="info-label">
            {{ detailCardsData.withdrawFee }}%
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            View contract
          </div>
          <div class="info-label">
            <a
              :href="generateExplorerLink(pool?.pool_address, 'contract')"
              target="_blank"
              class="market-penalty"
            >
              <i-app-export-icon color="#111" />
            </a>
          </div>
        </market-info-card>
      </div>
    </div>
  </section>
</template>
