<script lang="ts" setup>
import type { SupplyTableItem } from '~/types/table'

const emits = defineEmits(['showInfo'])

const marketsStore = useMarketsStore()

const pools = computed(() => marketsStore.state.pollsData)
const loading = computed(() => marketsStore.state.loading)

watch(pools, async (p) => {
  console.log(pools.value)
}, { immediate: true })

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'pool_size', label: 'Pool Size', align: 'right' },
  { key: 'price', label: 'Price', align: 'right' },
  { key: 'deposit_apy', label: 'Deposit APY', align: 'center' },
  { key: 'trust_ratio', label: 'Trust Ratio', align: 'right' },
  { key: 'risk_floor', label: 'Risk Floor', align: 'right' },
  { key: 'position', label: 'You Position', align: 'center' },
  { key: 'action', label: '' },
]

// const items: SupplyTableItem[] = [
//   {
//     asset: { name: 'Stellar', symbol: 'XLM', icon: stellarIcon },
//     pool_size: '23.89K',
//     price: '131,85 USD',
//     deposit_apy: '18.93%',
//     trust_ratio: '75.05%',
//     risk_floor: '75.05%',
//     position: '200,458 XLM',
//     action: 'Supply',
//   },
//   {
//     asset: { name: 'USDT', symbol: 'USDT', icon: 'https://icons.iconarchive.com/icons/cjdowner/cryptocurrency-flat/512/Tether-USDT-icon.png' },
//     pool_size: '23.89K',
//     price: '131,85 USD',
//     deposit_apy: '18.93%',
//     trust_ratio: '75.05%',
//     risk_floor: '75.05%',
//     position: '200,458 XLM',
//     action: 'Supply',
//   },
//   {
//     asset: { name: 'Solana', symbol: 'SOL', icon: 'https://upload.wikimedia.org/wikipedia/en/b/b9/Solana_logo.png' },
//     pool_size: '23.89K',
//     price: '131,85 USD',
//     deposit_apy: '18.93%',
//     trust_ratio: '75.05%',
//     risk_floor: '75.05%',
//     position: '200,458 XLM',
//     action: 'Supply',
//   },
// ]

const items = computed<SupplyTableItem[]>(() => {
  // return pools.value.map((p) => {
  //   const tokenName = p.token_ticker
  //   const icon = getTokenIcon(tokenName)
  //   return {
  //     asset: { name: tokenName, symbol: tokenName, icon },
  //     pool_size: '23.89K',
  //     price: '131,85 USD',
  //     deposit_apy: '18.93%',
  //     trust_ratio: '75.05%',
  //     risk_floor: '75.05%',
  //     position: '200,458 XLM',
  //     action: 'Supply',
  //   }
  // })
  return []
})

const dialog = ref(false)
const selectedItem = ref<SupplyTableItem>()

async function supplyDialogHandler(data: { item: SupplyTableItem }) {
  selectedItem.value = data.item
  dialog.value = true
}

function onRowClicked(item: any, _index: number, _event: any) {
  marketsStore.selectedMarketInfo = item
  emits('showInfo')
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

      <template #cell(pool_size)="data">
        <div class="table-cell justify-content-end">
          {{ data.item.pool_size }}
        </div>
      </template>

      <template #cell(price)="data">
        <div class="table-cell justify-content-end">
          {{ data.item.price }}
        </div>
      </template>

      <template #cell(deposit_apy)="data">
        <div class="table-cell justify-content-center">
          <j-pill-label
            color="#111"
            bg-color="#08b57680"
            size="md"
          >
            {{ data.item.deposit_apy }}
          </j-pill-label>
        </div>
      </template>

      <template #cell(trust_ratio)="data">
        <div class="table-cell justify-content-end">
          {{ data.item.trust_ratio }}
        </div>
      </template>

      <template #cell(risk_floor)="data">
        <div class="table-cell justify-content-end">
          {{ data.item.risk_floor }}
        </div>
      </template>

      <template #cell(position)="data">
        <div class="table-cell justify-content-center">
          <j-pill-label
            variant="secondary"
            size="md"
          >
            {{ data.item.position }}
          </j-pill-label>
        </div>
      </template>

      <template #cell(action)="data">
        <div class="table-cell justify-content-end">
          <j-btn
            size="lg"
            pill
            icon-right
            @click="supplyDialogHandler(data)"
          >
            Supply
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
    v-model="dialog"
    :data="selectedItem"
  />
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

  .no-data {
    display: flex;
    align-items: center;
    justify-content: center;
  }
}
</style>
