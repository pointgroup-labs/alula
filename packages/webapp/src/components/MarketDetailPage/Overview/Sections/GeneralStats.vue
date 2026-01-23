<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const selectedPool = inject<Ref<MarketTableItem>>('selectedPool')

const {
  availableToBorrow,
} = useBorrowDialog(selectedPool, false)
</script>

<template>
  <section
    id="stats"
    class="market-general-stats"
  >
    <div class="stat-cards">
      <!-- Supplied -->
      <j-tooltip>
        <market-info-card>
          <div class="info-label">
            {{ shortenNumber(selectedPool?.total_supply ?? 0) }} {{ selectedPool?.asset?.symbol }}
          </div>
          <div class="info-value">
            <img
              :src="selectedPool?.asset?.icon"
              alt="token icon"
              class="token-icon"
            >  Supplied
          </div>
        </market-info-card>
        <template #content>
          {{ formatPrice(selectedPool?.total_supply ?? 0) }} {{ selectedPool?.asset?.symbol }}
        </template>
      </j-tooltip>

      <!-- Borrowed -->
      <j-tooltip>
        <market-info-card>
          <div class="info-label">
            {{ shortenNumber(selectedPool?.total_borrowed ?? 0) }} {{ selectedPool?.asset?.symbol }}
          </div>
          <div class="info-value">
            <img
              :src="selectedPool?.asset?.icon"
              alt="token icon"
              class="token-icon"
            >  Borrowed
          </div>
        </market-info-card>
        <template #content>
          {{ formatPrice(selectedPool?.total_borrowed ?? 0) }} {{ selectedPool?.asset?.symbol }}
        </template>
      </j-tooltip>

      <!-- Liquidity -->
      <j-tooltip>
        <market-info-card>
          <div class="info-label">
            {{ shortenNumber(availableToBorrow ?? 0) }} {{ selectedPool?.asset?.symbol }}
          </div>
          <div class="info-value">
            <img
              :src="selectedPool?.asset?.icon"
              alt="token icon"
              class="token-icon"
            >
            Liquidity
          </div>
        </market-info-card>
        <template #content>
          {{ formatPrice(availableToBorrow ?? 0) }} {{ selectedPool?.asset?.symbol }}
        </template>
      </j-tooltip>

      <!-- Supply APY -->
      <market-info-card>
        <div class="info-label positive">
          {{ selectedPool?.deposit_apy }}
        </div>
        <div class="info-value">
          Supply APY
        </div>
      </market-info-card>

      <!-- Borrow APY -->
      <market-info-card>
        <div class="info-label warning">
          {{ selectedPool?.borrow_apy }}
        </div>
        <div class="info-value">
          Borrow APY
        </div>
      </market-info-card>
    </div>
  </section>
</template>

<style lang="scss">
section#stats {
  .stat-cards {
    width: 100%;
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    grid-template-areas: 'first second third fourth fifth';
    gap: $spacing-12;

    @media (max-width: 1215px) {
      grid-template-columns: repeat(6, 1fr);
      grid-template-areas:
        'first first second second third third'
        'fourth fourth fourth fifth fifth fifth';
    }

    @media (max-width: $breakpoint-sm) {
      grid-template-columns: repeat(2, 1fr);
      grid-template-areas:
        'first second'
        'third fourth'
        'fifth fifth';
    }

    & > div {
      display: flex;
      width: 100%;

      &:nth-child(1) {
        grid-area: first;
      }

      &:nth-child(2) {
        grid-area: second;
      }

      &:nth-child(3) {
        grid-area: third;
      }

      &:nth-child(4) {
        grid-area: fourth;
      }

      &:nth-child(5) {
        grid-area: fifth;
      }
    }

    .market-info-card {
      flex: 1;
    }

    .token-icon {
      width: 16px;
      height: 16px;
      object-fit: contain;
    }

    .info-label {
      &.positive {
        color: $success;
      }
      &.warning {
        color: $warning;
      }
    }
  }
}
</style>
