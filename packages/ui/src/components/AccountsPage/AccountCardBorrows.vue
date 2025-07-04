<script lang="ts" setup>
import type { BorrowCardTableItem } from '~/types/table'
import stellarIcon from '~/assets/img/assets/stellar.png'

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'debt', label: 'Debt', align: 'right' },
  { key: 'borrow_apy', label: 'Borrow APY', align: 'center' },
  { key: 'action', label: '' },
]

const items: BorrowCardTableItem[] = [
  {
    asset: { name: 'Stellar', symbol: 'XLM', icon: stellarIcon },
    debt: '789.5488',
    borrow_apy: '18.93%',
    action: 'Repay',
  },
  {
    asset: { name: 'USDT', symbol: 'USDT', icon: 'https://icons.iconarchive.com/icons/cjdowner/cryptocurrency-flat/512/Tether-USDT-icon.png' },
    debt: '23.89K',
    borrow_apy: '11.93%',
    action: 'Repay',
  },
]

const dialog = ref(false)
const selectedItem = ref()

function withdrawDialogHandler(data: { item: any }) {
  selectedItem.value = data.item
  dialog.value = true
}
</script>

<template>
  <div class="account-card">
    <div class="account-card__title">
      Your Borrows
    </div>

    <BTable
      v-if="items.length > 0"
      borderless
      :fields="fields"
      :items="items"
      responsive
      class="account-card__table"
    >
      <template v-for="field in fields" :key="field.key" #[`head(${field.key})`]="data">
        <span :style="{ '--align': field.align }">{{ data.label }}</span>
      </template>

      <template #cell(asset)="data">
        <div class="account-card__table__asset">
          <img :src="data.item.asset.icon" alt="">
          <div class="account-card__table__asset__info">
            <div class="account-card__table__asset__info__name">
              {{ data.item.asset.name }}
            </div>
            <div class="account-card__table__asset__info__symbol">
              {{ data.item.asset.symbol }}
            </div>
          </div>
        </div>
      </template>

      <template #cell(debt)="data">
        <div class="table-cell table-cell__dept justify-content-end">
          {{ data.item.debt }}
        </div>
      </template>

      <template #cell(borrow_apy)="data">
        <div class="table-cell justify-content-center">
          <j-pill-label color="#111" bg-color="#e49c0b80" size="md">
            {{ data.item.borrow_apy }}
          </j-pill-label>
        </div>
      </template>

      <template #cell(action)="data">
        <div class="table-cell justify-content-center">
          <j-btn pill variant="success" icon-right size="lg" class="repay-btn" @click="withdrawDialogHandler(data)">
            {{ data.item.action }}
          </j-btn>
        </div>
      </template>
    </BTable>

    <div v-else class="no-data">
      No data
    </div>
  </div>

  <repay-dialog v-model="dialog" :data="selectedItem" />
</template>

<style lang="scss">
.account-card {
  .table-cell__dept {
    color: $warning;
  }
}
</style>
