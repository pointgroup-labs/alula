<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { bigintToNumber, truncatePercent } from '~/utils'

const marketsStore = useMarketsStore()

const selectedPool = inject('selectedPool') as Ref<MarketTableItem>

const pool = computed(() => selectedPool.value?.raw?.pool)

const totalSupplied = computed(() => Number(bigintToNumber(pool.value?.total_borrowed + pool.value?.total_available, marketsStore.assetDecimals)) || 0)

const ltvCardData = computed(() => {
  if (!pool.value) {
    return {
      closeLTV: 0,
      openLTV: 0,
    }
  }
  const closeLTV = Number(pool.value?.config.health_config.close_ltv_bps) / 100
  const openLTV = Number(pool.value?.config.health_config.open_ltv_bps) / 100
  const insolvencyLTV = Number(pool.value?.config.health_config.insolvency_ltv_bps) / 10_000

  return {
    closeLTV: truncatePercent(closeLTV || 0, 2),
    openLTV: truncatePercent(openLTV || 0, 2),
    insolvencyLTV: truncatePercent(insolvencyLTV || 0, 2),
  }
})

const liquidationCardData = computed(() => {
  if (!pool.value) {
    return {
      liquidationPenalty: 0,
    }
  }
  const liquidationPenalty = (Number(pool.value?.config.health_config.max_liquidation_incentive_bps) / 100).toFixed(0)
  const accrualModel = pool.value?.config?.accrual_model?.tag ?? '-'
  const interestRateModel = pool.value?.config?.interest_rate_model?.tag ?? '-'
  return {
    liquidationPenalty,
    accrualModel,
    interestRateModel,
  }
})

const isSupplyLimit = computed(() => Number(pool.value?.config.health_config.supply_limit) > 0)
const supplyLimit = computed(() => isSupplyLimit.value ? Number(bigintToNumber(pool.value?.config.health_config.supply_limit, marketsStore.assetDecimals)) : 0)

const progress = computed(() => isSupplyLimit.value ? Number(totalSupplied.value / supplyLimit.value * 100).toFixed(2) : 100)
</script>

<template>
  <section
    id="supply"
    class="market-supply-overview"
  >
    <div class="market-stats-cards">
      <div class="stat-card apy">
        <div class="stat-title">
          <svg
            class="icon-apy"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
          >
            <circle
              cx="12"
              cy="12"
              r="10"
              stroke="#1dbf73"
              stroke-width="2"
            />
            <path
              d="M8 12l2.5 2.5L16 9"
              stroke="#1dbf73"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          LTV
        </div>
        <div class="stat-sub">
          <span class="stat-sub__title">Close LTV:</span>
          <span class="stat-sub__value">{{ ltvCardData.closeLTV }}%</span>
        </div>
        <div class="stat-sub">
          <span class="stat-sub__title">Open LTV:</span>
          <span class="stat-sub__value">{{ ltvCardData.openLTV }}%</span>
        </div>
        <div class="stat-sub">
          <span class="stat-sub__title">Insolvency LTV:</span>
          <span class="stat-sub__value">{{ ltvCardData.insolvencyLTV }}%</span>
        </div>
      </div>
      <div class="stat-card supply">
        <div class="stat-title">
          <svg
            class="icon-supply"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
          >
            <circle
              cx="12"
              cy="12"
              r="10"
              stroke="#006CE4"
              stroke-width="2"
            />
            <path
              d="M12 6v6l4 2"
              stroke="#006CE4"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          Supply
        </div>
        <market-progress
          is-progress
          :progress="progress"
          :cap="totalSupplied"
          :limit="supplyLimit"
          :symbol="selectedPool?.asset?.symbol"
          details-color="#006CE4"
        />
      </div>
      <div class="stat-card penalty">
        <div class="stat-title">
          <i-app-info-circle class="info-icon" />
          Details
        </div>
        <div class="stat-sub">
          <span class="stat-sub__title">Liquidation Penalty:</span>
          <span class="stat-sub__value">
            {{ liquidationCardData.liquidationPenalty }}%</span>
        </div>
        <div class="stat-sub">
          <span class="stat-sub__title">Accrual:</span>
          <span class="stat-sub__value">
            {{ liquidationCardData.accrualModel }}</span>
        </div>
        <div class="stat-sub">
          <span class="stat-sub__title">Interest Rate:</span>
          <span class="stat-sub__value">
            {{ liquidationCardData.interestRateModel }}</span>
        </div>
      </div>
    </div>
  </section>
</template>

<style lang="scss">
.market-stats-cards {
  display: flex;
  gap: 24px;

  @media (max-width: 900px) {
    flex-direction: column;
  }

  .stat-card {
    background: #fff;
    border-radius: 16px;
    box-shadow: 0 2px 12px #0001;
    padding: 24px 32px;
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    min-width: 0;

    .market-progress__wrapper {
      width: 100%;
      margin: 0 auto;
    }

    .stat-title {
      font-size: 16px;
      color: #888;
      margin-bottom: 4px;
      display: flex;
      align-items: center;
      gap: 8px;

      svg {
        flex-shrink: 0;
      }
    }

    .stat-sub {
      display: flex;
      align-items: flex-end;
      gap: 8px;

      &:not(:last-child) {
        margin-bottom: 12px;
      }

      &__title {
        font-size: 14px;
        color: #666;
        line-height: 20px;
      }

      &__value {
        font-size: 22px;
        line-height: 22px;
        font-weight: 700;
        color: #000;

        &.positive {
          color: #1dbf73;
        }

        &.warning {
          color: #ff9800;
        }
      }
    }

    &.penalty {
      .info-icon {
        color: $warning;
      }
    }

    .market-progress__wrapper {
      margin-top: auto;
    }

    .market-progress {
      width: 100%;
    }
  }
}
</style>
