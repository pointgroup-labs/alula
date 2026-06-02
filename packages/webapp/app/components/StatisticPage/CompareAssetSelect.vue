<script lang="ts" setup>
type CompareAssetOption = {
  icon: string
  label: string
  name: string
  price: number
  symbol: string
  value: string
}

const route = useRoute()
const router = useRouter()

const marketTableStore = useMarketTableStore()
const statisticsStore = useMarketStatisticsStore()

const dialog = ref(false)
const search = ref('')

const activeToken = ref<CompareAssetOption | undefined>()

const marketAddress = computed(() => route.params.market as string)
const poolAddresses = computed(() => {
  const routeAddr = route.params?.pool as string
  const [address, pairAddress] = routeAddr?.split(':')
  return {
    address,
    pairAddress,
  }
})

const market = computed(() => marketTableStore.marketWithTableItems.find(item => item.marketAddress === marketAddress.value))

const options = computed<CompareAssetOption[]>(() => {
  const pools = market.value?.tableItems ?? []
  const filteredAssets = pools.filter(pool => pool.pool_address !== poolAddresses.value.address)
  return filteredAssets?.map((a) => {
    return {
      ...a.asset,
      price: a.price,
      label: a.asset.symbol,
      value: a.pool_address,
    }
  })
})

const filteredOptions = computed<CompareAssetOption[]>(() => {
  return search.value
    ? options.value.filter(o =>
        o.label.toLowerCase().includes(search.value.toLowerCase())
        || o.value.toLowerCase().includes(search.value.toLowerCase()))
    : options.value
})

function tokenReference(token: any): string {
  if (token.label === 'XLM') {
    return 'Native asset'
  }

  const reference = token.value
  return token.assetIssuer
    ? `Issuer ${shortenAddress(reference, 4)}`
    : shortenAddress(reference, 4)
}

function tokenPrice(token: CompareAssetOption): string {
  if (token.price <= 0) {
    return 'Unpriced'
  }

  return `$${formatPrice(token.price, 2, token.price < 1 ? 4 : 2)}`
}

function select(token: CompareAssetOption): void {
  dialog.value = false
  activeToken.value = token
}

function unselect() {
  const pairAddress = poolAddresses.value?.pairAddress
  if (!pairAddress) {
    return
  }
  for (const poolKey of statisticsStore.historyMap.keys()) {
    if (poolKey.includes(pairAddress)) {
      statisticsStore.historyMap.delete(poolKey)
    }
  }
  statisticsStore.state.pairPool = undefined
  activeToken.value = undefined
}

watch([poolAddresses, options], () => {
  activeToken.value = options.value.find(o => o.value === poolAddresses.value.pairAddress)
}, { immediate: true })

watch(activeToken, (val) => {
  const poolsPath = val?.value ? `${poolAddresses.value.address}:${val.value}` : poolAddresses.value.address
  router.push(`/statistics/${marketAddress.value}/${poolsPath}`)
})
</script>

<template>
  <button
    id="select-compare-asset-btn"
    type="button"
    @click="dialog = true"
  >
    <template v-if="!activeToken">
      Compare Asset

      <i-app-chevron-down class="chevron-icon" />
    </template>
    <template v-else>
      <div class="asset-data">
        <img
          :src="activeToken.icon"
          alt="token icon"
        >
        {{ activeToken.label }}
      </div>

      <i-app-cross-icon
        class="cross-icon"
        @click.stop="unselect"
      />
    </template>
  </button>

  <select-entity-dialog
    v-model="dialog"
    v-model:search="search"
    search-placeholder="Search asset in current market"
    class="select-compare-asset-dialog"
  >
    <div
      v-if="filteredOptions.length > 0"
      class="compare-tokens-list"
    >
      <div
        v-for="token in filteredOptions"
        :key="token.value"
        class="compare-token"
        :class="{ 'compare-token--selected': token.value === activeToken?.value }"
        @click="select(token)"
      >
        <img
          :src="token.icon"
          alt="asset icon"
          class="asset-icon"
        >
        <div class="asset-data">
          <div class="asset-data__top-row">
            <div class="asset-data__symbol">{{ token.symbol }}</div>
            <div
              v-if="token.value === activeToken?.value"
              class="asset-data__badge"
            >
              Selected
            </div>
          </div>
          <div class="asset-data__name">{{ token.name }}</div>
          <div class="asset-data__meta">{{ tokenReference(token) }}</div>
        </div>
        <div class="asset-price">{{ tokenPrice(token) }}</div>
      </div>
    </div>
    <div
      v-else
      class="no-data"
    >
      No assets to compare
    </div>
  </select-entity-dialog>
</template>

<style lang="scss">
#select-compare-asset-btn {
  all: unset;
  margin-left: auto;
  font-size: 14px;
  background-color: $bg-secondary;
  padding: $spacing-sm $spacing-lg;
  border-radius: $radius-sm;
  border: 1px solid $border-primary;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;

  &:hover {
    background-color: $bg-tertiary;
  }

  .asset-data {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;

    img {
      width: 18px;
      height: 18px;
      object-fit: contain;
      border-radius: 50%;
    }
  }

  .chevron-icon {
    width: 8px;
    height: 8px;
  }

  .cross-icon {
    width: 12px;
    height: 12px;
    cursor: pointer;
  }
}

.select-compare-asset-dialog {
  .compare-tokens-list {
    display: flex;
    flex-direction: column;
    padding: 16px 0;
  }
  .compare-token {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    cursor: pointer;

    &:hover {
      background-color: color-mix(in oklab, $secondary 70%, transparent);
    }

    &--selected {
      background-color: color-mix(in oklab, $secondary 45%, transparent);
    }

    .asset-icon {
      width: 32px;
      height: 32px;
      object-fit: contain;
      border-radius: 50%;
    }

    .asset-data {
      min-width: 40px;
      flex: 1;

      &__top-row {
        display: flex;
        align-items: center;
        gap: 8px;
      }

      &__symbol {
        font-size: 14px;
        color: $text-primary;
      }

      &__badge {
        font-size: 10px;
        line-height: 1;
        color: $dark;
        padding: 4px 6px;
        border-radius: 999px;
        background-color: $brand-50;
      }

      &__name {
        font-size: $text-xs;
        color: $text-tertiary;
      }

      &__meta {
        font-size: 11px;
        color: $text-secondary;
      }
    }

    .asset-price {
      margin-left: auto;
      font-size: 12px;
      color: $text-secondary;
      white-space: nowrap;
    }
  }
}
</style>
