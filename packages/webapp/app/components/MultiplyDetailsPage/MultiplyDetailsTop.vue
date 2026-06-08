<script lang="ts" setup>
import type { Pool } from '@alula/market-sdk'
import { getApyRangeForMultiplier } from '~/utils/multiply'

const { width } = useWindowSize()

const multiplyStore = useMultiplyStore()

const {
  apyDisplay,
} = provideLeveragePosition()

const selectedVault = computed(() => multiplyStore.selectedVault)
const depositPoolData = computed<Pool | undefined>(() => selectedVault.value?.depositPoolData.pool)

const depositAsset = computed(() => selectedVault.value?.asset)
const borrowAsset = computed(() => selectedVault.value?.borrowAsset)
const price = computed(() => selectedVault.value?.price ?? 0)

const maxMultiplier = computed(() => selectedVault.value?.maxMultiplier ?? 0)

const maxApy = computed(() => {
  const depositApy = (selectedVault.value?.depositPoolData.apy.supply_bps ?? 0) / 100
  const borrowApy = (selectedVault.value?.borrowPoolData.apy.borrow_bps ?? 0) / 100
  return getApyRangeForMultiplier({
    supplyApy: depositApy,
    borrowApy,
    maxMultiplier: maxMultiplier.value,
  }).maxApy || 0
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
      v-if="depositAsset && borrowAsset"
      class="asset-data"
    >
      <img
        :src="depositAsset?.icon"
        alt="asset icon"
      >
      <img
        :src="borrowAsset?.icon"
        alt="asset icon"
        class="xlm-icon"
      >
      <div class="asset-data__coin">
        <span data="name">{{ depositAsset?.symbol }} / {{ borrowAsset?.symbol }}</span>
        <span data="symbol">Multiply {{ depositAsset?.symbol }} with {{ borrowAsset?.symbol }}</span>
      </div>
    </div>

    <template v-if="depositPoolData">
      <market-pill>
        {{ selectedVault?.market ?? '' }} Market
      </market-pill>

      <market-pill v-if="width > 650">
        Price: <span class="text-num"> ${{ formatPrice(price, 2, 2) }}</span>
      </market-pill>
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
            Estimated net APY at the suggested max multiplier ({{ truncatePercent(maxMultiplier, 2) }}x):
            supply APY × multiplier − borrow APY × (multiplier − 1).
            <br>
            The "suggested max" leaves headroom for swap slippage and fees;
            the contract's hard ceiling (1 / (1 − open LTV)) is higher.
            Realized APY varies with price, rate changes, and your chosen multiplier.
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
    gap: 0;
    font-size: 18px;
    font-weight: 500;

    img {
      width: 38px;
      height: 38px;
      border-radius: 50%;
    }

    // Stack the borrow icon over the deposit icon (secondary on top, slightly overlapping).
    // Matches the pattern used by the multiply table, portfolio "My Multiplies" table, and
    // .position-card__icons so all multiply surfaces share one visual language.
    .xlm-icon {
      position: relative;
      margin-left: -14px;
      z-index: 1;
      border: 2px solid $bg-card;
      background-color: $bg-card;
      box-sizing: content-box;
    }

    &__coin {
      display: flex;
      flex-direction: column;
      align-items: flex-start;
      margin-left: 12px;

      span[data='symbol'] {
        color: $text-tertiary;
        font-size: 12px;
        opacity: 0.8;
      }
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
