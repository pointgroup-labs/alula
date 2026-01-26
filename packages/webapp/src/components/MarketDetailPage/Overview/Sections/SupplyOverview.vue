<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { bigintToNumber } from '~/utils'

const marketsStore = useMarketsStore()

const selectedPool = inject('selectedPool') as Ref<MarketTableItem>

const pool = computed(() => selectedPool.value?.raw?.pool)

const detailCardsData = computed(() => {
  if (!pool.value) {
    return {
      closeLTV: 0,
      openLTV: 0,
      depositApy: 0,
      depositFee: 0,
      repayFee: 0,
    }
  }
  const closeLTV = Number(pool.value?.config.health_config.close_ltv_bps) / 100
  const openLTV = Number(pool.value?.config.health_config.open_ltv_bps) / 100

  const depositFee = Number(pool.value?.config.fee_config.deposit_fee_bps) / 100
  const repayFee = Number(pool.value?.config.fee_config.repay_fee_bps) / 100

  const price = Number(selectedPool.value?.price || 0)

  const depositApy = selectedPool.value?.deposit_apy ?? '0%'
  return {
    depositApy,
    closeLTV: truncatePercent(closeLTV || 0, 2),
    openLTV: truncatePercent(openLTV || 0, 2),
    depositFee: truncatePercent(depositFee || 0, 2),
    repayFee: truncatePercent(repayFee || 0, 2),
    price: `$${formatPrice(price, 2, 2)}`,
  }
})

const isSupplyLimit = computed(() => Number(pool.value?.config.health_config.supply_limit) > 0)
const supplyLimit = computed(() => isSupplyLimit.value ? Number(bigintToNumber(pool.value?.config.health_config.supply_limit, marketsStore.assetDecimals)) : 0)

const totalSupplied = computed(() => Number(bigintToNumber(pool.value?.total_borrowed + pool.value?.total_available, marketsStore.assetDecimals)) || 0)

const totalSuppliedInUsd = computed(() => totalSupplied.value * selectedPool.value?.price || 0)
const supplyLimitInUsd = computed(() => supplyLimit.value * selectedPool.value?.price || 0)
const progress = computed(() => isSupplyLimit.value ? Number(totalSupplied.value / supplyLimit.value * 100).toFixed(2) : 100)
</script>

<template>
  <section
    id="supply"
  >
    <div class="stat-card">
      <market-history-chart-supply />
    </div>
    <div class="market-stats-cards">
      <div class="stat-card">
        <div class="stat-title">
          <svg
            class="icon-supply"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
          >
            <path
              d="M12 2l6 6h-4v8h-4V8H6l6-6z"
              fill="#006CE4"
            />
            <rect
              x="4"
              y="20"
              width="16"
              height="2"
              fill="#006CE4"
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
        </market-progress>
      </div>
      <div class="cards-list">
        <market-info-card>
          <div class="info-value">
            Open LTV
          </div>
          <div class="info-label">
            {{ detailCardsData.openLTV }}%
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Close LTV
          </div>
          <div class="info-label">
            {{ detailCardsData.closeLTV }}%
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Supply APY
          </div>
          <div class="info-label positive">
            {{ detailCardsData.depositApy }}
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Deposit Fee
          </div>
          <div class="info-label">
            {{ detailCardsData.depositFee }}%
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Rapay Fee
          </div>
          <div class="info-label">
            {{ detailCardsData.repayFee }}%
          </div>
        </market-info-card>
        <market-info-card>
          <div class="info-value">
            Price
          </div>
          <div class="info-label">
            {{ detailCardsData.price }}
          </div>
        </market-info-card>
      </div>
    </div>
  </section>
</template>
