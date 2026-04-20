<script lang="ts" setup>
import type { Pool } from '@alula/market-sdk'
import { bigintToNumber } from '~/utils'

const multiplyStore = useMultiplyStore()
const selectedVault = computed(() => multiplyStore.selectedVault)

const pool = computed<Pool | undefined>(() => selectedVault.value?.depositPoolData.pool)

const detailCardsData = computed(() => {
  if (!pool.value) {
    return {
      closeLTV: 0,
      openLTV: 0,
      depositApy: 0,
      depositFee: 0,
      withdrawFee: 0,
    }
  }
  const closeLTV = Number(pool.value?.config.health_config.close_ltv_bps) / 100
  const openLTV = Number(pool.value?.config.health_config.open_ltv_bps) / 100

  const depositFee = Number(pool.value?.config.fee_config.deposit_fee_bps) / 100
  const withdrawFee = Number(pool.value?.config.fee_config.withdraw_fee_bps) / 100

  const price = Number(selectedVault.value?.price || 0)

  const apyRaw = Number(selectedVault.value?.depositPoolData.apy.supply_bps ?? 0) / 100 || 0
  const depositApy = `${truncatePercent(apyRaw)}%`
  return {
    depositApy,
    closeLTV: truncatePercent(closeLTV || 0, 2),
    openLTV: truncatePercent(openLTV || 0, 2),
    depositFee: truncatePercent(depositFee || 0, 2),
    withdrawFee: truncatePercent(withdrawFee || 0, 2),
    price: formatCompactUSD(price, 2, 2),
  }
})

const isSupplyLimit = computed(() => Number(pool.value?.config.health_config.supply_limit) > 0)
const supplyLimit = computed(() => isSupplyLimit.value ? Number(bigintToNumber(pool.value?.config.health_config.supply_limit ?? 0n, pool.value?.token_decimals ?? 7)) : 0)
const totalSupplied = computed(() => Number(bigintToNumber(((pool.value?.total_borrowed ?? 0n) + (pool.value?.total_available ?? 0n)) || 0n, pool.value?.token_decimals ?? 7)) || 0)

const totalSuppliedInUsd = computed(() => totalSupplied.value * (selectedVault.value?.price ?? 0) || 0)
const supplyLimitInUsd = computed(() => supplyLimit.value * (selectedVault.value?.price ?? 0) || 0)
const progress = computed(() => isSupplyLimit.value ? Number(totalSupplied.value / supplyLimit.value * 100).toFixed(2) : 100)
</script>

<template>
  <div class="pool-card stat-card stat-card--small">
    <div class="stat-card__header">
      <h3 class="pool-card-title">
        Collateral
      </h3>

      <j-pill-label
        size="sm"
        style="margin-left: auto;"
      >
        Supply APY {{ detailCardsData?.depositApy }}
      </j-pill-label>
    </div>

    <div class="stat-card__body">
      <market-progress
        is-progress
        :progress="Number(progress).toFixed(1)"
        color="#22d3ee"
      >
        <div class="progress-content">
          <div class="market-progress__info">
            <div class="market-progress__info__title">
              Supplied
            </div>
            <div class="market-progress__info__data">
              {{ shortenNumber(totalSupplied, 1, 1) }}
              <span>/ {{ formatCompactUSD(totalSuppliedInUsd, 1, 1) }}</span>
            </div>
          </div>

          <div class="separator-vert" />

          <div class="market-progress__info">
            <div class="market-progress__info__title">
              Supply Cap
            </div>
            <div class="market-progress__info__data">
              {{ isSupplyLimit ? shortenNumber(supplyLimit, 1, 1) : '-' }}
              <span>/ {{ formatCompactUSD(supplyLimitInUsd, 1, 1) }}</span>
            </div>
          </div>
        </div>
      </market-progress>

      <div class="detail-list">
        <div class="detail-list__item">
          <div class="detail-list__item__label">
            Max LTV

            <info-tooltip>
              Maximum loan-to-value ratio allowed when opening a borrow position.
            </info-tooltip>
          </div>
          <div class="detail-list__item__value">
            {{ detailCardsData.openLTV }}%
          </div>
        </div>

        <div class="detail-list__item">
          <div class="detail-list__item__label">
            Liquidation Threshold

            <info-tooltip>
              Loan-to-value ratio at which a position becomes eligible for liquidation.
            </info-tooltip>
          </div>
          <div class="detail-list__item__value">
            {{ detailCardsData.closeLTV }}%
          </div>
        </div>

        <div class="detail-list__item">
          <div class="detail-list__item__label">
            Deposit Fee
          </div>
          <div
            class="detail-list__item__value"
            style="color: #10b981;"
          >
            {{ detailCardsData.depositFee }}%
          </div>
        </div>

        <div class="detail-list__item">
          <div class="detail-list__item__label">
            Withdraw Fee
          </div>
          <div
            class="detail-list__item__value"
            style="color: #10b981;"
          >
            {{ detailCardsData.withdrawFee }}%
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
