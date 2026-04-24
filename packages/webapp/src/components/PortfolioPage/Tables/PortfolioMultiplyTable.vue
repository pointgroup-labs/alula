<script lang="ts" setup>
import type { MultiplyPortfolioTableItem } from '~/types/table'
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
const { getFullTokenData } = useTokensStore()
const multiplyStore = useMultiplyStore()
const vaults = computed(() => multiplyStore.vaults)

const market = useMarketActions()

const dialogLeverageWithdraw = toRef(marketsStore, 'dialogLeverageWithdraw')
const selectedVault = ref<MultiplyPortfolioTableItem>()

const markets = computed(() => Object.keys(marketsStore.state.markets) ?? [])
const isLoading = computed(() => (marketsStore.state.loadingLeveragePools || marketsStore.state.loading) || userStore.loading)

const fields = [
  { key: 'asset', label: 'Vault', align: 'left' },
  { key: 'multiplier', label: 'Multiplier', align: 'center' },
  { key: 'market', label: 'Market', align: 'center' },
  { key: 'deposited', label: 'Deposited', align: 'right' },
  { key: 'borrowed', label: 'Borrowed', align: 'right' },
  { key: 'hf', label: 'Health Factor', align: 'right' },
  { key: 'action', label: '' },
]

function calculatePositionHealthFactor(params: {
  deposited: number
  depositPrice: number
  closeLtvBps: number
  borrowed: number
  borrowPrice: number
  liabilityFactorBps: number
}) {
  const weightedDepositUsd = params.deposited * params.depositPrice * bpsToNumber(params.closeLtvBps)
  const weightedBorrowUsd = params.borrowed * params.borrowPrice * bpsToNumber(params.liabilityFactorBps)

  if (weightedBorrowUsd <= 0) {
    return 10
  }

  return Math.min(weightedDepositUsd / weightedBorrowUsd, 10)
}

const tableItems = computed<MultiplyPortfolioTableItem[]>(() => {
  const res = []
  for (const market in marketsStore.state.markets) {
    const state = marketsStore.state.markets[market]?.marketState
    const oraclePriceDecimals = state?.oracle_price_decimals ?? 0
    const marketVaults = vaults.value.filter(vault => vault.market === market)
    for (const vault of marketVaults) {
      const multiplyObligation = userStore.state.multiplyObligations[market]?.[vault.pairKey]
      const depositPoolAddress = vault.depositPoolData.pool.pool_address
      const borrowPoolAddress = vault.borrowPoolData.pool.pool_address
      const depositObligation = multiplyObligation?.deposits?.find(([address]) => address === depositPoolAddress)
      const borrowObligation = multiplyObligation?.borrows?.find(([address]) => address === borrowPoolAddress)
      const depositPoolData = vault.depositPoolData
      const borrowPoolData = vault.borrowPoolData
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
        }, borrowPoolData.pool.token_decimals) || 0
      const depositPoolPrice = Number(bigintToNumber(depositPoolData.oracle_asset_price, oraclePriceDecimals)) || 0
      const borrowPPoolPrice = Number(bigintToNumber(borrowPoolData.oracle_asset_price, oraclePriceDecimals)) || 0
      const healthFactor = calculatePositionHealthFactor({
        deposited,
        depositPrice: depositPoolPrice,
        closeLtvBps: Number(depositPoolData.pool.config.health_config.close_ltv_bps || 0),
        borrowed,
        borrowPrice: borrowPPoolPrice,
        liabilityFactorBps: Number(borrowPoolData.pool.config.health_config.liability_factor_bps || 0),
      })

      const currentMultiplier = calculateCurrentMultiplier(deposited, depositPoolPrice, borrowed, borrowPPoolPrice) || 0

      const data = {
        pairKey: vault.pairKey,
        market,
        depositPoolData,
        borrowPoolData,
        asset: getFullTokenData(depositPoolData?.pool.token_symbol),
        borrowAsset: getFullTokenData(borrowPoolData?.pool.token_symbol),
        deposited,
        borrowed,
        healthFactor,
        multiplier: currentMultiplier,
        maxAPY,
        price: depositPoolPrice,
        borrowPoolPrice: borrowPPoolPrice,
        pool_address: depositPoolData?.pool.pool_address || '',
        assetDecimals: depositPoolData.pool.token_decimals,
      }

      res.push(data)
    }
  }

  return res
})

const filteredData = computed(() => {
  const data = onlyMultiplied ? tableItems.value?.filter(item => isUserHaveMultiply(item.pool_address, String(item.market))) : tableItems.value
  return data.filter(Boolean)
})

async function multiplyDialogHandler(item: MultiplyPortfolioTableItem) {
  selectedVault.value = item
  dialogLeverageWithdraw.value = true
}

function isUserHaveMultiply(poolAddress: string, market: string) {
  return checkIsHaveMultiply(
    userStore.state.multiplyObligations,
    tableItems.value as any,
    poolAddress,
    market,
  )
}
</script>

<template>
  <div v-if="markets.length === 0 && isLoading">
    <portfolio-multiply-table-skeleton v-if="width > 1024" />
    <portfolio-multiply-table-skeleton-mobile v-else />
  </div>
  <div
    v-else
    class="table-wrapper"
  >
    <template v-if="filteredData.length > 0">
      <BTable
        v-if="width >= 1024"
        show-empty
        borderless
        :fields="fields"
        :items="filteredData"
        responsive
        class="market-table multiply-table portfolio-table"
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
          <div class="table-cell justify-content-end market-table__action">
            <j-btn
              v-if="isUserHaveMultiply(data.item.pool_address, String(data.item.market))"
              size="sm"
              variant="accent-outlined"
              :disabled="market.isDisabled(data.item.pool_address, 'withdrawLeverage', data.item.market!)"
              :loading="market.isLoading(data.item.pool_address, 'withdrawLeverage', data.item.market!)"
              @click="multiplyDialogHandler(data.item)"
            >
              Close
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

      <portfolio-multiply-table-mobile
        v-else
        :items="filteredData"
        show-in-accounts
        @dialog-handler="(e: any) => multiplyDialogHandler(e.item)"
      />
    </template>
    <div
      v-else
      class="no-data"
    >
      No multiplied assets
    </div>
  </div>

  <withdraw-multiply-dialog
    v-model="dialogLeverageWithdraw"
    :data="selectedVault"
  />
</template>

<style lang="scss">
.portfolio-multiply__cards {
  .multiply-table {
    thead {
      th:first-child {
        padding-left: 16px;
      }
      th:last-child {
        padding-right: 20px;
      }
    }

    tbody {
      tr {
        td:first-child {
          padding-left: 16px;
        }
        td:last-child {
          padding-right: 20px;
        }
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
          width 0.3s ease,
          background-color 0.3s ease;
      }
    }

    .hf-percent {
      font-size: 12px;
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
    padding: 32px;
  }
}
</style>
