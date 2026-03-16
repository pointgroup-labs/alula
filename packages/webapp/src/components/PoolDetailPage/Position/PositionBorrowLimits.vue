<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const {
  selectedPool,
  borrowLimit = 0,
  borrowedAmount = 0,
} = defineProps<{
  availableToBorrow: number
  borrowedAmount?: number
  borrowLimit?: number
  selectedPool?: MarketTableItem
}>()

const borrowLimitUsd = computed(() => borrowLimit * Number(selectedPool?.price ?? 0))
const borrowedAmountUsd = computed(() => borrowedAmount * Number(selectedPool?.price ?? 0))
</script>

<template>
  <div class="position-panel stat-card stat-card--small">
    <div class="position-panel__eyebrow">
      Borrow Limits
    </div>

    <div class="metric-list">
      <div class="metric-list__item">
        <div class="metric-list__label">
          Borrow Limit
        </div>
        <div class="metric-list__value metric-list__value--stacked">
          <span>{{ formatPrice(borrowLimit, 0, 5) }} {{ selectedPool?.asset.symbol }}</span>
          <small class="metric-list__sub-value">${{ formatPrice(borrowLimitUsd, 0, 2) }}</small>
        </div>
      </div>

      <div class="metric-list__item">
        <div class="metric-list__label">
          Borrowed
        </div>
        <div class="metric-list__value  metric-list__value--stacked">
          <span>{{ formatPrice(borrowedAmount, 0, 5) }} {{ selectedPool?.asset.symbol }}</span>
          <small class="metric-list__sub-value">${{formatPrice(borrowedAmountUsd, 0, 2)}}</small>
        </div>
      </div>

      <div class="separator" />

      <div class="metric-list__item">
        <div class="metric-list__label">
          Available to Borrow
        </div>
        <div class="metric-list__value borrow-color">
          {{ formatPrice(availableToBorrow, 0, 5) }} {{ selectedPool?.asset.symbol }}
        </div>
      </div>
    </div>
  </div>
</template>
