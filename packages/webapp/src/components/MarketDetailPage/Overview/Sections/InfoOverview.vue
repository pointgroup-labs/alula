<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const selectedPool = inject('selectedPool') as Ref<MarketTableItem>

const pool = computed(() => selectedPool.value?.raw?.pool)

const detailCardsData = computed(() => {
  if (!pool.value) {
    return {
      accrualModel: '-',
      interestRateModel: '-',
      liquidationCloseFactor: '-',
      maxLiquidationIncentive: '-',
      insolvencyLTV: '-',
      targetUtilizationRate: '-',
      poolStatus: '-',
      lastAccrual: '-',
      poolAddressShort: '-',
    }
  }

  const liquidationCloseFactor = Number(pool.value?.config.health_config.liquidation_close_factor_bps) / 100
  const maxLiquidationIncentive = Number(pool.value?.config.health_config.max_liquidation_incentive_bps) / 100
  const insolvencyLTV = Number(pool.value?.config.health_config.insolvency_ltv_bps) / 100
  const targetUtilizationRate = Number(pool.value?.target_utilization_ratio_bps || 0) / 100

  const lastAccrual = pool.value?.last_accrual_timestamp
    ? formatDateToDMY(new Date(Number(pool.value.last_accrual_timestamp) * 1000))
    : '-'

  const poolAddressShort = pool.value?.pool_address
    ? `${pool.value.pool_address.slice(0, 6)}...${pool.value.pool_address.slice(-4)}`
    : '-'

  return {
    accrualModel: pool.value?.config.accrual_model.tag ?? '-',
    interestRateModel: pool.value?.config.interest_rate_model.tag ?? '-',
    liquidationCloseFactor: truncatePercent(liquidationCloseFactor || 0, 2),
    maxLiquidationIncentive: truncatePercent(maxLiquidationIncentive || 0, 2),
    insolvencyLTV: truncatePercent(insolvencyLTV || 0, 2),
    targetUtilizationRate: truncatePercent(targetUtilizationRate || 0, 2),
    lastAccrual,
    poolAddressShort,
  }
})
</script>

<template>
  <section
    id="info"
    class="market-supply-overview"
  >
    <div class="stat-card">
      <market-interest-chart />
    </div>
    <div class="market-stats-cards">
      <div class="stat-card">
        <div class="stat-title">
          <i-app-info-circle />
          Market Info
        </div>

        <div class="market-model-card">
          <span :style="{ '--drop-color': '#006ce4' }" /> Current Utilization Rate
        </div>

        <div class="market-model-card">
          <span :style="{ '--drop-color': '#ffb726' }" /> Optimal Utilization Rate
        </div>

        <div
          class="separator"
          style="margin: 10px 0;"
        />

        <div class="market-info-details">
          <div class="info-row">
            <span class="info-label">Last Accrual:</span>
            <span class="info-value">{{ detailCardsData.lastAccrual }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">Pool Address:</span>
            <span class="info-value">{{ detailCardsData.poolAddressShort }}</span>
          </div>
        </div>

      </div>

      <div class="cards-list">
        <market-info-card>
          <div class="info-value">
            Accrual Model
          </div>
          <div class="info-label">
            {{ detailCardsData.accrualModel }}
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Interest Rate Model
          </div>
          <div class="info-label">
            {{ detailCardsData.interestRateModel }}
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Liquidation Close Factor
          </div>
          <div class="info-label">
            {{ detailCardsData.liquidationCloseFactor }}%
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Max Liquidation Incentive
          </div>
          <div class="info-label">
            {{ detailCardsData.maxLiquidationIncentive }}%
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Insolvency LTV
          </div>
          <div class="info-label">
            {{ detailCardsData.insolvencyLTV }}%
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Target Utilization Rate
          </div>
          <div class="info-label">
            {{ detailCardsData.targetUtilizationRate }}%
          </div>
        </market-info-card>
      </div>
    </div>
  </section>
</template>

<style scoped>
.market-info-details {
  width: 100%;
}

.info-row {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.info-row:last-child {
  margin-bottom: 0;
}

.info-row .info-label {
  color: #888;
  font-weight: 500;
}

.info-row .info-value {
  color: #333;
  font-weight: 600;
}
</style>
