<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { amountToUsdWithShort, formatPrice, shortenNumber } from '~/utils'

const { width } = useWindowSize()

const market = useMarketActions()

const {
  loading,
  tableItems,
  infoDialog,
  dialogSupply,
  dialogBorrow,
  activeMarket,
  selectedPool,
  selectedMarketAddress,
  selectedMarketDetails,
} = useMarketTable()

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'total_supply', label: 'Total Supply', align: 'right' },
  { key: 'total_borrowed', label: 'Total Borrow', align: 'right' },
  { key: 'deposit_apy', label: 'Deposit APY', align: 'center' },
  { key: 'borrow_apy', label: 'Borrow APY', align: 'center' },
  { key: 'utilization_rate', label: 'Utilization', align: 'right' },
  { key: 'max_ltv', label: 'Max LTV', align: 'center' },
  { key: 'action', label: '' },
]

async function supplyDialogHandler(item: MarketTableItem, action: 'supply' | 'borrow') {
  selectedMarketAddress.value = item?.pool_address
  action === 'supply' ? dialogSupply.value = true : dialogBorrow.value = true
}

function onRowClicked(item: MarketTableItem, _index: number, _event: any) {
  selectedMarketAddress.value = item.pool_address
  infoDialog.value = true
}

provide('selectedMarketDetails', selectedMarketDetails)
</script>

<template>
  <div v-if="activeMarket && loading">
    <table-skeleton v-if="width > 650" />
    <table-skeleton-mobile v-else />
  </div>
  <div
    v-else
    class="table-wrapper"
  >
    <BTable
      v-if="width >= 1024"
      show-empty
      borderless
      :fields="fields"
      :items="tableItems"
      responsive
      class="market-table"
      @row-clicked="onRowClicked"
    >
      <template
        v-for="field in fields"
        :key="field.key"
        #[`head(${field.key})`]="data"
      >
        <span :style="{ '--align': field.align }">{{ data.label }}</span>
      </template>

      <template #cell(asset)="data">
        <div class="market-table__asset">
          <img
            :src="data.item.asset.icon"
            alt=""
          >
          <div class="market-table__asset__info">
            <div class="market-table__asset__info__name">
              {{ data.item.asset.symbol }}
            </div>
            <div class="market-table__asset__info__symbol">
              {{ data.item.asset.name }}
            </div>
          </div>
        </div>
      </template>

      <template #cell(total_supply)="data">
        <div class="table-cell justify-content-end">
          <j-tooltip tooltip-class="with-price">
            <strong>{{ shortenNumber(data.item.total_supply) }}</strong>
            <span>${{ amountToUsdWithShort(data.item.total_supply, data.item.price) }}</span>
            <template #content>
              {{ formatPrice(data.item.total_supply) }} {{ data.item.asset.symbol }}
              <br>
              <span>${{ amountToUsdWithShort(data.item.total_supply, data.item.price, false) }}</span>
            </template>
          </j-tooltip>
        </div>
      </template>

      <template #cell(total_borrowed)="data">
        <div class="table-cell justify-content-end">
          <j-tooltip tooltip-class="with-price">
            <strong>{{ shortenNumber(data.item.total_borrowed) }}</strong>
            <span>${{ amountToUsdWithShort(data.item.total_borrowed, data.item.price) }}</span>
            <template #content>
              {{ formatPrice(data.item.total_borrowed) }} {{ data.item.asset.symbol }}
              <br>
              <span>${{ amountToUsdWithShort(data.item.total_borrowed, data.item.price, false) }}</span>
            </template>
          </j-tooltip>
        </div>
      </template>

      <template #cell(deposit_apy)="data">
        <div class="table-cell justify-content-center">
          <j-pill-label
            color="#111"
            variant="success"
            size="sm"
          >
            {{ data.item.deposit_apy }}
          </j-pill-label>
        </div>
      </template>

      <template #cell(borrow_apy)="data">
        <div class="table-cell justify-content-center">
          <j-pill-label
            color="#111"
            variant="warning"
            size="sm"
          >
            {{ data.item.borrow_apy }}
          </j-pill-label>
        </div>
      </template>

      <template #cell(utilization_rate)="data">
        <div class="table-cell justify-content-end">
          {{ data.item.utilization_rate }}
        </div>
      </template>

      <template #cell(max_ltv)="data">
        <div class="table-cell justify-content-end">
          {{ data.item.max_ltv }}
        </div>
      </template>

      <template #cell(action)="data">
        <div class="table-cell justify-content-end market-table__action">
          <j-btn
            size="md"
            pill
            icon-right
            :disabled="market.isDisabled(data.item.pool_address, 'deposit', data.item.market!)"
            :loading="market.isLoading(data.item.pool_address, 'deposit', data.item.market!)"
            @click="supplyDialogHandler(data.item, 'supply')"
          >
            Supply
          </j-btn>
          <j-btn
            size="md"
            pill
            icon-right
            variant="accent"
            :disabled="market.isDisabled(data.item.pool_address, 'borrow', data.item.market!)"
            :loading="market.isLoading(data.item.pool_address, 'borrow', data.item.market!)"
            @click="supplyDialogHandler(data.item, 'borrow')"
          >
            Borrow
          </j-btn>
        </div>
      </template>

      <template
        #empty
      >
        <div
          v-show="!loading"
          class="no-data"
        >
          No Pools
        </div>
      </template>
    </BTable>

    <market-table-mobile
      v-else
      :items="tableItems"
      @dialog-handler="(e: any) => supplyDialogHandler(e.item, e.action)"
      @on-row-clicked="onRowClicked"
    />

    <j-loading-spinner
      v-if="loading"
      class="table-loading-spinner"
    >
      Loading...
    </j-loading-spinner>
  </div>

  <supply-dialog
    v-model="dialogSupply"
    :data="selectedPool"
  />

  <borrow-dialog
    v-model="dialogBorrow"
    :data="selectedPool"
  />

  <market-info-dialog v-model="infoDialog" />
</template>
