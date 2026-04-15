<script lang="ts" setup>
import type { MultiplyVaultItem } from '~/types/table'

defineProps<{
  selectedVault: MultiplyVaultItem
}>()
</script>

<template>
  <section class="multiply-details__hero">
    <div>
      <div class="multiply-details__eyebrow">
        {{ selectedVault.market }} market
      </div>
      <h1 class="multiply-details__title">
        {{ selectedVault.asset.symbol }}/{{ selectedVault.borrowAsset.symbol }} multiply vault
      </h1>
      <p class="multiply-details__copy">
        This vault opens leveraged {{ selectedVault.asset.symbol }} exposure by using {{ selectedVault.borrowAsset.symbol }} as margin, routing the swap through the provider-resolved Soroswap router, and depositing the slippage-adjusted output as collateral.
      </p>
    </div>

    <div class="multiply-details__hero-stats">
      <div>
        <span>Max multiplier</span>
        <strong>x{{ truncatePercent(selectedVault.maxMultiplier, 2) }}</strong>
      </div>
      <div>
        <span>APY at max multiplier</span>
        <strong>{{ truncatePercent(selectedVault.apyAtMaxMultiplier, 2) }}%</strong>
      </div>
      <div>
        <span>Borrow liquidity</span>
        <strong>{{ formatPrice(selectedVault.liquidity, 2, 2) }} {{ selectedVault.borrowAsset.symbol }}</strong>
      </div>
      <div>
        <span>Collateral TVL</span>
        <strong>${{ amountToUsdWithShort(selectedVault.supplied, selectedVault.price, false) }}</strong>
      </div>
    </div>
  </section>

  <section class="multiply-details__content">
    <div class="multiply-details__overview">
      <div class="multiply-details__card">
        <span>Collateral asset</span>
        <strong>{{ selectedVault.asset.name }}</strong>
        <small>{{ formatCompactUSD(selectedVault.price, 2, 4) }}</small>
      </div>
      <div class="multiply-details__card">
        <span>Margin asset</span>
        <strong>{{ selectedVault.borrowAsset.name }}</strong>
        <small>{{ formatCompactUSD(selectedVault.borrowPoolPrice, 2, 4) }}</small>
      </div>
      <div class="multiply-details__card">
        <span>Deposit pool</span>
        <strong>
          {{ shortenAddress(selectedVault.depositPoolData.pool.pool_address, 20) }}
          <copy-to-clipboard :text="selectedVault.depositPoolData.pool.pool_address" />
        </strong>
      </div>
      <div class="multiply-details__card">
        <span>Borrow pool</span>
        <strong>
          {{ shortenAddress(selectedVault.borrowPoolData.pool.pool_address, 20) }}
          <copy-to-clipboard :text="selectedVault.borrowPoolData.pool.pool_address" />
        </strong>
      </div>
    </div>

    <multiply-supply-card :vault="selectedVault" />
  </section>
</template>
