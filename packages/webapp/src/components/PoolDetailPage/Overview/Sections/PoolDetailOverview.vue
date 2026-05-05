<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const selectedPool = inject('selectedPool') as Ref<MarketTableItem>

const pool = computed(() => selectedPool.value?.raw?.pool)
const optimalUtilizationRate = computed(() => {
  const interestRateModel = pool.value?.config.interest_rate_model?.values?.[0]
  if (!interestRateModel) {
    return null
  }

  return Number(interestRateModel.kink1_ur_bps ?? 8000n) / 100
})

const detailCardsData = computed(() => {
  if (!pool.value) {
    return {
      liquidationCloseFactor: '-',
      maxLiquidationIncentive: '-',
      withdrawScarcityLimitBps: '-',
      optimalUtilizationRate: '-',
    }
  }

  const liquidationCloseFactor = Number(pool.value?.config.health_config.liquidation_close_factor_bps) / 100
  const maxLiquidationIncentive = Number(pool.value?.config.health_config.max_liquidation_incentive_bps) / 100
  const withdrawScarcityLimitBps = Number(pool.value?.config.health_config.withdraw_scarcity_limit_bps) / 100

  return {
    liquidationCloseFactor: truncatePercent(liquidationCloseFactor || 0, 2),
    maxLiquidationIncentive: truncatePercent(maxLiquidationIncentive || 0, 2),
    withdrawScarcityLimitBps: truncatePercent(withdrawScarcityLimitBps || 0, 2),
    optimalUtilizationRate: optimalUtilizationRate.value && truncatePercent(optimalUtilizationRate.value || 0, 2),
  }
})
</script>

<template>
  <section id="pool-info-overview">
    <div class="stat-card">
      <div class="stat-card__body">
        <div class="info-list">
          <div class="info-list__item">
            <div class="title">
              Close Factor

              <info-tooltip>
                The maximum percentage of a debt that can be repaid in a single liquidation transaction. This mechanism
                allows a position to be gradually restored to health without selling all collateral at once.
              </info-tooltip>
            </div>
            <div class="value">
              {{ detailCardsData.liquidationCloseFactor }}%
            </div>
          </div>

          <div class="info-list__item">
            <div class="title">
              Max Scarcity Rate

              <info-tooltip>
                The peak borrow interest rate reached at 100% pool utilization. This high rate incentivizes borrowers to
                repay their loans, ensuring there is always liquidity available for suppliers to withdraw their funds.
              </info-tooltip>
            </div>
            <div class="value">
              {{ detailCardsData.withdrawScarcityLimitBps }}%
            </div>
          </div>

          <div class="info-list__item">
            <div class="title">
              Liquidation Bonus

              <info-tooltip>
                A discount provided to liquidators on the purchase of collateral when they repay a user's debt. This
                incentive ensures that risky positions are closed quickly to maintain protocol stability.
              </info-tooltip>
            </div>
            <div class="value">
              {{ detailCardsData.maxLiquidationIncentive }}%
            </div>
          </div>

          <div class="info-list__item">
            <div class="title">
              Optimal Utilization

              <info-tooltip>
                The target pool usage level where the interest rate curve reaches its "kink". Beyond this point, the
                cost of borrowing increases sharply to prevent liquidity shortages and protect the protocol's health
              </info-tooltip>
            </div>
            <div class="value">
              <template v-if="optimalUtilizationRate">
                {{ detailCardsData.optimalUtilizationRate }}%
              </template>
              <template v-else>
                -
              </template>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
