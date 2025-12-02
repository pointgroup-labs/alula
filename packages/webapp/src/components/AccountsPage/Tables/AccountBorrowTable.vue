<script lang="ts" setup>
import type { BorrowCardTableItem } from '~/types/table'
// import type { BorrowObligation } from '@jlend/sdk'
import { calculateBorrow } from '@alula/client-sdk/src/utils'
import {
  destructurePoolAsset,
  formatPrice,
  getTokenIcon,
  getTokenName,
  shortenNumber,
  truncatePercent,
} from '~/utils'

const { width } = useWindowSize()

const userStore = useUserStore()

const marketsStore = useMarketsStore()

const market = useMarketActions()

const loadingMarkets = computed(() => marketsStore.state.loadingLeveragePools || marketsStore.state.loading)

const isHasObligations = computed(() => Object.keys(userStore.state.obligations).length > 0)

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'debt', label: 'Debt', align: 'right' },
  { key: 'market', label: 'Market', align: 'center' },
  { key: 'borrow_apy', label: 'Borrow APY', align: 'center' },
  { key: 'action', label: '' },
]

const items: ComputedRef<BorrowCardTableItem[]> = computed(() => {
  const res = []
  for (const market in userStore.state.obligations) {
    const deposits = userStore.state.obligations[market]?.borrows ?? []
    const marketState = marketsStore.state.markets[market]?.marketState
    const poolsData = marketState?.pools_data
    const assetDecimals = marketState?.asset_decimals ?? 7
    const oraclePriceDecimals = marketState?.oracle_price_decimals ?? 0
    for (const deposit of deposits) {
      const [pool_address, borrow] = deposit
      const activePool = poolsData?.find(data => data.pool.pool_address === pool_address)
      if (!activePool) {
        continue
      }

      const tokenSymbol = getTokenSymbol(activePool.pool.token_symbol)
      const tokenName = getTokenName(tokenSymbol)
      const icon = getTokenIcon(tokenSymbol)
      const rawDept = calculateBorrow(borrow.d_tokens, {
        total_borrowed: activePool.pool.total_borrowed,
        total_d_tokens: activePool.pool.total_d_tokens,
      }, assetDecimals)

      const price = activePool.oracle_asset_price ? bigintToNumber(activePool.oracle_asset_price, oraclePriceDecimals) : 0

      const debt = Number(rawDept)
      const debtUsd = formatPrice(Number(debt) * Number(price), 2, 2)

      const [, asset_issuer] = destructurePoolAsset(activePool.pool.name)
      const borrowApy = activePool.apy.borrow_bps / 100

      const data = {
        raw: activePool,
        market,
        asset: { name: tokenName, symbol: tokenSymbol, icon },
        debt,
        debtUsd,
        price,
        borrow_apy: `${truncatePercent(borrowApy || 0, 2)}%`,
        action: 'Repay',
        pool_address,
        asset_issuer,
      }

      res.push(data)
    }
  }
  return res?.filter(Boolean) as BorrowCardTableItem[]
})

const dialog = ref(false)
const selectedMarket = ref({ market: '', pool_address: '' })
const selectedPool = computed(() =>
  items.value?.find(item => item.pool_address === selectedMarket.value.pool_address
    && item.market === selectedMarket.value.market))

function withdrawDialogHandler(item: BorrowCardTableItem) {
  selectedMarket.value = { market: String(item.market), pool_address: item?.pool_address }
  dialog.value = true
}

watch(selectedMarket, (p) => {
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

    <div v-if="!isHasObligations && (userStore.loading || loadingMarkets)">
      <table-skeleton v-if="width > 650" />
      <table-skeleton-mobile v-else />
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
              {{
                Number(data.item.debt) > 1000 ? shortenNumber(Number(data.item.debt)) : Number(data.item.debt).toFixed(5)
              }}
              <span>${{ data.item.debtUsd }}</span>
              <template #content>
                {{ formatPrice(data.item.debt) }}
              </template>
            </j-tooltip>
          </template>

          <template #cell(market)="data">
            <j-tooltip tooltip-class="table-cell justify-content-center market-cell">
              <span>{{ data.item.market }}</span>
              <template #content>
                {{ data.item.market }}
              </template>
            </j-tooltip>
          </template>

          <template #cell(borrow_apy)="data">
            <div class="table-cell justify-content-center">
              <j-pill-label
                color="#111"
                variant="warning"
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
                :disabled="market.isDisabled(data.item.pool_address, 'repay', data.item.market!)"
                :loading="market.isLoading(data.item.pool_address, 'repay', data.item.market!)"
                @click="withdrawDialogHandler(data.item)"
              >
                {{ data.item.action }}
              </j-btn>
            </div>
          </template>
        </BTable>

        <account-borrow-table-mobile
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
        no borrowed assets
      </div>

      <j-loading-spinner v-if="userStore.loading">
        Loading...
      </j-loading-spinner>
    </div>
  </div>

  <client-only>
    <repay-dialog
      v-model="dialog"
      :data="selectedPool"
    />
  </client-only>
</template>

<style lang="scss">
.account-card {
  .table-cell__dept {
    color: $warning;
  }
}
</style>
