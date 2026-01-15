<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { capitalize } from 'vue'
import { amountToUsdWithShort, formatPrice, shortenNumber } from '~/utils'

const {
  searchAsses,
} = defineProps<{
  searchAsses?: string
}>()

const { width } = useWindowSize()

const isHasMarkets = defineModel<boolean>('isHasMarkets', {
  default: true,
})

const marketActions = useMarketActions()

const {
  loading,
  search,
  marketWithTableItems,
  filteredMarkets,
  infoDialog,
  dialogSupply,
  dialogBorrow,
  selectedMarketName,
  selectedPool,
  selectedPoolAddress,
  selectedMarketDetails,
} = useMarketTable()

const { additionalMarketsData, generateMockAdditionalData } = useAdditionalApy()

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  // { key: 'status', label: 'Status', align: 'center', thClass: 'status', tdClass: 'status' },
  { key: 'total_supply', label: 'Supply', align: 'right' },
  { key: 'total_borrowed', label: 'Borrow', align: 'right' },
  { key: 'deposit_apy', label: 'Supply APY', align: 'center', thClass: 'apy', tdClass: 'apy' },
  { key: 'borrow_apy', label: 'Borrow APY', align: 'center', thClass: 'apy', tdClass: 'apy' },
  // { key: 'utilization_rate', label: 'Utilization', align: 'right' },
  // { key: 'max_ltv', label: 'Open LTV', align: 'right' },
  { key: 'action', label: '', thClass: 'action', tdClass: 'action' },
]

async function supplyDialogHandler(marketName: string, item: MarketTableItem, action: 'supply' | 'borrow') {
  selectedMarketName.value = marketName
  selectedPoolAddress.value = item?.pool_address
  action === 'supply' ? dialogSupply.value = true : dialogBorrow.value = true
}

function onRowClicked(marketName: string, item: MarketTableItem) {
  selectedMarketName.value = marketName
  selectedPoolAddress.value = item.pool_address
  infoDialog.value = true
}

provide('selectedMarketDetails', selectedMarketDetails)

watch(() => searchAsses, (val) => {
  search.value = val
})

watch(filteredMarkets, (val) => {
  isHasMarkets.value = val.length > 0
  if (additionalMarketsData.value.length > 0) {
    return
  }
  generateMockAdditionalData(marketWithTableItems.value)
}, { immediate: true })
</script>

<template>
  <div v-if="marketWithTableItems.length === 0 && loading">
    <j-skeleton
      full-width
      height="60"
      style="border-radius: 8px;"
    />
  </div>
  <div
    v-else
    class="table-wrapper"
  >
    <j-accordion
      v-for="(market, idx) in filteredMarkets"
      :key="market.marketName"
      :visible="!searchAsses ? idx === 0 : !!searchAsses"
    >
      <template #title>
        {{ capitalize(market.marketName) }} Market

        <div
          v-if="market.assets.length > 0"
          class="market-assets"
        >
          <p>Assets</p>

          <img
            v-for="asset in market.assets.slice(0, 2)"
            :key="asset.icon"
            :src="asset.icon"
            alt="market asset"
          >
          <span v-if="market.assets.length > 2">+{{ market.assets.length - 2 }}</span>
        </div>
      </template>

      <BTable
        v-if="width >= 1024"
        show-empty
        borderless
        :fields="fields"
        :items="market.tableItems"
        responsive
        class="market-table"
        @row-clicked="(e) => onRowClicked(market.marketName, e)"
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
            <div
              class="market-table__asset__info"
              style="gap: 0;"
            >
              <div class="market-table__asset__info__name">
                {{ data.item.asset.symbol }}
              </div>
              <div class="market-table__asset__info__symbol">
                {{ data.item.asset.name }}
              </div>
            </div>

            <pool-status :pool="data.item.raw.pool" />
          </div>
        </template>

        <!-- <template #cell(status)="data">
          <div class="table-cell justify-content-center">
            <pool-status :pool="data.item.raw.pool" />
          </div>
        </template> -->

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
          <div class="table-cell justify-content-center flex">
            <market-apy-with-additional
              :pool-data="data.item"
              :additional-markets-data="additionalMarketsData"
              :is-deposit="true"
            />
          </div>
        </template>

        <template #cell(borrow_apy)="data">
          <div class="table-cell justify-content-center">
            <market-apy-with-additional
              :pool-data="data.item"
              :additional-markets-data="additionalMarketsData"
              :is-deposit="false"
            />
          </div>
        </template>

        <!-- <template #cell(utilization_rate)="data">
          <div class="table-cell justify-content-end">
            {{ data.item.utilization_rate }}
          </div>
        </template>

        <template #cell(max_ltv)="data">
          <div class="table-cell justify-content-end">
            {{ data.item.open_ltv }}
          </div>
        </template> -->

        <template #cell(action)="data">
          <div class="table-cell justify-content-end market-table__action">
            <j-btn
              size="md"
              pill
              icon-right
              :disabled="marketActions.isDisabled(data.item.pool_address, 'deposit', data.item.market!)"
              :loading="marketActions.isLoading(data.item.pool_address, 'deposit', data.item.market!)"
              @click="supplyDialogHandler(market.marketName, data.item, 'supply')"
            >
              Supply
            </j-btn>
            <j-btn
              size="md"
              pill
              icon-right
              variant="accent"
              :disabled="marketActions.isDisabled(data.item.pool_address, 'borrow', data.item.market!)"
              :loading="marketActions.isLoading(data.item.pool_address, 'borrow', data.item.market!)"
              @click="supplyDialogHandler(market.marketName, data.item, 'borrow')"
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
            No pools
          </div>
        </template>
      </BTable>

      <markets-list-mobile
        v-else
        :items="market.tableItems"
        :additional-markets-data="additionalMarketsData"
        @dialog-handler="(e: any) => supplyDialogHandler(market.marketName, e.item, e.action)"
        @on-row-clicked="onRowClicked"
      />
    </j-accordion>

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
