<script lang="ts" setup>
import type { SuppliedCardTableItem } from '~/types/table'
import stellarIcon from '~/assets/img/assets/stellar.png'

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'balance', label: 'Balance', align: 'right' },
  { key: 'supply_apy', label: 'Supply APY', align: 'center' },
  { key: 'action', label: '' },
]

const items: SuppliedCardTableItem[] = [
  {
    asset: { name: 'Stellar', symbol: 'XLM', icon: stellarIcon },
    balance: '789.5488',
    supply_apy: '18.93%',
    action: 'Withdraw',
  },
  {
    asset: { name: 'USDT', symbol: 'USDT', icon: 'https://icons.iconarchive.com/icons/cjdowner/cryptocurrency-flat/512/Tether-USDT-icon.png' },
    balance: '32.5488',
    supply_apy: '11.93%',
    action: 'Withdraw',
  },
  {
    asset: { name: 'Solana', symbol: 'SOL', icon: 'https://upload.wikimedia.org/wikipedia/en/b/b9/Solana_logo.png' },
    balance: '7839.5488',
    supply_apy: '12.04%',
    action: 'Withdraw',
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
      Your Supplies
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

      <template #cell(balance)="data">
        <div class="table-cell justify-content-end">
          {{ data.item.balance }}
        </div>
      </template>

      <template #cell(supply_apy)="data">
        <div class="table-cell justify-content-center">
          <j-pill-label color="#111" bg-color="#08b57680" size="md">
            {{ data.item.supply_apy }}
          </j-pill-label>
        </div>
      </template>

      <template #cell(action)="data">
        <div class="table-cell justify-content-center">
          <j-btn pill variant="dark" size="lg" @click="withdrawDialogHandler(data)">
            {{ data.item.action }}
          </j-btn>
        </div>
      </template>
    </BTable>

    <div v-else class="no-data">
      No data
    </div>
  </div>

  <withdraw-dialog v-model="dialog" :data="selectedItem" />
</template>

<style lang="scss">
.account-card {
  display: flex;
  flex-direction: column;
  gap: $spacing-16;

  &__title {
    font-size: 20px;
    font-style: normal;
    font-weight: 700;
    line-height: 20px;
  }

  &__table {
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
  }

  .no-data {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100px;
    max-height: 200px;
  }
}
</style>
