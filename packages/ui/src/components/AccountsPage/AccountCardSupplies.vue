<script lang="ts" setup>
import type { DepositObligation } from 'sdk'
import type { SuppliedCardTableItem } from '~/types/table'
import { bigintToNumber, getTokenIcon, shortenNumber, truncatePercent } from '~/utils'

const clientStore = useClientStore()
const decimals = computed(() => clientStore.assetDecimals)

const userStore = useUserStore()
const obligation = computed(() => userStore.userObligation)

const marketsStore = useMarketsStore()
const pools = computed(() => marketsStore.state.pollsData)

const market = useMarket()

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'balance', label: 'Balance', align: 'right' },
  { key: 'supply_apy', label: 'Supply APY', align: 'center' },
  { key: 'action', label: '' },
]

const items: ComputedRef<SuppliedCardTableItem[]> = computed(() => {
  const deposits = obligation.value?.deposits || []
  return deposits.map((item: [string, DepositObligation]) => {
    const [pool_address, deposit] = item
    const pool = pools.value.find(p => p.pool_address === pool_address)
    if (!pool) {
      return {
        asset: { name: 'Unknown', symbol: 'Unknown', icon: '' },
        balance: '0',
        supply_apy: '0%',
        action: 'Withdraw',
        pool_address,
      }
    }
    const tokenName = pool.token_ticker
    const icon = getTokenIcon(tokenName)
    const userShares = bigintToNumber(deposit.shares, decimals.value)
    const totalShares = bigintToNumber(pool.total_shares, decimals.value)
    const userBorrowInPoolPercentage = Number(userShares) / Number(totalShares)

    const available = Number(bigintToNumber(pool.available, decimals.value))
    const totalBorrowed = Number(bigintToNumber(pool.total_borrowed, decimals.value))
    const totalSupplied = available + totalBorrowed

    const userSupplied = totalSupplied * userBorrowInPoolPercentage

    const poolApy = pool.pool_apy.supply_bps / 100
    return {
      asset: { name: tokenName, symbol: tokenName, icon },
      balance: userSupplied,
      available,
      supply_apy: `${truncatePercent(poolApy || 0, 2)}%`,
      action: 'Withdraw',
      pool_address,
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
      Your Supplies
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

        <template #cell(balance)="data">
          <div class="table-cell justify-content-end">
            {{ shortenNumber(Number(data.item.balance)) }}
          </div>
        </template>

        <template #cell(supply_apy)="data">
          <div class="table-cell justify-content-center">
            <j-pill-label
              color="#111"
              bg-color="#08b57680"
              size="md"
            >
              {{ data.item.supply_apy }}
            </j-pill-label>
          </div>
        </template>

        <template #cell(action)="data">
          <div class="table-cell justify-content-center">
            <j-btn
              pill
              variant="dark"
              size="lg"
              :disabled="market.isDisabled(data.item.pool_address)"
              :loading="market.isLoading(data.item.pool_address)"
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

  <withdraw-dialog
    v-model="dialog"
    :data="selectedPool"
  />
</template>

<style lang="scss">
.account-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: $spacing-16;

  .loading-spinner {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: rgba(255, 255, 255, 0.5);
    width: 100%;
    height: 100%;
  }

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
