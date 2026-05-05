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

    <div class="overview-metrics-list">
      <div class="overview-metrics-list__row">
        <div class="asset">
          <img
            :src="selectedVault?.asset.icon"
            alt="asset icon"
          >
          <div class="asset-data">
            <div class="asset-data__title">
              Collateral
            </div>
            <div class="asset-data__value">
              {{ shortenNumber(position.deposited, 2, maxDecimalsForShortenNumber(position.deposited)) }} {{ selectedVault?.asset.symbol }}
              <j-pill-label>
                {{ truncatePercent(position.supplyApy) }}% APY
              </j-pill-label>
            </div>
          </div>
        </div>
        <div class="value">
          ${{ formatPrice(position.positionValueUsd, 2, 2) }}
        </div>
      </div>

      <div class="overview-metrics-list__row">
        <div class="asset">
          <img
            :src="selectedVault?.borrowAsset.icon"
            alt="asset icon"
          >
          <div class="asset-data">
            <div class="asset-data__title">
              Borrowed
            </div>
            <div class="asset-data__value">
              {{ shortenNumber(position.borrowed, 2, maxDecimalsForShortenNumber(position.borrowed)) }} {{ selectedVault?.borrowAsset.symbol }}
              <j-pill-label variant="indigo">
                {{ truncatePercent(position.borrowApy) }}% APY
              </j-pill-label>
            </div>
          </div>
        </div>
        <div class="value">
          ${{ formatPrice(position.borrowedUsd, 2, 2) }}
        </div>
      </div>

      <div class="separator" />

      <div class="overview-metrics-list__row">
        <div class="label">
          Net Equity
          <info-tooltip>
            The net value of your position, calculated as supplied assets minus borrowed value.
            <br>
            This represents your real exposure and will fluctuate with asset prices and debt levels.
          </info-tooltip>
        </div>
        <div
          class="value"
          style="font-size: 14px;"
        >
          ${{ formatPrice(position.netEquityUsd, 2, 2) }}
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

<style lang="scss" scoped>
.overview-metrics-list {
  display: flex;
  flex-direction: column;

  &__row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: $spacing-md $spacing-lg;
    font-size: 12px;
  }

  .asset {
    display: flex;
    align-items: center;
    gap: 10px;

    img {
      width: 30px;
      height: 30px;
      object-fit: contain;
      border-radius: 50%;
    }

    &-data {
      display: flex;
      flex-direction: column;
      gap: 4px;

      &__title {
        color: $text-tertiary;
        font-size: 12px;
      }

      &__value {
        color: #fff;
        font-size: 13px;
        line-height: normal;
        display: flex;
        align-items: center;
        gap: 4px;
      }
    }

    .j-pill-label {
      font-size: 11px;
      padding: 2px 6px;
    }
  }

  .label {
    font-size: 14px;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .value {
    font-family: $font-JetBrainsMono;
  }

  .separator {
    margin: $spacing-lg 0;
  }
}
</style>
