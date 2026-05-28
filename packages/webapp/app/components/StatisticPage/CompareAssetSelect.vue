<script lang="ts" setup>
const route = useRoute()
const router = useRouter()

const pairAsset = ref()

const marketAddress = computed(() => route.params.market as string)
const poolAddresses = computed(() => {
  const routeAddr = route.params?.pool as string
  const [address, pairAddress] = routeAddr?.split(':')
  return {
    address,
    pairAddress,
  }
})

const marketTableStore = useMarketTableStore()
const market = computed(() => marketTableStore.marketWithTableItems.find(item => item.marketAddress === marketAddress.value))

const options = computed(() => {
  const pools = market.value?.tableItems ?? []
  const filteredAssets = pools.filter(pool => pool.pool_address !== poolAddresses.value.address)
  return filteredAssets?.map((a) => {
    return {
      label: a.asset.symbol,
      data: a.asset,
      value: a.pool_address,
    }
  })
})

watch([poolAddresses, options], () => {
  pairAsset.value = options.value.find(o => o.value === poolAddresses.value.pairAddress)
}, { immediate: true })

watch(pairAsset, (val) => {
  const poolsPath = val?.value ? `${poolAddresses.value.address}:${val.value}` : poolAddresses.value.address
  router.push(`/statistics/${marketAddress.value}/${poolsPath}`)
})
</script>

<template>
  <j-select
    v-model="pairAsset"
    :options="options"
    class="compare-asset-select"
  >
    <template #label>
      <template v-if="pairAsset">
        <div>
          <img
            :src="pairAsset.data?.icon"
            alt="asset icon"
          >
          <span>
            {{ pairAsset.label }}
          </span>
        </div>
      </template>
      <template v-else>
        <span class="compare-asset-select__placeholder">Compare</span>
      </template>
    </template>

    <template #option="{ option }">
      <img
        :src="option.data.icon"
        alt="asset icon"
      >  {{ option.label }}
    </template>
  </j-select>
</template>

<style lang="scss">
.compare-asset-select {
  margin-left: auto;
  width: 120px;

  @media (max-width: $breakpoint-xs) {
    display: none;
  }

  &__placeholder {
    color: $text-tertiary;
  }

  .btn {
    width: 100%;
    justify-content: space-between;
    display: flex;
    align-items: center;
    overflow: hidden;

    img {
      min-width: 22px;
      width: 18px;
      height: 18px;
      object-fit: contain;
      border-radius: 50%;
    }

    & > div {
      display: flex;
      align-items: center;
      overflow: hidden;
      gap: 4px;
    }

    span {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }

  .dropdown-menu {
    .select-item {
      img {
        width: 20px;
        height: 20px;
        object-fit: contain;
        border-radius: 50%;
        margin-right: 4px;
      }

      font-size: 12px;
    }
  }
}
</style>
