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

const marketActions = useMarketActions()

function onRowClicked(item: MarketTableItem) {
  emits('onRowClicked', item.market, item)
}

function dialogHandler(item: MarketTableItem, action: string) {
  emits('dialogHandler', { item, action })
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
            <j-pill-label
              variant="cyan"
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
              variant="indigo"
              size="sm"
            >
              {{ item.borrow_apy }}
            </j-pill-label>
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
        v-if="+item.position.borrowed === 0"
        size="sm"
        variant="brand-outlined"
        :disabled="marketActions.isDisabled(item.pool_address, 'deposit', item.market!)"
        :loading="marketActions.isLoading(item.pool_address, 'deposit', item.market!)"
        @click="dialogHandler(item, 'supply')"
      >
        Supply
      </j-btn>
      <j-btn
        v-else
        size="sm"
        variant="brand-secondary-outlined"
        :disabled="marketActions.isDisabled(item.pool_address, 'repay', item.market!)"
        :loading="marketActions.isLoading(item.pool_address, 'repay', item.market!)"
        @click="dialogHandler(item, 'repay')"
      >
        Repay
      </j-btn>
      <j-btn
        v-if="+item.position.supplied === 0"
        size="sm"
        variant="brand-secondary-outlined"
        :disabled="marketActions.isDisabled(item.pool_address, 'borrow', item.market!)"
        :loading="marketActions.isLoading(item.pool_address, 'borrow', item.market!)"
        @click="dialogHandler(item, 'borrow')"
      >
        Borrow
      </j-btn>
      <j-btn
        v-else
        size="sm"
        variant="brand-outlined"
        :disabled="marketActions.isDisabled(item.pool_address, 'withdraw', item.market!)"
        :loading="marketActions.isLoading(item.pool_address, 'withdraw', item.market!)"
        @click="dialogHandler(item, 'withdraw')"
      >
        Withdraw
      </j-btn>
    </div>
  </table-mobile-card>
</template>
