<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const selectedPool = inject<Ref<MarketTableItem>>('selectedPool')

const {
  poolBorrowLimit,
} = useBorrowDialog(selectedPool, toRef(false))

const poolData = computed(() => {
  const assetSymbol = selectedPool?.value?.asset?.symbol ?? '-'
  return {
    assetSymbol,
    supplied: selectedPool?.value?.total_supply ?? 0,
    borrowed: selectedPool?.value?.total_borrowed ?? 0,
    Limit: Number(selectedPool?.value?.raw?.pool?.config.health_config.utilization_ratio_limit_bps || 0) / 100,
    price: selectedPool?.value?.price ?? 0,
  }
})
</script>

<template>
  <section
    id="stats"
  >
    <div class="stat-cards">

      <!-- Supply -->
      <div class="stat-card stat-card--small">
        <div class="stat-title">
          <i-metrics-trend-up style="color: #22d3ee;" /> Total supplied
        </div>
        <div class="stat-value">
          <span>{{ shortenNumber(poolData.supplied) }}</span> {{ poolData.assetSymbol }}
        </div>
      </div>

      <!-- Borrow -->
      <div class="stat-card stat-card--small">
        <div class="stat-title">
          <i-metrics-trend-down style="color: #8a8df4;" /> Total borrowed
        </div>
        <div class="stat-value">
          <span>{{ shortenNumber(poolData.borrowed) }}</span> {{ poolData.assetSymbol }}
        </div>
      </div>

      <!-- Liquidity -->
      <div class="stat-card stat-card--small">
        <div class="stat-title">
          <i-metrics-liquidity style="color: #6b7a94;" /> Liquidity
        </div>
        <div class="stat-value">
          <span>{{ shortenNumber(poolBorrowLimit) }}</span> {{ poolData.assetSymbol }}
        </div>
      </div>

      <!-- Utilization -->
      <div class="stat-card stat-card--small">
        <div class="stat-title">
          <i-metrics-pulse style="color: #f43f5e;" /> Utilization Limit
          <info-tooltip>
            Maximum allowed pool utilization.
            <br>
            Borrowing is disabled once this limit is reached
          </info-tooltip>
        </div>
        <div class="stat-value">
          <span>{{ poolData.Limit.toFixed(2) }}%</span>
        </div>
      </div>

      <!-- Price -->
      <div class="stat-card stat-card--small">
        <div class="stat-title">
          <i-metrics-percent style="color: #10b981;" /> Price
        </div>
        <div class="stat-value">
          <span>{{ formatCompactUSD(poolData.price, 2, 2) }}</span>
        </div>
      </div>
    </div>
  </section>
</template>

<style lang="scss">
section#stats {
  .stat-cards {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 12px;

    .stat-card--small {
      padding: 14px 16px;
      display: flex;
      flex-direction: column;
      gap: 4px;

      .stat-title {
        color: $text-tertiary;
        font-size: 10px;
        display: flex;
        align-items: center;
        gap: 6px;

        svg {
          width: 12px;
          height: 12px;
        }
      }

      .stat-value {
        color: $text-tertiary;
        font-size: 10px;
        display: flex;
        align-items: flex-end;
        gap: 4px;

        span {
          font-family: $font-JetBrainsMono;
          font-size: 18px;
          font-weight: 700;
          color: $text-primary;
          line-height: 23px;
        }
      }
    }
  }
}
</style>
