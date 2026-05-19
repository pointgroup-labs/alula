<script lang="ts" setup>
const {
  collateralValueUsd,
  positions,
  selectedPool,
  weightedBorrowedValueUsd,
  liquidationBufferUsd,
  healthFactor,
} = useMyPosition()

const route = useRoute()
const router = useRouter()

const dialog = ref(false)

const healthIndicatorStyle = computed(() => ({
  '--indicator-width': `${Math.min(Math.max((((healthFactor.value ?? 1) - 1) * 100), 0), 100)}%`,
  '--indicator-color': healthFactorColor(healthFactor.value),
}))

const healthStatusLabel = computed(() => {
  if (!healthFactor.value) {
    return null
  }
  if (healthFactor.value < 1.2) {
    return 'At Risk'
  }
  if (healthFactor.value < 1.5) {
    return 'Caution'
  }
  return 'Health'
})

const healthStatusDetail = computed(() => {
  if (!healthFactor.value) {
    return null
  }
  const formattedBuffer = formatCompactUSD(liquidationBufferUsd.value, 2, 2)
  if (healthFactor.value < 1.2) {
    return 'Add collateral or repay'
  }
  if (healthFactor.value < 1.5) {
    return `Buffer left: ~${formattedBuffer}`
  }
  return `You can borrow ~${formattedBuffer} more before liquidation.`
})

const existSuppliedPools = computed(() => positions.value?.deposits?.map(p => p.address) ?? [])

function isCurrentPool(address: string) {
  return address === selectedPool.value.pool_address
}

async function navigateToPool(address: string) {
  if (!address || isCurrentPool(address)) {
    return
  }

  await router.push({
    name: route.name,
    params: {
      ...route.params,
      pool: address,
    },
    query: route.query,
  })
}

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
        <button
          v-for="position in positions.deposits"
          :key="position.address"
          type="button"
          class="overview-metric__item"
          :class="{ 'overview-metric__item--interactive': !isCurrentPool(position.address) }"
          :disabled="isCurrentPool(position.address)"
          @click="navigateToPool(position.address)"
        >
          <div class="asset">
            <img
              :src="position.icon"
              alt="asset icon"
            >
            {{ position.symbol }}
          </div>
          <div class="value value--interactive">
            {{ formatCompactUSD(position.usd, 2, 2) }}
            <i-app-chevron-down
              v-if="!isCurrentPool(position.address)"
              class="chevron"
            />
          </div>
        </button>
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
        <button
          v-for="position in positions.borrows"
          :key="position.address"
          type="button"
          class="overview-metric__item"
          :class="{ 'overview-metric__item--interactive': !isCurrentPool(position.address) }"
          :disabled="isCurrentPool(position.address)"
          @click="navigateToPool(position.address)"
        >
          <div class="asset">
            <img
              :src="position.icon"
              alt="asset icon"
            >
            {{ position.symbol }}
          </div>
          <div class="value value--interactive">
            {{ formatCompactUSD(position.usd, 2, 2) }}
            <i-app-chevron-down
              v-if="!isCurrentPool(position.address)"
              class="chevron"
            />
          </div>
        </button>
      </div>

      <div
        v-else
        class="no-borrow-card"
      >
        Start borrowing
        <j-btn
          variant="outlined-brand-secondary"
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
          <j-tooltip>
            <span
              v-if="healthStatusLabel"
              class="health-highlight__badge"
              :style="{ '--indicator-color': healthIndicatorStyle['--indicator-color'] }"
            >
              {{ healthStatusLabel }}
            </span>
            <template #content>
              {{ healthStatusDetail }}
            </template>
          </j-tooltip>
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

  <change-pool-dialog
    v-model="dialog"
    :filtered-positions="existSuppliedPools"
    is-borrow
  />
</template>
