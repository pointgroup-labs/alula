<script lang="ts" setup>
import type { SuppliedCardTableItem } from '~/types/table'
import { shortenNumber } from '~/utils'

const {
  items,
} = defineProps<{
  items?: SuppliedCardTableItem[]
}>()

const emits = defineEmits(['dialogHandler'])

const market = useMarketActions()
</script>

<template>
  <table-mobile-card
    v-for="item in items"
    :key="item.pool_address"
  >
    <div class="mobile-card-top">
      <div class="card-asset">
        <img
          :src="item.asset.icon"
          alt="asset icon"
        >
        <div class="card-asset__info">
          <div class="card-asset__info__name">
            {{ item.asset.symbol }}
          </div>
          <div class="card-asset__info__symbol">
            {{ item.asset.name }}
          </div>
        </div>
      </div>

      <div class="card-top-info">
        <div class="info-wrapper with-pill">
          <div class="info-wrapper__title text-center">
            Supply APY
          </div>
          <div class="info-wrapper__value">
            <j-pill-label
              variant="success"
              size="sm"
            >
              {{ item.supply_apy }}
            </j-pill-label>
          </div>
        </div>
      </div>
    </div>

    <div class="mobile-card-body">
      <div class="info-wrapper">
        <div class="info-wrapper__title text-end">
          Balance
        </div>
        <div class="info-wrapper__value">
          {{ Number(item.balance) > 1000 ? shortenNumber(Number(item.balance)) : Number(item.balance).toFixed(5) }}
        </div>
      </div>

    </div>

    <div
      class="mobile-card-footer"
    >
      <j-btn
        variant="success"
        size="sm"
        :disabled="market.isDisabled(item.pool_address, 'withdraw', item.market!)"
        :loading="market.isLoading(item.pool_address, 'withdraw', item.market!)"
        @click="emits('dialogHandler', { item })"
      >
        {{ item.action }}
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
