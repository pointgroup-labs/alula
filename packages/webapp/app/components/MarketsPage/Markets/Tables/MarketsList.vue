<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { capitalize } from 'vue'
import { amountToUsdWithShort, shortenNumber } from '~/utils'

const { width } = useWindowSize()

const marketActions = useMarketActions()
const userStore = useUserStore()

const isObligationsLoading = computed(() => userStore.loading)

const router = useRouter()

const marketTableStore = useMarketTableStore()
const {
  loading,
  search,
  marketWithTableItems,
  filteredMarkets,
  dialogSupply,
  dialogBorrow,
  dialogRepay,
  dialogWithdraw,
  selectedMarketName,
  selectedPool,
  selectedPoolAddress,
} = storeToRefs(marketTableStore)

const {
  opened,
  isOpened,
  toggleOpen } = useAccordionMarketsHandler('accordion-markets')

const marketsStore = useMarketsStore()

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'total_supply', label: 'Supplied', align: 'right', thClass: 'supply', tdClass: 'supply' },
  { key: 'total_borrowed', label: 'Borrowed', align: 'right', thClass: 'borrow', tdClass: 'borrow' },
  { key: 'utilization_rate', label: 'Utilization', align: 'right', thClass: 'utilization', tdClass: 'utilization' },
  { key: 'deposit_apy', label: 'Supply APY', align: 'center', thClass: 'apy', tdClass: 'apy' },
  { key: 'borrow_apy', label: 'Borrow rate', align: 'center', thClass: 'apy', tdClass: 'apy' },
  { key: 'position', label: 'My Position', align: 'right', thClass: 'position', tdClass: 'position' },
  { key: 'action', label: '', thClass: 'action', tdClass: 'action' },
]

async function dialogHandler(marketName: string, item: MarketTableItem, action: 'supply' | 'borrow' | 'repay' | 'withdraw') {
  selectedMarketName.value = marketName
  selectedPoolAddress.value = item?.pool_address
  if (action === 'repay') {
    dialogRepay.value = true
    return
  }
  if (action === 'supply') {
    dialogSupply.value = true
    return
  }
  if (action === 'borrow') {
    dialogBorrow.value = true
    return
  }
  if (action === 'withdraw') {
    dialogWithdraw.value = true
  }
}

function onRowClicked(marketName: string, item: MarketTableItem) {
  const marketAddress = marketsStore.state.markets[marketName]?.address
  const poolAddress = item.pool_address
  router.push(`/lend/${marketAddress}/${poolAddress}/pool`)
}

function rowClass(item: any): any {
  if (!item) {
    return
  }
  const util = item.utilization_rate_percent ?? 0
  if (util >= 80) {
    return 'row-danger'
  }
  return ''
}

const isNoMarketsData = computed(() => filteredMarkets.value.every(m => m?.tableItems?.length === 0))

const isInitialized = ref(false)
watch(loading, (val) => {
  if (!val) { isInitialized.value = true }
})

watch([
  filteredMarkets,
  search,
], ([markets, s]) => {
  if (markets.length === 0) {
    return
  }

  const allCollapsed = opened.value.length === 0

  if (allCollapsed && s) {
    for (const market of markets) {
      if (!isOpened(market.marketName)) {
        toggleOpen(market.marketName)
      }
    }
    return
  }

  if (allCollapsed) {
    const mainMarket = markets.find(m => m.marketName === 'main') ?? markets[0]
    if (mainMarket) {
      toggleOpen(mainMarket.marketName)
    }
  }
}, { immediate: true })
</script>

<template>
  <div v-if="marketWithTableItems.length === 0 && (loading || !isInitialized)">
    <market-table-skeleton v-if="width > 1024" />
    <market-table-skeleton-mobile v-else />
  </div>
  <div
    v-else-if="marketWithTableItems.length > 0"
    class="table-wrapper"
  >
    <template
      v-for="(market) in filteredMarkets"
      :key="market.marketName"
    >
      <j-accordion
        v-if="market.tableItems.length > 0"
        :visible="isOpened(market.marketName)"
        @toggle="toggleOpen(market.marketName)"
      >
        <template #title>
          {{ capitalize(market.marketName) }} Market

          <div class="market-info-wrapper">
            <market-info-badge>
              <span data-name="title">Market Size: </span>

              <span>${{ shortenNumber(market.marketSize.supplied) }}</span>
            </market-info-badge>

            <market-info-badge>
              <span data-name="title">Borrowed: </span>

              <span>${{ shortenNumber(market.marketSize.borrowed) }}</span>
            </market-info-badge>

            <market-info-badge v-if="market.assets.length > 0">
              <span data-name="title">Assets: </span>
              <span>{{ market.assets.length }}</span>
            </market-info-badge>

            <statistics-route-btn :market-name="market.marketName" />
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
          :tbody-tr-class="rowClass"
          :class="{ 'table-loading': loading || isObligationsLoading }"
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

                <pool-status :pool="data.item.raw.pool" />
              </div>
            </div>
          </template>

          <template #cell(total_supply)="data">
            <div class="table-cell justify-content-end">
              <div class="with-price">
                <strong>{{ shortenNumber(data.item.total_supply) }}</strong>
                <span>${{ amountToUsdWithShort(data.item.total_supply, data.item.price) }}</span>
              </div>
            </div>
          </template>

          <template #cell(total_borrowed)="data">
            <div class="table-cell justify-content-end">
              <div class="with-price">
                <strong>{{ shortenNumber(data.item.total_borrowed) }}</strong>
                <span>${{ amountToUsdWithShort(data.item.total_borrowed, data.item.price) }}</span>
              </div>
            </div>
          </template>

          <template #cell(utilization_rate)="data">
            <div class="table-cell justify-content-end">
              <j-circular-progress
                :progress="data.item.utilization_rate_percent ?? 0"
                :width="18"
                :stroke-width="30"
                stroke-bg="#262729"
                :stroke-color="utilRateColor(data.item.utilization_rate_percent, data.item.utilization_rate_limit * 100)"
                background="transparent"
                color="#fff"
                :with-progress="false"
              />
              {{ data.item.utilization_rate }}
            </div>
          </template>

          <template #cell(deposit_apy)="data">
            <div
              class="table-cell justify-content-center flex"
            >
              <j-insentive-apy
                :apy="data.item.deposit_apy"
                :pool-data="data.item.raw"
                farm-type="supply"
                :market-name="market.marketName"
              />
            </div>
          </template>

          <template #cell(borrow_apy)="data">
            <div
              class="table-cell justify-content-center"
            >
              <j-insentive-apy
                :apy="data.item.borrow_apy"
                :pool-data="data.item.raw"
                :market-name="market.marketName"
                farm-type="borrow"
                variant="indigo"
              />
            </div>
          </template>

          <template #cell(position)="data">
            <div class="table-cell justify-content-end with-price">
              <template v-if="+data.item.position.supplied > 0 || +data.item.position.borrowed > 0">
                <strong :style="{ color: +data.item.position.supplied > 0 ? '#22D3EE' : '#8A8DF4' }">
                  {{ shortenNumber(+data.item.position.supplied || +data.item.position.borrowed) }}</strong>
                <span>
                  ${{ amountToUsdWithShort(+data.item.position.supplied || +data.item.position.borrowed, data.item.price) }}</span>
              </template>
              <div
                v-else
                style="opacity: .3;"
              >
                -
              </div>
            </div>

          </template>

          <template #cell(action)="data">
            <div class="table-cell justify-content-end market-table__action">
              <j-btn
                v-if="+data.item.position.borrowed === 0"
                size="sm"
                variant="outlined-brand"
                :disabled="marketActions.isDisabled(data.item.pool_address, 'deposit', data.item.market!)"
                :loading="marketActions.isLoading(data.item.pool_address, 'deposit', data.item.market!)"
                @click="dialogHandler(market.marketName, data.item, 'supply')"
              >
                Supply
              </j-btn>
              <j-btn
                v-else
                size="sm"
                variant="outlined-brand-secondary"
                :disabled="marketActions.isDisabled(data.item.pool_address, 'repay', data.item.market!)"
                :loading="marketActions.isLoading(data.item.pool_address, 'repay', data.item.market!)"
                @click="dialogHandler(market.marketName, data.item, 'repay')"
              >
                Repay
              </j-btn>
              <j-btn
                v-if="+data.item.position.supplied === 0"
                size="sm"
                variant="outlined-brand-secondary"
                :disabled="marketActions.isDisabled(data.item.pool_address, 'borrow', data.item.market!)"
                :loading="marketActions.isLoading(data.item.pool_address, 'borrow', data.item.market!)"
                @click="dialogHandler(market.marketName, data.item, 'borrow')"
              >
                Borrow
              </j-btn>
              <j-btn
                v-else
                size="sm"
                variant="outlined-brand"
                :disabled="marketActions.isDisabled(data.item.pool_address, 'withdraw', data.item.market!)"
                :loading="marketActions.isLoading(data.item.pool_address, 'withdraw', data.item.market!)"
                @click="dialogHandler(market.marketName, data.item, 'withdraw')"
              >
                Withdraw
              </j-btn>
            </div>
          </template>

          <template #empty>
            <div
              v-show="!loading"
              class="no-table-data"
            >
              No pools
            </div>
          </template>
        </BTable>

        <markets-list-mobile
          v-else
          :items="market.tableItems"
          :market-name="market.marketName"
          @dialog-handler="(e: any) => dialogHandler(market.marketName, e.item, e.action)"
          @on-row-clicked="onRowClicked"
        />
      </j-accordion>
    </template>

    <div
      v-if="isNoMarketsData"
      class="no-markets-found"
    >
      No Markets found
    </div>
  </div>

  <div
    v-else
    class="no-markets-found"
  >
    No Markets found
  </div>

  <client-only>
    <supply-dialog
      v-model="dialogSupply"
      :data="selectedPool"
    />

    <repay-dialog
      v-model="dialogRepay"
    />

    <withdraw-dialog
      v-model="dialogWithdraw"
    />

    <borrow-dialog
      v-model="dialogBorrow"
      :data="selectedPool"
    />
  </client-only>
</template>

<style lang="scss">
.table-wrapper {
  .market-table {
    .position {
      .position-spinner {
        position: relative !important;
        background-color: transparent !important;
        align-items: flex-end;
        .spinner-border {
          color: $text-tertiary !important;
        }
      }
    }
  }
}
</style>
