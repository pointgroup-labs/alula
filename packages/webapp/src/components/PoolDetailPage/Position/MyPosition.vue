<script lang="ts" setup>
const {
  selectedPool,
  marketKey,
  netApy,
  netYearlyUsd,
  hasSupply,
  hasBorrow,
  hasPosition,
} = provideMyPosition()

const marketsStore = useMarketsStore()
const marketActions = useMarketActions()
const userStore = useUserStore()

const isLoadingObligations = computed(() => userStore.loading)

const netApyDisplay = computed(() => {
  const value = Number(netApy.value || 0)
  const absValue = Math.abs(value)

  if (absValue === 0) {
    return '0.00'
  }

  const decimals = Math.max(2, getZeroCountAfterDecimal(absValue))
  return truncatePercent(value, decimals)
})

const positionLabel = computed(() => {
  if (hasBorrow.value) {
    return 'Borrow'
  }
  if (hasSupply.value) {
    return 'Supply'
  }
  return null
})

const statusPillStyle = computed(() => {
  if (hasSupply.value && hasBorrow.value) {
    return {
      '--color': '#8a8df4',
      '--background-color': 'rgb(138 141 244 / 10%)',
    }
  }
  if (hasBorrow.value) {
    return {
      '--color': '#8a8df4',
      '--background-color': 'rgb(138 141 244 / 10%)',
    }
  }
  if (hasSupply.value) {
    return {
      '--color': '#22d3ee',
      '--background-color': 'rgb(34 211 238 / 10%)',
    }
  }
  return {
    '--color': '#6b7994',
    '--background-color': 'rgb(107 121 148 / 15%)',
  }
})

function openAction(action: 'supply' | 'withdraw' | 'borrow' | 'repay') {
  marketsStore.selectedMarketName = marketKey.value
  marketsStore.selectedPoolAddress = selectedPool.value.pool_address

  if (action === 'supply') {
    marketsStore.dialogSupply = true
    return
  }
  if (action === 'withdraw') {
    marketsStore.dialogWithdraw = true
    return
  }
  if (action === 'borrow') {
    marketsStore.dialogBorrow = true
    return
  }
  marketsStore.dialogRepay = true
}
</script>

<template>
  <section id="my-position">
    <div class="pool-card stat-card position-card">
      <div class="stat-card__header">
        <div class="position-card__heading">
          <div class="position-card__asset">
            <img
              :src="selectedPool?.asset.icon"
              :alt="selectedPool?.asset.symbol"
            >
            <div class="position-card__title-wrap">
              <h3 class="pool-card-title">
                My Position
              </h3>
            </div>
          </div>

          <div
            v-if="positionLabel"
            class="pool-card-pill"
            :style="statusPillStyle"
          >
            {{ positionLabel }}
          </div>

          <div
            v-if="hasPosition"
            class="net-apy"
          >
            NET APY
            <info-tooltip>
              Net APY = supply yield on interest-bearing deposits minus borrow cost, divided by your net position base.
              <br>
              Collateral-only assets do not earn supply APY.
              <br>
              Estimated yearly result: {{ netYearlyUsd >= 0 ? '+' : '' }}{{ formatCompactUSD(netYearlyUsd, 2, 2) }}.
            </info-tooltip>
            <span
              class="net-apy__value"
              :class="[`net-apy--${netApy > 0 ? 'positive' : 'negative'}`]"
            >
              {{ netApy > 0 ? '+' : '' }}{{ netApyDisplay }}%</span>
          </div>

          <div
            v-if="hasPosition"
            class="position-actions"
          >
            <j-btn
              v-if="hasSupply"
              size="sm"
              variant="brand-outlined"
              :disabled="marketActions.isDisabled(selectedPool.pool_address, 'deposit', selectedPool.market!)"
              :loading="marketActions.isLoading(selectedPool.pool_address, 'deposit', selectedPool.market!)"
              @click="openAction('supply')"
            >
              Supply
            </j-btn>
            <j-btn
              v-else
              size="sm"
              variant="brand-secondary-outlined"
              :disabled="marketActions.isDisabled(selectedPool.pool_address, 'repay', selectedPool.market!)"
              :loading="marketActions.isLoading(selectedPool.pool_address, 'repay', selectedPool.market!)"
              @click="openAction('repay')"
            >
              Repay
            </j-btn>
            <j-btn
              v-if="hasBorrow"
              size="sm"
              variant="brand-secondary-outlined"
              :disabled="marketActions.isDisabled(selectedPool.pool_address, 'borrow', selectedPool.market!)"
              :loading="marketActions.isLoading(selectedPool.pool_address, 'borrow', selectedPool.market!)"
              @click="openAction('borrow')"
            >
              Borrow
            </j-btn>
            <j-btn
              v-else
              size="sm"
              variant="brand-outlined"
              :disabled="marketActions.isDisabled(selectedPool.pool_address, 'withdraw', selectedPool.market!)"
              :loading="marketActions.isLoading(selectedPool.pool_address, 'withdraw', selectedPool.market!)"
              @click="openAction('withdraw')"
            >
              Withdraw
            </j-btn>
          </div>
        </div>
      </div>

      <div class="stat-card__body">
        <template v-if="hasPosition">
          <div class="position-layout">

            <position-overview />

            <position-supply-info v-if="hasSupply" />
            <position-borrow-limits v-else />

            <position-risk-metrics />

          </div>
        </template>

        <j-skeleton
          v-else-if="!hasPosition && isLoadingObligations"
          full-width
          height="178"
          style="border-radius: 12px;"
        />

        <template v-else>
          <div class="empty-state">
            <div class="empty-state__title">
              No active position in this pool
            </div>
            <div class="empty-state__text">
              Supply to start earning yield, or borrow once you have collateral in this market.
            </div>
          </div>
        </template>

      </div>
    </div>

    <client-only>
      <supply-dialog
        v-model="marketsStore.dialogSupply"
        :data="selectedPool"
      />

      <borrow-dialog
        v-model="marketsStore.dialogBorrow"
        :data="selectedPool"
      />

      <withdraw-dialog v-model="marketsStore.dialogWithdraw" />
      <repay-dialog v-model="marketsStore.dialogRepay" />
    </client-only>
  </section>
</template>

<style lang="scss">
section#my-position {
  .pool-card {
    background-color: $bg-card;
    .stat-card__header {
      gap: 12px;
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
    }
  }

  .position-card__heading {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 16px;

    @media (max-width: $breakpoint-xs) {
      flex-wrap: wrap;
      gap: 12px;
    }
  }

  .net-apy {
    display: flex;
    align-items: center;
    gap: 8px;
    color: $text-tertiary;
    font-size: 12px;
    font-style: normal;
    font-weight: 400;
    line-height: normal;
    margin: 0 $spacing-xs 0 auto;

    @media (max-width: $breakpoint-xs) {
      margin: 0;
      gap: 2px;
    }

    &__value {
      margin-left: $spacing-xs;
      font-family: $font-JetBrainsMono;
      font-size: 20px;
      font-style: normal;
      font-weight: 700;
      line-height: normal;

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

  .position-card__asset {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;

    img {
      width: 38px;
      height: 38px;
      border-radius: 50%;
      flex-shrink: 0;
    }
  }

  .position-card__title-wrap {
    min-width: 0;

    .subtitle {
      margin: 4px 0 0;
      color: $text-tertiary;
      font-size: 12px;
    }
  }

  .position-layout {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: auto auto;
    gap: 16px;
    margin-bottom: 16px;

    @media (max-width: $breakpoint-md) {
      grid-template-columns: 1fr;
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

      @media (max-width: $breakpoint-md) {
        grid-column: span 1;
      }
    }

    &__eyebrow {
      color: $text-primary;
      font-size: 12px;
      font-weight: 700;
      padding-bottom: $spacing-md;
    }
  }

  .overview-metric {
    display: flex;
    flex-direction: column;
    gap: 8px;

    &__title {
      color: $text-tertiary;
      font-size: 12px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
    }

    &__value {
      font-family: $font-JetBrainsMono;
      font-size: 14px;
      font-weight: 700;
      color: $text-primary;
    }

    &__list {
      display: flex;
      flex-direction: column;
      max-height: 120px;
      overflow: auto;
    }

    &__item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: $spacing-md $spacing-lg;
      width: 100%;
      border: 0;
      background: transparent;
      text-align: left;

      .asset {
        display: flex;
        align-items: center;
        gap: 12px;
        color: $text-tertiary;
        font-size: 12px;
        font-style: normal;
        font-weight: 400;
        line-height: normal;
      }

      img {
        width: 24px;
        height: 24px;
        object-fit: contain;
        border-radius: 50%;
      }

      .value {
        color: $text-tertiary;
        font-family: $font-JetBrainsMono;
        font-size: 12px;
        font-style: normal;
        font-weight: 400;
        line-height: normal;

        &--interactive {
          display: flex;
          align-items: center;
          gap: 8px;
        }
      }

      .chevron {
        width: 12px;
        transform: rotate(-90deg);
        opacity: 0.55;
        transition:
          transform 0.2s ease,
          opacity 0.2s ease;
      }

      &--interactive {
        cursor: pointer;
        border-radius: $radius-md;
        transition: background-color 0.2s ease;

        &:hover {
          background-color: color-mix(in oklab, $bg-secondary 55%, transparent);

          .chevron {
            opacity: 1;
            transform: translateX(2px) rotate(-90deg);
          }
        }

        &:focus-visible {
          outline: 1px solid $border-secondary;
          outline-offset: 0;
        }
      }
    }

    [class*='tooltip'] {
      cursor: default;
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

  .metric-list {
    display: flex;
    flex-direction: column;
    gap: 0;

    &__item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
      padding: $spacing-md 0;

      &:last-child {
        padding-bottom: 0;

        .metric-list__value {
          font-size: 14px;
          font-weight: 500;
        }
      }
    }

    &__label {
      color: $text-tertiary;
      font-size: 12px;
      display: flex;
      align-items: center;
      gap: 6px;
    }

    &__value {
      color: $text-primary;
      font-family: $font-JetBrainsMono;
      font-size: 12px;
      font-weight: 400;
      text-align: right;

      &--stacked {
        display: flex;
        flex-direction: column;
        align-items: flex-end;

        span {
          color: $text-primary;
          font-family: $font-JetBrainsMono;
          font-size: 12px;
          font-weight: 700;
        }

        small {
          color: $text-tertiary;
          font-family: $font-JetBrainsMono;
          font-size: 12px;
          font-style: normal;
          font-weight: 400;
          line-height: normal;
        }
      }
    }

    .separator {
      margin: $spacing-md 0;
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
        width 0.3s ease,
        background-color 0.3s ease;
    }
  }

  .supply-color {
    color: $cyan !important;
  }

  .borrow-color {
    color: $indigo !important;
  }

  .empty-state {
    padding: 64px 16px;
    border: 1px solid $border-primary;
    border-radius: $radius-xl;
    margin-bottom: 18px;
    text-align: center;

    &__title {
      color: $text-primary;
      font-size: 16px;
      font-weight: 600;
      margin-bottom: 4px;
    }

    &__text {
      color: $text-tertiary;
      font-size: 13px;
      line-height: 1.5;
    }
  }

  .no-borrow-card {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    border: 1px solid $border-secondary;
    height: 100%;
    border-radius: $radius-md;
    color: $text-tertiary;
    font-size: 14px;
    font-style: normal;
    font-weight: 500;
    line-height: 20px;
    gap: 12px;

    @media (max-width: $breakpoint-xs) {
      padding: 32px 0;
    }
  }

  .position-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
    flex-wrap: wrap;

    @media (max-width: $breakpoint-xs) {
      margin: 0 auto;
    }
  }
}
</style>
