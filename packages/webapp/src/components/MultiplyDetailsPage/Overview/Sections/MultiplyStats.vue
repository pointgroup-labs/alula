<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'

const poolData = inject<Ref<MultiplyTableItem>>('selectedPool')

const liquidity = computed(() => poolData?.value?.liquidity ?? 0)
const multiplier = computed(() => poolData?.value?.multiplier ?? 0)
const maxAPY = computed(() => poolData?.value?.maxAPY ?? 0)
const maxLtvPercent = computed(() =>
  Number(poolData?.value.depositPoolData.pool.config.health_config.open_ltv_bps ?? 0) / 100,
)
const liquidationLtvPercent = computed(() =>
  Number(poolData?.value.depositPoolData.pool.config.health_config.insolvency_ltv_bps ?? 0) / 100,
)
const liquidationBuffer = computed(() => Math.max(liquidationLtvPercent.value - maxLtvPercent.value, 0))
const assets = computed(() => {
  return {
    supplyAsset: poolData?.value?.asset,
    borrowAsset: poolData?.value?.borrowAsset,
  }
})
const apyClass = computed(() => {
  switch (true) {
    case maxAPY.value > 0:
      return 'positive'
    case maxAPY.value < 0:
      return 'warning'
    default:
      return 'default'
  }
})
</script>

<template>
  <section id="multiply-stats">
    <h2>Looping Overview</h2>

    <div class="stat-cards">
      <j-tooltip>
        <market-info-detail-card>
          <div class="info-value">
            Liquidity
            <i-app-info-circle class="info-icon" />
          </div>
          <div class="info-label">
            <img
              :src="assets.borrowAsset?.icon"
              alt="token icon"
              class="token-icon"
            > {{ shortenNumber(liquidity) }} {{ assets.borrowAsset?.symbol }}
          </div>
        </market-info-detail-card>
        <template #content>
          {{ formatPrice(liquidity) }} {{ assets.borrowAsset?.symbol }}
        </template>
      </j-tooltip>

      <market-info-detail-card>
        <div class="info-value">
          Max Leverage
        </div>
        <div class="info-label">
          {{ multiplier }}x
        </div>
      </market-info-detail-card>

      <market-info-detail-card>
        <div
          class="info-value"
        >
          Max Net APY
        </div>
        <div
          class="info-label"
          :class="apyClass"
        >
          {{ truncatePercent(maxAPY, 2) }}%
        </div>
      </market-info-detail-card>
    </div>

    <div class="separator" />

    <div class="stats-table">
      <!-- Collateral -->
      <div class="stats-table__item">
        <div class="stat-label">
          Collateral Asset
        </div>
        <div class="stat-value">
          <img
            :src="assets.supplyAsset?.icon"
            alt="collateral asset icon"
          >
          {{ assets.supplyAsset?.symbol }}
        </div>
      </div>

      <!-- Debt -->
      <div class="stats-table__item">
        <div class="stat-label">
          Debt Asset
        </div>
        <div class="stat-value">
          <img
            :src="assets.borrowAsset?.icon"
            alt="collateral asset icon"
          >
          {{ assets.borrowAsset?.symbol }}
        </div>
      </div>

      <!-- MAx LTV -->
      <div class="stats-table__item">
        <div class="stat-label">
          Max LTV
        </div>
        <div class="stat-value">
          {{ maxLtvPercent }}%
        </div>
      </div>

      <!-- Liquidation LTV -->
      <div class="stats-table__item">
        <div class="stat-label">
          Liquidation LTV
        </div>
        <div class="stat-value">
          {{ liquidationLtvPercent }}%
        </div>
      </div>

      <!-- Liquidation LTV -->
      <div class="stats-table__item">
        <div class="stat-label">
          Liquidation Buffer
        </div>
        <div class="stat-value">
          {{ liquidationBuffer }}%
        </div>
      </div>
    </div>
  </section>
</template>

<style lang="scss">
section#multiply-stats {
  .separator {
    margin: 12px 0;
  }

  .stat-cards {
    width: 100%;
    padding: 0 0 $spacing-16;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: $spacing-12;

    [class*='tooltip'] {
      width: 100%;
    }

    .market-info-detail-card {
      gap: $spacing-6;
    }

    .info-label {
      display: flex;
      align-items: center;
      gap: $spacing-6;
      line-height: normal;
    }

    .info-value {
      margin: 0;
      font-size: 16px;
    }

    .token-icon {
      width: 22px;
      height: 22px;
      margin-bottom: -2px;
    }
  }

  .stats-table {
    padding-top: $spacing-16;
    display: block;
    column-count: 2;
    column-gap: 20px;

    &__item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-size: 14px;
      font-weight: 500;
      line-height: 20px;
      margin-bottom: 10px;
      padding: 12px 16px;
      background: linear-gradient(135deg, rgba(249, 250, 251, 0.6) 0%, rgba(255, 255, 255, 0.4) 100%);
      border: 1px solid rgba(226, 232, 240, 0.8);
      border-radius: 10px;
      transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
      position: relative;
      overflow: hidden;
      break-inside: avoid;

      &::before {
        content: '';
        position: absolute;
        left: 0;
        top: 0;
        bottom: 0;
        width: 3px;
        background: linear-gradient(180deg, rgba(99, 102, 241, 0.6) 0%, rgba(139, 92, 246, 0.6) 100%);
        opacity: 0;
        transition: opacity 0.2s ease;
      }

      &:hover {
        background: linear-gradient(135deg, rgba(255, 255, 255, 0.9) 0%, rgba(249, 250, 251, 0.8) 100%);
        border-color: rgba(99, 102, 241, 0.2);
        box-shadow: 0 2px 8px rgba(99, 102, 241, 0.08);
        transform: translateX(2px);

        &::before {
          opacity: 1;
        }
      }
    }

    .stat-label {
      color: #64748b;
      font-weight: 500;
      font-size: 13px;
      letter-spacing: 0.01em;
    }

    .stat-value {
      display: flex;
      align-items: center;
      gap: $spacing-6;
      font-weight: 600;
      color: #1e293b;
      font-size: 14px;

      img {
        width: 18px;
        height: 18px;
        object-fit: contain;
        filter: drop-shadow(0px 1px 2px rgba(0, 0, 0, 0.08));
      }
    }
  }

  .theme-dark & {
    .stats-table {
      &__item {
        background: linear-gradient(135deg, rgba(30, 41, 59, 0.4) 0%, rgba(15, 23, 42, 0.6) 100%);
        border-color: rgba(71, 85, 105, 0.3);

        &:hover {
          background: linear-gradient(135deg, rgba(30, 41, 59, 0.6) 0%, rgba(15, 23, 42, 0.8) 100%);
          border-color: rgba(139, 92, 246, 0.4);
          box-shadow: 0 2px 8px rgba(139, 92, 246, 0.15);
        }
      }

      .stat-label {
        color: #94a3b8;
      }

      .stat-value {
        color: #e2e8f0;
      }
    }
  }
}
</style>
