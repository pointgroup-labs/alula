<script lang="ts" setup>
import type { MultiplyVaultItem } from '~/types/table'
import { amountToUsdWithShort, formatCompactUSD, formatPrice, shortenNumber, truncatePercent } from '~/utils'

const { width } = useWindowSize()
const marketsStore = useMarketsStore()
const marketActions = useMarketActions()
const userStore = useUserStore()

const multiplyStore = useMultiplyStore()
const vaults = computed(() => multiplyStore.vaults)

const dialogLeverage = toRef(marketsStore, 'dialogLeverage')
const withdrawDialogOpen = toRef(marketsStore, 'dialogLeverageWithdraw')
const selectedVault = ref<MultiplyVaultItem>()
const selectedWithdrawVault = ref<MultiplyVaultItem>()

const fields = [
  { key: 'asset', label: 'Vault', align: 'left' },
  { key: 'market', label: 'Market', align: 'center' },
  { key: 'price', label: 'Collateral Price', align: 'right' },
  { key: 'liquidity', label: 'Borrow Liquidity', align: 'right' },
  { key: 'supplied', label: 'Collateral TVL', align: 'right' },
  { key: 'apyAtMaxMultiplier', label: 'APY at Max Multiplier', align: 'center' },
  { key: 'maxMultiplier', label: 'Max Multiplier', align: 'center' },
  { key: 'action', label: '', align: 'right' },
]

function openDialog(vault: MultiplyVaultItem) {
  selectedVault.value = vault
  dialogLeverage.value = true
}

function openWithdrawDialog(vault: MultiplyVaultItem) {
  selectedWithdrawVault.value = vault
  withdrawDialogOpen.value = true
}

function onRowClicked(vault: MultiplyVaultItem) {
  multiplyStore.openVault(vault)
}

function isUserHaveMultiply(vault: MultiplyVaultItem) {
  const obligation = userStore.state.multiplyObligations[vault.market]?.[vault.pairKey]
  const deposits: any[] = obligation?.deposits ?? []
  const borrows: any[] = obligation?.borrows ?? []

  if (deposits.length === 0 || borrows.length === 0) {
    return false
  }

  const hasDeposit = deposits.some(deposit => deposit.includes(vault.depositPoolData.pool.pool_address))
  const hasBorrow = borrows.some(borrow => borrow.includes(vault.borrowPoolData.pool.pool_address))

  return hasDeposit && hasBorrow
}
</script>

<template>
  <div class="multiply-table">
    <div
      v-if="vaults.length === 0"
      class="multiply-table__empty"
    >
      No multiply vaults available.
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
        :items="vaults"
        responsive
        class="market-table multiply-table__desktop"
        @row-clicked="onRowClicked"
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
              :alt="data.item.asset.symbol"
            >
            <img
              :src="data.item.borrowAsset.icon"
              :alt="data.item.borrowAsset.symbol"
              class="xlm-icon"
            >
            <div class="market-table__asset__info">
              <div class="market-table__asset__info__name">
                {{ data.item.asset.symbol }}/{{ data.item.borrowAsset.symbol }}
              </div>
              <div class="market-table__asset__info__symbol">
                Borrow {{ data.item.borrowAsset.symbol }}, swap into {{ data.item.asset.symbol }}
              </div>
            </div>
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

        <template #cell(price)="data">
          <div class="table-cell justify-content-end">
            <div class="with-price">
              <strong>{{ formatCompactUSD(data.item.price, 2, 4) }}</strong>
              <span>{{ data.item.asset.symbol }}</span>
            </div>
          </div>
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
              <strong>{{ shortenNumber(data.item.supplied || 0) }} {{ data.item.asset.symbol }}</strong>
              <span>${{ amountToUsdWithShort(data.item.supplied, data.item.price) }}</span>
              <template #content>
                {{ formatPrice(data.item.supplied) }} {{ data.item.asset.symbol }}
                <br>
                <span>${{ amountToUsdWithShort(data.item.supplied, data.item.price, false) }}</span>
              </template>
            </j-tooltip>
          </div>
        </template>

        <template #cell(apyAtMaxMultiplier)="data">
          <div
            class="table-cell justify-content-center multiply-table__apy"
            :class="[`multiply-table__apy--${data.item.apyAtMaxMultiplier < 0 ? 'negative' : 'positive'}`]"
          >
            {{ truncatePercent(data.item.apyAtMaxMultiplier || 0, 2) }}%
          </div>
        </template>

        <template #cell(maxMultiplier)="data">
          <div class="table-cell justify-content-center">
            <j-pill-label
              size="sm"
              variant="success"
            >
              {{ truncatePercent(data.item.maxMultiplier || 0, 2) }}x
            </j-pill-label>
          </div>
        </template>

        <template #cell(action)="data">
          <div class="table-cell justify-content-end market-table__action">
            <j-btn
              size="sm"
              variant="brand-outlined"
              :disabled="marketActions.isDisabled(data.item.pool_address, 'multiplyOpen', data.item.market)"
              :loading="marketActions.isLoading(data.item.pool_address, 'multiplyOpen', data.item.market)"
              @click.stop="openDialog(data.item)"
            >
              Open Multiply
            </j-btn>
            <j-btn
              v-if="isUserHaveMultiply(data.item)"
              size="sm"
              variant="brand-secondary-outlined"
              :disabled="marketActions.isDisabled(data.item.pool_address, 'withdrawLeverage', data.item.market)"
              :loading="marketActions.isLoading(data.item.pool_address, 'withdrawLeverage', data.item.market)"
              @click.stop="openWithdrawDialog(data.item)"
            >
              Withdraw
            </j-btn>
          </div>
        </template>

        <template #empty>
          <div class="multiply-table__empty">
            No multiply vaults available.
          </div>
        </template>
      </BTable>

      <multiply-table-mobile
        v-else
        :items="vaults as any"
        @dialog-handler="(e: any) => e?.action === 'supply'
          ? openDialog(e.item)
          : e?.action === 'withdraw'
            ? openWithdrawDialog(e.item)
            : multiplyStore.openVault(e.item)"
      />
    </div>

    <client-only>
      <multiply-dialog
        v-model="dialogLeverage"
        :data="selectedVault"
      />
      <withdraw-multiply-dialog
        v-model="withdrawDialogOpen"
        :data="selectedWithdrawVault"
      />
    </client-only>
  </div>
</template>

<style lang="scss">
.multiply-table {
  &__desktop {
    thead {
      th {
        border-bottom: 1px solid $border-primary;
      }
    }
    tbody tr {
      cursor: pointer;
    }
  }

  &__apy {
    color: $success;
    font-size: 14px;
    font-style: normal;
    font-weight: 700;

    &--negative {
      color: $danger;
    }
  }

  &__empty {
    color: $text-secondary;
    font-size: 12px;
    font-style: normal;
    font-weight: 400;
    line-height: 16px;
    color: $text-secondary;
    text-align: center;
  }
}
</style>
