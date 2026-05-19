<script lang="ts" setup>
import type { SuppliedCardTableItem } from '~/types/table'
import { calculateTotalStake } from '@alula/client-sdk/src/utils'
import { formatPrice, shortenNumber } from '~/utils'

const { width } = useWindowSize()

const userStore = useUserStore()
const { getFullTokenData } = useTokensStore()

const marketsStore = useMarketsStore()

const market = useMarketActions()

const markets = computed(() => Object.keys(marketsStore.state.markets) ?? [])
const isLoading = computed(() => (marketsStore.state.loadingLeveragePools || marketsStore.state.loading) || userStore.loading)

// const isHasObligations = computed(() => Object.keys(userStore.state.obligations).length > 0)

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'balance', label: 'Supply', align: 'right' },
  { key: 'supply_apy', label: 'Supply APY', align: 'center' },
  { key: 'earning', label: 'Earning', align: 'center' },
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

      const deposited = +calculateTotalStake(dep.j_tokens || 0n, {
        total_j_tokens: activePool.pool.total_j_tokens,
        total_borrowed: activePool.pool.total_borrowed,
        total_available: activePool.total_available_adjusted,
      }, assetDecimals) || 0
      const collateral = Number(bigintToNumber(BigInt(dep.collateral || 0n), assetDecimals)) || 0
      const balance = deposited + collateral

      const depositedPercent = calcStakePercent(deposited, balance)
      const collateralPercent = calcStakePercent(collateral, balance)

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
        market,
        deposited,
        depositedPercent,
        collateral,
        collateralPercent,
      }

      res.push(data)
    }
  }
  return res.filter(Boolean) as SuppliedCardTableItem[]
})

const totalSupplyUsdRaw = computed(() => {
  let sum = 0
  for (const item of items.value) {
    sum += Number(item.balanceUsd)
  }
  return sum
})

const totalSupplyUsd = computed(() => formatCompactUSD(totalSupplyUsdRaw.value, 2, 2))

function withdrawDialogHandler(item: SuppliedCardTableItem) {
  marketsStore.selectedMarketName = String(item.market)
  marketsStore.selectedPoolAddress = item.pool_address
  marketsStore.dialogWithdraw = true
}

function calcStakePercent(stake: number, total: number) {
  return (stake / total) * 100
}
</script>

<template>
  <div class="portfolio-card">
    <div class="portfolio-card__title">
      My Supplies

      <metric-indicator
        v-if="totalSupplyUsdRaw > 0"
        label="Total Supplied"
        :value="`${totalSupplyUsd}`"
        color="#17B26A"
      />
    </div>

    <div v-if="markets.length === 0 && isLoading">
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
          class="portfolio-table market-table"
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
              {{ Number(data.item.balance) > 1000 ? shortenNumber(Number(data.item.balance)) : Number(data.item.balance).toFixed(5) }}
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

          <template #cell(earning)="data">
            <div class="table-cell justify-content-center">
              <j-tooltip tooltip-class="earning-tip">
                <div
                  class="earning-indicator"
                  :style="{
                    '--deposit-width': `${data.item.depositedPercent}%`,
                    '--collateral-width': `${data.item.collateralPercent}%`,
                  }"
                />
                <div class="earning-percent">
                  <span
                    class="text-num"
                    :class="[`text-${Number(data.item.depositedPercent) > 0 ? 'positive' : 'accent'}`]"
                  >
                    {{ truncatePercent(data.item.depositedPercent, 2) }}%
                  </span>
                </div>

                <template #content>
                  This shows how much of your deposit is actively earning yield.
                  <br>
                  <br>
                  • {{ formatCompactUSD(Number(data.item.deposited) * Number(data.item.price)) }} in supply is earning interest
                  <template v-if="Number(data.item.collateral) > 0">
                    <br>
                    • {{ formatCompactUSD(Number(data.item.collateral) * Number(data.item.price)) }} in collateral is not earning
                    <br>
                    <br>

                    Move funds from collateral to supply to start earning yield.
                  </template>
                </template>
              </j-tooltip>
            </div>
          </template>

          <template #cell(action)="data">
            <div class="table-cell justify-content-end">
              <j-btn
                variant="outlined-brand"
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

        <portfolio-supply-table-mobile
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
.portfolio-card {
  position: relative;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: 12px;
  width: 100%;
  display: flex;
  flex-direction: column;

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
    height: 100%;
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 14px;
    font-style: normal;
    font-weight: 400;
    line-height: 16px;
    color: $navi-100;
  }
}

.portfolio-table {
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

  .earning-tip {
    display: flex;
    align-items: center;
  }

  .earning-indicator {
    position: relative;
    width: 50px;
    height: 4px;
    border-radius: 10px;
    background-color: color-mix(in oklab, #1a2335 70%, transparent);
    overflow: hidden;
    flex-shrink: 0;
    margin-right: 4px;
    font-family: 'JetBrainsMono', monospace;

    &::before {
      content: '';
      position: absolute;
      left: 0;
      top: 0;
      height: 100%;
      width: var(--deposit-width);
      background-color: $success;
    }

    &::after {
      content: '';
      position: absolute;
      right: 0;
      top: 0;
      height: 100%;
      width: var(--collateral-width);
      background-color: $accent;
    }
  }

  .earning-percent {
    font-size: 11px;
    color: $text-tertiary;
  }
}
</style>
