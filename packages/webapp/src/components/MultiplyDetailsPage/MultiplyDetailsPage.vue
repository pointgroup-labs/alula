<script lang="ts" setup>
import { amountToUsdWithShort, formatCompactUSD, formatPrice, truncatePercent } from '~/utils'

const marketsStore = useMarketsStore()
const loading = computed(() => marketsStore.state.loading)

const { selectedVault } = useMultiplyCatalog()
</script>

<template>
  <main class="multiply-details container">
    <back-btn to="/multiply" />

    <j-loading-spinner
      v-if="loading"
      class="table-loading-spinner"
    >
      Loading market data...
    </j-loading-spinner>

    <template v-else-if="selectedVault">
      <section class="multiply-details__hero">
        <div>
          <div class="multiply-details__eyebrow">
            {{ selectedVault.market }} market
          </div>
          <h1 class="multiply-details__title">
            {{ selectedVault.asset.symbol }}/{{ selectedVault.borrowAsset.symbol }} multiply vault
          </h1>
          <p class="multiply-details__copy">
            This vault opens leveraged {{ selectedVault.asset.symbol }} exposure by using {{ selectedVault.borrowAsset.symbol }} as margin, routing the swap through the provider-resolved Soroswap router, and depositing the slippage-adjusted output as collateral.
          </p>
        </div>

        <div class="multiply-details__hero-stats">
          <div>
            <span>Max multiplier</span>
            <strong>x{{ truncatePercent(selectedVault.maxMultiplier, 2) }}</strong>
          </div>
          <div>
            <span>APY at max multiplier</span>
            <strong>{{ truncatePercent(selectedVault.apyAtMaxMultiplier, 2) }}%</strong>
          </div>
          <div>
            <span>Borrow liquidity</span>
            <strong>{{ formatPrice(selectedVault.liquidity, 2, 2) }} {{ selectedVault.borrowAsset.symbol }}</strong>
          </div>
          <div>
            <span>Collateral TVL</span>
            <strong>${{ amountToUsdWithShort(selectedVault.supplied, selectedVault.price, false) }}</strong>
          </div>
        </div>
      </section>

      <section class="multiply-details__content">
        <div class="multiply-details__overview">
          <div class="multiply-details__card">
            <span>Collateral asset</span>
            <strong>{{ selectedVault.asset.name }}</strong>
            <small>{{ formatCompactUSD(selectedVault.price, 2, 4) }}</small>
          </div>
          <div class="multiply-details__card">
            <span>Margin asset</span>
            <strong>{{ selectedVault.borrowAsset.name }}</strong>
            <small>{{ formatCompactUSD(selectedVault.borrowPoolPrice, 2, 4) }}</small>
          </div>
          <div class="multiply-details__card">
            <span>Deposit pool</span>
            <strong>
              {{ shortenAddress(selectedVault.depositPoolData.pool.pool_address, 20) }}
              <copy-to-clipboard :text="selectedVault.depositPoolData.pool.pool_address" />
            </strong>
          </div>
          <div class="multiply-details__card">
            <span>Borrow pool</span>
            <strong>
              {{ shortenAddress(selectedVault.borrowPoolData.pool.pool_address, 20) }}
              <copy-to-clipboard :text="selectedVault.borrowPoolData.pool.pool_address" />
            </strong>
          </div>
        </div>

        <multiply-window :vault="selectedVault" />
      </section>
    </template>

    <div
      v-else
      class="multiply-details__empty"
    >
      Market or pool not found.
    </div>
  </main>
</template>

<style lang="scss">
.multiply-details {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding-bottom: 72px;

  &__hero {
    display: grid;
    grid-template-columns: minmax(0, 1.4fr) minmax(320px, 0.9fr);
    gap: 20px;
    padding: 32px;
    border-radius: 32px;
    background:
      radial-gradient(circle at top left, rgba(34, 211, 238, 0.12), transparent 34%),
      linear-gradient(180deg, rgba(17, 24, 39, 0.96) 0%, rgba(13, 18, 31, 0.96) 100%);
    border: 1px solid $border-primary;
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.24);

    @media (max-width: $breakpoint-lg) {
      grid-template-columns: 1fr;
    }
  }

  &__eyebrow {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: $text-brand;
  }

  &__title {
    margin: 10px 0;
    font-size: clamp(28px, 4vw, 44px);
    line-height: 1;
    color: $text-primary;
  }

  &__copy {
    margin: 0;
    max-width: 720px;
    line-height: 1.7;
    color: $text-tertiary;
  }

  &__hero-stats,
  &__overview {
    height: fit-content;
    display: grid;
    gap: 14px;
    grid-template-columns: repeat(2, minmax(0, 1fr));

    @media (max-width: $breakpoint-sm) {
      grid-template-columns: 1fr;
    }
  }

  &__hero-stats > div,
  &__card {
    height: fit-content;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 16px;
    border-radius: 18px;
    background: $surface-neutral-04;
    border: 1px solid $surface-neutral-08;

    span,
    small {
      color: $text-tertiary;
      font-size: 12px;
    }

    strong {
      font-size: 15px;
      color: $text-primary;
      word-break: break-all;
      display: flex;
      align-items: center;
      gap: 12px;
    }
  }

  &__content {
    display: flex;
    justify-content: space-between;
    gap: 20px;
  }

  &__empty {
    padding: 32px;
    border-radius: 24px;
    background: $bg-card;
    border: 1px solid $border-primary;
    color: $text-secondary;
    text-align: center;
  }

  .multiply-trade-panel {
    max-width: 500px;
    width: 100%;
    background-color: $bg-card;
    padding: $spacing-xl;
    border-radius: $radius-2xl;
    border: 1px solid $border-primary;
  }
}
</style>
