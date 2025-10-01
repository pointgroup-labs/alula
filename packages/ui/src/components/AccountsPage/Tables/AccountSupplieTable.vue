<script lang="ts" setup>
import type { SuppliedCardTableItem } from '~/types/table'
import { calculateTotalStake } from '@alula/client-sdk/src/utils'
// import type { DepositObligation } from '@jlend/sdk'
import { formatPrice, shortenNumber } from '~/utils'

const { width } = useWindowSize()

const userStore = useUserStore()

const marketsStore = useMarketsStore()
const decimals = computed(() => marketsStore.assetDecimals)

const market = useMarketActions()

const loadingMarkets = computed(() => marketsStore.state.loadingLeveragePools || marketsStore.state.loading)

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'balance', label: 'Balance', align: 'right' },
  { key: 'supply_apy', label: 'Supply APY', align: 'center' },
  { key: 'action', label: '' },
]

const items: ComputedRef<SuppliedCardTableItem[] | []> = computed(() => {
  const res = []
  for (const market in userStore.state.obligations) {
    const deposits = userStore.state.obligations[market]?.deposits ?? []
    const pools = marketsStore.state.markets[market]?.pools
    for (const deposit of deposits) {
      const [pool_address, dep] = deposit
      const pool = pools?.find(p => p.pool_address === pool_address)
      if (!pool) {
        continue
      }

      const tokenSymbol = pool.token_ticker
      const tokenName = getTokenName(tokenSymbol)
      const icon = getTokenIcon(tokenSymbol)
      const available = Number(bigintToNumber(pool.total_available, decimals.value))

      const deposited = calculateTotalStake(dep.j_tokens, {
        total_j_tokens: pool.total_j_tokens,
        total_borrowed: pool.total_borrowed,
        total_available: pool.total_available,
      })
      const userCollateral = bigintToNumber(dep.collateral, decimals.value)
      const balance = Number(deposited) + Number(userCollateral)

      const poolApy = pool.pool_apy.supply_bps / 100

      const data = {
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
        market,
      }

      res.push(data)
    }
  }
  return res.filter(Boolean) as SuppliedCardTableItem[]
})

const dialog = ref(false)
const selectedMarket = ref({ market: '', pool_address: '' })
const selectedPool = computed(() =>
  items.value?.find(item => item.pool_address === selectedMarket.value.pool_address
    && item.market === selectedMarket.value.market))

function withdrawDialogHandler(item: SuppliedCardTableItem) {
  selectedMarket.value = { market: String(item.market), pool_address: item?.pool_address }
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

    <div v-if="items.length === 0 && (userStore.loading || loadingMarkets)">
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
