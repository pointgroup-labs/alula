<script lang="ts" setup>
import type { RewardsTableItem } from '~/types/table'
import { shortenNumber } from '~/utils'

const {
  items,
} = defineProps<{
  items?: RewardsTableItem[]
}>()

const emits = defineEmits(['dialogHandler'])
</script>

<template>
  <table-mobile-card
    v-for="item in items"
    :key="item.asset?.symbol"
  >
    <div class="mobile-card-top">
      <div class="card-asset">
        <img
          :src="item.asset?.icon"
          alt="asset icon"
        >
        <div class="card-asset__info">
          <div class="card-asset__info__name">
            {{ item.asset?.symbol }}
          </div>
          <div class="card-asset__info__symbol">
            {{ item.asset?.name }}
          </div>
        </div>
      </div>

      <div class="card-top-info">
        <div class="info-wrapper with-pill">
          <div class="info-wrapper__title text-center">
            Market
          </div>
          <div class="info-wrapper__value text-capitalize">
            {{ item.market }}
          </div>
        </div>
      </div>
    </div>

    <div class="mobile-card-body">
      <div class="info-wrapper align-items-center">
        <div class="info-wrapper__title text-end">
          Balance
        </div>
        <div class="info-wrapper__value">
          {{ Number(item.pending.amount) > 1000 ? shortenNumber(Number(item.pending.amount)) : Number(item.pending.amount).toFixed(5) }}
          <span
            class="text-tertiary"
            style="font-size: 12px;"
          >/ {{ formatCompactUSD(item.pending.usd, 2, 2) }}</span>
        </div>
      </div>

    </div>

    <div
      class="mobile-card-footer"
    >
      <j-btn
        variant="outlined-brand"
        size="sm"
        @click="emits('dialogHandler', { item })"
      >
        Claim
      </j-btn>
    </div>
  </table-mobile-card>
</template>

<style lang="scss" scoped>
.mobile-card-body {
  justify-content: center;
  gap: 16px;
}
</style>
