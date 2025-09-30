<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { bigintToNumber, truncatePercent } from '~/utils'

const { width } = useWindowSize()

const marketsStore = useMarketsStore()

const selectedMarketDetails = inject('selectedMarketDetails') as Ref<MarketTableItem>

const pool = computed(() => selectedMarketDetails.value?.raw)

const assetDecimals = computed(() => marketsStore.assetDecimals)

const utilizationRate = computed(() => {
  if (!pool.value) {
    return '0%'
  }

  const totalBorrowed = Number(bigintToNumber(pool.value.total_borrowed, assetDecimals.value))
  const available = Number(bigintToNumber(pool.value.total_available, assetDecimals.value))
  const totalSupplied = totalBorrowed + available

  const utilization = (totalBorrowed / totalSupplied) * 100

  return `${truncatePercent(utilization || 0, 2)}%`
})
</script>

<template>
  <div class="interest-wrapper">
    <div class="interest-wrapper__title">
      Interest rate
    </div>
    <div class="interest-wrapper__details">
      <div class="interest-rate">
        Utilization Rate
        <span>{{ utilizationRate }}</span>
      </div>

      <div class="separator-vert" />

      <a
        href="#"
        target="_blank"
        class="interest-link"
      >
        Rate Strategy <i-app-export-icon />
      </a>

      <div class="separator-vert hide-xs" />
    </div>
    <div class="separator" />

    <client-only>
      <dynamic-teleport
        :is-teleport="width <= 650"
        to=".market-interest-chart"
      >
        <div class="market-interest-legend">
          <div
            class="market-interest-legend__item"
            :style="{ '--legend-color': '#006CE4' }"
          >
            Borrow APR
          </div>
          <div
            class="market-interest-legend__item"
            :style="{ '--legend-color': '#FFD101' }"
          >
            Utilization Rate
          </div>
        </div>
      </dynamic-teleport>
    </client-only>
  </div>
</template>

<style lang="scss">
.market-info-dialog {
  .interest-wrapper {
    display: flex;
    flex-direction: column;
    gap: $spacing-16;

    &__title {
      font-size: 14px;
      font-style: normal;
      font-weight: 700;
      line-height: 16px;
    }

    &__details {
      padding: 0 $spacing-12;
      display: flex;
      gap: $spacing-24;

      @media (max-width: $breakpoint-xs) {
        justify-content: space-between;
      }
    }

    .interest-rate {
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

    .interest-link {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 10px;
      color: $neutral-6;
      font-size: 11px;
      font-style: normal;
      font-weight: 500;
      line-height: 12px;
      text-decoration: none;

      svg {
        color: $dark;
        margin-bottom: -2px;
      }
    }
  }
}

.market-interest-legend {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: $spacing-12;
  font-size: 11px;
  font-style: normal;
  font-weight: 500;
  line-height: 12px;

  @media (max-width: $breakpoint-xs) {
    justify-content: center;
    padding: $spacing-12 0 $spacing-16;
  }

  &__item {
    display: flex;
    align-items: center;
    gap: $spacing-4;

    &::before {
      content: '';
      display: block;
      width: 12px;
      height: 12px;
      border-radius: 50px;
      background-color: var(--legend-color);
    }
  }
}

body.body--dark {
  .market-info-dialog .interest-wrapper {
    .interest-rate span {
      color: #fff;
    }

    .interest-link {
      color: $neutral-9;

      svg {
        color: $neutral-9;
      }
    }
  }
}
</style>
