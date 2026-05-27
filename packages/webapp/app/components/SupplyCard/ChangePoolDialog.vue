<script lang="ts" setup>
const {
  filteredPositions = [],
  isBorrow = false,
} = defineProps<{
  filteredPositions?: string[]
  isBorrow?: boolean
}>()

const dialog = defineModel({ default: false })

const { getFullTokenData } = useTokensStore()

const route = useRoute()
const router = useRouter()

const selectedOption = ref()

const search = ref()

const marketsStore = useMarketsStore()
const { nativeBalance, getAssetBalance } = useWalletComposable()

const selectedPoolAddress = computed(() => marketsStore.selectedPoolAddress)

const options = computed(() => {
  return marketsStore.selectedMarketPools
    ?.filter(p => !filteredPositions.includes(p.pool.pool_address))
    ?.map((data) => {
      const asset = getFullTokenData(data.pool.token_symbol)
      const balance = asset?.symbol === 'XLM' ? nativeBalance.value : getAssetBalance(destructurePoolAsset(data.pool.name)[1])
      const oraclePriceDecimals = marketsStore.activeMarket?.marketState.oracle_price_decimals ?? 0
      const price = balance > 0 ? Number(bigintToNumber(data.oracle_asset_price, oraclePriceDecimals)) || 0 : 0
      const balanceUsd = price * balance
      return {
        label: asset.symbol,
        value: data.pool.pool_address,
        name: asset.name,
        icon: asset.icon,
        apy: {
          borrow: data.apy.borrow_bps / 100,
          supply: data.apy.supply_bps / 100,
        },
        balance: balanceUsd,
      }
    }) ?? []
})

const filteredOptions = computed(() => {
  return search.value ? options.value.filter(option => option.label.toLowerCase().includes(search.value.toLowerCase())) : options.value
})

const escHandler = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    dialog.value = false
  }
}

function cleanUp() {
  globalThis.removeEventListener('keydown', escHandler)
  search.value = ''
}

function handleSelect(option: typeof options.value[number]) {
  selectedOption.value = option
  dialog.value = false
}

watch(selectedOption, (opt) => {
  if (!opt) {
    return
  }

  router.push({
    name: route.name as string,
    params: {
      ...route.params,
      pool: opt.value,
      page: isBorrow ? 'pool' : route.params.page,
    },
    query: {
      ...route.query,
      action: isBorrow ? 'borrow' : 'supply',
    },
  })
})

watch(dialog, async (isOpen) => {
  globalThis.removeEventListener('keydown', escHandler)

  if (isOpen) {
    globalThis.addEventListener('keydown', escHandler)

    await sleep(300)
    focusInput('.select-entity-input')
  } else {
    cleanUp()
  }
})

onUnmounted(() => {
  cleanUp()
})

console.log(filteredOptions.value)
</script>

<template>
  <select-entity-dialog
    v-model="dialog"
    v-model:search="search"
    search-placeholder="Search token in current market"
    class="change-pool-dialog"
  >
    <div
      v-if="filteredOptions.length > 0"
      class="pool-list"
    >
      <div
        v-for="option in filteredOptions"
        :key="option.value"
        class="pool-list__item"
        @click="() => handleSelect(option)"
      >
        <img
          :src="option.icon"
          alt="asset icon"
          class="asset-icon"
        >
        <div class="asset-data">
          <div class="asset-data__symbol">{{ option.label }}</div>
          <div class="asset-data__name">{{ option.name }}</div>
        </div>

        <div class="apy-data">
          <div :style="{ '--color': '#22d3ee' }">
            Supply APY: <span>{{ option.apy.supply }}%</span>
          </div>
          <div :style="{ '--color': '#8a8df4' }">
            Borrow APY: <span>{{ option.apy.borrow }}%</span>
          </div>
        </div>

        <div
          v-if="selectedPoolAddress === option.value"
          class="selected-badge"
        >
          Selected
        </div>

        <div class="balance-data">
          <div class="label">My Balance</div>
          <div class="value">${{ formatPrice(option.balance, 2, 2) }}</div>
        </div>
      </div>
    </div>

    <div
      v-else
      class="no-data"
    >
      {{ filteredPositions.length > 0 ? 'No pools to borrow' : 'No pools' }}
    </div>
  </select-entity-dialog>
</template>

<style lang="scss">
.change-pool-dialog {
  .pool-list {
    display: flex;
    flex-direction: column;
    padding: 16px 0;

    &__item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 10px 16px;
      cursor: pointer;

      &:hover {
        background-color: color-mix(in oklab, $secondary 70%, transparent);
      }

      .asset-icon {
        width: 32px;
        height: 32px;
        object-fit: contain;
        border-radius: 50%;
      }

      .asset-data {
        min-width: 40px;
        &__symbol {
          font-size: 14px;
          color: $text-primary;
        }

        &__name {
          font-size: $text-xs;
          color: $text-tertiary;
        }
      }

      .apy-data {
        display: flex;
        align-items: flex-start;
        flex-direction: column;
        font-size: $text-xs;
        color: $text-primary;

        > div {
          display: flex;
          align-items: center;
          justify-content: flex-end;
          gap: 8px;

          &::before {
            content: '';
            display: block;
            width: 4px;
            height: 4px;
            border-radius: 50%;
            background-color: var(--color);
          }

          span {
            color: var(--color);
            font-family: $font-JetBrainsMono;
          }
        }
      }

      .balance-data {
        display: flex;
        align-items: flex-end;
        flex-direction: column;
        margin-left: auto;

        .label {
          color: $text-tertiary;
          font-size: 12px;
          font-style: normal;
          font-weight: 400;
          line-height: normal;
        }

        .value {
          color: $text-primary;
          font-family: $font-JetBrainsMono;
          font-size: 14px;
          font-style: normal;
          font-weight: 500;
          line-height: 20px;
        }
      }
    }
  }

  .selected-badge {
    font-size: 10px;
    line-height: 1;
    color: #111;
    padding: 4px 6px;
    border-radius: 999px;
    background-color: $cyan;
    margin-bottom: auto;
    margin-left: -2px;
  }
}
</style>
