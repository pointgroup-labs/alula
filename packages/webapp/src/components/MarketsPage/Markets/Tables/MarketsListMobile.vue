<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const {
  items,
} = defineProps<{
  items: MarketTableItem[]
}>()

const emits = defineEmits(['dialogHandler', 'onRowClicked'])

const market = useMarketActions()

function onRowClicked(item: MarketTableItem) {
  emits('onRowClicked', item)
}
</script>

<template>
  <div
    v-if="items?.length === 0"
    class="no-table-data"
  >
    No Pools
  </div>
  <table-mobile-card
    v-for="item in items"
    v-else
    :key="item.pool_address"
    @click="onRowClicked(item)"
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
            Deposit APY
          </div>
          <div class="info-wrapper__value">
            <j-pill-label
              color="#111"
              variant="success"
              size="sm"
            >
              {{ item.deposit_apy }}
            </j-pill-label>
          </div>
        </div>
        <div class="info-wrapper with-pill">
          <div class="info-wrapper__title text-center">
            Borrow APY
          </div>
          <div class="info-wrapper__value">
            <j-pill-label
              color="#111"
              variant="warning"
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
        <div class="info-wrapper__title">
          Total Supply
        </div>
        <div class="info-wrapper__value text-end">
          {{ shortenNumber(item.total_supply) }}
        </div>
      </div>

      <div class="separator-vert" />

      <div class="info-wrapper">
        <div class="info-wrapper__title">
          Total Borrowed
        </div>
        <div class="info-wrapper__value text-end">
          {{ shortenNumber(item.total_borrowed) }}
        </div>
      </div>

      <div class="separator-vert" />

      <div class="info-wrapper">
        <div class="info-wrapper__title">
          Max LTV
        </div>
        <div class="info-wrapper__value text-end">
          {{ item.max_ltv }}
        </div>
      </div>

      <div class="separator-vert" />

      <div class="info-wrapper">
        <div class="info-wrapper__title">
          Util. Rate
        </div>
        <div class="info-wrapper__value text-end">
          {{ item.utilization_rate }}
        </div>
      </div>
    </div>

    <div class="mobile-card-footer">
      <j-btn
        size="sm"
        pill
        icon-right
        :disabled="market.isDisabled(item.pool_address, 'deposit', item.market!)"
        :loading="market.isLoading(item.pool_address, 'deposit', item.market!)"
        @click.stop="emits('dialogHandler', { item, action: 'supply' })"
      >
        Supply
      </j-btn>
      <j-btn
        size="sm"
        pill
        icon-right
        variant="accent"
        :disabled="market.isDisabled(item.pool_address, 'borrow', item.market!)"
        :loading="market.isLoading(item.pool_address, 'borrow', item.market!)"
        @click.stop="emits('dialogHandler', { item, action: 'borrow' })"
      >
        Borrow
      </j-btn>
    </div>
  </table-mobile-card>
</template>
