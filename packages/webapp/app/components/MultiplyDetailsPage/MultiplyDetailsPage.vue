<script lang="ts" setup>
const route = useRoute()
const router = useRouter()

const marketsStore = useMarketsStore()
const loading = computed(() => marketsStore.state.loading)

const multiplyStore = useMultiplyStore()
const selectedVault = computed(() => multiplyStore.selectedVault)

const tabs = [{
  label: 'Pool',
  value: 'pool',
},
{
  label: 'Info / Risks',
  value: 'info',
},
{
  label: 'Position',
  value: 'position',
},
{
  label: 'Statistics',
  value: 'statistics',
}]

const defaultTab = tabs[0]!

function resolveTab(tabValue?: string | null) {
  return tabs.find(tab => tab.value === tabValue) ?? defaultTab
}

const activeTab = ref(resolveTab(route.params.page as string | undefined))

watch(() => activeTab.value?.value, (val) => {
  if (route.params.page === val) {
    return
  }

  router.push({
    params: {
      ...route.params,
      page: val,
    },
  })
})
</script>

<template>
  <j-loading-spinner
    v-if="loading"
    class="table-loading-spinner"
  >
    Loading market data...
  </j-loading-spinner>

  <template v-else-if="selectedVault">
    <j-line-tab
      v-model="activeTab"
      :tabs="tabs"
      style="margin-bottom: -12px;"
    />

    <multiply-details-overview
      v-if="activeTab.value === 'pool'"
      :selected-vault="selectedVault"
    />
    <multiply-info-risks
      v-if="activeTab.value === 'info'"
    />
    <leverage-position v-if="activeTab.value === 'position'" />
    <multiply-statistics v-if="activeTab.value === 'statistics'" />
  </template>

  <div
    v-else
    class="multiply-details__empty"
  >
    Market or pool not found.
  </div>
</template>

<style lang="scss">
.multiply-details {
  display: flex;
  flex-direction: column;
  gap: 32px;
  padding-bottom: 72px;

  .table-loading-spinner,
  .multiply-details__empty {
    padding: 32px;
    margin: 0 auto;
    color: $navi-50;
    font-size: 12px;
    font-style: normal;
    font-weight: 400;
    line-height: 16px;
    gap: 12px;
  }
}
</style>
