<script lang="ts" setup>
const {
  collateralValueUsd,
  positions,
  weightedBorrowedValueUsd,
  healthFactor,
} = useMyPosition()

const dialog = ref(false)

const healthIndicatorStyle = computed(() => ({
  '--indicator-width': `${Math.min(Math.max((((healthFactor.value ?? 1) - 1) * 100), 0), 100)}%`,
  '--indicator-color': healthFactorColor(healthFactor.value),
}))

function handleClick() {
  dialog.value = !dialog.value
}
</script>

<template>
  <div class="position-panel position-panel--overview stat-card stat-card--small">
    <div class="position-panel__eyebrow">
      Position Overview
    </div>

    <div class="overview-metric">
      <div class="overview-metric__title">
        Collateral Value

        <div class="overview-metric__value">
          {{ formatCompactUSD(collateralValueUsd, 2, 2) }}
        </div>
      </div>

      <div
        v-if="positions?.deposits"
        class="overview-metric__list"
      >
        <div
          v-for="position in positions.deposits"
          :key="position.address"
          class="overview-metric__item"
        >
          <div class="asset">
            <img
              :src="position.icon"
              alt="asset icon"
            >
            {{ position.symbol }}
          </div>
          <div class="value">
            {{ formatCompactUSD(position.usd, 2, 2) }}
          </div>
        </div>
      </div>
    </div>

    <div
      class="separator"
      style="margin: 12px 0;"
    />

    <div
      class="overview-metric"
      :style="{ height: `${positions?.borrows && positions?.borrows?.length > 0 ? 'auto' : '100%'}` }"
    >
      <div class="overview-metric__title">
        Borrow Value

        <div class="overview-metric__value">
          {{ formatCompactUSD(weightedBorrowedValueUsd, 2, 2) }}
        </div>
      </div>

      <div
        v-if="positions?.borrows && positions?.borrows?.length > 0"
        class="overview-metric__list"
      >
        <div
          v-for="position in positions.borrows"
          :key="position.address"
          class="overview-metric__item"
        >
          <div class="asset">
            <img
              :src="position.icon"
              alt="asset icon"
            >
            {{ position.symbol }}
          </div>
          <div class="value">
            {{ formatCompactUSD(position.usd, 2, 2) }}
          </div>
        </div>
      </div>

      <div
        v-else
        class="no-borrow-card"
      >
        Start borrowing
        <j-btn
          variant="brand-secondary-outlined"
          size="sm"
          @click="handleClick"
        >
          Borrow
        </j-btn>
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
            v-if="healthFactor !== null"
            class="hf-indicator"
            :style="healthIndicatorStyle"
          />
          <div
            class="health-highlight__value"
            :style="{ color: healthIndicatorStyle['--indicator-color'] }"
          >
            {{ healthFactor === null ? 'No debt' : truncatePercent(healthFactor, 2) }}
          </div>
        </div>
      </div>

    </div>
  </div>

  <change-pool-dialog v-model="dialog" />
</template>
