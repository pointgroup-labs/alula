<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'
import { amountToUsdWithShort, formatPrice, shortenNumber, truncatePercent } from '~/utils'

const {
  onlyMultiplied = false,
} = defineProps<{
  onlyMultiplied?: boolean
}>()

const { width } = useWindowSize()

const {
  tableItems,
  selectedMarketAddress,
  dialogLeverage,
  dialogLeverageWithdraw,
  markets,
  isLoading,
  selectedPool,
  activeLeverageMarket,
} = useMultiplyTable()

const market = useMarketActions()

const userStore = useUserStore()

const fields = [
  { key: 'asset', label: 'Vault', align: 'left' },
  { key: 'maxAPY', label: 'APY', align: 'center' },
  { key: 'multiplier', label: 'Multiplier', align: 'center' },
  { key: 'market', label: 'Market', align: 'center' },
  { key: 'liquidity', label: 'Borrow', align: 'right' },
  { key: 'supplied', label: 'Supply', align: 'right' },
  { key: 'borrowing', label: 'Borrow Token', align: 'right' },
  { key: 'action', label: '' },
]

const filteredData = computed(() => {
  const data = onlyMultiplied ? tableItems.value?.filter(item => isUserHaveMultiply(item.pool_address, String(item.market))) : tableItems.value
  return data.filter(Boolean)
})

async function multiplyDialogHandler(item: MultiplyTableItem, action: 'supply' | 'withdraw') {
  selectedMarketAddress.value = item?.pool_address
  activeLeverageMarket.value = String(item.market)
  action === 'supply' ? dialogLeverage.value = true : dialogLeverageWithdraw.value = true
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
          v-show="!isLoading"
          class="no-data"
        >
          No pools/positions opened
        </div>
      </template>
    </BTable>

    <multiply-table-mobile
      v-else
      :items="filteredData"
      @dialog-handler="(e: any) => multiplyDialogHandler(e.item, e.action)"
    />

    <j-loading-spinner
      v-if="isLoading"
      class="table-loading-spinner"
    >
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

    &--negative {
      color: $danger;
    }
  }
}
</style>
