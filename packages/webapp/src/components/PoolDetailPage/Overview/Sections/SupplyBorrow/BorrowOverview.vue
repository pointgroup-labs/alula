<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { bpsToNumber } from '@alula/client-sdk'
import { bigintToNumber } from '~/utils'

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
const totalSupplied = computed(() => Number(bigintToNumber(selectedPool.value?.raw?.total_supply, selectedPool.value?.assetDecimals)) || 0)

const maxBorrow = computed(() => {
  if (!pool.value) {
    return 0
  }

  const utilRatioLimit = bpsToNumber(Number(pool.value.config.health_config.utilization_ratio_limit_bps || 0))
  return Number(totalSupplied.value * utilRatioLimit || 0)
})

const availableBorrow = computed(() => maxBorrow.value - totalBorrowed.value)

const totalBorrowedUsd = computed(() => Number(totalBorrowed.value * Number(selectedPool.value?.price || 0)) || 0)
const availableBorrowUsd = computed(() => Number(availableBorrow.value * Number(selectedPool.value?.price || 0)) || 0)

const reserve = computed(() => {
  if (!pool.value) {
    return 0
  }
  const reserve = Number(pool.value?.config.fee_config.take_rate_bps) / 100
  return Number(reserve || 0).toFixed(0)
})
const progress = computed(() => {
  if (!maxBorrow.value) {
    return '0.00'
  }

  return ((totalBorrowed.value / maxBorrow.value) * 100).toFixed(2)
})
</script>

<template>
  <div class="pool-card stat-card stat-card--small">
    <div class="stat-card__header">
      <h3 class="pool-card-title">
        Borrow
      </h3>

      <j-pill-label
        size="sm"
        variant="indigo"
        style="margin-left: auto;"
      >
        Borrow rate {{ selectedPool.borrow_apy }}
      </j-pill-label>
    </div>

    <div class="stat-card__body">
      <market-progress
        is-progress
        :progress="Number(progress).toFixed(1)"
        color="#8a8df4"
      >
        <div class="progress-content">
          <div class="market-progress__info">
            <div class="market-progress__info__title">
              Borrowed
            </div>
            <div class="market-progress__info__data">
              {{ shortenNumber(totalBorrowed, 1, 1) }}
              <span> / {{ formatCompactUSD(totalBorrowedUsd, 1, 1) }}</span>
            </div>
          </div>

          <div class="separator-vert" />

          <div class="market-progress__info">
            <div class="market-progress__info__title">
              Available
            </div>
            <div class="market-progress__info__data">
              {{ shortenNumber(availableBorrow, 1, 1) }}
              <span>/ {{ formatCompactUSD(availableBorrowUsd, 1, 1) }}</span>
            </div>
          </div>
        </div>
      </market-progress>

      <div class="detail-list">
        <div class="detail-list__item">
          <div class="detail-list__item__label">
            Utilization Rate

            <info-tooltip>
              Percentage of supplied assets currently borrowed.
              <br>
              Higher utilization leads to higher borrow rates.
            </info-tooltip>
          </div>
          <div class="detail-list__item__value">
            {{ detailCardsData.utilRate }}
          </div>
        </div>

        <div class="detail-list__item">
          <div class="detail-list__item__label">
            Max Borrow Utilization
          </div>
          <div class="detail-list__item__value">
            {{ detailCardsData.utilRatioLimit }}%
          </div>
        </div>

        <div class="detail-list__item">
          <div class="detail-list__item__label">
            Reserve Factor

            <info-tooltip>
              Percentage of borrower interest taken as a protocol fee.
            </info-tooltip>
          </div>
          <div class="detail-list__item__value">
            {{ reserve }}%
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
      </div>
    </div>
  </div>
</template>
