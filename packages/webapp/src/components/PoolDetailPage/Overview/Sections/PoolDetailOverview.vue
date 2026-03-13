<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const selectedPool = inject('selectedPool') as Ref<MarketTableItem>

const pool = computed(() => selectedPool.value?.raw?.pool)

const detailCardsData = computed(() => {
  if (!pool.value) {
    return {
      liquidationCloseFactor: '-',
      maxLiquidationIncentive: '-',
      withdrawScarcityLimitBps: '-',
      targetUtilizationRate: '-',
    }
  }

  const liquidationCloseFactor = Number(pool.value?.config.health_config.liquidation_close_factor_bps) / 100
  const maxLiquidationIncentive = Number(pool.value?.config.health_config.max_liquidation_incentive_bps) / 100
  const withdrawScarcityLimitBps = Number(pool.value?.config.health_config.withdraw_scarcity_limit_bps) / 100
  const targetUtilizationRate = Number(pool.value?.target_utilization_ratio_bps || 0) / 100

  return {
    liquidationCloseFactor: truncatePercent(liquidationCloseFactor || 0, 2),
    maxLiquidationIncentive: truncatePercent(maxLiquidationIncentive || 0, 2),
    withdrawScarcityLimitBps: truncatePercent(withdrawScarcityLimitBps || 0, 2),
    targetUtilizationRate: truncatePercent(targetUtilizationRate || 0, 2),
  }
})
</script>

<template>
  <section id="pool-info-overview">
    <div class="stat-card">
      <div class="stat-card__header">
        <h3 class="title">
          Pool Overview
        </h3>
      </div>

      <div class="stat-card__body">
        <div class="info-list">
          <div class="info-list__item">
            <div class="title">
              Liquidation Close Factor
            </div>
            <div class="value">
              {{ detailCardsData.liquidationCloseFactor }}%
            </div>
          </div>

          <div class="info-list__item">
            <div class="title">
              Withdraw Scarcity Limit
            </div>
            <div class="value">
              {{ detailCardsData.withdrawScarcityLimitBps }}%
            </div>
          </div>

          <div class="info-list__item">
            <div class="title">
              Max Liquidation Incentive
            </div>
            <div class="value">
              {{ detailCardsData.maxLiquidationIncentive }}%
            </div>
          </div>

          <div class="info-list__item">
            <div class="title">
              Target Utilization Rate
            </div>
            <div class="value">
              {{ detailCardsData.targetUtilizationRate }}%
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
