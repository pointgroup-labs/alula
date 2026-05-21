<script lang="ts" setup>
import type { BorrowCardTableItem } from '~/types/table'
import { shortenNumber } from '~/utils'

const {
  items,
} = defineProps<{
  items: BorrowCardTableItem[]
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
          alt=""
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
            Borrow APY
          </div>
          <div class="info-wrapper__value">
            <j-pill-label
              variant="indigo"
              size="sm"
            >
              {{ item.borrow_apy }}
            </j-pill-label>
          </div>
        </div>
      </div>
    </div>

    <div class="mobile-card-body">
      <div class="info-wrapper">
        <div class="info-wrapper__title text-end">
          Debt
        </div>
        <div class="info-wrapper__value">
          {{
            Number(item.debt) > 1000 ? shortenNumber(Number(item.debt)) : Number(item.debt).toFixed(5)
          }}
        </div>
      </div>
    </div>

    <div class="mobile-card-footer">
      <j-btn
        variant="outlined-brand-secondary"
        size="sm"
        class="repay-btn"
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
