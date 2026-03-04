<script lang="ts" setup>
import type { MultiplyAccountTableItem } from '~/types/table'
import { bpsToNumber } from '@alula/client-sdk'
import { calculateBorrow, calculateTotalStake } from '@alula/client-sdk/src/utils'
import { amountToUsdWithShort, calculateCurrentMultiplier, formatPrice, shortenNumber, truncatePercent } from '~/utils'

const {
  onlyMultiplied = false,
} = defineProps<{
  onlyMultiplied?: boolean
}>()

const { width } = useWindowSize()

const marketsStore = useMarketsStore()
const userStore = useUserStore()

const market = useMarketActions()

const selectedPoolAddress = toRef(marketsStore, 'selectedPoolAddress')
const dialogLeverageWithdraw = toRef(marketsStore, 'dialogLeverageWithdraw')

const markets = computed(() => Object.keys(marketsStore.state.markets) ?? [])
const isLoading = computed(() => (marketsStore.state.loadingLeveragePools || marketsStore.state.loading) || userStore.loading)

const fields = [
  { key: 'asset', label: 'Vault', align: 'left' },
  { key: 'maxAPY', label: 'APY', align: 'center' },
  { key: 'multiplier', label: 'Multiplier', align: 'center' },
  { key: 'market', label: 'Market', align: 'center' },
  { key: 'deposited', label: 'Deposited', align: 'right' },
  { key: 'borrowed', label: 'Borrowed', align: 'right' },
  { key: 'action', label: '' },
]

const tableItems = computed<MultiplyAccountTableItem[]>(() => {
  const res = []
  for (const market in marketsStore.state.markets) {
    const state = marketsStore.state.markets[market]?.marketState
    const poolsData = state?.pools_data ?? []
    const leveragePools = state?.multiply_pairs ?? []
    const oraclePriceDecimals = state?.oracle_price_decimals ?? 0
    const assetDecimals = state?.asset_decimals ?? 7
    const multiplyObligations = userStore.state.multiplyObligations[market]
    for (const { borrow_pool, deposit_pool } of leveragePools) {
      const depositObligation = multiplyObligations?.deposits?.find(([address]) => address === deposit_pool)
      const borrowObligation = multiplyObligations?.borrows?.find(([address]) => address === borrow_pool)
      const depositPoolData = poolsData.find(p => p.pool.pool_address === deposit_pool)
      const borrowPoolData = poolsData.find(p => p.pool.pool_address === borrow_pool)
      if (!depositObligation || !borrowObligation || !depositPoolData || !borrowPoolData) {
        continue
      }

      const [, depOblData] = depositObligation
      const [, borrowOblData] = borrowObligation

      const supplyBPS = bpsToNumber(Number(depositPoolData?.apy.supply_bps || 0))
      const borrowBPS = bpsToNumber(Number(borrowPoolData?.apy.borrow_bps || 0))
      const ltv = Number(depositPoolData?.pool.config.health_config.open_ltv_bps) || 0
      const multiplier = calculateMaxMultiplierFromBps(ltv)
      const maxAPY = (supplyBPS * multiplier - borrowBPS * (multiplier - 1)) * 100
      const deposited = +calculateTotalStake(depOblData.j_tokens, {
        total_j_tokens: depositPoolData.pool.total_j_tokens,
        total_borrowed: depositPoolData.pool.total_borrowed,
        total_available: depositPoolData.total_available_adjusted,
      }) || 0
      const borrowed
        = +calculateBorrow(borrowOblData.d_tokens, {
          total_borrowed: borrowPoolData.pool.total_borrowed,
          total_d_tokens: borrowPoolData.pool.total_d_tokens,
        }, assetDecimals) || 0
      const depositPoolPrice = Number(bigintToNumber(depositPoolData.oracle_asset_price, oraclePriceDecimals)) || 0
      const borrowPPoolPrice = Number(bigintToNumber(borrowPoolData.oracle_asset_price, oraclePriceDecimals)) || 0

      const currentMultiplier = calculateCurrentMultiplier(deposited, depositPoolPrice, borrowed, borrowPPoolPrice) || 0

      const data = {
        market,
        depositPoolData,
        borrowPoolData,
        asset: getFullTokenData(depositPoolData?.pool.token_symbol),
        borrowAsset: getFullTokenData(borrowPoolData?.pool.token_symbol),
        deposited,
        borrowed,
        multiplier: currentMultiplier,
        maxAPY,
        price: depositPoolPrice,
        borrowPoolPrice: borrowPPoolPrice,
        pool_address: depositPoolData?.pool.pool_address || '',
        assetDecimals,
      }

      res.push(data)
    }
  }

  return res
})

const activeLeverageMarket = toRef(marketsStore, 'activeLeverageMarket')
const selectedPool = computed(() =>
  tableItems.value.find(item => item.pool_address === selectedPoolAddress.value
    && activeLeverageMarket.value === item.market))

const filteredData = computed(() => {
  const data = onlyMultiplied ? tableItems.value?.filter(item => isUserHaveMultiply(item.pool_address, String(item.market))) : tableItems.value
  return data.filter(Boolean)
})

async function multiplyDialogHandler(item: MultiplyAccountTableItem) {
  selectedPoolAddress.value = item?.pool_address
  activeLeverageMarket.value = String(item.market)
  dialogLeverageWithdraw.value = true
}

function isUserHaveMultiply(poolAddress: string, market: string) {
  return checkIsHaveMultiply(
    userStore.state.multiplyObligations,
    tableItems.value ?? [],
    poolAddress,
    market,
  )
}
</script>

<template>
  <div v-if="markets.length === 0 && isLoading">
    <market-table-skeleton v-if="width > 650" />
    <market-table-skeleton-mobile v-else />
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
      class="market-table multiply-table multiply-table-accounts"
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
        <div
          class="table-cell cell-apy"
          :class="[`cell-apy--${data.item.maxAPY < 0 ? 'negative' : 'positive'}`]"
        >
          {{ truncatePercent(data.item.maxAPY || 0, 2) }}%
        </div>
      </template>

      <template #cell(multiplier)="data">
        <div class="table-cell justify-content-center">
          <j-pill-label
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

      <template #cell(deposited)="data">
        <div class="table-cell justify-content-end">
          <j-tooltip tooltip-class="with-price">
            <strong>{{ shortenNumber(data.item.deposited.toFixed(2) || 0) }} {{ data.item.asset.symbol }}</strong>
            <span>${{ amountToUsdWithShort(data.item.deposited, data.item.price) }}</span>
            <template #content>
              {{ formatPrice(data.item.deposited) }} {{ data.item.asset.symbol }}
              <br>
              <span>${{ amountToUsdWithShort(data.item.deposited, data.item.price, false) }}</span>
            </template>
          </j-tooltip>
        </div>
      </template>

      <template #cell(borrowed)="data">
        <div class="table-cell justify-content-end">
          <j-tooltip tooltip-class="with-price">
            <strong>{{ shortenNumber(data.item.borrowed || 0) }} {{ data.item.borrowAsset.symbol }}</strong>
            <span>${{ amountToUsdWithShort(data.item.borrowed, data.item.borrowPoolPrice) }}</span>
            <template #content>
              {{ formatPrice(data.item.borrowed) }} {{ data.item.borrowAsset.symbol }}
              <br>
              <span>${{ amountToUsdWithShort(data.item.borrowed, data.item.borrowPoolPrice, false) }}</span>
            </template>
          </j-tooltip>
        </div>
      </template>

      <template #cell(action)="data">
        <div class="table-cell justify-content-end market-table__action">
          <j-btn
            v-if="isUserHaveMultiply(data.item.pool_address, String(data.item.market))"
            size="xs"
            variant="accent"
            :disabled="market.isDisabled(data.item.pool_address, 'withdrawLeverage', data.item.market!)"
            :loading="market.isLoading(data.item.pool_address, 'withdrawLeverage', data.item.market!)"
            @click="multiplyDialogHandler(data.item)"
          >
            Withdraw
          </j-btn>
        </div>
      </template>

      <template #empty>
        <div
          v-show="!isLoading"
          class="no-data"
        >
          No Multiply vaults
        </div>
      </template>
    </BTable>

    <multiply-table-mobile
      v-else
      :items="filteredData"
      show-in-accounts
      @dialog-handler="(e: any) => multiplyDialogHandler(e.item)"
    />

    <j-loading-spinner
      v-if="isLoading"
      class="table-loading-spinner"
    >
      Loading...
    </j-loading-spinner>
  </div>

  <withdraw-leverage-dialog
    v-model="dialogLeverageWithdraw"
    :data="selectedPool"
  />
</template>

<style lang="scss">
.multiply-table {
  &.multiply-table-accounts {
    tbody tr {
      cursor: default;
    }
  }
  .cell-apy {
    color: $success;
    font-size: 14px;
    font-style: normal;
    font-weight: 700;

    &--negative {
      color: $danger;
    }
  }

  .no-data {
    color: $text-secondary;
    font-size: 12px;
    font-style: normal;
    font-weight: 400;
    line-height: 16px;
    text-align: center;
  }
}
</style>
