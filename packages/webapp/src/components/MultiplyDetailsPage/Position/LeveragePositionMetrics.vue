<script lang="ts" setup>
const {
  position,
  selectedVault,
} = useLeveragePosition()
</script>

<template>
  <template v-if="position">
    <div class="pool-card position-panel">
      <div class="position-panel__eyebrow">
        Risk Metrics
      </div>

      <div class="metrics-list">
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Leverage
            <info-tooltip>
              Current position multiple based on total exposure relative to your invested capital.
              <br>
              Open LTV limit: {{ truncatePercent(position.openLtv, 2) }}%.
            </info-tooltip>
          </div>
          <div class="metrics-list__row__value">
            <strong>{{ truncatePercent(position.currentMultiplier, 2) }}x</strong>
          </div>
        </div>
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Liquidation Price
            <info-tooltip>
              Estimated {{ selectedVault?.asset.symbol }} price where this position reaches liquidation.
              <br>
              Based on Close LTV {{ truncatePercent(position.closeLtv, 2) }}% and liability factor {{ truncatePercent(position.liabilityFactor, 2) }}%.
            </info-tooltip>
          </div>
          <div class="metrics-list__row__value metrics-list__row__value--stacked">
            <strong>{{ position.liquidationPrice !== null ? `$${formatPrice(position.liquidationPrice, 2, position.liquidationPrice < 1 ? 6 : 4)}` : '—' }}</strong>
            <small v-if="position.distanceToLiquidationPercent !== null">
              {{ truncatePercent(position.distanceToLiquidationPercent, 2) }}% above liquidation
            </small>
          </div>
        </div>
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Liquidation Buffer
            <info-tooltip>
              Estimated downside room before liquidation at current debt and collateral settings.
              <br>
              Current LTV: {{ truncatePercent(position.currentLtv, 2) }}%. Close LTV limit: {{ truncatePercent(position.closeLtv, 2) }}%.
            </info-tooltip>
          </div>
          <div class="metrics-list__row__value metrics-list__row__value--stacked">
            <strong>{{ formatCompactUSD(position.liquidationBufferUsd, 2, 2) }}</strong>
          </div>
        </div>
      </div>
    </div>

    <div class="pool-card position-panel">
      <div class="position-panel__eyebrow">
        Yield Breakdown
      </div>

      <div class="metrics-list">
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Collateral supply APY
          </div>
          <div
            class="metrics-list__row__value"
            :class="position.supplyApy >= 0 ? 'text-positive' : 'text-negative'"
          >
            <strong>+{{ truncatePercent(position.supplyApy, 2) }}%</strong>
          </div>
        </div>
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Borrow cost APY
          </div>
          <div class="metrics-list__row__value">
            <span>
              <strong>{{ truncatePercent(position.borrowApy, 2) }}%</strong>
            </span>
          </div>
        </div>
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Estimated yearly result
          </div>
          <div class="metrics-list__row__value">
            ≈ <span :class="position.yearlyResultUsd >= 0 ? 'text-positive' : 'text-negative'">
              <strong>{{ position.yearlyResultUsd >= 0 ? '+' : '' }}{{ formatCompactUSD(position.yearlyResultUsd, 2, 2) }}</strong>
            </span>
          </div>
        </div>
      </div>
    </div>
  </template>
</template>
