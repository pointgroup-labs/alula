<script lang="ts" setup>
import type { SuppliedCardTableItem } from '~/types/table'
import { calculateTotalStake } from '@alula/client-sdk/src/utils'
import { formatPrice, shortenNumber } from '~/utils'

const { width } = useWindowSize()

const userStore = useUserStore()
const { getFullTokenData } = useTokensStore()

const marketsStore = useMarketsStore()

const market = useMarketActions()

const loadingMarkets = computed(() => marketsStore.state.loadingLeveragePools || marketsStore.state.loading)

const isHasObligations = computed(() => Object.keys(userStore.state.obligations).length > 0)

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'balance', label: 'Supply', align: 'right' },
  { key: 'supply_apy', label: 'Supply APY', align: 'center' },
  { key: 'action', label: '', thClass: 'profile-action', tdClass: 'profile-action' },
]

const items: ComputedRef<SuppliedCardTableItem[] | []> = computed(() => {
  const res = []
  for (const market in userStore.state.obligations) {
    const deposits = userStore.state.obligations[market]?.deposits ?? []
    const marketState = marketsStore.state.markets[market]?.marketState
    const poolsData = marketState?.pools_data
    const oraclePriceDecimals = marketState?.oracle_price_decimals ?? 0
    for (const deposit of deposits) {
      const [pool_address, dep] = deposit
      const activePool = poolsData?.find(data => data.pool.pool_address === pool_address)
      if (!activePool) {
        continue
      }

      const assetDecimals = activePool.pool.token_decimals ?? 7

      const available = Number(bigintToNumber(activePool.total_available_adjusted, assetDecimals))

      const deposited = calculateTotalStake(dep.j_tokens, {
        total_j_tokens: activePool.pool.total_j_tokens,
        total_borrowed: activePool.pool.total_borrowed,
        total_available: activePool.total_available_adjusted,
      })
      const userCollateral = bigintToNumber(dep.collateral, assetDecimals)
      const balance = Number(deposited) + Number(userCollateral)

      const price = activePool.oracle_asset_price ? bigintToNumber(activePool.oracle_asset_price, oraclePriceDecimals) : 0

      const poolApy = activePool.apy.supply_bps / 100

      const data = {
        raw: activePool,
        asset: getFullTokenData(activePool.pool.token_symbol),
        assetDecimals,
        balance,
        balanceUsd: balance * Number(price),
        price: Number(price),
        available,
        supply_apy: `${truncatePercent(poolApy || 0, 2)}%`,
        action: 'Withdraw',
        pool_address,
        collateral: userCollateral,
        market,
      }

      res.push(data)
    }
  }
  return res.filter(Boolean) as SuppliedCardTableItem[]
})

const totalSupplyUsd = computed(() => {
  let sum = 0
  for (const item of items.value) {
    sum += Number(item.balanceUsd)
  }
  return formatCompactUSD(sum, 2, 2)
})

function withdrawDialogHandler(item: SuppliedCardTableItem) {
  marketsStore.selectedMarketName = String(item.market)
  marketsStore.selectedPoolAddress = item.pool_address
  marketsStore.dialogWithdraw = true
}
</script>

<template>
  <div class="account-card">
    <div class="account-card__title">
      My Supplies

      <metric-indicator
        v-if="isHasObligations"
        label="Total Supplied"
        :value="`${totalSupplyUsd}`"
        color="#17B26A"
      />
    </div>

    <div v-if="!isHasObligations && (userStore.loading || loadingMarkets)">
      <supply-table-skeleton />
    </div>

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
          :class="{ 'table-loading': userStore.loading }"
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
                <div
                  class="market-table__asset__info__symbol"
                  style="text-transform: capitalize;"
                >
                  {{ data.item.market }}
                </div>
              </div>
            </div>
          </template>

          <template #cell(balance)="data">
            <div class="table-cell justify-content-end with-price">
              {{
                Number(data.item.balance) > 1000 ? shortenNumber(Number(data.item.balance)) : Number(data.item.balance).toFixed(5)
              }}
              <span>${{ formatPrice(data.item.balanceUsd, 2, 2) }}</span>
            </div>
          </template>

          <template #cell(supply_apy)="data">
            <div class="table-cell justify-content-center">
              <j-pill-label
                variant="success"
                size="sm"
              >
                {{ data.item.supply_apy }}
              </j-pill-label>
            </div>
          </template>

          <template #cell(action)="data">
            <div class="table-cell justify-content-end">
              <j-btn
                variant="brand-outlined"
                size="sm"
                :disabled="market.isDisabled(data.item.pool_address, 'withdraw', data.item.market!)"
                :loading="market.isLoading(data.item.pool_address, 'withdraw', data.item.market!)"
                @click="withdrawDialogHandler(data.item)"
              >
                {{ data.item.action }}
              </j-btn>
            </div>
          </template>
        </BTable>

        <account-supply-table-mobile
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
        No supplied assets
      </div>
    </div>
  </div>

  <client-only>
    <withdraw-dialog v-model="marketsStore.dialogWithdraw" />
  </client-only>
</template>

<style lang="scss">
.account-card {
  position: relative;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: 12px;
  overflow: hidden;

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
    color: $navi-25;
    font-size: 20px;
    font-style: normal;
    font-weight: 700;
    line-height: 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: $spacing-xl;
    border-bottom: 1px solid $border-primary;

    @media (max-width: $breakpoint-sm) {
      font-size: 16px;
    }
  }

  .no-data {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-height: 116px;
    max-height: 200px;
    font-size: 14px;
    font-style: normal;
    font-weight: 400;
    line-height: 16px;
    color: $navi-100;
  }
}

.account-table {
  th {
    &:first-child {
      padding-left: 16px;
    }
    &:last-child {
      padding-right: 16px;
    }
  }
  td {
    &:first-child {
      padding-left: 16px;
    }
    &:last-child {
      padding-right: 16px;
    }
  }
  tbody tr {
    cursor: default;
    border-color: $border-primary;

    &:hover {
      background-color: $navi-700;
    }

    &:last-child {
      border-radius: 0 0 12px 12px !important;
    }
  }

  .profile-action {
    width: 100px;
  }
}
</style>
