<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { formatPrice, getTokenIcon, shortenNumber, truncatePercent } from '~/utils'

const infoDialog = ref(false)

const client = useClientStore()
const marketsStore = useMarketsStore()

const assetDecimals = computed(() => client.assetDecimals)

const pools = computed(() => marketsStore.selectedMarketPools)
const loading = computed(() => marketsStore.state.loading)

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'total_supply', label: 'Total Supply', align: 'right' },
  { key: 'total_borrowed', label: 'Total Borrowed', align: 'right' },
  { key: 'deposit_apy', label: 'Deposit APY', align: 'center' },
  { key: 'borrow_apy', label: 'Borrow APY', align: 'center' },
  { key: 'utilization_rate', label: 'Utilization Rate', align: 'right' },
  { key: 'max_ltv', label: 'Max LTV', align: 'center' },
  { key: 'action', label: '' },
]

const items = computed<MarketTableItem[]>(() => {
  return pools.value.map((p, i) => {
    const tokenName = p.token_ticker
    const icon = getTokenIcon(tokenName)
    const supply = (Number(p.available) + Number(p.total_borrowed)) / 10 ** assetDecimals.value
    const borrowed = Number(p.total_borrowed) / 10 ** assetDecimals.value
    const depositApy = p.pool_apy.supply_bps / 100
    const borrowApy = p.pool_apy.borrow_bps / 100
    const utilRate = Number(p.total_borrowed) / Number((p.available + p.total_borrowed)) * 100
    const maxLTV = Number(p.config.open_ltv_bps) / 100
    const supplyLimit = i % 2 === 0 ? 0 : 1000
    return {
      raw: p,
      asset: { name: tokenName, symbol: tokenName, icon },
      total_supply: supply,
      total_borrowed: borrowed,
      deposit_apy: `${truncatePercent(depositApy || 0, 2)}%`,
      borrow_apy: `${truncatePercent(borrowApy || 0, 2)}%`,
      utilization_rate: `${truncatePercent(utilRate || 0, 2)}%`,
      max_ltv: `${truncatePercent(maxLTV || 0, 2)}%`,
      action: 'Supply',
      price: p.pool_price,
      supply_limit: supplyLimit,
      available: Number(p.available) / (10 ** assetDecimals.value),
    }
  })
})

const dialogSupply = ref(false)
const dialogBorrow = ref(false)
const selectedItem = ref<MarketTableItem>()

async function supplyDialogHandler(data: { item: MarketTableItem }, action: 'supply' | 'borrow') {
  selectedItem.value = data.item
  action === 'supply' ? dialogSupply.value = true : dialogBorrow.value = true
}

function onRowClicked(item: any, _index: number, _event: any) {
  marketsStore.selectedMarketInfo = item
  infoDialog.value = true
}

function amountToUsd(amount: number, price: number) {
  return shortenNumber((Number(amount) * Number(price)) || 0)
}
</script>

<template>
  <div class="table-wrapper">
    <BTable
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
              {{ data.item.asset.name }}
            </div>
            <div class="market-table__asset__info__symbol">
              {{ data.item.asset.symbol }}
            </div>
          </div>
        </div>
      </template>

      <template #cell(total_supply)="data">
        <j-tooltip tooltip-class="table-cell justify-content-end with-price">
          {{ data.item.total_supply > 1000 ? shortenNumber(data.item.total_supply) : data.item.total_supply }}
          <span>${{ amountToUsd(data.item.total_supply, data.item.price) }}</span>
          <template #content>
            {{ formatPrice(data.item.total_supply) }}
          </template>
        </j-tooltip>
      </template>

      <template #cell(total_borrowed)="data">
        <j-tooltip tooltip-class="table-cell justify-content-end with-price">
          <div class="table-cell justify-content-end with-price">
            {{ data.item.total_borrowed > 1000 ? shortenNumber(data.item.total_borrowed) : data.item.total_borrowed }}
            <span>${{ amountToUsd(data.item.total_borrowed, data.item.price) }}</span>
          </div>
          <template #content>
            {{ formatPrice(data.item.total_borrowed) }}
          </template>
        </j-tooltip>

      </template>

      <template #cell(deposit_apy)="data">
        <div class="table-cell justify-content-center">
          <j-pill-label
            color="#111"
            bg-color="rgba(8, 181, 118, 0.50)"
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
            bg-color="rgba(228, 156, 11, 0.50)"
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
            @click="supplyDialogHandler(data, 'supply')"
          >
            Supply
          </j-btn>
          <j-btn
            size="lg"
            pill
            icon-right
            variant="accent"
            @click="supplyDialogHandler(data, 'borrow')"
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

    <j-loading-spinner v-if="loading">
      Loading...
    </j-loading-spinner>
  </div>

  <supply-dialog
    v-model="dialogSupply"
    :data="selectedItem"
  />

  <borrow-dialog
    v-model="dialogBorrow"
    :data="selectedItem"
  />

  <market-info-dialog v-model="infoDialog" />
</template>

<style lang="scss">
.market-table {
  th {
    color: $neutral-6;
    font-size: 16px;
    font-style: normal;
    font-weight: 600;
    line-height: 20px;

    span {
      width: 100%;
      display: block;
      text-align: var(--align, center);
      white-space: nowrap;
    }
  }

  tbody {
    tr {
      height: 80px;
      cursor: pointer;

      &:nth-child(even) {
        td {
          background-color: $neutral-2;
        }
      }
    }

    td {
      text-align: center;
      vertical-align: middle;
    }
  }

  .table-cell {
    width: 100%;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    font-style: normal;
    font-weight: 600;
    line-height: 20px;
    white-space: nowrap;
  }

  .with-price {
    flex-direction: column;
    align-items: flex-end;

    span {
      color: $neutral-12;
      font-size: 12px;
      font-weight: 500;
      line-height: 16px;
    }
  }

  &__asset {
    display: flex;
    align-items: center;
    gap: $spacing-8;

    img {
      width: 40px;
      height: 40px;
      object-fit: contain;
    }

    &__info {
      display: flex;
      flex-direction: column;
      gap: 2px;
      font-style: normal;

      &__name {
        font-size: 20px;
        font-weight: 500;
        line-height: 20px;
        text-align: left;
      }

      &__symbol {
        color: $neutral-6;
        font-size: 12px;
        font-weight: 500;
        line-height: 16px;
        text-align: left;
      }
    }
  }

  &__action {
    gap: $spacing-8;
  }

  .no-data {
    display: flex;
    align-items: center;
    justify-content: center;
  }
}
</style>
