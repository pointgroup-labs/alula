<script lang="ts" setup>
import type { Pool } from '@alula/market-sdk'

const { width } = useWindowSize()

const multiplyStore = useMultiplyStore()

const {
  apyDisplay,
} = provideLeveragePosition()

const selectedVault = computed(() => multiplyStore.selectedVault)
const depositPoolData = computed<Pool | undefined>(() => selectedVault.value?.depositPoolData.pool)

const asset = computed(() => selectedVault.value?.asset)
const price = computed(() => selectedVault.value?.price ?? 0)

const maxMultiplier = computed(() => selectedVault.value?.maxMultiplier ?? 0)
const maxApy = computed(() => {
  const depositApy = (selectedVault.value?.depositPoolData.apy.supply_bps ?? 0) / 100
  const borrowApy = (selectedVault.value?.borrowPoolData.apy.borrow_bps ?? 0) / 100
  const maxApy = depositApy * maxMultiplier.value - borrowApy * (maxMultiplier.value - 1)
  return maxApy || 0
})

const myApyClass = computed(() => {
  if (Number(apyDisplay.value) < 0) {
    return 'text-negative'
  }
  return 'text-cyan'
})

const maxApyClass = computed(() => {
  if (maxApy.value <= 0) {
    return 'text-negative'
  }
  if (maxApy.value <= (selectedVault.value?.depositPoolData.apy.supply_bps ?? 0) / 100) {
    return 'text-warning'
  }
  return 'text-positive'
})
</script>

<template>
  <div class="multiply-details-top">
    <back-btn to="/multiply" />

    <div
      v-if="asset"
      class="asset-data"
    >
      <img
        :src="asset?.icon"
        alt="asset icon"
      >
      <div class="asset-data__coin">
        <span data="name">{{ asset?.symbol }}</span>
        <span data="symbol">{{ asset?.name }}</span>
      </div>
    </div>

    <template v-if="depositPoolData">
      <div class="market-pill">
        {{ asset?.symbol ?? '' }} Market
      </div>

      <div
        v-if="width > 650"
        class="market-pill"
      >
        <div class="market-name">
          Price: <span class="text-num"> ${{ formatPrice(price, 2, 2) }}</span>
        </div>
      </div>
    </template>

    <div class="pool-metrics">
      <div class="pool-metrics__item">
        <span>My APY
          <info-tooltip>
            Your real-time yield for this position, including rewards and borrow costs, adjusted by your
            multiplier.
          </info-tooltip>
        </span>
        <span
          class="my-apy"
          :class="myApyClass"
        >{{ apyDisplay }}%</span>
      </div>

      <div class="separator-vert" />

      <div class="pool-metrics__item">
        <span>Max APY
          <info-tooltip>
            The highest estimated yield achievable with maximum leverage. Accounts for both collateral
            rewards and
            borrowing costs at peak multiplier.
          </info-tooltip>
        </span>
        <span
          class="max-apy"
          :class="maxApyClass"
        >{{ truncatePercent(maxApy) }}%</span>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.multiply-details-top {
  display: flex;
  align-items: center;
  gap: 16px;

  @media (max-width: $breakpoint-sm) {
    flex-wrap: wrap;
    gap: 12px;
  }

  .asset-data {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 18px;
    font-weight: 500;

    img {
      width: 38px;
      height: 38px;
      border-radius: 50%;
    }

    &__coin {
      display: flex;
      flex-direction: column;
      align-items: flex-start;

      span[data='symbol'] {
        color: $text-tertiary;
        font-size: 12px;
        opacity: 0.8;
      }
    }
  }

  .market-pill {
    display: flex;
    align-items: center;
    flex-direction: column;
    gap: 2px;
    padding: 4px 12px;
    font-size: $text-xs;
    color: $text-tertiary;
    letter-spacing: 0.05em;
    font-weight: 500;
    text-transform: capitalize;
    background-color: color-mix(in oklab, $secondary 60%, transparent);
    border-radius: $radius-full;

    span {
      color: $text-primary;
    }

    @media (max-width: $breakpoint-xs) {
      padding: 2px 16px;
    }
  }

  .pool-metrics {
    display: flex;
    align-items: stretch;
    gap: 24px;
    margin-left: auto;

    @media (max-width: $breakpoint-sm) {
      margin: 16px auto 0;
      overflow-x: auto;
    }

    &__item {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: space-between;
      gap: 4px;
      font-size: 12px;
      color: $text-tertiary;

      span {
        font-size: 14px;
        color: $text-primary;
      }

      span:nth-child(1) {
        font-size: $text-xs;
        color: $text-tertiary;
        letter-spacing: 0.05em;
        display: flex;
        align-items: center;
        gap: 4px;

        @media (max-width: $breakpoint-sm) {
          align-items: flex-start;
          gap: 8px;

          i {
            margin-top: 4px;
          }
        }
      }

      span:nth-child(2) {
        font-family: $font-JetBrainsMono;
        font-size: 18px;
        font-weight: 700;
        line-height: 30px;
      }

      .my-apy {
        color: $cyan;
      }

      .max-apy {
        color: $success;
      }
    }
  }
}
</style>
