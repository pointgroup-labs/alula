<script lang="ts" setup>
import type { DepositObligation } from '@jlend/sdk'
import type { SuppliedCardTableItem } from '~/types/table'
import { bigintToNumber, formatPrice, getTokenIcon, getTokenName, shortenNumber, truncatePercent } from '~/utils'

const { width } = useWindowSize()

const clientStore = useClientStore()
const decimals = computed(() => clientStore.assetDecimals)

const userStore = useUserStore()
const obligation = computed(() => userStore.userObligation)

const marketsStore = useMarketsStore()
const pools = computed(() => marketsStore.state.pools)

const market = useMarket()

const loadingMarkets = computed(() => marketsStore.state.loadingLeveragePools || marketsStore.state.loading)

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
      return null
    }
    const tokenSymbol = pool.token_ticker
    const tokenName = getTokenName(tokenSymbol)
    const icon = getTokenIcon(tokenSymbol)
    const userShares = bigintToNumber(deposit.shares, decimals.value)
    const totalShares = bigintToNumber(pool.total_shares, decimals.value)
    const userBorrowInPoolPercentage = Number(userShares) / Number(totalShares)

    const available = Number(bigintToNumber(pool.available, decimals.value))
    const totalBorrowed = Number(bigintToNumber(pool.total_borrowed, decimals.value))
    const totalSupplied = available + totalBorrowed

    const userSupplied = totalSupplied * userBorrowInPoolPercentage
    const userCollateral = bigintToNumber(deposit.collateral, decimals.value)
    const balance = Number(userSupplied) + Number(userCollateral)

    const poolApy = pool.pool_apy.supply_bps / 100

    return {
      raw: pool,
      asset: { name: tokenName, symbol: tokenSymbol, icon },
      balance,
      balanceUsd: formatPrice(balance * Number(pool.pool_price), 2, 2),
      price: Number(pool.pool_price),
      available,
      supply_apy: `${truncatePercent(poolApy || 0, 2)}%`,
      action: 'Withdraw',
      pool_address,
      collateral: userCollateral,
    }
  })?.filter((item: SuppliedCardTableItem) => item)
})

const dialog = ref(false)
const selectedPoolAddress = ref()
const selectedPool = computed(() => items.value.find(item => item.pool_address === selectedPoolAddress.value))

function withdrawDialogHandler(item: SuppliedCardTableItem) {
  selectedPoolAddress.value = item?.pool_address
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
      Your Supplies
    </div>

    <template v-if="items.length === 0 && (userStore.loading || loadingMarkets)">
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
      <template v-if="items.length > 0">
        <BTable
          v-if="width >= 1024"
          borderless
          :fields="fields"
          :items="items"
          responsive
          class="account-table market-table"
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

          <template #cell(balance)="data">
            <j-tooltip tooltip-class="table-cell justify-content-end with-price">
              {{
                Number(data.item.balance) > 1000 ? shortenNumber(Number(data.item.balance)) : Number(data.item.balance).toFixed(5)
              }}
              <span>${{ data.item.balanceUsd }}</span>
              <template #content>
                {{ formatPrice(data.item.balance) }}
              </template>
            </j-tooltip>
          </template>

          <template #cell(supply_apy)="data">
            <div class="table-cell justify-content-center">
              <j-pill-label
                color="#111"
                variant="success"
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
                :disabled="market.isDisabled(data.item.pool_address, 'withdraw')"
                :loading="market.isLoading(data.item.pool_address, 'withdraw')"
                @click="withdrawDialogHandler(data.item)"
              >
                {{ data.item.action }}
              </j-btn>
            </div>
          </template>
        </BTable>

        <account-supplie-table-mobile
          v-else
          :items="items"
          @dialog-handler="(e: any) => withdrawDialogHandler(e.item)"
        />

      </template>
      <div
        v-else
        class="no-data"
      >
        <i-app-strongbox-icon />
        no supplied assets
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

    @media (max-width: $breakpoint-sm) {
      font-size: 16px;
    }
  }

  .no-data {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: $spacing-8;
    min-height: 100px;
    max-height: 200px;
    font-size: 14px;
    font-style: normal;
    font-weight: 400;
    line-height: 16px;
  }
}

.account-table {
  tbody tr {
    cursor: default;
  }
}

body.body--dark {
  .account-card {
    &__title {
      color: #fff;
    }
    .no-data {
      color: $neutral-9;
    }
    .loading-spinner {
      background: rgba(0, 0, 0, 0.5);
      color: #fff;

      .spinner-border {
        color: #fff !important;
      }
    }
  }
}
</style>
