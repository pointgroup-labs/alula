<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { bigintToNumber, shortenNumber, truncatePercent } from '~/utils'

const { width } = useWindowSize()

const marketsStore = useMarketsStore()

const selectedMarketDetails = inject('selectedMarketDetails') as Ref<MarketTableItem>

const pool = computed(() => selectedMarketDetails.value?.raw)

const totalSupplied = computed(() => Number(bigintToNumber(pool.value?.total_borrowed + pool.value?.total_available, marketsStore.assetDecimals)) || 0)

const closeLTV = computed(() => {
  if (!pool.value) {
    return 0
  }
  const closeLtv = Number(pool.value?.config.health_config.close_ltv_bps) / 100
  return truncatePercent(closeLtv || 0, 2)
})

const openLTV = computed(() => {
  if (!pool.value) {
    return 0
  }
  const closeLtv = Number(pool.value?.config.health_config.open_ltv_bps) / 100
  return truncatePercent(closeLtv || 0, 2)
})

const liquidationPenalty = computed(() => (Number(pool.value?.config.health_config.liquidation_close_factor_bps) / 100).toFixed(0))

const isSupplyLimit = computed(() => Number(pool.value?.config.health_config.supply_limit) > 0)
const supplyLimit = computed(() => isSupplyLimit.value ? Number(bigintToNumber(pool.value?.config.health_config.supply_limit, marketsStore.assetDecimals)) : 0)

const totalSuppliedInUsd = computed(() => totalSupplied.value * selectedMarketDetails.value?.price || 0)
const supplyLimitInUsd = computed(() => supplyLimit.value * selectedMarketDetails.value?.price || 0)
const progress = computed(() => isSupplyLimit.value ? Number(totalSupplied.value / supplyLimit.value * 100).toFixed(2) : 100)
</script>

<template>
  <div class="market-details">
    <div class="market-details__title">
      Supply Details
    </div>

    <div class="market-stats">
      <div
        class="market-stats__apy"
      >
        <div class="stats-apy">
          Supply APY
          <span>{{ selectedMarketDetails?.deposit_apy || '-' }}</span>
        </div>
        <div class="stats-params">
          <div class="stats-params__item">
            Close LTV:
            <span>{{ closeLTV }}%</span>
          </div>
          <div class="stats-params__item">
            Open LTV:
            <span>{{ openLTV }}%</span>
          </div>
        </div>
      </div>

      <div class="separator-vert" />

      <market-progress
        is-progress
        :progress="progress"
        :cap="totalSupplied"
        :limit="supplyLimit"
        details-color="#006CE4"
      >
        <div class="market-progress__info">
          <div class="market-progress__info__title">
            Total Supply
          </div>
          <div class="market-progress__info__data">
            {{ shortenNumber(totalSupplied) }} / {{ isSupplyLimit ? shortenNumber(supplyLimit) : '-' }}

            <span>${{ shortenNumber(totalSuppliedInUsd, 2) }} / {{ isSupplyLimit ? `$${shortenNumber(supplyLimitInUsd, 2)}` : '-' }}</span>
          </div>
        </div>
        <div
          v-if="width <= 650"
          class="market-penalty"
        >
          Liquidation Penalty:

          <span>{{ liquidationPenalty }}%</span>
        </div>
      </market-progress>

      <div class="separator-vert hide-xs" />

      <div class="market-penalty hide-xs">
        Liquidation Penalty:

        <span>{{ liquidationPenalty }}%</span>
      </div>
    </div>

    <div class="separator" />

    <market-history-chart-supply />
  </div>
</template>

<style lang="scss">
.market-info-dialog {
  .market-details {
    width: 504px;
    display: flex;
    flex-direction: column;
    gap: $spacing-16;

    @media (max-width: $breakpoint-xs) {
      width: 100%;
    }

    &__title {
      padding: 0 $spacing-12;
      font-size: 14px;
      font-style: normal;
      font-weight: 700;
      line-height: 16px;
    }
  }

  .market-stats {
    display: flex;
    justify-content: space-between;
    align-items: stretch;
    gap: $spacing-16;
    padding: 0 $spacing-12;

    &__apy {
      width: 110px;
      display: flex;
      flex-direction: column;
      gap: 6px;

      .stats-apy {
        display: flex;
        flex-direction: column;
        gap: 2px;
        color: $neutral-12;
        font-size: 12px;
        font-style: normal;
        font-weight: 500;
        line-height: 16px;

        span {
          color: $dark;
          font-size: 20px;
          font-style: normal;
          font-weight: 700;
          line-height: 20px;
        }
      }

      .stats-params {
        display: flex;
        flex-direction: column;

        &__item {
          color: $neutral-6;
          font-size: 11px;
          font-style: normal;
          font-weight: 500;
          line-height: 12px;
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 4px;
          white-space: nowrap;

          span {
            color: $dark;
            font-size: 12px;
            font-style: normal;
            font-weight: 500;
            line-height: 16px;
          }
        }
      }
    }

    .market-penalty {
      width: 105px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      color: $neutral-6;
      font-size: 11px;
      font-style: normal;
      font-weight: 500;
      line-height: 12px;

      @media (max-width: $breakpoint-xs) {
        width: 100%;
        justify-content: flex-start;
        gap: $spacing-12;
      }

      span {
        color: $dark;
        font-size: 12px;
        font-weight: 500;
        line-height: 16px;
      }
    }
  }
}

body.body--dark {
  .market-info-dialog {
    .market-stats__apy {
      .stats-apy {
        color: $neutral-12;

        span {
          color: #fff;
        }
      }

      .stats-params__item {
        color: $neutral-12;

        span {
          color: $neutral-9;
        }
      }
    }

    .market-progress__info__title {
      color: $neutral-7;
    }

    .market-progress__info__data span {
      color: $neutral-9;
    }

    .market-penalty {
      color: $neutral-9;

      span {
        color: $neutral-9;
      }

      svg {
        color: $neutral-9;
      }
    }
  }
}
</style>
