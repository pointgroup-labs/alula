<script lang="ts" setup>
const {
  currentLtv,
  liquidationLtv,
  liquidationBufferPercent,
  liquidationBufferUsd,
  healthFactor,
  liquidationPrice,
} = useMyPosition()
</script>

<template>
  <div class="position-panel stat-card stat-card--small">
    <div class="position-panel__eyebrow">
      Risk Metrics
    </div>

    <div class="metric-list">
      <div class="metric-list__item">
        <div class="metric-list__label">
          Current LTV
        </div>
        <div class="metric-list__value">
          {{ truncatePercent(currentLtv, 2) }}%
        </div>
      </div>

      <div class="metric-list__item">
        <div class="metric-list__label">
          Liquidation LTV
        </div>
        <div class="metric-list__value">
          {{ truncatePercent(liquidationLtv, 2) }}%
        </div>
      </div>

      <div class="metric-list__item">
        <div class="metric-list__label">
          Liquidation Buffer
        </div>
        <div class="metric-list__value metric-list__value--stacked">
          <span>{{ truncatePercent(liquidationBufferPercent, 2) }}%</span>
          <small v-if="healthFactor">{{ formatCompactUSD(liquidationBufferUsd, 2, 2) }} until liquidation</small>
        </div>
      </div>

      <div class="separator" />

      <div class="metric-list__item">
        <div class="metric-list__label">
          Liquidation Price
        </div>
        <div class="metric-list__value">
          {{ liquidationPrice !== null ? formatCompactUSD(liquidationPrice, 2, 2) : '—' }}
        </div>
      </div>
    </div>
  </div>
</template>
