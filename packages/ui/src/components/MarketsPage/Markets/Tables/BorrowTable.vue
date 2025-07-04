<script lang="ts" setup>
import type { BorrowTableItem } from '~/types/table'
import stellarIcon from '~/assets/img/assets/stellar.png'

const emits = defineEmits(['showInfo'])

const marketsStore = useMarketsStore()

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'available', label: 'Available', align: 'right' },
  { key: 'price', label: 'Price', align: 'right' },
  { key: 'borrow_apy', label: 'Borrow APY', align: 'center' },
  { key: 'utilization_rate', label: 'Utilization Rate', align: 'right' },
  { key: 'position', label: 'You Position', align: 'center' },
  { key: 'action', label: '' },
]

const items = [
  {
    asset: { name: 'Stellar', symbol: 'XLM', icon: stellarIcon },
    available: '23.89K',
    price: '131,85 USD',
    borrow_apy: '18.93%',
    utilization_rate: '75.05%',
    position: '200,458 XLM',
    action: 'Supply',
  },
  {
    asset: { name: 'Stellar', symbol: 'XLM', icon: stellarIcon },
    available: '23.89K',
    price: '131,85 USD',
    borrow_apy: '18.93%',
    utilization_rate: '75.05%',
    position: '200,458 XLM',
    action: 'Supply',
  },
  {
    asset: { name: 'Stellar', symbol: 'XLM', icon: stellarIcon },
    available: '23.89K',
    price: '131,85 USD',
    borrow_apy: '18.93%',
    utilization_rate: '75.05%',
    position: '200,458 XLM',
    action: 'Supply',
  },
]

const dialog = ref(false)
const selectedItem = ref<BorrowTableItem>()

function supplyDialogHandler(data: { item: BorrowTableItem }) {
  selectedItem.value = data.item
  dialog.value = true
}

function onRowClicked(item: any, _index: number, _event: any) {
  marketsStore.selectedMarketInfo = item
  emits('showInfo')
}
</script>

<template>
  <BTable
    borderless
    :fields="fields"
    :items="items"
    responsive
    class="market-table"
    @row-clicked="onRowClicked"
  >
    <template v-for="field in fields" :key="field.key" #[`head(${field.key})`]="data">
      <span :style="{ '--align': field.align }">{{ data.label }}</span>
    </template>

    <template #cell(asset)="data">
      <div class="market-table__asset">
        <img :src="data.item.asset.icon" alt="">
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

    <template #cell(available)="data">
      <div class="table-cell justify-content-end">
        {{ data.item.available }}
      </div>
    </template>

    <template #cell(price)="data">
      <div class="table-cell justify-content-end">
        {{ data.item.price }}
      </div>
    </template>

    <template #cell(borrow_apy)="data">
      <div class="table-cell justify-content-center">
        <j-pill-label color="#111" bg-color="#e49c0b80" size="md">
          {{ data.item.borrow_apy }}
        </j-pill-label>
      </div>
    </template>

    <template #cell(utilization_rate)="data">
      <div class="table-cell justify-content-end">
        {{ data.item.utilization_rate }}
      </div>
    </template>

    <template #cell(position)="data">
      <div class="table-cell justify-content-center">
        <j-pill-label variant="secondary" size="md">
          {{ data.item.position }}
        </j-pill-label>
      </div>
    </template>

    <template #cell(action)="data">
      <div class="table-cell justify-content-end">
        <j-btn variant="accent" size="lg" pill icon-right @click="supplyDialogHandler(data)">
          Borrow
        </j-btn>
      </div>
    </template>
  </BTable>

  <borrow-dialog v-model="dialog" :data="selectedItem" />
</template>
