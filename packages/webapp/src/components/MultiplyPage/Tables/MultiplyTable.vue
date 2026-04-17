<script lang="ts" setup>
import type { MultiplyVaultItem } from '~/types/table'
import { capitalize } from 'vue'
import { truncatePercent } from '~/utils'

const { width } = useWindowSize()
const marketsStore = useMarketsStore()
const marketActions = useMarketActions()
const userStore = useUserStore()

const multiplyStore = useMultiplyStore()
const vaults = computed(() => multiplyStore.vaults)

const isLoading = computed(() => (marketsStore.state.loading || userStore.loading) && vaults.value.length === 0)

const positions = computed(() => multiplyStore.positions)

const dialogLeverage = toRef(marketsStore, 'dialogLeverage')
const selectedVault = ref<MultiplyVaultItem>()

const fields = [
  { key: 'asset', label: 'Pair', align: 'left' },
  { key: 'maxMultiplier', label: 'Multiplier', align: 'center' },
  { key: 'apyAtMaxMultiplier', label: 'Net APY', align: 'center' },
  { key: 'netEquity', label: 'Net Equity', align: 'right' },
  { key: 'action', label: '', align: 'right' },
]

const vaultsByMarket = computed(() => {
  const grouped = Object.values(
    vaults.value.reduce((acc, item) => {
      const key = item.market

      if (!acc[key]) {
        acc[key] = {
          market: key,
          items: [],
        }
      }

      const netEquityUsd = getNetEquity(item)
      item.netEquityUsd = netEquityUsd

      acc[key].items.push(item)

      return acc
    }, {} as Record<string, { market: string, items: MultiplyVaultItem[] }>),
  )
  return grouped
})

function openDialog(vault: MultiplyVaultItem) {
  selectedVault.value = vault
  dialogLeverage.value = true
}

function onRowClicked(vault: MultiplyVaultItem) {
  multiplyStore.openVault(vault)
}

function getNetEquity(vault: MultiplyVaultItem): number {
  const position = positions.value.find(position => position.market === vault.market && position.pairKey === vault.pairKey)
  return position?.netEquityUsd ?? 0
}

function isUserHaveMultiply(vault: MultiplyVaultItem) {
  return checkIsHaveMultiply(userStore.state.multiplyObligations, [vault] as any, vault.depositPoolData.pool.pool_address, vault.market)
}
</script>

<template>
  <div class="multiply-table">
    <template v-if="isLoading">
      <multiply-table-skeleton v-if="width > 1024" />
      <multiply-table-skeleton-mobile v-else />
    </template>
    <div
      v-else-if="vaults.length === 0"
      class="multiply-table__empty"
    >
      No multiply vaults available.
    </div>

    <div
      class="table-wrapper"
    >
      <j-accordion
        v-for="(vault) in vaultsByMarket"
        :key="vault.market"
        visible
      >
        <template #title>
          {{ capitalize(vault.market) }} Market

          <div class="market-info-wrapper">
            <market-info-badge>
              <span data-name="title">Strategies: </span>
              <span>{{ vault.items.length }}</span>
            </market-info-badge>
          </div>

        </template>

        <BTable
          v-if="width >= 1024"
          show-empty
          borderless
          :fields="fields"
          :items="vault.items"
          responsive
          class="market-table multiply-table__desktop"
          :class="{ 'table-loading': userStore.loading }"
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

          <template #cell(maxMultiplier)="data">
            <div class="table-cell justify-content-center">
              {{ truncatePercent(data.item.maxMultiplier || 0, 2) }}x
            </div>
          </template>

          <template #cell(apyAtMaxMultiplier)="data">
            <div
              class="table-cell justify-content-center multiply-table__apy"
              :class="[`multiply-table__apy--${data.item.apyAtMaxMultiplier < 0 ? 'negative' : 'positive'}`]"
            >
              <j-pill-label
                size="sm"
                :variant="data.item.apyAtMaxMultiplier < 0 ? 'danger' : 'success'"
              >
                {{ truncatePercent(data.item.apyAtMaxMultiplier || 0, 2) }}x
              </j-pill-label>
            </div>
          </template>

          <template #cell(netEquity)="data">
            <div
              class="table-cell justify-content-end"
              :class="[`multiply-table__netEquity--${data.item?.netEquityUsd ? (data.item?.netEquityUsd < 0 ? 'negative' : 'positive') : 'neutral'}`]"
            >
              <template v-if="data.item.netEquityUsd">
                ${{ formatPrice(data.item.netEquityUsd ?? 0, 2, 2) }}
              </template>
              <template v-else>
                —
              </template>
            </div>
          </template>

          <template #cell(action)="data">
            <div class="table-cell justify-content-end market-table__action">
              <j-btn
                v-if="isUserHaveMultiply(data.item)"
                size="sm"
                variant="positive-outlined"
                :disabled="marketActions.isDisabled(data.item.pool_address, 'withdrawLeverage', data.item.market)"
                :loading="marketActions.isLoading(data.item.pool_address, 'withdrawLeverage', data.item.market)"
                @click.stop="onRowClicked(data.item)"
              >
                Manage
              </j-btn>
              <j-btn
                v-else
                size="sm"
                variant="positive-outlined"
                :disabled="marketActions.isDisabled(data.item.pool_address, 'multiplyOpen', data.item.market)"
                :loading="marketActions.isLoading(data.item.pool_address, 'multiplyOpen', data.item.market)"
                @click.stop="openDialog(data.item)"
              >
                Multiply
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
          :items="vaults"
          @dialog-handler="(e: any) => e?.action === 'supply'
            ? openDialog(e.item)
            : onRowClicked(e.item)"
        />
      </j-accordion>

    </div>

    <client-only>
      <multiply-dialog
        v-model="dialogLeverage"
        :data="selectedVault"
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
    &--positive {
      color: $success;
    }

    &--negative {
      color: $danger;
    }
  }

  &__netEquity {
    &--neutral {
      color: $text-tertiary;
    }
    &--positive {
      color: $success;
    }
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
