<script lang="ts" setup>
import type { Pool } from '@alula/market-sdk'

const multiplyStore = useMultiplyStore()
const selectedVault = computed(() => multiplyStore.selectedVault)

const depositPoolData = computed<Pool | undefined>(() => selectedVault.value?.depositPoolData.pool)
const borrowPoolData = computed<Pool | undefined>(() => selectedVault.value?.borrowPoolData.pool)

const { generateExplorerLink } = useExplorerLink()

const detailCardsData = computed(() => {
  if (!depositPoolData.value) {
    return {
      accrualModel: '-',
      interestRateModel: '-',
      lastAccrual: '-',
      poolAddressShort: '-',
    }
  }

  const lastAccrual = depositPoolData.value?.last_accrual_timestamp
    ? formatDateToDMY(new Date(Number(depositPoolData.value.last_accrual_timestamp) * 1000))
    : '-'

  return {
    lastAccrual,
    accrualModel: depositPoolData.value?.config.accrual_model.tag ?? '-',
    depositPoolAddress: depositPoolData.value?.pool_address ?? '-',
    borrowPoolAddress: borrowPoolData.value?.pool_address,
  }
})
</script>

<template>
  <section id="pool-info-overview">
    <div class="stat-card">
      <div class="stat-card__header">
        <h3 class="title">
          Market Info
        </h3>
      </div>

      <div class="stat-card__body">
        <div class="info-list">
          <div class="info-list__item">
            <div class="title">
              Last Accrual
            </div>
            <div class="value">
              {{ detailCardsData.lastAccrual }}
            </div>
          </div>

          <div class="info-list__item">
            <div class="title">
              Accrual Model
            </div>
            <div class="value">
              {{ detailCardsData.accrualModel }}
            </div>
          </div>

          <div class="info-list__item">
            <div class="title">
              Deposit Pool
            </div>
            <a
              class="value"
              :href="generateExplorerLink(String(detailCardsData.depositPoolAddress), 'contract')"
              target="_blank"
              rel="noopener noreferrer nofollow"
            >
              {{ shortenAddress(detailCardsData.depositPoolAddress ?? '', 4) }}
              <i-app-export-icon class="export-icon" />
            </a>
          </div>

          <div class="info-list__item">
            <div class="title">
              Borrow Pool
            </div>
            <a
              class="value"
              :href="generateExplorerLink(String(detailCardsData.borrowPoolAddress), 'contract')"
              target="_blank"
              rel="noopener noreferrer nofollow"
            >
              {{ shortenAddress(detailCardsData.borrowPoolAddress ?? '', 4) }}
              <i-app-export-icon class="export-icon" />
            </a>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style lang="scss">
.info-list {
  display: flex;
  align-items: stretch;
  gap: 32px;

  &__item {
    position: relative;
    width: 100%;
    display: flex;
    align-items: flex-start;
    flex-direction: column;
    gap: 4px;
    white-space: nowrap;

    &::after {
      content: '';
      right: -16px;
      top: -16px;
      position: absolute;
      width: 1px;
      height: calc(100% + 32px);
      background-color: $border-primary;
    }
  }

  .auditor-item {
    img {
      width: 18px;
      height: 12px;
    }
  }

  .title {
    color: $text-tertiary;
    font-size: $text-xs;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .value {
    color: $text-primary;
    font-family: $font-JetBrainsMono;
    font-size: 14px;
  }

  a {
    display: flex;
    align-items: center;
    gap: 6px;
    text-decoration: none;
    transition: opacity 0.1s ease;

    &:hover {
      .export-icon {
        color: $text-primary;
      }
    }
  }

  .export-icon {
    width: 12px;
    height: 12px;
    color: $navi-300;
  }
}
</style>
