<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { capitalize } from 'vue'
import { amountToUsdWithShort, formatPrice, shortenNumber } from '~/utils'

const {
  searchAsset,
} = defineProps<{
  searchAsset?: string
}>()

const { width } = useWindowSize()

const isHasMarkets = defineModel<boolean>('isHasMarkets', {
  default: true,
})

const marketActions = useMarketActions()

const router = useRouter()

const {
  loading,
  search,
  marketWithTableItems,
  filteredMarkets,
  dialogSupply,
  dialogBorrow,
  selectedMarketName,
  selectedPool,
  selectedPoolAddress,
} = useMarketTable()

const {
  opened,
  isOpened,
  toggleOpen } = useAccordionMarketsHandler('accordion-markets')

const { additionalMarketsData, generateMockAdditionalData } = useAdditionalApy()

const marketsStore = useMarketsStore()

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  // { key: 'status', label: 'Status', align: 'center', thClass: 'status', tdClass: 'status' },
  // { key: 'price', label: 'Price', align: 'right', thClass: 'price', tdClass: 'price' },
  { key: 'total_supply', label: 'Supply', align: 'right', thClass: 'supply', tdClass: 'supply' },
  { key: 'total_borrowed', label: 'Borrow', align: 'right', thClass: 'borrow', tdClass: 'borrow' },
  { key: 'deposit_apy', label: 'Supply APY', align: 'center', thClass: 'apy', tdClass: 'apy' },
  { key: 'borrow_apy', label: 'Borrow APY', align: 'center', thClass: 'apy', tdClass: 'apy' },
  // { key: 'utilization_rate', label: 'Utilization', align: 'right' },
  // { key: 'max_ltv', label: 'Open LTV', align: 'right' },
  { key: 'action', label: '', thClass: 'action', tdClass: 'action' },
]

async function dialogHandler(marketName: string, item: MarketTableItem, action: 'supply' | 'borrow') {
  selectedMarketName.value = marketName
  selectedPoolAddress.value = item?.pool_address
  action === 'supply' ? dialogSupply.value = true : dialogBorrow.value = true
}

function onRowClicked(marketName: string, item: MarketTableItem) {
  const marketAddress = marketsStore.state.markets[marketName]?.address
  const poolAddress = item.pool_address
  router.push(`/lend/${marketAddress}/${poolAddress}`)
}

watch(() => searchAsset, (val) => {
  search.value = val
})

watch(filteredMarkets, (val) => {
  isHasMarkets.value = val.length > 0
  if (additionalMarketsData.value.length > 0) {
    return
  }
  generateMockAdditionalData(marketWithTableItems.value)
}, { immediate: true })

const stop = watch(additionalMarketsData, () => {
  if (opened.value.length === 0 && additionalMarketsData.value.length > 0) {
    toggleOpen(filteredMarkets.value[0]!.marketName)
    stop()
  }
})
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
    v-else-if="marketWithTableItems.length > 0"
    class="table-wrapper"
  >
    <j-accordion
      v-for="(market) in filteredMarkets"
      :key="market.marketName"
      :visible="!searchAsset ? isOpened(market.marketName) : !!searchAsset"
      @toggle="toggleOpen(market.marketName)"
    >
      <template #title>
        {{ capitalize(market.marketName) }} Market

        <div class="market-info-wrapper">
          <market-info-pill>
            <span data-name="title">Market Size: </span>

            <span>${{ shortenNumber(market.marketSize) }}</span>
          </market-info-pill>

          <market-info-pill v-if="market.assets.length > 0">
            <span data-name="title">Assets </span>

            <img
              v-for="asset in market.assets.slice(0, 2)"
              :key="asset.icon"
              :src="asset.icon"
              alt="market asset"
            >
            <span v-if="market.assets.length > 2">+{{ market.assets.length - 2 }}</span>
          </market-info-pill>
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

        <template #cell(price)="data">
          <div class="table-cell justify-content-end">
            <j-tooltip>
              {{ formatCompactUSD(data.item.price, 2, 2) }}
              <template #content>
                {{ formatPrice(data.item.price) }}
              </template>
            </j-tooltip>
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
              size="xs"
              variant="blue"
              :disabled="marketActions.isDisabled(data.item.pool_address, 'deposit', data.item.market!)"
              :loading="marketActions.isLoading(data.item.pool_address, 'deposit', data.item.market!)"
              @click="dialogHandler(market.marketName, data.item, 'supply')"
            >
              Supply
            </j-btn>
            <j-btn
              size="xs"
              variant="accent"
              :disabled="marketActions.isDisabled(data.item.pool_address, 'borrow', data.item.market!)"
              :loading="marketActions.isLoading(data.item.pool_address, 'borrow', data.item.market!)"
              @click="dialogHandler(market.marketName, data.item, 'borrow')"
            >
              Borrow
            </j-btn>
          </div>
        </template>

        <template #empty>
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
        @dialog-handler="(e: any) => dialogHandler(market.marketName, e.item, e.action)"
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

  <div
    v-else
    class="no-data"
  >
    No markets
  </div>

  <supply-dialog
    v-model="dialogSupply"
    :data="selectedPool"
  />

  <borrow-dialog
    v-model="dialogBorrow"
    :data="selectedPool"
  />
</template>
