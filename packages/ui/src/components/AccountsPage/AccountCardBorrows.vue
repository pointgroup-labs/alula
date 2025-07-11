<script lang="ts" setup>
import type { BorrowObligation } from 'sdk'
import type { BorrowCardTableItem } from '~/types/table'
import { bigintToNumber, getTokenIcon, shortenNumber, truncatePercent } from '~/utils'

const clientStore = useClientStore()
const decimals = computed(() => clientStore.assetDecimals)

const userStore = useUserStore()
const obligation = computed(() => userStore.userObligation)

const marketsStore = useMarketsStore()
const pools = computed(() => marketsStore.state.pollsData)

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'debt', label: 'Debt', align: 'right' },
  { key: 'borrow_apy', label: 'Borrow APY', align: 'center' },
  { key: 'action', label: '' },
]

const items: ComputedRef<BorrowCardTableItem[]> = computed(() => {
  const borrows = obligation.value?.borrows || []
  return borrows.map((item: [string, BorrowObligation]) => {
    const [pool_address, borrow] = item
    const pool = pools.value.find(p => p.pool_address === pool_address)
    if (!pool) {
      return {
        asset: { name: 'Unknown', symbol: 'Unknown', icon: '' },
        balance: '0',
        supply_apy: '0%',
        action: 'Repay',
      }
    }
    const tokenName = pool.token_ticker
    const icon = getTokenIcon(tokenName)
    const userShares = bigintToNumber(borrow.borrowed, decimals.value)
    const totalShares = bigintToNumber(pool.total_shares, decimals.value)
    const userBorrowInPoolPercentage = Number(userShares) / Number(totalShares)

    const available = Number(bigintToNumber(pool.available, decimals.value))
    const totalBorrowed = Number(bigintToNumber(pool.total_borrowed, decimals.value))
    const totalSupplied = available + totalBorrowed

    const userBorrowed = totalSupplied * userBorrowInPoolPercentage
    const asset_issuer = pool.name.split(':')[1]
    const borrowApy = pool.pool_apy.borrow_bps / 100

    return {
      asset: { name: tokenName, symbol: tokenName, icon },
      debt: userBorrowed,
      borrow_apy: `${truncatePercent(borrowApy || 0, 2)}%`,
      action: 'Repay',
      pool_address,
      asset_issuer,
    }
  })
})

const dialog = ref(false)
const selectedPoolAddress = ref()
const selectedPool = computed(() => items.value.find(item => item.pool_address === selectedPoolAddress.value))

function withdrawDialogHandler(data: { item: any }) {
  selectedPoolAddress.value = data.item?.pool_address
  dialog.value = true
}
</script>

<template>
  <div class="account-card">
    <div class="account-card__title">
      Your Borrows
    </div>

    <div class="table-wrapper">

      <BTable
        v-if="items.length > 0"
        borderless
        :fields="fields"
        :items="items"
        responsive
        class="account-card__table"
      >
        <template
          v-for="field in fields"
          :key="field.key"
          #[`head(${field.key})`]="data"
        >
          <span :style="{ '--align': field.align }">{{ data.label }}</span>
        </template>

        <template #cell(asset)="data">
          <div class="account-card__table__asset">
            <img
              :src="data.item.asset.icon"
              alt=""
            >
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
            {{ shortenNumber(Number(data.item.debt) || 0) }}
          </div>
        </template>

        <template #cell(borrow_apy)="data">
          <div class="table-cell justify-content-center">
            <j-pill-label
              color="#111"
              bg-color="#e49c0b80"
              size="md"
            >
              {{ data.item.borrow_apy }}
            </j-pill-label>
          </div>
        </template>

        <template #cell(action)="data">
          <div class="table-cell justify-content-center">
            <j-btn
              pill
              variant="success"
              icon-right
              size="lg"
              class="repay-btn"
              @click="withdrawDialogHandler(data)"
            >
              {{ data.item.action }}
            </j-btn>
          </div>
        </template>
      </BTable>

      <div
        v-else
        class="no-data"
      >
        No data
      </div>

      <j-loading-spinner v-if="userStore.loading">
        Loading...
      </j-loading-spinner>
    </div>

  </div>

  <repay-dialog
    v-model="dialog"
    :data="selectedPool"
  />
</template>

<style lang="scss">
.account-card {
  .table-cell__dept {
    color: $warning;
  }
}
</style>
