<script lang="ts" setup>
const {
  position,
  selectedVault,
  healthIndicatorStyle,
} = useLeveragePosition()
</script>

<template>
  <div
    v-if="position"
    class="pool-card position-panel position-panel--overview"
  >
    <div class="position-panel__eyebrow">
      Position Overview
    </div>

    <div class="overview-grid">
      <div class="overview-metric overview-metric--primary">
        <div class="overview-metric__title">
          Exposure ({{ selectedVault?.asset.symbol }})
        </div>
        <div class="overview-metric__value">
          {{ shortenNumber(position.deposited, 2, maxDecimalsForShortenNumber(position.deposited)) }} {{ selectedVault?.asset.symbol }}
        </div>
        <div class="overview-metric__caption">
          Long {{ selectedVault?.asset.symbol }} exposure at current market price.
        </div>
      </div>

      <div class="overview-metric overview-metric--featured">
        <div class="overview-metric__title">
          Leverage
        </div>
        <div class="overview-metric__value overview-metric__value--accent overview-metric__value--featured">
          {{ truncatePercent(position.currentMultiplier, 2) }}x
        </div>
        <div class="overview-metric__caption">
          Max vault multiplier: {{ truncatePercent(selectedVault?.maxMultiplier || 0, 2) }}x
        </div>
      </div>

      <div class="overview-metric">
        <div class="overview-metric__title">
          Position Value
        </div>
        <div class="overview-metric__value">
          {{ formatCompactUSD(position.positionValueUsd, 2, 2) }}
        </div>
        <div class="overview-metric__caption">
          Total market value of your {{ selectedVault?.asset.symbol }} exposure.
        </div>
      </div>

      <div class="overview-metric">
        <div class="overview-metric__title">
          Borrowed
        </div>
        <div class="overview-metric__value">
          {{ shortenNumber(position.borrowed, 2, maxDecimalsForShortenNumber(position.borrowed)) }} {{ selectedVault?.borrowAsset.symbol }}
        </div>
        <div class="overview-metric__caption">
          ${{ amountToUsdWithShort(position.borrowed, selectedVault?.borrowPoolPrice || 0, false) }} outstanding debt
        </div>
      </div>
    </div>

    <div class="health-highlight">
      <div class="health-highlight__meta">
        <div class="health-highlight__label">
          Health Factor
          <info-tooltip>
            Health Factor = weighted collateral at Close LTV divided by weighted debt with liability factor.
            <br>
            Lower values mean higher liquidation risk.
          </info-tooltip>
        </div>

        <div class="health-highlight__value-row">
          <div
            v-if="position.healthFactor !== null"
            class="hf-indicator"
            :style="healthIndicatorStyle"
          />
          <div
            class="health-highlight__value"
            :style="{ color: healthIndicatorStyle['--indicator-color'] }"
          >
            {{ position.healthFactor === null ? 'No debt' : truncatePercent(position.healthFactor, 2) }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
