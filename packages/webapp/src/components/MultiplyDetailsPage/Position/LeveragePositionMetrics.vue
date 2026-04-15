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
            Current LTV
          </div>
          <div class="metrics-list__row__value">
            <strong>{{ truncatePercent(position.currentLtv, 2) }}%</strong>
          </div>
        </div>
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Open LTV limit
          </div>
          <div class="metrics-list__row__value">
            <strong>{{ truncatePercent(position.openLtv, 2) }}%</strong>
          </div>
        </div>
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Close LTV limit
          </div>
          <div class="metrics-list__row__value">
            <strong>{{ truncatePercent(position.closeLtv, 2) }}%</strong>
          </div>
        </div>
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Liability factor
          </div>
          <div class="metrics-list__row__value">
            <strong>{{ truncatePercent(position.liabilityFactor, 2) }}%</strong>
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
          <div class="metrics-list__row__value">
            <strong>+{{ truncatePercent(position.supplyApy, 2) }}%</strong>
          </div>
        </div>
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Borrow cost APY
          </div>
          <div class="metrics-list__row__value">
            <span class="text-negative">
              <strong>-{{ truncatePercent(position.borrowApy, 2) }}%</strong>
            </span>
          </div>
        </div>
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Estimated yearly result
          </div>
          <div class="metrics-list__row__value">
            <span :class="position.yearlyResultUsd >= 0 ? 'text-positive' : 'text-negative'">
              <strong>{{ position.yearlyResultUsd >= 0 ? '+' : '' }}{{ formatCompactUSD(position.yearlyResultUsd, 2, 2) }}</strong>
            </span>
          </div>
        </div>
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Borrow liquidity
          </div>
          <div class="metrics-list__row__value">
            <strong>{{ formatPrice(selectedVault?.liquidity || 0, 2, 2) }} {{ selectedVault?.borrowAsset.symbol }}</strong>
          </div>
        </div>

        <div class="separator" />

        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Liquidation buffer:
          </div>
          <div class="metrics-list__row__value">
            <strong> {{ formatCompactUSD(position.liquidationBufferUsd, 2, 2) }}</strong>
          </div>
        </div>
      </div>
    </div>
  </template>
</template>
