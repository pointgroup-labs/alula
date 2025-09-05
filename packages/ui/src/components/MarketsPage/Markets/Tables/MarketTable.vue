<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { amountToUsdWithShort, formatPrice, shortenNumber } from '~/utils'

const { width } = useWindowSize()

const client = useClientStore()
const marketsStore = useMarketsStore()

const market = useMarket()

const dialogSupply = toRef(marketsStore, 'dialogSupply')
const dialogBorrow = toRef(marketsStore, 'dialogBorrow')
const infoDialog = toRef(marketsStore, 'marketInfoDialog')

const selectedMarketAddress = toRef(marketsStore, 'selectedMarketAddress')

const assetDecimals = computed(() => client.assetDecimals)

const pools = computed(() => marketsStore.selectedMarketPools)
const loading = computed(() => marketsStore.state.loading)

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

const items = computed<MarketTableItem[]>(() => {
  return pools.value?.map((p) => {
    const tokenSymbol = p.token_ticker
    const tokenName = getTokenName(tokenSymbol)
    const icon = getTokenIcon(tokenSymbol) || ''
    const total_supply = Number(bigintToNumber(p.available + p.total_borrowed + p.total_collateral, assetDecimals.value)) || 0
    const total_borrowed = Number(bigintToNumber(p.total_borrowed, assetDecimals.value)) || 0
    const depositApy = p.pool_apy.supply_bps / 100
    const borrowApy = p.pool_apy.borrow_bps / 100
    const utilRate = Number(p.total_borrowed) / Number((p.available + p.total_borrowed)) * 100
    const maxLTV = Number(p.config.open_ltv_bps) / 100
    const supply_limit = Number(bigintToNumber(p.config.supply_limit, assetDecimals.value)) || 0
    return {
      raw: p,
      asset: { name: tokenName, symbol: tokenSymbol, icon },
      total_supply,
      total_borrowed,
      deposit_apy: `${truncatePercent(depositApy || 0, 2)}%`,
      borrow_apy: `${truncatePercent(borrowApy || 0, 2)}%`,
      utilization_rate: `${truncatePercent(utilRate || 0, 2)}%`,
      max_ltv: `${truncatePercent(maxLTV || 0, 2)}%`,
      action: 'Supply',
      price: Number(p.pool_price),
      supply_limit,
      available: Number(p.available) / (10 ** assetDecimals.value),
      pool_address: p.pool_address,
    }
  })
})

const selectedPool = computed(() => items.value.find(item => item.pool_address === selectedMarketAddress.value))

async function supplyDialogHandler(item: MarketTableItem, action: 'supply' | 'borrow') {
  selectedMarketAddress.value = item?.pool_address
  action === 'supply' ? dialogSupply.value = true : dialogBorrow.value = true
}

function onRowClicked(item: MarketTableItem, _index: number, _event: any) {
  selectedMarketAddress.value = item.pool_address
  infoDialog.value = true
}

const selectedMarketDetails = computed(() => items.value.find(item => item.pool_address === selectedMarketAddress.value))

provide('selectedMarketDetails', selectedMarketDetails)
</script>

<template>
  <div v-if="pools.length === 0 && loading">
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
      :items="items"
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
            size="md"
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
            size="md"
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
            size="lg"
            pill
            icon-right
            :disabled="market.isDisabled(data.item.pool_address, 'deposit')"
            :loading="market.isLoading(data.item.pool_address, 'deposit')"
            @click="supplyDialogHandler(data.item, 'supply')"
          >
            Supply
          </j-btn>
          <j-btn
            size="lg"
            pill
            icon-right
            variant="accent"
            :disabled="market.isDisabled(data.item.pool_address, 'borrow')"
            :loading="market.isLoading(data.item.pool_address, 'borrow')"
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
          No Markets
        </div>
      </template>
    </BTable>

    <market-table-mobile
      v-else
      :items="items"
      @dialog-handler="(e) => supplyDialogHandler(e.item, e.action)"
      @on-row-clicked="onRowClicked"
    />

    <j-loading-spinner v-if="loading">
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
