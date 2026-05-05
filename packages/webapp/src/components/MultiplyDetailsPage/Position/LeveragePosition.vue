<script lang="ts" setup>
import { formatCompactUSD } from '~/utils'

const marketsStore = useMarketsStore()
const marketActions = useMarketActions()

const dialogLeverage = toRef(marketsStore, 'dialogLeverage')
const dialogLeverageWithdraw = toRef(marketsStore, 'dialogLeverageWithdraw')

const {
  apyDisplay,
  position,
  hasPosition,
  isLoadingPosition,
  selectedVault,
  openMultiply,
  closeMultiply,
} = provideLeveragePosition()
</script>

<template>
  <section id="my-leverage-position">
    <div class="pool-card stat-card position-card">
      <div class="stat-card__header">
        <div class="position-card__asset">
          <div class="position-card__icons">
            <img
              :src="selectedVault?.asset.icon"
              :alt="selectedVault?.asset.symbol"
            >
            <img
              :src="selectedVault?.borrowAsset.icon"
              :alt="selectedVault?.borrowAsset.symbol"
              class="position-card__icons-secondary"
            >
          </div>

          <div class="position-card__title-wrap">
            <h3 class="pool-card-title">
              Long {{ selectedVault?.asset.symbol }}
            </h3>
          </div>
        </div>

        <div
          v-if="position?.currentMultiplier"
          class="pool-card-pill pool-card-pill--multiply"
        >
          Multiplier: {{ Number.isFinite(position?.currentMultiplier) ? truncatePercent(position?.currentMultiplier || 0, 2) : 0 }}x
        </div>

        <div
          v-if="hasPosition && position"
          class="net-apy"
        >
          Net APY
          <info-tooltip>
            Net APY for the active multiply position. It combines deposit yield on the collateral leg and borrow cost on the margin leg at the current multiplier.
            <br>
            Estimated yearly result: {{ position.yearlyResultUsd >= 0 ? '+' : '' }}{{ formatCompactUSD(position.yearlyResultUsd, 2, 2) }}.
          </info-tooltip>
          <span
            class="net-apy__value"
            :class="[`net-apy--${position.currentApy >= 0 ? 'positive' : 'negative'}`]"
          >
            {{ position.currentApy >= 0 ? '+' : '' }}{{ apyDisplay }}%
          </span>
        </div>

        <div
          v-if="hasPosition"
          class="position-actions"
          :style="{ 'margin-left': hasPosition ? 0 : 'auto' }"
        >
          <j-btn
            size="sm"
            variant="outline-positive"
            :disabled="!selectedVault || marketActions.isDisabled(selectedVault.pool_address, 'leverage', selectedVault.market)"
            :loading="!!selectedVault && marketActions.isLoading(selectedVault.pool_address, 'leverage', selectedVault.market)"
            @click="openMultiply"
          >
            Add
          </j-btn>
          <j-btn
            size="sm"
            variant="outlined-negative"
            :disabled="!selectedVault || marketActions.isDisabled(selectedVault.pool_address, 'withdrawLeverage', selectedVault.market)"
            :loading="!!selectedVault && marketActions.isLoading(selectedVault.pool_address, 'withdrawLeverage', selectedVault.market)"
            @click="closeMultiply"
          >
            Close
          </j-btn>
        </div>

      </div>

      <div class="stat-card__body">
        <template v-if="hasPosition && position">
          <leverage-position-overview />
          <leverage-position-metrics />
        </template>

        <j-skeleton
          v-else-if="isLoadingPosition"
          full-width
          height="178"
          style="border-radius: 12px;"
          class="empty-state-skeleton"
        />

        <template v-else>
          <div class="empty-state">
            <div class="empty-state__title">
              No active multiply position in this vault
            </div>
            <div class="empty-state__text">
              Open a multiply position to see live collateral, debt, multiplier and liquidation metrics for this pair.
            </div>
            <j-btn
              size="sm"
              variant="outline-positive"
              :disabled="!selectedVault || marketActions.isDisabled(selectedVault.pool_address, 'leverage', selectedVault.market)"
              :loading="!!selectedVault && marketActions.isLoading(selectedVault.pool_address, 'leverage', selectedVault.market)"
              @click="openMultiply"
            >
              Open Position
            </j-btn>
          </div>
        </template>
      </div>
    </div>

    <client-only>
      <multiply-dialog
        v-model="dialogLeverage"
        :data="selectedVault"
        :has-position="hasPosition"
      />

      <withdraw-multiply-dialog
        v-model="dialogLeverageWithdraw"
        :data="selectedVault"
        :has-position="hasPosition"
      />
    </client-only>
  </section>
</template>

<style lang="scss">
section#my-leverage-position {
  h3 {
    margin: 0;
  }

  .pool-card {
    background-color: $bg-card;
    border: 1px solid #1a2335;
    border-radius: 16px;

    .stat-card__header {
      width: 100%;
      display: flex;
      align-items: center;
      flex-wrap: wrap;
      gap: 16px;
      padding: 14px 20px;
      border-bottom: 1px solid $border-primary;
    }

    .stat-card__body {
      display: grid;
      grid-template-columns: 1.25fr 1fr;
      grid-template-rows: auto auto;
      gap: 16px;
      margin-bottom: 16px;
      padding: $spacing-xl;

      @media (max-width: $breakpoint-md) {
        grid-template-columns: 1fr;
      }
    }

    &-title {
      font-size: 14px;
    }

    &-pill {
      font-size: $text-xs;
      padding: 2px 6px;
      border-radius: 50px;
      color: var(--color);
      background-color: var(--background-color);
      white-space: nowrap;

      &--multiply {
        --color: #1dc978;
        --background-color: rgba(29, 201, 121, 0.1);
        border: 1px solid #1dc978;
      }

      &--position {
        position: relative;
        display: flex;
        align-items: center;
        gap: 6px;
        --color: #{$text-tertiary};
        --background-color: rgb(138 141 244 / 10%);

        &::before {
          content: '';
          display: block;
          width: 6px;
          height: 6px;
          border-radius: 50%;
          background-color: $success;
        }
      }
    }
  }

  .position-card__asset {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .position-card__icons {
    position: relative;
    width: 62px;
    height: 38px;
    flex-shrink: 0;

    img {
      position: absolute;
      top: 0;
      width: 38px;
      height: 38px;
      border-radius: 50%;
      border: 2px solid $bg-card;
      background-color: $bg-card;
      object-fit: cover;
    }
  }

  .position-card__icons-secondary {
    left: 20px;
  }

  .position-card__title-wrap {
    min-width: 0;

    .subtitle {
      margin: 4px 0 0;
      color: $text-tertiary;
      font-size: 12px;
    }
  }

  .net-apy {
    display: flex;
    align-items: center;
    gap: 8px;
    color: $text-tertiary;
    font-size: 12px;
    font-weight: 400;
    margin: 0 $spacing-xs 0 auto;

    @media (max-width: $breakpoint-xs) {
      margin: 0;
      gap: 2px;
    }

    &__value {
      margin-left: $spacing-xs;
      font-family: $font-JetBrainsMono;
      font-size: 20px;
      font-weight: 700;

      @media (max-width: $breakpoint-xs) {
        font-size: 14px;
      }
    }

    &--positive {
      color: $success;
    }

    &--negative {
      color: $danger;
    }
  }

  .position-panel {
    padding: $spacing-xl;
    display: flex;
    flex-direction: column;
    background-color: $bg-tertiary;
    border-color: $border-secondary;

    &--overview {
      grid-row: span 2;
    }

    &__eyebrow {
      color: $text-primary;
      font-size: 12px;
      font-weight: 700;
      padding-bottom: $spacing-md;
    }
  }

  .metrics-list {
    display: flex;
    flex-direction: column;
  }

  .metrics-list__row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: $spacing-md 0;
    color: $text-secondary;
    font-size: 12px;

    &:last-child {
      padding-bottom: 0;
    }

    &__label {
      color: $text-tertiary;
      display: flex;
      align-items: center;
      gap: 6px;
    }

    &__value {
      color: $text-primary;
      font-family: $font-JetBrainsMono;
      text-align: right;

      &--stacked {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 4px;

        small {
          color: $text-tertiary;
          font-size: 11px;
          line-height: 1.4;
        }
      }
    }
  }

  .health-highlight {
    padding-top: 12px;
    border-top: 1px solid $border-primary;
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: auto;

    &__meta {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
    }

    &__label {
      color: $text-tertiary;
      font-size: 12px;
      display: flex;
      align-items: center;
      gap: 6px;
      flex-wrap: wrap;
    }

    &__badge {
      display: inline-flex;
      align-items: center;
      padding: 2px 8px;
      border-radius: 999px;
      color: var(--indicator-color, $text-primary);
      background-color: color-mix(in oklab, var(--indicator-color, $border-secondary) 12%, transparent);
      font-size: 11px;
      font-weight: 600;
      line-height: 1.2;
      letter-spacing: 0.02em;
    }

    &__status {
      font-size: 12px;
      font-weight: 600;
      letter-spacing: 0.06em;
    }

    &__value-row {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
    }

    &__value {
      font-family: $font-JetBrainsMono;
      font-size: 14px;
      font-weight: 700;
      line-height: 1;
      color: $text-primary;
    }

    &__hint {
      font-size: 12px;
      line-height: 1.4;
      color: $text-tertiary;
    }
  }

  .hf-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .hf-indicator {
    position: relative;
    width: 120px;
    height: 4px;
    border-radius: $radius-lg;
    background-color: color-mix(in oklab, $border-primary 70%, transparent);
    overflow: hidden;
    flex-shrink: 0;

    &::after {
      content: '';
      position: absolute;
      top: 0;
      right: 0;
      width: var(--indicator-width, 0%);
      height: 100%;
      border-radius: $radius-lg;
      background-color: var(--indicator-color, var(--hf-success));
      transition:
        width $transition-base ease,
        background-color $transition-base ease;
    }
  }

  .text-positive {
    color: $success !important;
  }

  .text-negative {
    color: $danger !important;
  }
  .position-actions {
    display: flex;
    gap: 12px;

    @media (max-width: $breakpoint-xs) {
      margin: 0 auto !important;
    }
  }

  .empty-state {
    min-height: 178px;
    grid-column: span 2;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 24px;
    text-align: center;
    background-color: $bg-tertiary;
    border: 1px solid $border-secondary;
    border-radius: 16px;

    &__title {
      color: $text-primary;
      font-size: 16px;
      font-weight: 600;
    }

    &__text {
      max-width: 520px;
      color: $text-secondary;
      font-size: 13px;
      line-height: 1.6;
    }
  }

  .empty-state-skeleton {
    grid-column: span 2;
  }
}
</style>
