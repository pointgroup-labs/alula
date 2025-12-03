<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'
import { amountToUsdWithShort, bigintToNumber, formatPrice, getTokenIcon, getTokenName, shortenNumber, truncatePercent } from '~/utils'

const {
  onlyMultiplied = false,
} = defineProps<{
  onlyMultiplied?: boolean
}>()

const { width } = useWindowSize()

const marketsStore = useMarketsStore()

const selectedMarketAddress = toRef(marketsStore, 'selectedMarketAddress')

const dialogLeverage = toRef(marketsStore, 'dialogLeverage')
const dialogLeverageWithdraw = toRef(marketsStore, 'dialogLeverageWithdraw')

const market = useMarketActions()

const userStore = useUserStore()

const markets = computed(() => Object.keys(marketsStore.state.markets) ?? [])
const loading = computed(() => (marketsStore.state.loadingLeveragePools || marketsStore.state.loading) || userStore.loading)

const fields = [
  { key: 'asset', label: 'Vault', align: 'left' },
  { key: 'maxAPY', label: 'Max APY', align: 'center' },
  { key: 'multiplier', label: 'Multiplier', align: 'center' },
  { key: 'market', label: 'Market', align: 'center' },
  { key: 'liquidity', label: 'Liquidity', align: 'right' },
  { key: 'supplied', label: 'Supply', align: 'right' },
  { key: 'borrowing', label: 'Borrow', align: 'right' },
  { key: 'action', label: '' },
]

const items = computed<MultiplyTableItem[]>(() => {
  const res = []
  for (const market in marketsStore.state.markets) {
    const state = marketsStore.state.markets[market]?.marketState
    const poolsData = state?.pools_data ?? []
    const leveragePools = state?.multiply_pairs ?? []
    const oraclePriceDecimals = state?.oracle_price_decimals ?? 0
    const assetDecimals = state?.asset_decimals ?? 0
    for (const { borrow_pool, deposit_pool } of leveragePools) {
      const depositPoolData = poolsData.find(p => p.pool.pool_address === deposit_pool)!
      const borrowPoolData = poolsData.find(p => p.pool.pool_address === borrow_pool)!
      const depositTokenSymbol = getTokenSymbol(depositPoolData?.pool.token_symbol)
      const borrowTokenSymbol = getTokenSymbol(borrowPoolData?.pool.token_symbol)
      const depositTokenName = getTokenName(String(depositTokenSymbol))
      const depositTokenIcon = getTokenIcon(String(depositTokenSymbol)) || ''
      const borrowTokenName = getTokenName(String(borrowTokenSymbol))
      const borrowTokenIcon = getTokenIcon(String(borrowTokenSymbol)) || ''
      const ltv = Number(depositPoolData?.pool.config.health_config.open_ltv_bps) || 0
      const multiplier = calculateMaxMultiplierFromBps(ltv)
      const supplyBPS = Number(depositPoolData?.apy.supply_bps || 0) / 10_000
      const borrowBPS = Number(borrowPoolData?.apy.borrow_bps || 0) / 10_000
      const maxAPY = (supplyBPS * multiplier - borrowBPS * (multiplier - 1)) * 100
      const supplied = depositPoolData && depositPoolData.pool.total_available ? Number(bigintToNumber(depositPoolData.pool.total_available, assetDecimals)) : 0
      const liquidity
        = borrowPoolData && borrowPoolData.pool.total_available
          ? Number(bigintToNumber(borrowPoolData.pool.total_available/*  + borrowPoolData.total_borrowed + borrowPoolData.total_collateral */, assetDecimals))
          : 0
      const depositPoolPrice = Number(bigintToNumber(depositPoolData.oracle_asset_price, oraclePriceDecimals)) || 0
      const borrowPPoolPrice = Number(bigintToNumber(borrowPoolData.oracle_asset_price, oraclePriceDecimals)) || 0

      const data = {
        market,
        depositPoolData,
        borrowPoolData,
        asset: { name: depositTokenName, symbol: depositTokenSymbol, icon: depositTokenIcon },
        borrowAsset: { name: borrowTokenName, symbol: borrowTokenSymbol, icon: borrowTokenIcon },
        liquidity,
        multiplier,
        maxAPY,
        price: depositPoolPrice,
        borrowPoolPrice: borrowPPoolPrice,
        pool_address: depositPoolData?.pool.pool_address || '',
        supplied,
        assetDecimals,
      }

      res.push(data)
    }
  }

  return res
})

const filteredData = computed(() => {
  const data = onlyMultiplied ? items.value?.filter(item => isUserHaveMultiply(item.pool_address, String(item.market))) : items.value
  return data.filter(Boolean)
})

const activeLeverageMarket = toRef(marketsStore, 'activeLeverageMarket')
const selectedPool = computed(() =>
  items.value.find(item => item.pool_address === selectedMarketAddress.value
    && activeLeverageMarket.value === item.market))

async function multiplyDialogHandler(item: MultiplyTableItem, action: 'supply' | 'withdraw') {
  selectedMarketAddress.value = item?.pool_address
  activeLeverageMarket.value = String(item.market)
  action === 'supply' ? dialogLeverage.value = true : dialogLeverageWithdraw.value = true
}

function isUserHaveMultiply(poolAddress: string, market: string) {
  return checkIsHaveMultiply(
    userStore.state.multiplyObligations,
    items.value ?? [],
    poolAddress,
    market,
  )
}
</script>

<template>
  <div v-if="markets.length === 0 && loading">
    <table-skeleton v-if="width > 650" />
    <table-skeleton-mobile v-else />
  </div>
  <div
    v-else
    class="table-wrapper"
  >
    <BTable
      v-if="width >= 1024"
      show-empty
      borderless
      :fields="fields"
      :items="filteredData"
      responsive
      class="market-table multiply-table"
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
            alt="asset icon"
          >
          <img
            :src="data.item.borrowAsset.icon"
            alt="XLM icon"
            class="xlm-icon"
          >
          <div class="market-table__asset__info">
            <div class="market-table__asset__info__name">
              {{ data.item.asset.symbol }}/{{ data.item.borrowAsset.symbol }}
            </div>
            <div class="market-table__asset__info__symbol">
              {{ data.item.asset.name }} / {{ data.item.borrowAsset.symbol }}
            </div>
          </div>
        </div>
      </template>

      <template #cell(maxAPY)="data">
        <div class="table-cell cell-apy">
          {{ truncatePercent(data.item.maxAPY || 0, 2) }}%
        </div>
      </template>

      <template #cell(multiplier)="data">
        <div class="table-cell justify-content-center">
          <j-pill-label
            color="#111"
            size="sm"
            variant="success"
          >
            {{ truncatePercent(data.item.multiplier || 0, 2) }}x
          </j-pill-label>
        </div>
      </template>

      <template #cell(market)="data">
        <j-tooltip tooltip-class="table-cell justify-content-center market-cell">
          <span>{{ data.item.market }}</span>
          <template #content>
            {{ data.item.market }}
          </template>
        </j-tooltip>
      </template>

      <template #cell(liquidity)="data">
        <div class="table-cell justify-content-end">
          <j-tooltip tooltip-class="with-price">
            <strong>{{ shortenNumber(data.item.liquidity || 0) }} {{ data.item.borrowAsset.symbol }}</strong>
            <span>${{ amountToUsdWithShort(data.item.liquidity, data.item.borrowPoolPrice) }}</span>
            <template #content>
              {{ formatPrice(data.item.liquidity) }} {{ data.item.borrowAsset.symbol }}
              <br>
              <span>${{ amountToUsdWithShort(data.item.liquidity, data.item.borrowPoolPrice, false) }}</span>
            </template>
          </j-tooltip>
        </div>
      </template>

      <template #cell(supplied)="data">
        <div class="table-cell justify-content-end">
          <j-tooltip tooltip-class="with-price">
            <strong>{{ shortenNumber(data.item.supplied.toFixed(2) || 0) }} {{ data.item.asset.symbol }}</strong>
            <span>${{ amountToUsdWithShort(data.item.supplied, data.item.price) }}</span>
            <template #content>
              {{ formatPrice(data.item.supplied) }} {{ data.item.asset.symbol }}
              <br>
              <span>${{ amountToUsdWithShort(data.item.supplied, data.item.price, false) }}</span>
            </template>
          </j-tooltip>
        </div>
      </template>

      <template #cell(borrowing)="data">
        <div class="table-cell justify-content-end">
          {{ data.item.borrowAsset.symbol }}
        </div>
      </template>

      <template #cell(action)="data">
        <div class="table-cell justify-content-end market-table__action">
          <j-btn
            size="md"
            pill
            icon-right
            :loading="market.isLoading(data.item.pool_address, 'leverage', data.item.market!)"
            :disabled="market.isDisabled(data.item.pool_address, 'leverage', data.item.market!)"
            @click="multiplyDialogHandler(data.item, 'supply')"
          >
            Multiply
          </j-btn>
          <j-btn
            v-if="isUserHaveMultiply(data.item.pool_address, String(data.item.market))"
            size="md"
            variant="accent"
            pill
            icon-right
            :disabled="market.isDisabled(data.item.pool_address, 'withdrawLeverage', data.item.market!)"
            :loading="market.isLoading(data.item.pool_address, 'withdrawLeverage', data.item.market!)"
            @click="multiplyDialogHandler(data.item, 'withdraw')"
          >
            Withdraw
          </j-btn>
        </div>
      </template>

      <template
        #empty
      >
        <div
          v-show="!loading"
          class="no-data"
        >
          No Pools
        </div>
      </template>
    </BTable>

    <multiply-table-mobile
      v-else
      :items="filteredData"
      @dialog-handler="(e: any) => multiplyDialogHandler(e.item, e.action)"
    />

    <j-loading-spinner v-if="loading">
      Loading...
    </j-loading-spinner>
  </div>

  <multiply-dialog
    v-model="dialogLeverage"
    :data="selectedPool"
  />

  <withdraw-leverage-dialog
    v-model="dialogLeverageWithdraw"
    :data="selectedPool"
  />
</template>

<style lang="scss">
.multiply-table {
  tbody tr {
    cursor: default;
  }

  .cell-apy {
    color: $success;
    font-size: 14px;
    font-style: normal;
    font-weight: 700;
  }
}
</style>
