<script lang="ts" setup>
import type { BorrowCardTableItem } from '~/types/table'
import { calculateBorrow } from '@alula/client-sdk/src/utils'
import { calcHealthFactor,
  destructurePoolAsset,
  formatPrice,
  shortenNumber,
  truncatePercent } from '~/utils'

const { width } = useWindowSize()

const userStore = useUserStore()
const { getFullTokenData } = useTokensStore()

const marketsStore = useMarketsStore()

const market = useMarketActions()

const markets = computed(() => Object.keys(marketsStore.state.markets) ?? [])
const isLoading = computed(() => (marketsStore.state.loadingLeveragePools || marketsStore.state.loading) || userStore.loading)

const isHasObligations = computed(() => Object.keys(userStore.state.obligations).length > 0)

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'debt', label: 'Debt', align: 'right' },
  { key: 'borrow_apy', label: 'Borrow APY', align: 'center' },
  { key: 'hf', label: 'Health Factor', align: 'right' },
  { key: 'action', label: '' },
]

const items: ComputedRef<BorrowCardTableItem[]> = computed(() => {
  const res = []
  for (const market in userStore.state.obligations) {
    const obligation = userStore.state.obligations[market]
    const borrows = obligation?.borrows ?? []
    const marketState = marketsStore.state.markets[market]?.marketState
    const poolsData = marketState?.pools_data
    const oraclePriceDecimals = marketState?.oracle_price_decimals ?? 0
    for (const borrow of borrows) {
      const [pool_address, bor] = borrow
      const activePool = poolsData?.find(data => data.pool.pool_address === pool_address)
      if (!activePool) {
        continue
      }
      const assetDecimals = activePool.pool.token_decimals ?? 7

      const rawDept = calculateBorrow(bor.d_tokens, {
        total_borrowed: activePool.pool.total_borrowed,
        total_d_tokens: activePool.pool.total_d_tokens,
      }, assetDecimals)

      const price = activePool.oracle_asset_price ? bigintToNumber(activePool.oracle_asset_price, oraclePriceDecimals) : 0

      const debt = Number(rawDept)
      const debtUsd = Number(debt) * Number(price)

      const [, asset_issuer] = destructurePoolAsset(activePool.pool.name)
      const borrowApy = activePool.apy.borrow_bps / 100

      const healthFactor = calcHealthFactor(obligation!, poolsData!, assetDecimals, oraclePriceDecimals)

      const data = {
        raw: activePool,
        market,
        asset: getFullTokenData(activePool.pool.token_symbol),
        debt,
        debtUsd,
        price,
        borrow_apy: `${truncatePercent(borrowApy || 0, 2)}%`,
        action: 'Repay',
        pool_address,
        asset_issuer,
        healthFactor,
      }

      res.push(data)
    }
  }
  return res?.filter(Boolean) as BorrowCardTableItem[]
})

const totalDebtRaw = computed(() => {
  let sum = 0
  for (const item of items.value) {
    sum += Number(item.debtUsd)
  }
  return sum
})

const totalDebt = computed(() => formatCompactUSD(totalDebtRaw.value, 2, 2))

function withdrawDialogHandler(item: BorrowCardTableItem) {
  marketsStore.selectedMarketName = String(item.market)
  marketsStore.selectedPoolAddress = item.pool_address
  marketsStore.dialogRepay = true
}
</script>

<template>
  <div class="portfolio-card">
    <div class="portfolio-card__title">
      My Borrows

      <metric-indicator
        v-if="totalDebtRaw > 0"
        label="Total Borrowed"
        :value="`${totalDebt}`"
        color="#f04438"
      />
    </div>

    <div v-if="markets.length === 0 && isLoading">
      <borrow-table-skeleton />
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
          class="portfolio-table market-table "
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

          <template #cell(debt)="data">
            <div class="table-cell table-cell__dept justify-content-end with-price">
              {{
                Number(data.item.debt) > 1000 ? shortenNumber(Number(data.item.debt)) : Number(data.item.debt).toFixed(5)
              }}
              <span>${{ formatPrice(data.item.debtUsd, 2, 2) }}</span>
            </div>
          </template>

          <template #cell(borrow_apy)="data">
            <div class="table-cell justify-content-center">
              <j-pill-label
                variant="indigo"
                size="sm"
              >
                {{ data.item.borrow_apy }}
              </j-pill-label>
            </div>
          </template>

          <template #cell(hf)="data">
            <div class="table-cell justify-content-end">
              <div
                class="hf-indicator"
                :style="{
                  '--indicator-width': `${Math.min(Math.max((data.item.healthFactor - 1) * 100, 0), 100)}%`,
                  '--indicator-color': healthFactorColor(data.item.healthFactor),
                }"
              />
              <span
                :style="{
                  color: healthFactorColor(data.item.healthFactor),
                }"
                class="text-num hf-percent"
              >
                {{ truncatePercent(data.item.healthFactor, 2) }}
              </span>
            </div>
          </template>

          <template #cell(action)="data">
            <div class="table-cell justify-content-end">
              <j-btn
                variant="brand-secondary-outlined"
                size="sm"
                class="repay-btn"
                :disabled="market.isDisabled(data.item.pool_address, 'repay', data.item.market!)"
                :loading="market.isLoading(data.item.pool_address, 'repay', data.item.market!)"
                @click="withdrawDialogHandler(data.item)"
              >
                {{ data.item.action }}
              </j-btn>
            </div>
          </template>
        </BTable>

        <portfolio-borrow-table-mobile
          v-else
          :items="items"
          @dialog-handler="(e: any) => withdrawDialogHandler(e.item)"
        />
      </template>

      <div
        v-else
        class="no-data"
      >
        <i-app-percentage-square-icon />
        No borrowed assets
      </div>
    </div>
  </div>

  <client-only>
    <repay-dialog v-model="marketsStore.dialogRepay" />
  </client-only>
</template>

<style lang="scss">
.portfolio-card {
  .table-cell__dept {
    color: $indigo;
  }

  .hf-indicator {
    position: relative;
    width: 50px;
    height: 4px;
    border-radius: $radius-lg;
    background-color: color-mix(in oklab, $border-primary 70%, transparent);
    overflow: hidden;
    flex-shrink: 0;
    margin-right: 4px;
    font-family: $font-JetBrainsMono;

    &::after {
      content: '';
      position: absolute;
      right: 0;
      top: 0;
      height: 100%;
      width: var(--indicator-width, 0%);
      border-radius: $radius-lg;
      background-color: var(--indicator-color, #{$success});
      transition:
        width $transition-base ease,
        background-color $transition-base ease;
    }
  }

  .hf-percent {
    font-size: 12px;
  }
}
</style>
