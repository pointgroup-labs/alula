<script lang="ts" setup>
const {
  selectedPool,
  suppliedAmount,
  suppliedUsd,
  collateralOnly,
  collateralOnlyUsd,
  currentPoolInterestBearingAmount,
  currentPoolInterestBearingUsd,
  currentPoolYearlyYieldUsd,
} = useMyPosition()
</script>

<template>
  <div class="position-panel stat-card stat-card--small">
    <div class="position-panel__eyebrow">
      Supply Info
    </div>

    <div class="metric-list">
      <div class="metric-list__item">
        <div class="metric-list__label">
          Total Supplied
        </div>
        <div class="metric-list__value metric-list__value--stacked">
          <span>{{ formatPrice(suppliedAmount, 0, 5) }} {{ selectedPool?.asset.symbol }}</span>
          <small>${{ formatPrice(suppliedUsd, 0, 2) }}</small>
        </div>
      </div>

      <div class="metric-list__item">
        <div class="metric-list__label">
          Earning Balance
        </div>
        <div class="metric-list__value metric-list__value--stacked supply-color">
          <span>{{ formatPrice(currentPoolInterestBearingAmount, 0, 5) }} {{ selectedPool?.asset.symbol }}</span>
          <small>${{ formatPrice(currentPoolInterestBearingUsd, 0, 2) }}</small>
        </div>
      </div>

      <div class="metric-list__item">
        <div class="metric-list__label">
          Collateral Only
        </div>
        <div class="metric-list__value metric-list__value--stacked">
          <span>{{ formatPrice(collateralOnly, 0, 5) }} {{ selectedPool?.asset.symbol }}</span>
          <small>${{ formatPrice(collateralOnlyUsd, 0, 2) }}</small>
        </div>
      </div>

      <div class="separator" />

      <div class="metric-list__item">
        <div class="metric-list__label">
          Est. Yearly Yield
        </div>
        <div class="metric-list__value metric-list__value--stacked supply-color">
          <span>${{ formatPrice(currentPoolYearlyYieldUsd, 0, 2) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
