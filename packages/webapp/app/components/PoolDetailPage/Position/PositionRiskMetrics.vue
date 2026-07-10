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
          <info-tooltip>
            The percentage of your collateral that is currently borrowed.
            <br>
            Higher LTV means higher risk. If it gets too close to the liquidation threshold, your position can be liquidated.
          </info-tooltip>
        </div>
        <div class="metric-list__value">
          {{ truncatePercent(currentLtv, 2) }}%
        </div>
      </div>

      <div class="metric-list__item">
        <div class="metric-list__label">
          Liquidation LTV
          <info-tooltip>
            The maximum loan-to-value ratio before your position becomes eligible for liquidation.
            <br>
            If your Current LTV reaches this level, your collateral may be partially or fully liquidated to repay the debt.
          </info-tooltip>
        </div>
        <div class="metric-list__value">
          {{ truncatePercent(liquidationLtv, 2) }}%
        </div>
      </div>

      <div class="metric-list__item">
        <div class="metric-list__label">
          Liquidation Buffer
          <info-tooltip>
            The safety margin before your position reaches liquidation.
            <br>
            Shows how much your position can lose in value before hitting the liquidation threshold.
          </info-tooltip>
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
          <info-tooltip>
            The asset price at which your position becomes eligible for liquidation.
            <br>
            If the market price reaches this level, your collateral may be liquidated to repay the debt.
          </info-tooltip>
        </div>
        <div class="metric-list__value">
          {{ liquidationPrice !== null ? formatCompactUSD(liquidationPrice, 2, 2) : '—' }}
        </div>
      </div>
    </div>
  </div>
</template>
