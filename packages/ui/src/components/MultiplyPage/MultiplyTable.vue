<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'
import Decimal from 'decimal.js'
import { bigintToNumber, formatPrice, getTokenIcon, getTokenName, shortenNumber, truncatePercent } from '~/utils'

const client = useClientStore()
const marketsStore = useMarketsStore()

const market = useMarket()

const assetDecimals = computed(() => client.assetDecimals)

const pools = computed(() => marketsStore.selectedMarketPools)
const leveragePools = computed(() => marketsStore.state.leveragePools)
const loading = computed(() => marketsStore.state.loadingLeveragePools || marketsStore.state.loading)

/**
 * @param ltvByBps — LTV в basis points (0…10000)
 * @returns number ≥1, max multiplyer
 */
function calculateMaxMultiplierFromBps(ltvByBps: number): number {
  if (!Number.isInteger(ltvByBps) || ltvByBps < 0 || ltvByBps >= 10_000) {
    throw new Error(`ltvByBps must be integer in [0,10000), got ${ltvByBps}`)
  }
  const openLtv = new Decimal(ltvByBps).div(10_000)
  return openLtv.eq(1)
    ? Infinity
    : new Decimal(1).div(new Decimal(1).minus(openLtv)).toNumber()
}

const fields = [
  { key: 'asset', label: 'Vault', align: 'left' },
  { key: 'maxAPY', label: 'Max APY', align: 'center' },
  { key: 'multiplier', label: 'Multiplier', align: 'center' },
  { key: 'liquidity', label: 'Liquidity', align: 'right' },
  { key: 'supplied', label: 'Supplied', align: 'right' },
  { key: 'borrowing', label: 'Borrowing', align: 'right' },
  { key: 'action', label: '' },
]

const items = computed<MultiplyTableItem[]>(() => {
  return leveragePools.value
    ?.map(({ borrow_pool, deposit_pool }) => {
      const depositPool = pools.value.find(pool => pool.pool_address === deposit_pool)!
      const borrowPool = pools.value.find(pool => pool.pool_address === borrow_pool)!
      const depositTokenSymbol = depositPool?.token_ticker
      const borrowTokenSymbol = borrowPool?.token_ticker
      const depositTokenName = getTokenName(String(depositTokenSymbol))
      const depositTokenIcon = getTokenIcon(String(depositTokenSymbol))
      const borrowTokenName = getTokenName(String(borrowTokenSymbol))
      const borrowTokenIcon = getTokenIcon(String(borrowTokenSymbol))
      const liquidity = depositPool && depositPool.available ? Number(bigintToNumber(depositPool.available, assetDecimals.value)) : 0
      const ltv = Number(depositPool?.config.open_ltv_bps) || 0
      const multiplier = calculateMaxMultiplierFromBps(ltv)
      const maxAPY
       = ((Number(depositPool?.pool_apy.supply_bps || 0) - Number(depositPool?.pool_apy.borrow_bps || 0))
         * multiplier + Number(depositPool?.pool_apy.borrow_bps || 0)) / 100
      const supplied
      = borrowPool && borrowPool.available ? Number(bigintToNumber(borrowPool.available + borrowPool.total_borrowed, assetDecimals.value)) : 0
      return {
        depositPool,
        borrowPool,
        asset: { name: depositTokenName, symbol: depositTokenSymbol, icon: depositTokenIcon },
        borrowAsset: { name: borrowTokenName, symbol: borrowTokenSymbol, icon: borrowTokenIcon },
        liquidity,
        multiplier,
        maxAPY,
        price: Number(depositPool?.pool_price) || 0,
        borrowPoolPrice: Number(borrowPool?.pool_price) || 0,
        pool_address: depositPool?.pool_address || '',
        supplied,
      }
    })
})

const dialogSupply = ref(false)
const dialogBorrow = ref(false)
const selectedPoolAddress = ref()
const selectedPool = computed(() => items.value.find(item => item.pool_address === selectedPoolAddress.value))

async function multiplyDialogHandler(data: { item: MultiplyTableItem }, action: 'supply' | 'borrow') {
  selectedPoolAddress.value = data.item?.pool_address
  action === 'supply' ? dialogSupply.value = true : dialogBorrow.value = true
}

function amountToUsd(amount: number, price: number) {
  const usd = (Number(amount) * Number(price)) || 0
  return shortenNumber(usd)
}
</script>

<template>
  <template v-if="pools.length === 0 && loading">
    <j-skeleton
      height="36"
      full-width
      style="border-radius: 8px;"
    />
    <j-skeleton
      height="80"
      full-width
      style="margin-top: 8px; border-radius: 8px;"
    />
  </template>
  <div
    v-else
    class="table-wrapper"
  >
    <BTable
      show-empty
      borderless
      :fields="fields"
      :items="items"
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
            :src="getTokenIcon('XLM')"
            alt="XLM icon"
            class="xlm-icon"
          >
          <div class="market-table__asset__info">
            <div class="market-table__asset__info__name">
              {{ data.item.asset.symbol }}/XLM
            </div>
            <div class="market-table__asset__info__symbol">
              {{ data.item.asset.name }} / Stellar
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
            bg-color="rgba(8, 181, 118, 0.50)"
            size="md"
          >
            {{ truncatePercent(data.item.multiplier || 0, 2) }}x
          </j-pill-label>
        </div>
      </template>

      <template #cell(liquidity)="data">
        <j-tooltip tooltip-class="table-cell justify-content-end with-price">
          {{ shortenNumber(data.item.liquidity || 0) }} {{ data.item.asset.symbol }}
          <span>${{ amountToUsd(data.item.liquidity, data.item.price) }}</span>
          <template #content>
            {{ formatPrice(data.item.liquidity) }}
          </template>
        </j-tooltip>
      </template>

      <template #cell(supplied)="data">
        <j-tooltip tooltip-class="table-cell justify-content-end with-price">
          {{ shortenNumber(data.item.supplied.toFixed(2) || 0) }} XLM
          <span>${{ amountToUsd(data.item.supplied, data.item.borrowPoolPrice) }}</span>
          <template #content>
            {{ formatPrice(data.item.supplied) }}
          </template>
        </j-tooltip>
      </template>

      <template #cell(borrowing)="data">
        <div class="table-cell justify-content-end">
          {{ data.item.borrowAsset.symbol }}
        </div>
      </template>

      <template #cell(action)="data">
        <div class="table-cell justify-content-end market-table__action">
          <j-btn
            size="lg"
            pill
            icon-right
            :disabled="market.isDisabled(data.item.pool_address, 'deposit')"
            :loading="market.isLoading(data.item.pool_address, 'deposit')"
            @click="multiplyDialogHandler(data, 'supply')"
          >
            Multiply
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
          No Markets
        </div>
      </template>
    </BTable>

    <j-loading-spinner v-if="loading">
      Loading...
    </j-loading-spinner>
  </div>

  <multiply-dialog
    v-model="dialogSupply"
    :data="selectedPool"
  />
</template>

<style lang="scss">
.multiply-table {
  .cell-apy {
    color: $success;
    font-size: 16px;
    font-style: normal;
    font-weight: 700;
    line-height: 20px;
  }
}
</style>
