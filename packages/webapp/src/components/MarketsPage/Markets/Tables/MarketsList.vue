<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { capitalize } from 'vue'
import { amountToUsdWithShort, shortenNumber } from '~/utils'

const {
  searchAsset,
} = defineProps<{
  searchAsset?: string
}>()

const { width } = useWindowSize()

const marketActions = useMarketActions()
const userStore = useUserStore()

const isObligationsLoading = computed(() => userStore.loading)

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

const { additionalMarketsData } = useAdditionalApy()

const marketsStore = useMarketsStore()

const dialogRepay = ref(false)
const dialogWithdraw = ref(false)

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
  router.push(`/lend/${marketAddress}/${poolAddress}`)
}

function utilRateColor(value?: number) {
  if (!value) {
    return 'transparent'
  }
  switch (true) {
    case value >= 80: return '#f43f5e'
    case value >= 60: return '#8a8df4'
    default: return 'rgb(0, 201, 80)'
  }
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

watch(() => searchAsset, (val) => {
  search.value = val
})

const stop = watch(additionalMarketsData, () => {
  if (opened.value.length === 0 && additionalMarketsData.value.length > 0) {
    toggleOpen(filteredMarkets.value[0]!.marketName)
    stop()
  }
})
</script>

<template>
  <div v-if="marketWithTableItems.length === 0 && loading">
    <market-table-skeleton />
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
        :tbody-tr-class="rowClass"
        :class="{ 'table-loading': loading }"
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

        <template #cell(price)="data">
          <div class="table-cell justify-content-end">
            {{ formatCompactUSD(data.item.price, 2, 2) }}
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
              :stroke-color="utilRateColor(data.item.utilization_rate_percent ?? 0)"
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
            style="opacity: .8;"
          >
            <market-apy-with-additional
              :pool-data="data.item"
              :additional-markets-data="additionalMarketsData"
              :is-deposit="true"
            />
          </div>
        </template>

        <template #cell(borrow_apy)="data">
          <div
            class="table-cell justify-content-center"
            style="opacity: .8;"
          >
            <market-apy-with-additional
              :pool-data="data.item"
              :additional-markets-data="additionalMarketsData"
              :is-deposit="false"
            />
          </div>
        </template>

        <template #cell(position)="data">
          <div class="table-cell justify-content-end with-price">
            <j-loading-spinner
              v-if="isObligationsLoading"
              class="position-spinner"
              width="16px"
            />
            <template v-else-if="+data.item.position.supplied > 0 || +data.item.position.borrowed > 0">
              <strong :style="{ color: +data.item.position.supplied > 0 ? '#22d3ee' : '#8a8df4' }">
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
              size="xs"
              variant="outline-cyan"
              pill
              :disabled="marketActions.isDisabled(data.item.pool_address, 'deposit', data.item.market!)"
              :loading="marketActions.isLoading(data.item.pool_address, 'deposit', data.item.market!)"
              @click="dialogHandler(market.marketName, data.item, 'supply')"
            >
              Supply
            </j-btn>
            <j-btn
              v-else
              size="xs"
              variant="outline-cyan"
              pill
              :disabled="marketActions.isDisabled(data.item.pool_address, 'repay', data.item.market!)"
              :loading="marketActions.isLoading(data.item.pool_address, 'repay', data.item.market!)"
              @click="dialogHandler(market.marketName, data.item, 'repay')"
            >
              Repay
            </j-btn>
            <j-btn
              v-if="+data.item.position.supplied === 0"
              size="xs"
              variant="outline-purple"
              pill
              :disabled="marketActions.isDisabled(data.item.pool_address, 'borrow', data.item.market!)"
              :loading="marketActions.isLoading(data.item.pool_address, 'borrow', data.item.market!)"
              @click="dialogHandler(market.marketName, data.item, 'borrow')"
            >
              Borrow
            </j-btn>
            <j-btn
              v-else
              size="xs"
              variant="outline-success"
              pill
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
  </div>

  <div
    v-else
    class="no-markets-found"
  >
    No markets
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
.market-table {
  .position {
    .position-spinner {
      position: relative !important;
      background-color: transparent !important;
      align-items: flex-end;
      .spinner-border {
        color: $muted-foreground !important;
      }
    }
  }
}
</style>
