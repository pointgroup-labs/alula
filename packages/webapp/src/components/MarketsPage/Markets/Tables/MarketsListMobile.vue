<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { shortenNumber } from '~/utils'

const {
  items,
} = defineProps<{
  items: MarketTableItem[]
  additionalMarketsData: any
}>()

const emits = defineEmits(['dialogHandler', 'onRowClicked'])

const market = useMarketActions()

function onRowClicked(item: MarketTableItem) {
  emits('onRowClicked', item.market, item)
}
</script>

<template>
  <div
    v-if="items?.length === 0"
    class="no-table-data"
  >
    No pools
  </div>
  <table-mobile-card
    v-for="item in items"
    v-else
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

      <div
        class="card-top-info"
      >
        <div class="info-wrapper with-pill">
          <div class="info-wrapper__title text-center">
            Supply APY
          </div>
          <div class="info-wrapper__value">
            <market-apy-with-additional
              :pool-data="item"
              :additional-markets-data="additionalMarketsData"
              :is-deposit="true"
              style="height: 24px; font-size: 12px;"
            />
          </div>
        </div>
        <div class="info-wrapper with-pill">
          <div class="info-wrapper__title text-center">
            Borrow APY
          </div>
          <div class="info-wrapper__value">
            <market-apy-with-additional
              :pool-data="item"
              :additional-markets-data="additionalMarketsData"
              :is-deposit="false"
              style="height: 24px; font-size: 12px;"
            />
          </div>
        </div>
        <div
          class="info-wrapper with-pill"
          style="gap: 6px; margin-left: 4px;"
          @click="onRowClicked(item)"
        >
          <div class="info-wrapper__title text-center">
            Details
          </div>
          <div class="info-wrapper__value text-center">
            <i-app-export-icon :color="isDark ? '#fff' : '#111'" />
          </div>
        </div>
      </div>
    </div>

    <div class="mobile-card-body">
      <div class="info-wrapper">
        <div class="info-wrapper__title">
          Supply
        </div>
        <div class="info-wrapper__value text-end">
          {{ shortenNumber(item?.total_supply) }}
        </div>
      </div>

      <div class="separator-vert" />

      <div class="info-wrapper">
        <div class="info-wrapper__title">
          Borrow
        </div>
        <div class="info-wrapper__value text-end">
          {{ shortenNumber(item?.total_borrowed) }}
        </div>
      </div>

      <div class="separator-vert" />

      <div class="info-wrapper">
        <div class="info-wrapper__title">
          Open LTV
        </div>
        <div class="info-wrapper__value text-end">
          {{ item.open_ltv }}
        </div>
      </div>

      <div class="separator-vert" />

      <div class="info-wrapper">
        <div class="info-wrapper__title">
          Util.
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
