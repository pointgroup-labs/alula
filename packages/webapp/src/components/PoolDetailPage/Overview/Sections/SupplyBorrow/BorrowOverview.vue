<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { bpsToNumber } from '@alula/client-sdk'
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
  const utilRatioLimit = bpsToNumber(Number(pool.value?.config.health_config.utilization_ratio_limit_bps || 0)) * 100
  const withdrawFee = bpsToNumber(Number(pool.value?.config.fee_config.withdraw_fee_bps)) * 100
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
  const utilRatio = bpsToNumber(Number(pool.value?.config.health_config.utilization_ratio_limit_bps))
  return (supplied * utilRatio) || 0
})

const totalBorrowedUsd = computed(() => Number(totalBorrowed.value * Number(selectedPool.value?.price || 0)) || 0)

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
  <div class="pool-card stat-card stat-card--small">
    <div class="stat-card__header">
      <div
        class="header-icon"
        :style="{ '--icon-bg': 'rgba(99, 102, 241, 10%)', '--icon-color': '#6366F1' }"
      >
        <i-metrics-arrow-down />
      </div>

      <h3 class="pool-card-title">
        Borrow
      </h3>

      <div
        class="pool-card-pill"
        :style="{ '--color': '#f43f5e', '--background-color': 'rgba(244, 63, 94, 10%)' }"
      >
        APY {{ selectedPool.borrow_apy }}
      </div>
    </div>

    <div class="stat-card__body">
      <market-progress
        is-progress
        :progress="Number(progress).toFixed(1)"
        color="#6366F1"
      >
        <div class="market-progress__info">
          <div class="market-progress__info__title">
            Total Borrow
          </div>
          <div class="market-progress__info__data">
            {{ shortenNumber(totalBorrowed ?? 0) }}
            <span>/ {{ shortenNumber(totalSupplied ?? 0) }}</span>
          </div>
        </div>
      </market-progress>

      <div class="detail-list">
        <div class="detail-list__item">
          <div class="detail-list__item__label">
            Utilization Rate
          </div>
          <div class="detail-list__item__value">
            {{ detailCardsData.utilRate }}
          </div>
        </div>

        <div class="detail-list__item">
          <div class="detail-list__item__label">
            Take Rate
          </div>
          <div class="detail-list__item__value">
            {{ reserve }}%
          </div>
        </div>

        <div class="detail-list__item">
          <div class="detail-list__item__label">
            Borrowed
          </div>
          <div class="detail-list__item__value">
            {{ formatCompactUSD(totalBorrowedUsd) }}
          </div>
        </div>

        <div class="detail-list__item">
          <div class="detail-list__item__label">
            Util. Rate Limit
          </div>
          <div class="detail-list__item__value">
            {{ detailCardsData.utilRatioLimit }}%
          </div>
        </div>

        <div class="detail-list__item">
          <div class="detail-list__item__label">
            Withdraw Fee
          </div>
          <div
            class="detail-list__item__value"
            style="color: #10b981;"
          >
            {{ detailCardsData.withdrawFee }}%
          </div>
        </div>

        <div class="detail-list__item">
          <div class="detail-list__item__label">
            View contract
          </div>
          <div class="detail-list__item__value">
            <a
              :href="generateExplorerLink(pool?.pool_address, 'contract')"
              target="_blank"
            >
              <i-app-export-icon class="export-icon" />
            </a>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
