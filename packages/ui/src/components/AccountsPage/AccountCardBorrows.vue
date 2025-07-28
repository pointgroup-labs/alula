<script lang="ts" setup>
import type { BorrowObligation } from 'sdk'
import type { BorrowCardTableItem } from '~/types/table'
import { bigintToNumber, destructurePoolAsset, formatPrice, getTokenIcon, getTokenName, shortenNumber, truncatePercent } from '~/utils'

const clientStore = useClientStore()
const decimals = computed(() => clientStore.assetDecimals)

const userStore = useUserStore()
const obligation = computed(() => userStore.userObligation)

const marketsStore = useMarketsStore()
const pools = computed(() => marketsStore.state.pools)

const market = useMarket()

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
    const tokenSymbol = pool.token_ticker
    const tokenName = getTokenName(tokenSymbol)
    const icon = getTokenIcon(tokenSymbol)
    const userBorrowed = bigintToNumber(borrow.borrowed + borrow.unpaid_interest, decimals.value)
    const userBorrowedUsd = formatPrice(Number(userBorrowed) * Number(pool.pool_price), 2, 2)

    const [, asset_issuer] = destructurePoolAsset(pool.name)
    const borrowApy = pool.pool_apy.borrow_bps / 100

    return {
      raw: pool,
      asset: { name: tokenName, symbol: tokenSymbol, icon },
      debt: userBorrowed,
      debtUsd: userBorrowedUsd,
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

watch(selectedPool, (p) => {
  if (!p) {
    dialog.value = false
  }
})
</script>

<template>
  <div class="account-card">
    <div class="account-card__title">
      Your Borrows
    </div>

    <template v-if="items.length === 0 && userStore.loading">
      <j-skeleton
        height="36"
        full-width
      />
      <j-skeleton
        height="80"
        full-width
        style="margin-top: -8px;"
      />
    </template>

    <div
      v-else
      class="table-wrapper"
    >
      <BTable
        v-if="items.length > 0"
        borderless
        :fields="fields"
        :items="items"
        responsive
        class="account-card__table market-table"
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

        <template #cell(debt)="data">
          <j-tooltip tooltip-class="table-cell table-cell__dept justify-content-end with-price">
            {{ Number(data.item.debt) > 1000 ? shortenNumber(Number(data.item.debt)) : Number(data.item.debt).toFixed(5) }}
            <span>${{ data.item.debtUsd }}</span>
            <template #content>
              {{ formatPrice(data.item.debt) }}
            </template>
          </j-tooltip>
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
              :disabled="market.isDisabled(data.item.pool_address, 'repay')"
              :loading="market.isLoading(data.item.pool_address, 'repay')"
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
        <i-app-percentage-square-icon /> no borrowed assets
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
